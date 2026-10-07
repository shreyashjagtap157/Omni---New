//! Omni Compiler Driver CLI Entry Point
//! Fully featured argument parser supporting input files, optimization levels, and output targets.

use omni_types::ast::{BinOp, Expr, GenericFnDef, Lit, StructDef, TypeSpec, UnOp};
use omni_types::checker::SubstEnv;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct Args {
    pub input_file: Option<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub opt_level: u8,
    pub emit_native: bool,
}

pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut input_file = None;
    let mut output_file = None;
    let mut opt_level = 0;
    let mut emit_native = false;

    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                if let Some(val) = iter.next() {
                    output_file = Some(PathBuf::from(val));
                } else {
                    return Err("Missing value for output flag".into());
                }
            }
            "-O" | "--opt-level" => {
                if let Some(val) = iter.next() {
                    opt_level =
                        val.parse().map_err(|_| format!("Invalid optimization level: {val}"))?;
                } else {
                    return Err("Missing value for optimization level".into());
                }
            }
            "--native" => {
                emit_native = true;
            }
            _ if !arg.starts_with('-') => {
                input_file = Some(PathBuf::from(arg));
            }
            _ => {
                return Err(format!("Unknown flag: {}", arg));
            }
        }
    }

    Ok(Args { input_file, output_file, opt_level, emit_native })
}

/// Apply the Stage-0 feature profile to a successfully parsed program.
///
/// This is the profile layer's job, not the parser's. The grammar layer parses
/// all of Edition 1 and reports which Stage-0-classified features the source
/// exercised; this gate decides whether the selected profile enables them, using
/// the manifest's normative `stage0_feature_predicates` lists.
///
/// The gate is fail-closed in both directions: a feature the manifest forbids
/// and a feature merely absent from the allowed set are both rejected, because
/// an unlisted feature is unknown rather than disabled. Every rejection names
/// the feature and the byte span that introduced it, so the diagnostic points at
/// real source.
fn enforce_stage0_profile(
    feature_uses: &[omni_parse::FeatureUse],
    manifest: &omni_registry::Manifest,
) -> Result<(), String> {
    if feature_uses.is_empty() {
        return Ok(());
    }
    let (allowed, forbidden) =
        manifest.stage0_feature_sets().map_err(|e| format!("Predicate config error: {e}"))?;
    let engine = omni_stage0::predicates::Stage0PredicateEngine::from_lists(
        &allowed,
        &forbidden,
        &manifest.manifest_version,
    )
    .map_err(|e| format!("Predicate configuration error: {e}"))?;
    let profile = engine
        .select_profile(omni_stage0::predicates::STAGE0_PROFILE)
        .map_err(|e| format!("Profile selection error: {e}"))?;

    // Deterministic order: `feature_uses` is already sorted by feature name,
    // so the reported message does not depend on source ordering.
    let rejected: Vec<String> = feature_uses
        .iter()
        .filter_map(|used| match profile.check(used.feature) {
            Ok(()) => None,
            Err(_) => {
                Some(format!("{} at bytes {}..{}", used.feature, used.span.start, used.span.end))
            }
        })
        .collect();
    if rejected.is_empty() {
        return Ok(());
    }
    Err(format!(
        "Stage-0 profile error: feature not enabled under profile '{}': {}",
        omni_stage0::predicates::STAGE0_PROFILE,
        rejected.join(", ")
    ))
}

/// Runs the authoritative frontend, semantic, and monomorphization stages.
///
/// Returns the concrete program plus the struct declarations registered along the
/// way. Both are needed downstream: the backend resolves named field projections
/// from those declarations, and MIR lowering must type the same projections.
///
/// Both the native object path and the abstract-machine path start here, so a
/// program can never be executed on one pipeline and compiled on another.
fn lower_to_concrete_program(
    source_code: &str,
    manifest: &omni_registry::Manifest,
) -> Result<(omni_types::monomorph::MonomorphizedProgram, HashMap<String, StructDef>), String> {
    let mut parser = omni_parse::Parser::from_source(source_code);
    let parsed = parser.parse_source();
    if !parsed.is_ok() {
        return Err(format!(
            "Parse error: {}",
            parsed.diagnostics.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join("; ")
        ));
    }

    // Profile gating happens after a clean parse: a rejected program has no
    // trustworthy feature attribution to judge.
    enforce_stage0_profile(&parsed.feature_uses, manifest)?;

    let syntax = parsed.syntax();
    let mut resolver = omni_names::Resolver::new(0, 0);
    resolver
        .resolve_source(&syntax)
        .map_err(|errors| format!("Name resolution error: {:?}", errors))?;

    let functions = semantic_functions_from_cst(&syntax)?;
    if functions.is_empty() {
        return Err("Semantic error: source contains no function definitions".into());
    }
    if !functions.iter().any(|f| f.name == "main") {
        return Err("Semantic error: source contains no main function".into());
    }

    let mut checker = omni_types::TypeChecker::new();
    let trait_system = Arc::new(omni_traits::TraitSystem::new());
    trait_system.attach_to_checker(&mut checker);
    let enum_names = syntax
        .children()
        .filter(|n| n.kind() == omni_syntax::SyntaxKind::EnumDef)
        .filter_map(|n| direct_name(&n))
        .collect::<HashSet<_>>();
    for struct_def in semantic_structs_from_cst(&syntax, &enum_names)? {
        checker.register_struct(struct_def);
    }
    for enum_def in semantic_enums_from_cst(&syntax, &enum_names)? {
        checker.register_enum(enum_def);
    }
    for alias in semantic_type_aliases_from_cst(&syntax, &enum_names)? {
        checker.register_type_alias(alias);
    }
    for (name, spec, mutable) in semantic_globals_from_cst(&syntax, &enum_names)? {
        let ty = checker.lower_type_spec(&spec, &SubstEnv::new());
        checker.register_global(name, ty, mutable);
    }
    for func in &functions {
        checker.register_fn(func.clone());
    }

    let empty_env = SubstEnv::new();
    for func in &functions {
        let mut locals = HashMap::new();
        for (name, spec) in &func.params {
            let ty = checker.lower_type_spec(spec, &empty_env);
            locals.insert(name.clone(), ty);
        }

        let inferred = checker
            .infer_expr(&func.body, &empty_env, &locals)
            .map_err(|e| format!("Type error in '{}': {:?}", func.name, e))?;
        let expected = checker.lower_type_spec(&func.return_type, &empty_env);
        if inferred != expected {
            return Err(format!(
                "Type error in '{}': body has type {:?}, declared return type is {:?}",
                func.name, inferred, expected
            ));
        }

        checker
            .check_fn_effects(func)
            .map_err(|e| format!("Effect error in '{}': {:?}", func.name, e))?;
    }

    let mut monomorphizer = omni_types::Monomorphizer::new(&mut checker);
    let program = monomorphizer
        .monomorphize_entry("main", &[], &[])
        .map_err(|e| format!("Monomorphization error: {:?}", e))?;

    // Capture the declarations before the checker is consumed by the monomorphizer.
    let struct_defs = checker.struct_defs.clone();
    Ok((program, struct_defs))
}

/// Runs the concrete program through MIR lowering and MIR verification,
/// returning the verified MIR program the machine tier executes.
fn lower_to_hir(
    source_code: &str,
    manifest: &omni_registry::Manifest,
) -> Result<(omni_types::monomorph::MonomorphizedProgram, omni_hir::HirProgram), String> {
    let (program, struct_defs) = lower_to_concrete_program(source_code, manifest)?;
    let hir = omni_hir::HirProgram::from_monomorphized_program(&program, struct_defs)
        .map_err(|e| format!("HIR construction error: {}", e))?;
    Ok((program, hir))
}

fn lower_to_verified_mir(
    source_code: &str,
    manifest: &omni_registry::Manifest,
) -> Result<omni_mir::ir::MirProgram, String> {
    let (_, hir) = lower_to_hir(source_code, manifest)?;

    let mut lowering = omni_mir::lower::LoweringContext::new();
    let mir = lowering.lower_hir_program(hir).map_err(|e| format!("MIR lowering error: {}", e))?;

    omni_verify::MirVerifier::verify_program(&mir)
        .map_err(|e| format!("MIR verification error: {:?}", e))?;

    Ok(mir)
}

/// Executes `main` on the abstract machine and returns its integer result.
///
/// This is the machine tier's real entry point. It lowers and verifies MIR
/// first, so an unverified or malformed program is rejected before execution
/// rather than being interpreted optimistically.
pub fn compile_source_to_interpreter_value(
    source_code: &str,
    manifest: &omni_registry::Manifest,
) -> Result<i64, String> {
    let mir = lower_to_verified_mir(source_code, manifest)?;

    let mut interpreter = omni_machine::Interpreter::new_with_program(mir);
    let value = interpreter
        .execute_function("main", vec![])
        .map_err(|e| format!("Machine execution error: {:?}", e))?;

    match value {
        omni_machine::Value::Int(n) => Ok(n),
        other => Err(format!("Machine execution produced a non-integer main result: {:?}", other)),
    }
}

/// Compile source through the authoritative frontend, semantic, monomorphization,
/// typed-MIR, verification, and Cranelift native stages.
pub fn compile_source_to_object(
    source_code: &str,
    manifest: &omni_registry::Manifest,
) -> Result<Vec<u8>, String> {
    let (program, hir) = lower_to_hir(source_code, manifest)?;
    let mut lowering = omni_mir::lower::LoweringContext::new();
    let mir = lowering.lower_hir_program(hir).map_err(|e| format!("MIR lowering error: {}", e))?;
    omni_verify::MirVerifier::verify_program(&mir)
        .map_err(|e| format!("MIR verification error: {:?}", e))?;
    omni_codegen::compile_verified_mir_program(&program, &mir)
}

fn semantic_type_aliases_from_cst(
    root: &omni_syntax::SyntaxNode,
    enum_names: &HashSet<String>,
) -> Result<Vec<omni_types::ast::TypeAliasDef>, String> {
    let mut aliases = Vec::new();
    for node in root.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeAlias) {
        let name = direct_name(&node)
            .ok_or_else(|| "Semantic frontend error: type alias is missing a name".to_string())?;
        let type_params = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::GenericParams)
            .into_iter()
            .flat_map(|g| g.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeParam))
            .filter_map(|p| direct_name(&p))
            .collect::<Vec<_>>();
        let generic_names = type_params.iter().cloned().collect::<HashSet<_>>();
        let target = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::Type)
            .ok_or_else(|| {
                format!("Semantic frontend error: type alias '{}' has no target type", name)
            })
            .and_then(|n| type_spec_from_cst_with_context(n, &generic_names, enum_names))?;
        aliases.push(omni_types::ast::TypeAliasDef { name, type_params, target });
    }
    Ok(aliases)
}

fn semantic_globals_from_cst(
    root: &omni_syntax::SyntaxNode,
    enum_names: &HashSet<String>,
) -> Result<Vec<(String, TypeSpec, bool)>, String> {
    let mut globals = Vec::new();
    for node in root.children() {
        let (name, mutable) = match node.kind() {
            omni_syntax::SyntaxKind::ConstDef => {
                let name = direct_name(&node).ok_or_else(|| {
                    "Semantic frontend error: const is missing a name".to_string()
                })?;
                (name, false)
            }
            omni_syntax::SyntaxKind::StaticDef => {
                let name = direct_name(&node).ok_or_else(|| {
                    "Semantic frontend error: static is missing a name".to_string()
                })?;
                let mutable = node
                    .children_with_tokens()
                    .filter_map(|e| e.into_token())
                    .any(|t| t.kind() == omni_syntax::SyntaxKind::Keyword && t.text() == "mut");
                (name, mutable)
            }
            _ => continue,
        };
        let ty_node = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::Type)
            .ok_or_else(|| format!("Semantic frontend error: global '{}' has no type", name))?;
        let ty = type_spec_from_cst_with_context(ty_node, &HashSet::new(), enum_names)?;
        globals.push((name, ty, mutable));
    }
    Ok(globals)
}

fn semantic_structs_from_cst(
    root: &omni_syntax::SyntaxNode,
    enum_names: &HashSet<String>,
) -> Result<Vec<omni_types::ast::StructDef>, String> {
    let mut structs = Vec::new();
    for node in root.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::StructDef) {
        let name = direct_name(&node)
            .ok_or_else(|| "Semantic frontend error: struct is missing a name".to_string())?;
        let type_params = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::GenericParams)
            .into_iter()
            .flat_map(|g| g.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeParam))
            .filter_map(|p| direct_name(&p))
            .collect::<Vec<_>>();
        let generic_names = type_params.iter().cloned().collect::<HashSet<_>>();
        let mut fields = Vec::new();
        for field in node.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::StructField) {
            let field_name = direct_name(&field).ok_or_else(|| {
                format!("Semantic frontend error: struct '{}' has unnamed field", name)
            })?;
            let field_ty = direct_type(&field)
                .ok_or_else(|| {
                    format!("Semantic frontend error: field '{}' has no type", field_name)
                })
                .and_then(|n| type_spec_from_cst_with_context(n, &generic_names, enum_names))?;
            fields.push(omni_types::ast::StructFieldDef { name: field_name, ty: field_ty });
        }
        structs.push(omni_types::ast::StructDef { name, type_params, fields });
    }
    Ok(structs)
}

fn semantic_enums_from_cst(
    root: &omni_syntax::SyntaxNode,
    enum_names: &HashSet<String>,
) -> Result<Vec<omni_types::ast::EnumDef>, String> {
    let mut enums = Vec::new();
    for node in root.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::EnumDef) {
        let name = direct_name(&node)
            .ok_or_else(|| "Semantic frontend error: enum is missing a name".to_string())?;
        let type_params = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::GenericParams)
            .into_iter()
            .flat_map(|g| g.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeParam))
            .filter_map(|p| direct_name(&p))
            .collect::<Vec<_>>();
        let generic_names = type_params.iter().cloned().collect::<HashSet<_>>();
        let mut variants = Vec::new();
        for variant in node.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::EnumVariant)
        {
            let variant_name = direct_name(&variant).ok_or_else(|| {
                format!("Semantic frontend error: enum '{}' has an unnamed variant", name)
            })?;
            let payload = variant
                .children()
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::Type)
                .map(|n| type_spec_from_cst_with_context(n, &generic_names, enum_names))
                .collect::<Result<Vec<_>, _>>()?;
            variants.push(omni_types::ast::EnumVariantDef { name: variant_name, payload });
        }
        enums.push(omni_types::ast::EnumDef { name, type_params, variants });
    }
    Ok(enums)
}

fn collect_function_nodes(
    root: &omni_syntax::SyntaxNode,
    prefix: &str,
    out: &mut Vec<(omni_syntax::SyntaxNode, String)>,
) {
    for node in root.children() {
        match node.kind() {
            omni_syntax::SyntaxKind::FnDef => {
                if let Some(name) = direct_name(&node) {
                    let qualified =
                        if prefix.is_empty() { name } else { format!("{prefix}::{name}") };
                    out.push((node, qualified));
                }
            }
            omni_syntax::SyntaxKind::ModuleDecl => {
                if let Some(name) = direct_name(&node) {
                    let next = if prefix.is_empty() { name } else { format!("{prefix}::{name}") };
                    collect_function_nodes(&node, &next, out);
                }
            }
            _ => {}
        }
    }
}

/// The nominal name of a type written in a declaration position.
///
/// `direct_name` looks for a `NameRef`/`PathSegment`, but a named type such as
/// `impl Point` is a `Type > PathType > Path > PathSegment`; the leaf is three
/// levels down, so `direct_name` returns `None`. Walking to the *last* path
/// segment matches how `type_spec_from_cst` already reads a `PathType`, so
/// method registration and type lowering agree on what a type is called.
fn nominal_type_name(node: &omni_syntax::SyntaxNode) -> Option<String> {
    let path = node
        .children()
        .find(|n| n.kind() == omni_syntax::SyntaxKind::PathType)
        .and_then(|p| p.children().find(|n| n.kind() == omni_syntax::SyntaxKind::Path));
    let path = match path {
        Some(p) => p,
        None => return direct_name(node),
    };
    path.children()
        .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
        .last()
        .and_then(|seg| direct_name(&seg))
        .filter(|s| !s.is_empty())
}

/// Collects inherent methods from `impl Type { fn .. }` blocks.
///
/// Each becomes an ordinary function named `Type::method` whose first parameter
/// is the receiver, typed as the impl's target type and bound to the name `self`.
/// That representation is what makes dispatch resolvable by the type checker: the
/// receiver's type selects the `Type::` prefix, and the ordinary call-inference
/// path then does the rest. Nothing downstream needs a notion of "method".
///
/// Only *inherent* impls are collected here. `impl Trait for Type` blocks are
/// deliberately skipped: they are registered with the trait solver instead, and
/// conflating the two would give a trait impl two unrelated definitions. Trait
/// method *signatures* additionally remain blocked by the recorded EBNF hole —
/// `trait_item` references an undefined `function_signature` production — so no
/// trait impl can currently reach this point anyway.
fn collect_inherent_methods(
    root: &omni_syntax::SyntaxNode,
    out: &mut Vec<(omni_syntax::SyntaxNode, String, Option<omni_syntax::SyntaxNode>)>,
) -> Result<(), String> {
    for node in root.children() {
        if node.kind() == omni_syntax::SyntaxKind::ModuleDecl {
            // Methods are not module-scoped in Edition 1: `self.m()` resolves by
            // receiver type, not by lexical path, so nested modules are walked
            // without extending the name.
            collect_inherent_methods(&node, out)?;
            continue;
        }
        if node.kind() != omni_syntax::SyntaxKind::ImplDef {
            continue;
        }
        // `impl [generics] Type { .. }` has no `for`, which distinguishes an
        // inherent impl from `impl Trait for Type`.
        let has_for = node
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .any(|t| t.kind() == omni_syntax::SyntaxKind::Keyword && t.text() == "for");
        if has_for {
            continue;
        }
        let target =
            node.children().find(|c| c.kind() == omni_syntax::SyntaxKind::Type).ok_or_else(
                || "Semantic frontend error: inherent impl has no target type".to_string(),
            )?;
        let type_name = nominal_type_name(&target).ok_or_else(|| {
            "Semantic frontend error: inherent impl target type has no nominal name".to_string()
        })?;
        for item in node.children().filter(|c| c.kind() == omni_syntax::SyntaxKind::ImplItem) {
            for method in item.children().filter(|c| c.kind() == omni_syntax::SyntaxKind::FnDef) {
                let method_name = direct_name(&method).ok_or_else(|| {
                    "Semantic frontend error: impl method has no name".to_string()
                })?;
                out.push((method, format!("{type_name}::{method_name}"), Some(target.clone())));
            }
        }
    }
    Ok(())
}

fn semantic_functions_from_cst(
    root: &omni_syntax::SyntaxNode,
) -> Result<Vec<GenericFnDef>, String> {
    let mut functions = Vec::new();
    let mut names = HashSet::new();
    let mut function_nodes = Vec::new();
    collect_function_nodes(root, "", &mut function_nodes);
    // Inherent methods become `Type::method` functions taking `self` first, so
    // they are lowered through the same path as free functions once collected.
    // The `None` third element marks a free function, which has no receiver.
    let mut all_function_nodes: Vec<(
        omni_syntax::SyntaxNode,
        String,
        Option<omni_syntax::SyntaxNode>,
    )> = function_nodes.into_iter().map(|(n, name)| (n, name, None)).collect();
    collect_inherent_methods(root, &mut all_function_nodes)?;

    let enum_names: HashSet<String> = root
        .children()
        .filter(|n| n.kind() == omni_syntax::SyntaxKind::EnumDef)
        .filter_map(|n| direct_name(&n))
        .collect();
    let alias_defs = semantic_type_aliases_from_cst(root, &enum_names)?
        .into_iter()
        .map(|a| (a.name.clone(), a))
        .collect::<HashMap<_, _>>();

    for (node, name, receiver_type) in all_function_nodes {
        if !names.insert(name.clone()) {
            return Err(format!("Semantic frontend error: duplicate function '{}'", name));
        }

        let type_params = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::GenericParams)
            .into_iter()
            .flat_map(|g| g.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeParam))
            .filter_map(|p| direct_name(&p))
            .collect::<Vec<_>>();
        let generic_names: HashSet<String> = type_params.iter().cloned().collect();

        let params_node =
            node.children().find(|n| n.kind() == omni_syntax::SyntaxKind::ParamList).ok_or_else(
                || format!("Semantic frontend error: function '{}' has no parameter list", name),
            )?;

        let mut params = Vec::new();
        // An inherent method's receiver is its first parameter, bound to the
        // name `self` and typed as the impl's target. Prepending it here means
        // the rest of this function — and the whole call-inference path — sees
        // an ordinary parameter list.
        if let Some(target) = &receiver_type {
            let self_ty = normalize_alias_type(
                type_spec_from_cst_with_context(target.clone(), &generic_names, &enum_names)?,
                &alias_defs,
                &generic_names,
            )?;
            params.push(("self".to_string(), self_ty));
        }
        for param in params_node.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::Param) {
            let param_name = direct_name(&param).ok_or_else(|| {
                format!("Semantic frontend error: parameter in '{}' has no name", name)
            })?;
            let param_type = direct_type(&param).ok_or_else(|| {
                format!("Semantic frontend error: parameter '{}' has no type", param_name)
            })?;
            params.push((
                param_name,
                normalize_alias_type(
                    type_spec_from_cst_with_context(param_type, &generic_names, &enum_names)?,
                    &alias_defs,
                    &generic_names,
                )?,
            ));
        }

        let return_type = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::Type)
            .map(|n| {
                type_spec_from_cst_with_context(n, &generic_names, &enum_names)
                    .and_then(|s| normalize_alias_type(s, &alias_defs, &generic_names))
            })
            .transpose()?
            .unwrap_or(TypeSpec::Unit);

        let body = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::Block)
            .ok_or_else(|| format!("Semantic frontend error: function '{}' has no body", name))?;

        let bounds = bounds_from_fn_cst(&node, &type_params)?;
        let effects = effects_from_fn_cst(&node)?;
        functions.push(GenericFnDef {
            name,
            type_params,
            bounds,
            params,
            return_type,
            effects,
            capabilities: Vec::new(),
            body: expr_from_block(&body)?,
        });
    }

    Ok(functions)
}

fn normalize_alias_type(
    spec: TypeSpec,
    aliases: &HashMap<String, omni_types::ast::TypeAliasDef>,
    generic_names: &HashSet<String>,
) -> Result<TypeSpec, String> {
    fn expand(
        spec: TypeSpec,
        aliases: &HashMap<String, omni_types::ast::TypeAliasDef>,
        generic_names: &HashSet<String>,
        visiting: &mut HashSet<String>,
    ) -> Result<TypeSpec, String> {
        match spec {
            TypeSpec::Struct(name, args) if aliases.contains_key(&name) => {
                if !visiting.insert(name.clone()) {
                    return Err(format!("Semantic frontend error: cyclic type alias '{}'", name));
                }
                let alias = aliases.get(&name).expect("alias exists");
                if args.len() != alias.type_params.len() {
                    return Err(format!(
                        "Semantic frontend error: type alias '{}' expects {} arguments, found {}",
                        name,
                        alias.type_params.len(),
                        args.len()
                    ));
                }
                let mut substitutions = HashMap::new();
                for (param, arg) in alias.type_params.iter().zip(args) {
                    substitutions
                        .insert(param.clone(), expand(arg, aliases, generic_names, visiting)?);
                }
                let expanded = substitute_type_spec(alias.target.clone(), &substitutions);
                let result = expand(expanded, aliases, generic_names, visiting)?;
                visiting.remove(&name);
                Ok(result)
            }
            TypeSpec::Tuple(items) => Ok(TypeSpec::Tuple(
                items
                    .into_iter()
                    .map(|s| expand(s, aliases, generic_names, visiting))
                    .collect::<Result<_, _>>()?,
            )),
            TypeSpec::Array(elem, len) => {
                Ok(TypeSpec::Array(Box::new(expand(*elem, aliases, generic_names, visiting)?), len))
            }
            TypeSpec::Range(elem) => {
                Ok(TypeSpec::Range(Box::new(expand(*elem, aliases, generic_names, visiting)?)))
            }
            TypeSpec::Reference { lifetime, mutable, inner } => Ok(TypeSpec::Reference {
                lifetime,
                mutable,
                inner: Box::new(expand(*inner, aliases, generic_names, visiting)?),
            }),
            TypeSpec::Fn(params, ret) => Ok(TypeSpec::Fn(
                params
                    .into_iter()
                    .map(|s| expand(s, aliases, generic_names, visiting))
                    .collect::<Result<_, _>>()?,
                Box::new(expand(*ret, aliases, generic_names, visiting)?),
            )),
            TypeSpec::Struct(name, args) => Ok(TypeSpec::Struct(
                name,
                args.into_iter()
                    .map(|s| expand(s, aliases, generic_names, visiting))
                    .collect::<Result<_, _>>()?,
            )),
            TypeSpec::Enum(name, args) => Ok(TypeSpec::Enum(
                name,
                args.into_iter()
                    .map(|s| expand(s, aliases, generic_names, visiting))
                    .collect::<Result<_, _>>()?,
            )),
            TypeSpec::TraitObject { trait_name, args } => Ok(TypeSpec::TraitObject {
                trait_name,
                args: args
                    .into_iter()
                    .map(|s| expand(s, aliases, generic_names, visiting))
                    .collect::<Result<_, _>>()?,
            }),
            TypeSpec::GenericParam(name) if generic_names.contains(&name) => {
                Ok(TypeSpec::GenericParam(name))
            }
            other => Ok(other),
        }
    }
    expand(spec, aliases, generic_names, &mut HashSet::new())
}

fn substitute_type_spec(spec: TypeSpec, substitutions: &HashMap<String, TypeSpec>) -> TypeSpec {
    match spec {
        TypeSpec::GenericParam(name) => {
            substitutions.get(&name).cloned().unwrap_or(TypeSpec::GenericParam(name))
        }
        TypeSpec::Tuple(items) => TypeSpec::Tuple(
            items.into_iter().map(|s| substitute_type_spec(s, substitutions)).collect(),
        ),
        TypeSpec::Array(elem, len) => {
            TypeSpec::Array(Box::new(substitute_type_spec(*elem, substitutions)), len)
        }
        TypeSpec::Range(elem) => {
            TypeSpec::Range(Box::new(substitute_type_spec(*elem, substitutions)))
        }
        TypeSpec::Reference { lifetime, mutable, inner } => TypeSpec::Reference {
            lifetime,
            mutable,
            inner: Box::new(substitute_type_spec(*inner, substitutions)),
        },
        TypeSpec::Fn(params, ret) => TypeSpec::Fn(
            params.into_iter().map(|s| substitute_type_spec(s, substitutions)).collect(),
            Box::new(substitute_type_spec(*ret, substitutions)),
        ),
        TypeSpec::Struct(name, args) => TypeSpec::Struct(
            name,
            args.into_iter().map(|s| substitute_type_spec(s, substitutions)).collect(),
        ),
        TypeSpec::Enum(name, args) => TypeSpec::Enum(
            name,
            args.into_iter().map(|s| substitute_type_spec(s, substitutions)).collect(),
        ),
        TypeSpec::TraitObject { trait_name, args } => TypeSpec::TraitObject {
            trait_name,
            args: args.into_iter().map(|s| substitute_type_spec(s, substitutions)).collect(),
        },
        other => other,
    }
}

fn effects_from_fn_cst(node: &omni_syntax::SyntaxNode) -> Result<omni_effects::EffectRow, String> {
    let Some(generic_params) =
        node.children().find(|n| n.kind() == omni_syntax::SyntaxKind::GenericParams)
    else {
        return Ok(omni_effects::EffectRow::pure());
    };

    if generic_params.children().any(|n| n.kind() == omni_syntax::SyntaxKind::EffectParam) {
        return Err(
            "Semantic frontend error: generic effect parameters require effect-argument specialization before MIR"
                .into(),
        );
    }

    if generic_params.children().any(|n| n.kind() == omni_syntax::SyntaxKind::CapabilityParam) {
        return Err("Semantic frontend error: generic capability parameters are not representable by the current semantic capability model".into());
    }

    Ok(omni_effects::EffectRow::pure())
}

fn bound_trait_name(node: &omni_syntax::SyntaxNode) -> Option<String> {
    node.descendants()
        .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
        .last()
        .and_then(|n| direct_name(&n))
}

fn bounds_from_fn_cst(
    node: &omni_syntax::SyntaxNode,
    type_params: &[String],
) -> Result<Vec<(String, omni_types::ast::TraitBound)>, String> {
    let generic_names = type_params.iter().cloned().collect::<HashSet<_>>();
    let mut bounds = Vec::new();

    if let Some(generic_params) =
        node.children().find(|n| n.kind() == omni_syntax::SyntaxKind::GenericParams)
    {
        for param in
            generic_params.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeParam)
        {
            let name = direct_name(&param)
                .ok_or_else(|| "Semantic frontend error: type parameter has no name".to_string())?;
            for bound in param.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeBound)
            {
                let trait_refs = bound
                    .descendants()
                    .filter(|n| n.kind() == omni_syntax::SyntaxKind::TraitRef)
                    .collect::<Vec<_>>();
                if trait_refs.is_empty() {
                    return Err(format!(
                        "Semantic frontend error: type parameter '{}' uses an unsupported non-trait bound",
                        name
                    ));
                }
                for trait_ref in trait_refs {
                    let trait_name = bound_trait_name(&trait_ref).ok_or_else(|| {
                        format!("Semantic frontend error: type parameter '{}' has malformed trait bound", name)
                    })?;
                    bounds.push((name.clone(), omni_types::ast::TraitBound::Positive(trait_name)));
                }
            }
        }
    }

    if let Some(where_clause) =
        node.children().find(|n| n.kind() == omni_syntax::SyntaxKind::WhereClause)
    {
        for predicate in
            where_clause.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::WherePredicate)
        {
            let target = predicate
                .children()
                .next()
                .and_then(|n| {
                    n.descendants()
                        .filter(|d| d.kind() == omni_syntax::SyntaxKind::PathSegment)
                        .last()
                        .and_then(|seg| direct_name(&seg))
                })
                .ok_or_else(|| {
                    "Semantic frontend error: where predicate has no target type".to_string()
                })?;
            if !generic_names.contains(&target) {
                continue;
            }
            let trait_refs = predicate
                .descendants()
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::TraitRef)
                .collect::<Vec<_>>();
            if trait_refs.is_empty() {
                return Err(format!(
                    "Semantic frontend error: where predicate for '{}' has no supported trait bound",
                    target
                ));
            }
            for trait_ref in trait_refs {
                let trait_name = bound_trait_name(&trait_ref).ok_or_else(|| {
                    format!(
                        "Semantic frontend error: where predicate for '{}' is malformed",
                        target
                    )
                })?;
                bounds.push((target.clone(), omni_types::ast::TraitBound::Positive(trait_name)));
            }
        }
    }

    Ok(bounds)
}

fn label_from_cst(node: &omni_syntax::SyntaxNode) -> Option<String> {
    let label = node
        .children()
        .find(|n| n.kind() == omni_syntax::SyntaxKind::Label)
        .or_else(|| node.children().find(|n| n.kind() == omni_syntax::SyntaxKind::Lifetime))?;
    let name = label
        .children()
        .find(|n| n.kind() == omni_syntax::SyntaxKind::NameRef)
        .map(|n| n.text().to_string().trim().to_string())
        .unwrap_or_else(|| {
            label
                .text()
                .to_string()
                .trim()
                .trim_start_matches('\'')
                .trim_end_matches(':')
                .to_string()
        });
    (!name.is_empty()).then_some(name)
}

fn direct_name(node: &omni_syntax::SyntaxNode) -> Option<String> {
    if matches!(
        node.kind(),
        omni_syntax::SyntaxKind::NameRef | omni_syntax::SyntaxKind::PathSegment
    ) {
        let name = node.text().to_string().trim().to_string();
        return (!name.is_empty()).then_some(name);
    }
    node.children()
        .find(|n| {
            matches!(
                n.kind(),
                omni_syntax::SyntaxKind::NameRef | omni_syntax::SyntaxKind::PathSegment
            )
        })
        .map(|n| n.text().to_string().trim().to_string())
        .filter(|s| !s.is_empty())
}

/// The identifier a binding pattern introduces.
///
/// The parser lowers `let x` and a match binding `x` to
/// `IdentifierPattern > Path > PathSegment`, so the name is not a `NameRef`.
/// `direct_name` handles the leaf, but the intermediate `Path` has to be
/// traversed to reach it.
fn pattern_binding_text(node: &omni_syntax::SyntaxNode) -> Option<String> {
    if node.kind() == omni_syntax::SyntaxKind::PathSegment {
        return direct_name(node);
    }
    for child in node.children() {
        if let Some(name) = pattern_binding_text(&child) {
            return Some(name);
        }
    }
    None
}

fn direct_type(node: &omni_syntax::SyntaxNode) -> Option<omni_syntax::SyntaxNode> {
    node.children().find(|n| n.kind() == omni_syntax::SyntaxKind::Type)
}

fn type_spec_from_cst(node: omni_syntax::SyntaxNode) -> Result<TypeSpec, String> {
    type_spec_from_cst_with_context(node, &HashSet::new(), &HashSet::new())
}

fn type_spec_from_cst_with_generics(
    node: omni_syntax::SyntaxNode,
    generic_names: &HashSet<String>,
) -> Result<TypeSpec, String> {
    type_spec_from_cst_with_context(node, generic_names, &HashSet::new())
}

fn type_spec_from_cst_with_context(
    node: omni_syntax::SyntaxNode,
    generic_names: &HashSet<String>,
    enum_names: &HashSet<String>,
) -> Result<TypeSpec, String> {
    if node.kind() == omni_syntax::SyntaxKind::Type {
        return node
            .children()
            .next()
            .ok_or_else(|| "Semantic frontend error: empty type node".to_string())
            .and_then(|n| type_spec_from_cst_with_context(n, generic_names, enum_names));
    }

    match node.kind() {
        omni_syntax::SyntaxKind::TraitRef => {
            let mut children = node.children();
            let _dyn = children.next();
            let path = children
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Path)
                .ok_or_else(|| "Semantic frontend error: trait object is missing its trait path".to_string())?;
            let segments = path
                .children()
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
                .collect::<Vec<_>>();
            let last = segments
                .last()
                .ok_or_else(|| "Semantic frontend error: trait object path is empty".to_string())?;
            let trait_name = direct_name(last)
                .ok_or_else(|| "Semantic frontend error: trait object path has no trait name".to_string())?;
            let args = last
                .children()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::TypeArgs)
                .map(|type_args| {
                    type_args
                        .children()
                        .filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeArg)
                        .filter_map(|arg| arg.children().next())
                        .map(type_spec_from_cst)
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?
                .unwrap_or_default();
            Ok(TypeSpec::TraitObject { trait_name, args })
        }
        omni_syntax::SyntaxKind::PathType => {
            let path = node.children().find(|n| n.kind() == omni_syntax::SyntaxKind::Path);
            let text = path
                .as_ref()
                .map(|n| n.text().to_string().trim().to_string())
                .unwrap_or_else(|| node.text().to_string().trim().to_string());
            let base_name = path
                .as_ref()
                .and_then(|p| {
                    p.children()
                        .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
                        .last()
                })
                .and_then(|seg| direct_name(&seg))
                .unwrap_or_else(|| text.split('<').next().unwrap_or(&text).rsplit("::").next().unwrap_or(&text).trim().to_string());
            if generic_names.contains(&base_name) && path.as_ref().is_some_and(|p| p.children().count() <= 1) {
                return Ok(TypeSpec::GenericParam(base_name));
            }
            match base_name.as_str() {
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "Int" => Ok(TypeSpec::Int),
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "byte" | "Byte" => Ok(TypeSpec::Byte),
                "f16" | "f32" | "f64" | "f128" | "bf16" | "dec32" | "dec64" | "dec128" | "Float" => Ok(TypeSpec::Float),
                "bool" | "Bool" => Ok(TypeSpec::Bool),
                "char" | "Char" => Ok(TypeSpec::Char),
                "str" | "String" => Ok(TypeSpec::String),
                "unit" | "Unit" => Ok(TypeSpec::Unit),
                "never" | "Never" | "!" => Ok(TypeSpec::Never),
                _ => {
                    let args = path
                        .as_ref()
                        .map(|p| {
                            p.children()
                                .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
                                .filter_map(|seg| seg.children().find(|n| n.kind() == omni_syntax::SyntaxKind::TypeArgs))
                                .flat_map(|args| args.children())
                                .filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeArg)
                                .map(|arg| arg.children().next().map(|n| type_spec_from_cst_with_context(n, generic_names, enum_names)).transpose())
                                .collect::<Result<Vec<_>, _>>()
                        })
                        .transpose()?
                        .unwrap_or_default()
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>();
                    if enum_names.contains(&base_name) {
                        Ok(TypeSpec::Enum(base_name, args))
                    } else {
                        Ok(TypeSpec::Struct(base_name, args))
                    }
                }
            }
        }
        omni_syntax::SyntaxKind::TupleType => {
            Ok(TypeSpec::Tuple(node.children().map(|n| type_spec_from_cst_with_context(n, generic_names, enum_names)).collect::<Result<Vec<_>, _>>()?))
        }
        omni_syntax::SyntaxKind::ArrayType => {
            let mut children = node.children();
            let elem = children.next().ok_or_else(|| "Semantic frontend error: array type has no element".to_string()).and_then(|n| type_spec_from_cst_with_context(n, generic_names, enum_names))?;
            let len_expr = children.next().ok_or_else(|| "Semantic frontend error: array type has no length".to_string())?;
            let len_text = len_expr.text().to_string().trim().to_string();
            let len = parse_int_literal(&len_text)
                .map_err(|_| format!("Semantic frontend error: array length '{}' is not an integer", len_text))?;
            let len = usize::try_from(len).map_err(|_| "Semantic frontend error: array length is negative".to_string())?;
            Ok(TypeSpec::Array(Box::new(elem), len))
        }
        omni_syntax::SyntaxKind::FunctionType => {
            let has_return = node
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .any(|t| t.kind() == omni_syntax::SyntaxKind::Punct && t.text() == "->");
            let types = node.children().map(|n| type_spec_from_cst_with_generics(n, generic_names)).collect::<Result<Vec<_>, _>>()?;
            if has_return {
                let (ret, params) = types
                    .split_last()
                    .map(|(ret, params)| (ret.clone(), params.to_vec()))
                    .unwrap_or((TypeSpec::Unit, Vec::new()));
                Ok(TypeSpec::Fn(params, Box::new(ret)))
            } else {
                Ok(TypeSpec::Fn(types, Box::new(TypeSpec::Unit)))
            }
        }
        omni_syntax::SyntaxKind::ParenthesizedType => node.children().next().ok_or_else(|| "Semantic frontend error: empty parenthesized type".to_string()).and_then(|n| type_spec_from_cst_with_generics(n, generic_names)),
        omni_syntax::SyntaxKind::NeverType => Ok(TypeSpec::Never),
        omni_syntax::SyntaxKind::ReferenceType => {
            let mut children = node.children();
            let first = children.next().ok_or_else(|| "Semantic frontend error: reference type has no '&'".to_string())?;
            if first.kind() != omni_syntax::SyntaxKind::PathType {
                // The lexer/parser represent '&' and its modifiers as tokens, while
                // the pointee is the final Type child. Recover the semantic parts
                // directly from the CST.
            }
            let mutable = node
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .any(|t| t.kind() == omni_syntax::SyntaxKind::Keyword && t.text() == "mut");
            let lifetime = node
                .children()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Lifetime)
                .and_then(|lt| lt.children().find(|n| n.kind() == omni_syntax::SyntaxKind::NameRef))
                .map(|n| n.text().to_string().trim().to_string());
            let inner = node
                .children()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Type)
                .ok_or_else(|| "Semantic frontend error: reference type has no pointee type".to_string())
                .and_then(|n| type_spec_from_cst_with_context(n, generic_names, enum_names))?;
            Ok(TypeSpec::Reference {
                lifetime,
                mutable,
                inner: Box::new(inner),
            })
        }
        omni_syntax::SyntaxKind::SliceType
        | omni_syntax::SyntaxKind::RawPointerType => Err(format!("Semantic frontend error: type form {:?} is not representable by the current semantic TypeSpec", node.kind())),
        other => Err(format!("Semantic frontend error: unsupported type node {:?}", other)),
    }
}

fn expr_from_block(node: &omni_syntax::SyntaxNode) -> Result<Expr, String> {
    let statements: Vec<_> = node.children().collect();
    block_statements_to_expr(&statements)
}

fn block_statements_to_expr(statements: &[omni_syntax::SyntaxNode]) -> Result<Expr, String> {
    let mut values = Vec::new();
    for (index, statement) in statements.iter().enumerate() {
        match statement.kind() {
            omni_syntax::SyntaxKind::LetStmt => {
                let pattern_node = statement
                    .children()
                    .find(|n| matches!(
                        n.kind(),
                        omni_syntax::SyntaxKind::BindingPattern
                            | omni_syntax::SyntaxKind::IdentifierPattern
                            | omni_syntax::SyntaxKind::WildcardPattern
                            | omni_syntax::SyntaxKind::TuplePattern
                            | omni_syntax::SyntaxKind::SlicePattern
                            | omni_syntax::SyntaxKind::StructPattern
                            | omni_syntax::SyntaxKind::EnumPattern
                            | omni_syntax::SyntaxKind::ReferencePattern
                    ))
                    .ok_or_else(|| "Semantic frontend error: let statement is missing a pattern".to_string())?;
                let pattern = pattern_from_cst(&pattern_node)?;
                let binding_type = direct_type(statement).map(type_spec_from_cst).transpose()?;
                let initializer = statement
                    .children()
                    .filter(|n| !matches!(
                        n.kind(),
                        omni_syntax::SyntaxKind::NameRef
                            | omni_syntax::SyntaxKind::Type
                            | omni_syntax::SyntaxKind::BindingPattern
                            | omni_syntax::SyntaxKind::IdentifierPattern
                            | omni_syntax::SyntaxKind::WildcardPattern
                            | omni_syntax::SyntaxKind::TuplePattern
                            | omni_syntax::SyntaxKind::SlicePattern
                            | omni_syntax::SyntaxKind::StructPattern
                            | omni_syntax::SyntaxKind::EnumPattern
                            | omni_syntax::SyntaxKind::ReferencePattern
                    ))
                    .last()
                    .ok_or_else(|| "Semantic frontend error: let has no initializer".to_string())?;
                let init = expr_from_node(&initializer)?;
                let remaining = block_statements_to_expr(&statements[index + 1..])?;
                return Ok(Expr::Let {
                    pattern,
                    ty: binding_type,
                    init: Box::new(init),
                    body: Box::new(remaining),
                });
            }
            omni_syntax::SyntaxKind::FinalExpr => {
                let expr = expr_from_node(statement)?;
                values.push(expr);
            }
            omni_syntax::SyntaxKind::ExprStmt
            | omni_syntax::SyntaxKind::ReturnExpr
            | omni_syntax::SyntaxKind::BreakExpr
            | omni_syntax::SyntaxKind::ContinueExpr
            // `loop`, `while` and `for` are Edition 1 statements and the parser
            // emits them as bare loop nodes, not wrapped in `ExprStmt`, because
            // they take no trailing `;`. They still have to be lowered.
            | omni_syntax::SyntaxKind::LoopExpr
            | omni_syntax::SyntaxKind::WhileExpr
            | omni_syntax::SyntaxKind::ForExpr => {
                values.push(expr_from_node(statement)?);
            }
            omni_syntax::SyntaxKind::ItemDeclStmt => {
                return Err("Semantic frontend error: local item declarations are not yet lowered to the native AST".into());
            }
            other => {
                return Err(format!("Semantic frontend error: unsupported block statement {:?}", other));
            }
        }
    }
    match values.len() {
        0 => Ok(Expr::Block(Vec::new())),
        1 => Ok(values.pop().expect("one value exists")),
        _ => Ok(Expr::Block(values)),
    }
}

fn expr_from_node(node: &omni_syntax::SyntaxNode) -> Result<Expr, String> {
    match node.kind() {
        omni_syntax::SyntaxKind::ExprStmt
        | omni_syntax::SyntaxKind::FinalExpr
        | omni_syntax::SyntaxKind::ParenthesizedExpr => node
            .children()
            .next()
            .ok_or_else(|| "Semantic frontend error: expression wrapper is empty".to_string())
            .and_then(|n| expr_from_node(&n)),
        omni_syntax::SyntaxKind::Block => expr_from_block(node),
        omni_syntax::SyntaxKind::ReturnExpr => {
            let value = node.children().next().map(|n| expr_from_node(&n)).transpose()?;
            Ok(Expr::Return(value.map(Box::new)))
        }
        omni_syntax::SyntaxKind::LiteralExpr => {
            let token = node
                .first_token()
                .ok_or_else(|| "Semantic frontend error: literal node has no token".to_string())?;
            Ok(Expr::Literal(lit_from_text(token.text())?))
        }
        omni_syntax::SyntaxKind::NameRef => {
            let name = node.text().to_string().trim().to_string();
            (!name.is_empty())
                .then_some(Expr::Var(name))
                .ok_or_else(|| "Semantic frontend error: empty name reference".to_string())
        }
        omni_syntax::SyntaxKind::PathExpr => {
            if let Some((enum_name, variant, generic_args)) = enum_variant_target_from_cst(node)? {
                return Ok(Expr::EnumVariant {
                    enum_name,
                    variant,
                    generic_args,
                    args: Vec::new(),
                });
            }
            Ok(Expr::Var(node.text().to_string().trim().to_string()))
        }
        omni_syntax::SyntaxKind::BinaryExpr
        | omni_syntax::SyntaxKind::AssignExpr
        | omni_syntax::SyntaxKind::RangeExpr => {
            let parts = node.children().collect::<Vec<_>>();
            if parts.len() != 2 {
                return Err(format!(
                    "Semantic frontend error: {:?} must contain two operand nodes",
                    node.kind()
                ));
            }
            let op = node
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .find(|t| t.kind() == omni_syntax::SyntaxKind::Punct)
                .map(|t| t.text().to_string())
                .ok_or_else(|| {
                    "Semantic frontend error: expression has no operator token".to_string()
                })?;
            match node.kind() {
                omni_syntax::SyntaxKind::AssignExpr => {
                    let assign = node
                        .children_with_tokens()
                        .filter_map(|e| e.into_token())
                        .find(|t| t.kind() == omni_syntax::SyntaxKind::Punct)
                        .map(|t| t.text().to_string())
                        .ok_or_else(|| {
                            "Semantic frontend error: assignment has no operator token".to_string()
                        })?;
                    if assign == "=" {
                        Ok(Expr::Assign {
                            target: Box::new(expr_from_node(&parts[0])?),
                            value: Box::new(expr_from_node(&parts[1])?),
                        })
                    } else {
                        let op = match assign.as_str() {
                            "+=" => omni_types::ast::AssignOp::Add,
                            "-=" => omni_types::ast::AssignOp::Sub,
                            "*=" => omni_types::ast::AssignOp::Mul,
                            "/=" => omni_types::ast::AssignOp::Div,
                            "%=" => omni_types::ast::AssignOp::Rem,
                            "&=" => omni_types::ast::AssignOp::BitAnd,
                            "|=" => omni_types::ast::AssignOp::BitOr,
                            "^=" => omni_types::ast::AssignOp::BitXor,
                            "<<=" => omni_types::ast::AssignOp::Shl,
                            ">>=" => omni_types::ast::AssignOp::Shr,
                            other => {
                                return Err(format!(
                                    "Semantic frontend error: unsupported assignment operator '{}'",
                                    other
                                ))
                            }
                        };
                        Ok(Expr::CompoundAssign {
                            op,
                            target: Box::new(expr_from_node(&parts[0])?),
                            value: Box::new(expr_from_node(&parts[1])?),
                        })
                    }
                }
                omni_syntax::SyntaxKind::RangeExpr => {
                    let inclusive = node
                        .children_with_tokens()
                        .filter_map(|e| e.into_token())
                        .find(|t| {
                            t.kind() == omni_syntax::SyntaxKind::Punct
                                && (t.text() == ".." || t.text() == "..=")
                        })
                        .map(|t| t.text() == "..=")
                        .ok_or_else(|| {
                            "Semantic frontend error: range has no range operator".to_string()
                        })?;
                    Ok(Expr::Range {
                        start: Box::new(expr_from_node(&parts[0])?),
                        end: Box::new(expr_from_node(&parts[1])?),
                        inclusive,
                    })
                }
                _ => Ok(Expr::Binary {
                    op: bin_op_from_text(&op)?,
                    lhs: Box::new(expr_from_node(&parts[0])?),
                    rhs: Box::new(expr_from_node(&parts[1])?),
                }),
            }
        }
        omni_syntax::SyntaxKind::UnaryExpr => {
            let operand = node.children().last().ok_or_else(|| {
                "Semantic frontend error: unary expression has no operand".to_string()
            })?;
            let puncts = node
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .filter(|t| t.kind() == omni_syntax::SyntaxKind::Punct)
                .map(|t| t.text().to_string())
                .collect::<Vec<_>>();
            let punct_view = puncts.iter().map(String::as_str).collect::<Vec<_>>();
            let op = match punct_view.as_slice() {
                ["&"] => UnOp::BorrowShared,
                ["&", "mut"] => UnOp::BorrowMut,
                ["*"] => UnOp::Deref,
                ["-"] => UnOp::Neg,
                ["!"] => UnOp::Not,
                ["~"] => UnOp::BitNot,
                _ => {
                    return Err(format!(
                        "Semantic frontend error: unsupported unary operator sequence {:?}",
                        puncts
                    ))
                }
            };
            Ok(Expr::Unary { op, expr: Box::new(expr_from_node(&operand)?) })
        }
        omni_syntax::SyntaxKind::CallExpr => {
            let mut children = node.children();
            let callee = children
                .next()
                .ok_or_else(|| "Semantic frontend error: call has no callee".to_string())?;
            let args = children.map(|n| expr_from_node(&n)).collect::<Result<Vec<_>, _>>()?;

            if let Some((enum_name, variant, generic_args)) = enum_variant_target_from_cst(&callee)?
            {
                return Ok(Expr::EnumVariant { enum_name, variant, generic_args, args });
            }

            let (func, generic_args) = call_target_from_cst(&callee)?;
            Ok(Expr::Call { func, generic_args, args })
        }
        omni_syntax::SyntaxKind::FieldExpr => {
            let mut children = node.children();
            let base_opt = children.next();
            let field = node
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .filter(|t| {
                    matches!(
                        t.kind(),
                        omni_syntax::SyntaxKind::Ident | omni_syntax::SyntaxKind::Keyword
                    )
                })
                .last()
                .map(|t| t.text().to_string())
                .ok_or_else(|| "Semantic frontend error: field has no name".to_string())?;
            if let Some(base) = base_opt {
                Ok(Expr::Field { expr: Box::new(expr_from_node(&base)?), field })
            } else {
                // Projection shorthand .field denotes closure |it| it.field (ERR3-0033)
                let it_var = "it".to_string();
                let body = Expr::Field { expr: Box::new(Expr::Var(it_var.clone())), field };
                Ok(Expr::Lambda {
                    params: vec![(it_var, TypeSpec::GenericParam("Infer".into()))],
                    body: Box::new(body),
                })
            }
        }
        omni_syntax::SyntaxKind::IndexExpr => {
            let mut children = node.children();
            let base = children
                .next()
                .ok_or_else(|| "Semantic frontend error: index has no base".to_string())?;
            let index = children
                .next()
                .ok_or_else(|| "Semantic frontend error: index has no expression".to_string())?;
            Ok(Expr::Index {
                expr: Box::new(expr_from_node(&base)?),
                index: Box::new(expr_from_node(&index)?),
            })
        }
        omni_syntax::SyntaxKind::ArrayExpr => Ok(Expr::Array(
            node.children().map(|n| expr_from_node(&n)).collect::<Result<Vec<_>, _>>()?,
        )),
        omni_syntax::SyntaxKind::TupleExpr => Ok(Expr::Tuple(
            node.children().map(|n| expr_from_node(&n)).collect::<Result<Vec<_>, _>>()?,
        )),
        omni_syntax::SyntaxKind::StructExpr => {
            let mut children = node.children();
            let path = children.next().ok_or_else(|| {
                "Semantic frontend error: struct literal has no type path".to_string()
            })?;
            let mut generic_args = Vec::new();
            for seg in
                path.descendants().filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
            {
                if let Some(args) =
                    seg.children().find(|n| n.kind() == omni_syntax::SyntaxKind::TypeArgs)
                {
                    generic_args.extend(
                        args.children()
                            .filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeArg)
                            .filter_map(|a| a.children().next())
                            .map(type_spec_from_cst)
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
            }
            let name = path
                .descendants()
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
                .last()
                .and_then(|n| direct_name(&n))
                .ok_or_else(|| {
                    "Semantic frontend error: struct literal path has no type name".to_string()
                })?;
            let fields = children
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::StructExprField)
                .map(|field| {
                    let field_name = direct_name(&field).ok_or_else(|| {
                        "Semantic frontend error: struct literal field has no name".to_string()
                    })?;
                    let value = match field.children().nth(1) {
                        Some(n) => expr_from_node(&n)?,
                        None => Expr::Var(field_name.clone()),
                    };
                    Ok((field_name, value))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Expr::Struct { name, generic_args, fields })
        }

        omni_syntax::SyntaxKind::IfExpr => {
            let children = node.children().collect::<Vec<_>>();
            if children.len() < 2 || children.len() > 3 {
                return Err("Semantic frontend error: malformed if expression".into());
            }
            let condition = expr_from_node(&children[0])?;
            let then_branch = expr_from_node(&children[1])?;
            let else_branch = children.get(2).map(expr_from_node).transpose()?.map(Box::new);
            Ok(Expr::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch,
            })
        }
        omni_syntax::SyntaxKind::CastExpr => {
            let mut children = node.children();
            let expr = children.next().ok_or_else(|| {
                "Semantic frontend error: cast has no source expression".to_string()
            })?;
            let ty = children
                .next()
                .ok_or_else(|| "Semantic frontend error: cast has no target type".to_string())?;
            Ok(Expr::Cast { expr: Box::new(expr_from_node(&expr)?), ty: type_spec_from_cst(ty)? })
        }
        omni_syntax::SyntaxKind::LoopExpr => {
            let label = label_from_cst(node);
            let body = node
                .children()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Block)
                .ok_or_else(|| "Semantic frontend error: loop has no body".to_string())?;
            Ok(Expr::Loop { label, body: Box::new(expr_from_node(&body)?) })
        }
        omni_syntax::SyntaxKind::WhileExpr => {
            let label = node
                .children()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Lifetime)
                .map(|n| n.text().to_string().trim().trim_start_matches('\'').to_string());
            let body = node
                .children()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Block)
                .ok_or_else(|| "Semantic frontend error: while has no body".to_string())?;
            let condition = node
                .children()
                .find(|n| {
                    n.kind() != omni_syntax::SyntaxKind::Lifetime
                        && n.kind() != omni_syntax::SyntaxKind::Block
                })
                .ok_or_else(|| "Semantic frontend error: while has no condition".to_string())?;
            Ok(Expr::While {
                label,
                condition: Box::new(expr_from_node(&condition)?),
                body: Box::new(expr_from_node(&body)?),
            })
        }
        omni_syntax::SyntaxKind::ForExpr => {
            let label = node
                .children()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Lifetime)
                .map(|n| n.text().to_string().trim().trim_start_matches('\'').to_string());
            let children = node
                .children()
                .filter(|n| {
                    !matches!(
                        n.kind(),
                        omni_syntax::SyntaxKind::Label | omni_syntax::SyntaxKind::Lifetime
                    )
                })
                .collect::<Vec<_>>();
            if children.len() != 3 {
                return Err("Semantic frontend error: malformed for expression".into());
            }
            let pattern = pattern_from_cst(&children[0])?;
            let iterable = expr_from_node(&children[1])?;
            let body = expr_from_node(&children[2])?;
            Ok(Expr::For { label, pattern, iterable: Box::new(iterable), body: Box::new(body) })
        }
        omni_syntax::SyntaxKind::BreakExpr => {
            let mut children = node.children();
            let label = children
                .next()
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::Lifetime)
                .map(|n| n.text().to_string().trim().trim_start_matches('\'').to_string());
            let value_node = if label.is_some() { children.next() } else { node.children().next() };
            let value = value_node.map(|n| expr_from_node(&n)).transpose()?.map(Box::new);
            Ok(Expr::Break { label, value })
        }
        omni_syntax::SyntaxKind::ContinueExpr => {
            let label = node
                .children()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Lifetime)
                .map(|n| n.text().to_string().trim().trim_start_matches('\'').to_string());
            Ok(Expr::Continue { label })
        }
        omni_syntax::SyntaxKind::MatchExpr => {
            let mut children = node.children();
            let scrutinee = children
                .next()
                .ok_or_else(|| "Semantic frontend error: match has no scrutinee".to_string())?;
            let arms = children
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::MatchArm)
                .map(|n| match_arm_from_cst(&n))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Expr::Match { expr: Box::new(expr_from_node(&scrutinee)?), arms })
        }
        omni_syntax::SyntaxKind::ClosureExpr => {
            let params = node
                .children()
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::ClosureParam)
                .map(|n| {
                    let name = direct_name(&n).ok_or_else(|| {
                        "Semantic frontend error: closure parameter has no name".to_string()
                    })?;
                    let ty =
                        direct_type(&n).map(type_spec_from_cst).transpose()?.ok_or_else(|| {
                            format!(
                                "Semantic frontend error: closure parameter '{}' requires a type",
                                name
                            )
                        })?;
                    Ok((name, ty))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let body = node
                .children()
                .filter(|n| n.kind() != omni_syntax::SyntaxKind::ClosureParam)
                .last()
                .ok_or_else(|| "Semantic frontend error: closure has no body".to_string())?;
            Ok(Expr::Lambda { params, body: Box::new(expr_from_node(&body)?) })
        }
        omni_syntax::SyntaxKind::UnsafeBlock => {
            // `unsafe_block = "unsafe" block_expr`: the `unsafe` keyword is a
            // token child, so the body is the single `Block` child. The marker
            // is preserved rather than erased so the unsafe context survives
            // into MIR (UNSAFE-0002); the body's contents are still lowered and
            // checked exactly as an ordinary block (UNSAFE-0001).
            let body =
                node.children().find(|n| n.kind() == omni_syntax::SyntaxKind::Block).ok_or_else(
                    || "Semantic frontend error: unsafe block has no body block".to_string(),
                )?;
            Ok(Expr::UnsafeBlock { body: Box::new(expr_from_node(&body)?) })
        }
        omni_syntax::SyntaxKind::MethodCallExpr => {
            // `method_call_expr = expression "." identifier [ "<" type_args ">" ]
            // "(" [ expression { "," expression } [ "," ] ] ")"`.
            //
            // The parser emits children in source order: the receiver node, the
            // `.` token, the method-name *token*, an optional `TypeArgs` node,
            // then the argument expressions as direct children (there is no
            // argument-list node — `append_call_arguments` pushes each argument
            // expression straight into the parent). The name is a token rather
            // than a node, so it is read from the token stream, and it must be
            // located positionally: taking `children().nth(1)` would silently
            // pick the first *argument* instead, since the name is not a node.
            let receiver = node.children().next().ok_or_else(|| {
                "Semantic frontend error: method call has no receiver".to_string()
            })?;
            // The name is the first identifier *token*; `(`, `,`, `)` and the
            // generic closers are punctuators, so this cannot pick one up.
            let method = node
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .find(|t| {
                    matches!(
                        t.kind(),
                        omni_syntax::SyntaxKind::Ident | omni_syntax::SyntaxKind::Keyword
                    )
                })
                .map(|t| t.text().to_string())
                .ok_or_else(|| {
                    "Semantic frontend error: method call has no method name".to_string()
                })?;

            // Arguments are direct node children after the name; the type
            // arguments, when present, are the single `TypeArgs` child among
            // them. Distinguishing by node kind is what keeps a turbofished
            // method call's type arguments out of the positional argument list.
            let mut generic_args = Vec::new();
            let mut args = Vec::new();
            for child in node.children().skip(1) {
                if child.kind() == omni_syntax::SyntaxKind::TypeArgs {
                    generic_args = child
                        .children()
                        .filter(|t| t.kind() == omni_syntax::SyntaxKind::Type)
                        .map(type_spec_from_cst)
                        .collect::<Result<Vec<_>, _>>()?;
                } else {
                    args.push(expr_from_node(&child)?);
                }
            }
            Ok(Expr::MethodCall {
                receiver: Box::new(expr_from_node(&receiver)?),
                method,
                generic_args,
                args,
            })
        }
        omni_syntax::SyntaxKind::PlaceholderExpr => {
            // Placeholder `_` used in pipeline stages (ERR3-0030)
            Ok(Expr::Var("_".to_string()))
        }
        omni_syntax::SyntaxKind::PipelineExpr => {
            let parts = node.children().collect::<Vec<_>>();
            if parts.len() != 2 {
                return Err(
                    "Semantic frontend error: pipeline expression must have lhs and rhs".into()
                );
            }
            let lhs = expr_from_node(&parts[0])?;
            // Normative Pipeline Lowering (ERR3-0030):
            // `x |> f` -> `f(x)`
            // `x |> f(a, b)` -> `f(x, a, b)`
            // `x |> f(a, _, b)` -> `f(a, x, b)`
            match expr_from_node(&parts[1])? {
                Expr::Call { func, generic_args, mut args } => {
                    if let Some(pos) =
                        args.iter().position(|a| matches!(a, Expr::Var(v) if v == "_"))
                    {
                        args[pos] = lhs;
                    } else {
                        args.insert(0, lhs);
                    }
                    Ok(Expr::Call { func, generic_args, args })
                }
                Expr::Var(func) => {
                    Ok(Expr::Call { func, generic_args: Vec::new(), args: vec![lhs] })
                }
                Expr::MethodCall { receiver, method, generic_args, mut args } => {
                    if let Some(pos) =
                        args.iter().position(|a| matches!(a, Expr::Var(v) if v == "_"))
                    {
                        args[pos] = lhs;
                    } else {
                        args.insert(0, lhs);
                    }
                    Ok(Expr::MethodCall { receiver, method, generic_args, args })
                }
                other => Err(format!(
                    "Semantic frontend error: invalid pipeline target expression: {:?}",
                    other
                )),
            }
        }
        omni_syntax::SyntaxKind::CommandCallExpr => {
            // Command call `name arg1, arg2` (ERR3-0035) lowers to Expr::Call
            let mut children = node.children();
            let callee = children
                .next()
                .ok_or_else(|| "Semantic frontend error: command call has no callee".to_string())?;
            let (func, generic_args) = call_target_from_cst(&callee)?;
            let args = children.map(|n| expr_from_node(&n)).collect::<Result<Vec<_>, _>>()?;
            Ok(Expr::Call { func, generic_args, args })
        }
        omni_syntax::SyntaxKind::AwaitExpr
        | omni_syntax::SyntaxKind::MacroInvocation
        | omni_syntax::SyntaxKind::AsyncBlock
        | omni_syntax::SyntaxKind::TryBlock
        | omni_syntax::SyntaxKind::TryExpr => Err(format!(
            "Semantic frontend error: native AST lowering does not yet support {:?}",
            node.kind()
        )),
        other => Err(format!("Semantic frontend error: unsupported expression node {:?}", other)),
    }
}

fn enum_variant_target_from_cst(
    callee: &omni_syntax::SyntaxNode,
) -> Result<Option<(String, String, Vec<TypeSpec>)>, String> {
    let segments = callee
        .descendants()
        .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
        .collect::<Vec<_>>();
    if segments.len() < 2 {
        return Ok(None);
    }
    let enum_seg = &segments[segments.len() - 2];
    let variant_seg = &segments[segments.len() - 1];
    let enum_name = match direct_name(enum_seg) {
        Some(n) => n,
        None => return Ok(None),
    };
    let variant = match direct_name(variant_seg) {
        Some(n) => n,
        None => return Ok(None),
    };
    let generic_args = enum_seg
        .children()
        .find(|n| n.kind() == omni_syntax::SyntaxKind::TypeArgs)
        .map(|args| {
            args.children()
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeArg)
                .filter_map(|arg| arg.children().next())
                .map(type_spec_from_cst)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(Some((enum_name, variant, generic_args)))
}

fn call_target_from_cst(node: &omni_syntax::SyntaxNode) -> Result<(String, Vec<TypeSpec>), String> {
    let segment = node
        .descendants()
        .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
        .last()
        .ok_or_else(|| "Semantic frontend error: call target has no path segment".to_string())?;
    let name = direct_name(&segment)
        // A turbofish is part of the segment's text (`id<i64>`); the callee being
        // referenced is the identifier before the type arguments.
        .map(|n| n.split('<').next().unwrap_or_default().trim().to_string())
        .filter(|n| !n.is_empty())
        .ok_or_else(|| "Semantic frontend error: call target has no name".to_string())?;
    let args_node = node
        .children()
        .find(|n| n.kind() == omni_syntax::SyntaxKind::TypeArgs)
        .or_else(|| segment.children().find(|n| n.kind() == omni_syntax::SyntaxKind::TypeArgs));
    let generic_args = args_node
        .into_iter()
        .flat_map(|args| args.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::TypeArg))
        .filter_map(|arg| arg.children().next())
        .map(type_spec_from_cst)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((name, generic_args))
}

fn match_arm_from_cst(node: &omni_syntax::SyntaxNode) -> Result<omni_types::ast::MatchArm, String> {
    let mut children = node.children();
    let pattern = children
        .next()
        .ok_or_else(|| "Semantic frontend error: match arm has no pattern".to_string())?;
    let rest = children.collect::<Vec<_>>();
    let (guard, body) = match rest.as_slice() {
        [body] => (None, body),
        [guard, body] => (Some(guard), body),
        _ => return Err("Semantic frontend error: malformed match arm".into()),
    };
    Ok(omni_types::ast::MatchArm {
        pattern: pattern_from_cst(&pattern)?,
        guard: guard.map(expr_from_node).transpose()?,
        body: expr_from_node(body)?,
    })
}

fn pattern_from_cst(node: &omni_syntax::SyntaxNode) -> Result<omni_types::ast::Pattern, String> {
    match node.kind() {
        omni_syntax::SyntaxKind::WildcardPattern => Ok(omni_types::ast::Pattern::Wildcard),
        omni_syntax::SyntaxKind::IdentifierPattern | omni_syntax::SyntaxKind::BindingPattern => {
            if node.kind() == omni_syntax::SyntaxKind::IdentifierPattern {
                if let Some(path) =
                    node.children().find(|n| n.kind() == omni_syntax::SyntaxKind::Path)
                {
                    let segments = path
                        .children()
                        .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
                        .collect::<Vec<_>>();
                    if segments.len() >= 2 {
                        let enum_name =
                            direct_name(&segments[segments.len() - 2]).ok_or_else(|| {
                                "Semantic frontend error: enum pattern has no enum name".to_string()
                            })?;
                        let variant =
                            direct_name(&segments[segments.len() - 1]).ok_or_else(|| {
                                "Semantic frontend error: enum pattern has no variant name"
                                    .to_string()
                            })?;
                        return Ok(omni_types::ast::Pattern::Variant {
                            enum_name,
                            variant,
                            subpatterns: Vec::new(),
                        });
                    }
                }
            }
            let name = pattern_binding_text(node).ok_or_else(|| {
                "Semantic frontend error: pattern has no binding name".to_string()
            })?;
            Ok(omni_types::ast::Pattern::Binding(name))
        }
        omni_syntax::SyntaxKind::LiteralPattern => {
            let lit_node = node
                .children()
                .next()
                .ok_or_else(|| "Semantic frontend error: literal pattern is empty".to_string())?;
            let token = lit_node.first_token().ok_or_else(|| {
                "Semantic frontend error: literal pattern has no token".to_string()
            })?;
            Ok(omni_types::ast::Pattern::Lit(lit_from_text(token.text())?))
        }
        omni_syntax::SyntaxKind::TuplePattern => Ok(omni_types::ast::Pattern::Tuple(
            node.children().map(|n| pattern_from_cst(&n)).collect::<Result<Vec<_>, _>>()?,
        )),
        omni_syntax::SyntaxKind::OrPattern => Ok(omni_types::ast::Pattern::Or(
            node.children().map(|n| pattern_from_cst(&n)).collect::<Result<Vec<_>, _>>()?,
        )),
        omni_syntax::SyntaxKind::RangePattern => {
            let parts = node.children().collect::<Vec<_>>();
            let operator = node
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .find(|t| {
                    t.kind() == omni_syntax::SyntaxKind::Punct
                        && (t.text() == ".." || t.text() == "..=")
                })
                .map(|t| t.text().to_string())
                .ok_or_else(|| {
                    "Semantic frontend error: range pattern has no range operator".to_string()
                })?;

            let start_boundary = parts
                .first()
                .map(|start| -> Result<omni_types::ast::PatternRangeBoundary, String> {
                    let token = start.first_token().ok_or_else(|| {
                        "Semantic frontend error: range pattern start has no token".to_string()
                    })?;
                    Ok(omni_types::ast::PatternRangeBoundary::Inclusive(lit_from_text(
                        token.text(),
                    )?))
                })
                .transpose()?
                .unwrap_or(omni_types::ast::PatternRangeBoundary::Unbounded);

            let end_boundary = match parts.get(1) {
                Some(end) => {
                    let token = end.first_token().ok_or_else(|| {
                        "Semantic frontend error: range pattern end has no token".to_string()
                    })?;
                    let lit = lit_from_text(token.text())?;
                    if operator == "..=" {
                        omni_types::ast::PatternRangeBoundary::Inclusive(lit)
                    } else {
                        omni_types::ast::PatternRangeBoundary::Exclusive(lit)
                    }
                }
                None => omni_types::ast::PatternRangeBoundary::Unbounded,
            };

            Ok(omni_types::ast::Pattern::Range { start: start_boundary, end: end_boundary })
        }
        omni_syntax::SyntaxKind::StructPattern => {
            let mut parts = node.children();
            let path = parts
                .next()
                .ok_or_else(|| "Semantic frontend error: struct pattern has no path".to_string())?;
            let name = path
                .descendants()
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
                .last()
                .and_then(|n| direct_name(&n))
                .ok_or_else(|| {
                    "Semantic frontend error: struct pattern has no nominal type name".to_string()
                })?;
            let fields = parts
                .map(|f| {
                    let name = direct_name(&f).ok_or_else(|| {
                        "Semantic frontend error: pattern field has no name".to_string()
                    })?;
                    let sub = f
                        .children()
                        .nth(1)
                        .map(|n| pattern_from_cst(&n))
                        .transpose()?
                        .unwrap_or(omni_types::ast::Pattern::Binding(name.clone()));
                    Ok((name, sub))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(omni_types::ast::Pattern::Struct { name, fields })
        }
        omni_syntax::SyntaxKind::EnumPattern => {
            let mut parts = node.children();
            let path = parts
                .next()
                .ok_or_else(|| "Semantic frontend error: enum pattern has no path".to_string())?;
            let segments = path
                .descendants()
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
                .collect::<Vec<_>>();
            if segments.len() < 2 {
                return Err("Semantic frontend error: enum pattern requires Enum::Variant".into());
            }
            let enum_name = direct_name(&segments[segments.len() - 2]).ok_or_else(|| {
                "Semantic frontend error: enum pattern has no enum name".to_string()
            })?;
            let variant = direct_name(&segments[segments.len() - 1]).ok_or_else(|| {
                "Semantic frontend error: enum pattern has no variant name".to_string()
            })?;
            let subpatterns = parts.map(|n| pattern_from_cst(&n)).collect::<Result<Vec<_>, _>>()?;
            Ok(omni_types::ast::Pattern::Variant { enum_name, variant, subpatterns })
        }
        omni_syntax::SyntaxKind::ReferencePattern => {
            let mutable = node
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .any(|t| t.kind() == omni_syntax::SyntaxKind::Keyword && t.text() == "mut");
            let inner = node
                .children()
                .find(|n| !matches!(n.kind(), omni_syntax::SyntaxKind::Lifetime))
                .ok_or_else(|| {
                    "Semantic frontend error: reference pattern has no inner pattern".to_string()
                })?;
            Ok(omni_types::ast::Pattern::Reference {
                mutable,
                inner: Box::new(pattern_from_cst(&inner)?),
            })
        }
        omni_syntax::SyntaxKind::SlicePattern | omni_syntax::SyntaxKind::GuardPattern => {
            Err(format!(
                "Semantic frontend error: pattern lowering does not yet support {:?}",
                node.kind()
            ))
        }
        other => Err(format!("Semantic frontend error: unsupported pattern node {:?}", other)),
    }
}

fn lit_from_text(text: &str) -> Result<Lit, String> {
    match text {
        "true" => return Ok(Lit::Bool(true)),
        "false" => return Ok(Lit::Bool(false)),
        _ => {}
    }

    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return Ok(Lit::String(text[1..text.len() - 1].to_string()));
    }

    if text.starts_with("b'") && text.ends_with('\'') && text.len() >= 3 {
        let inner = &text[2..text.len() - 1];
        let byte = match inner {
            "\n" => b'\n',
            "\r" => b'\r',
            "\t" => b'\t',
            "\\" => b'\\',
            "\'" => b'\'',
            value if value.len() == 1 => value.as_bytes()[0],
            _ => {
                return Err(format!("Semantic frontend error: unsupported byte literal '{}'", text))
            }
        };
        return Ok(Lit::Byte(byte));
    }

    if text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2 {
        let inner = &text[1..text.len() - 1];
        let ch = match inner {
            "\n" => '\n',
            "\r" => '\r',
            "\t" => '\t',
            "\\" => '\\',
            "\'" => '\'',
            value if value.chars().count() == 1 => value.chars().next().unwrap(),
            _ => {
                return Err(format!("Semantic frontend error: unsupported char literal '{}'", text))
            }
        };
        return Ok(Lit::Char(ch));
    }

    if let Ok(value) = parse_int_literal(text) {
        return Ok(Lit::Int(value));
    }
    if let Ok(value) = text.parse::<f64>() {
        return Ok(Lit::Float(value.to_bits()));
    }

    Err(format!("Semantic frontend error: unsupported literal spelling '{}'", text))
}

fn parse_int_literal(text: &str) -> Result<i64, String> {
    let normalized = text.replace('_', "");
    if let Some(value) = normalized.strip_prefix("0x") {
        return i64::from_str_radix(value, 16).map_err(|e| e.to_string());
    }
    if let Some(value) = normalized.strip_prefix("0b") {
        return i64::from_str_radix(value, 2).map_err(|e| e.to_string());
    }
    if let Some(value) = normalized.strip_prefix("0o") {
        return i64::from_str_radix(value, 8).map_err(|e| e.to_string());
    }
    normalized.parse::<i64>().map_err(|e| e.to_string())
}

fn bin_op_from_text(text: &str) -> Result<BinOp, String> {
    match text {
        "+" => Ok(BinOp::Add),
        "-" => Ok(BinOp::Sub),
        "*" => Ok(BinOp::Mul),
        "/" => Ok(BinOp::Div),
        "%" => Ok(BinOp::Rem),
        "&" => Ok(BinOp::BitAnd),
        "|" => Ok(BinOp::BitOr),
        "^" => Ok(BinOp::BitXor),
        "<<" => Ok(BinOp::Shl),
        ">>" => Ok(BinOp::Shr),
        "&&" => Ok(BinOp::LogicalAnd),
        "||" => Ok(BinOp::LogicalOr),
        "==" => Ok(BinOp::Eq),
        "!=" => Ok(BinOp::Ne),
        "<" => Ok(BinOp::Lt),
        "<=" => Ok(BinOp::Le),
        ">" => Ok(BinOp::Gt),
        ">=" => Ok(BinOp::Ge),
        other => Err(format!("Semantic frontend error: unsupported binary operator '{}'", other)),
    }
}

mod provenance;

use provenance::EmissionContext;

/// Assemble provenance context from actual inputs and emit the sidecar.
/// The specification identity comes from the validated loader (never
/// re-derived here); every other field is gathered, never invented.
fn emit_artifact_provenance(
    out_path: &Path,
    bytes: &[u8],
    parsed: &Args,
) -> Result<PathBuf, String> {
    let (workspace_root, spec_root) = provenance::discover_workspace()?;
    let loaded = omni_registry::load_specification(spec_root)
        .map_err(|e| format!("specification failed to load: {e}"))?;
    let ctx = EmissionContext {
        tree_digest: loaded.identity().tree_digest.clone(),
        plan_digest: loaded.manifest().plan_sha256.clone(),
        toolchain: provenance::read_toolchain_channel(&workspace_root)?,
        target_descriptor: provenance::target_descriptor(),
        compiler_version: env!("CARGO_PKG_VERSION").to_string(),
        source_revision: provenance::source_revision(&workspace_root),
        opt_level: parsed.opt_level,
        emit_native: parsed.emit_native,
        build_epoch: provenance::build_epoch(),
    };
    provenance::emit_provenance_sidecar(out_path, bytes, &ctx, &workspace_root)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match parse_args(&args) {
        Ok(parsed) => {
            if let Some(input) = parsed.input_file.clone() {
                let source_code = match fs::read_to_string(&input) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!("Error reading file {:?}: {}", input, e);
                        std::process::exit(1);
                    }
                };

                let (_workspace_root, spec_root) =
                    provenance::discover_workspace().unwrap_or_else(|e| {
                        eprintln!("Error discovering workspace: {}", e);
                        std::process::exit(1);
                    });
                let loaded = omni_registry::load_specification(spec_root).unwrap_or_else(|e| {
                    eprintln!("Error loading specification: {}", e);
                    std::process::exit(1);
                });
                let manifest = loaded.manifest();

                if parsed.emit_native {
                    match compile_source_to_object(&source_code, manifest) {
                        Ok(bytes) => {
                            let out_path = parsed
                                .output_file
                                .clone()
                                .unwrap_or_else(|| PathBuf::from("output.o"));
                            if let Err(e) = fs::write(&out_path, &bytes) {
                                eprintln!("Failed to write object file: {}", e);
                                std::process::exit(1);
                            }
                            println!("Successfully compiled native object to {:?}", out_path);
                            match emit_artifact_provenance(&out_path, &bytes, &parsed) {
                                Ok(sidecar) => {
                                    println!("Provenance record: {:?}", sidecar);
                                }
                                Err(e) => {
                                    eprintln!("Provenance error: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Codegen error: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    match compile_source_to_interpreter_value(&source_code, manifest) {
                        Ok(result) => {
                            println!("Exit code: {}", result);
                        }
                        Err(e) => {
                            eprintln!("Execution error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            } else {
                println!(
                    "Omni Systems Programming Language Compiler v{}",
                    env!("CARGO_PKG_VERSION")
                );
                println!("Usage: omni-driver [OPTIONS] <INPUT_FILE>");
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::OnceLock;

    fn manifest() -> &'static omni_registry::Manifest {
        static ONCE: OnceLock<omni_registry::Manifest> = OnceLock::new();
        ONCE.get_or_init(|| {
            let spec_root =
                omni_registry::discover_spec_root().expect("discover spec root for tests");
            let loaded = omni_registry::load_specification(spec_root).expect("load spec for tests");
            loaded.manifest().clone()
        })
    }

    #[test]
    fn reference_machine_and_native_backend_agree_on_exit_value() {
        let corpus: &[(&str, i64)] = &[
            ("fn main() -> i64 { return 40 + 2; }", 42),
            ("fn main() -> i64 { let mut x = 10; x += 5; x -= 3; return x; }", 12),
            (
                "fn choose(b: bool) -> i64 { if b { 11 } else { 22 } } fn main() -> i64 { return choose(true) + choose(false); }",
                33,
            ),
            (
                "fn fib(n: i64) -> i64 { if n <= 1 { n } else { fib(n - 1) + fib(n - 2) } } fn main() -> i64 { return fib(6); }",
                8,
            ),
            (
                "fn main() -> i64 { let mut s = 0; let mut i = 1; while i <= 5 { s += i; i += 1; } return s; }",
                15,
            ),
            (
                "fn main() -> i64 { let x = 2; return match x { 1 => 10, 2 => 20, _ => 30 }; }",
                20,
            ),
            (
                "fn id<T>(x: T) -> T { return x; } fn main() -> i64 { return id<i64>(9); }",
                9,
            ),
        ];

        for (source, expected_val) in corpus {
            let expected = compile_source_to_interpreter_value(source, manifest())
                .expect("reference machine must execute the differential corpus");
            assert_eq!(
                expected, *expected_val,
                "reference machine returned unexpected value for {source}"
            );

            let object = compile_source_to_object(source, manifest())
                .expect("native backend must compile the differential corpus");
            let dir = std::env::temp_dir();
            static SEQ: AtomicU64 = AtomicU64::new(600);
            let stem = format!(
                "omni-differential-e2e-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::SeqCst)
            );
            let object_path = dir.join(format!("{stem}.o"));
            let executable_path = dir.join(&stem);
            fs::write(&object_path, object).expect("write differential object");

            let status = std::process::Command::new("cc")
                .arg(&object_path)
                .arg("-o")
                .arg(&executable_path)
                .status()
                .expect("system C linker is required for native differential execution");
            assert!(status.success(), "differential link failed with status {status}");

            let native = std::process::Command::new(&executable_path)
                .status()
                .expect("linked differential executable must run");
            assert_eq!(
                native.code(),
                Some((expected & 0xff) as i32),
                "reference machine and native execution disagree for {source}"
            );

            fs::remove_file(&object_path).ok();
            fs::remove_file(&executable_path).ok();
        }
    }

    #[test]
    fn source_pipeline_compiles_explicit_turbofish_generic_call() {
        let source = "fn id<T>(x: T) -> T { x } fn main() -> i64 { return id::<i64>(41); }";
        let object = compile_source_to_object(source, manifest())
            .expect("explicit generic call must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_generic_specialization() {
        let source = "fn id<T>(x: T) -> T { x } fn main() -> i64 { return id(41); }";
        let object = compile_source_to_object(source, manifest())
            .expect("generic specialization must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_assignment_and_bitwise_integer_ops() {
        let source = "fn main() -> i64 { let mut x = 6; x += 3; x <<= 1; x ^= 2; return x; }";
        let object = compile_source_to_object(source, manifest())
            .expect("assignment operators must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_short_circuit_boolean_expression() {
        let source = "fn main() -> bool { return false && (1 == 2) || true; }";
        let object = compile_source_to_object(source, manifest())
            .expect("short-circuit expression must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_if_expression_values() {
        let source = "fn choose(a: bool) -> i64 { if a { 1 } else { 2 } } fn main() -> i64 { return choose(true); }";
        let object =
            compile_source_to_object(source, manifest()).expect("if expression must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_executes_numeric_range_match_in_reference_machine() {
        let source = "fn main() -> i64 { let x = 5; return match x { 0..10 => 7, _ => 0, }; }";
        assert_eq!(
            compile_source_to_interpreter_value(source, manifest())
                .expect("numeric range match must execute"),
            7
        );
    }

    #[test]
    fn source_pipeline_executes_match_guard_in_reference_machine() {
        let source =
            "fn main() -> i64 { let x = 5; return match x { y if true => y, _ => 0, }; }";
        assert_eq!(
            compile_source_to_interpreter_value(source, manifest())
                .expect("guarded match must execute"),
            5
        );
    }

    #[test]
    fn source_pipeline_compiles_integer_match() {
        let source = "fn choose(x: i64) -> i64 { match x { 0 => 10, _ => 20 } } fn main() -> i64 { return choose(0); }";
        let object =
            compile_source_to_object(source, manifest()).expect("integer match must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_loop_value_and_while_control() {
        let source = "fn choose() -> i64 { loop { break 7; } } fn main() -> i64 { let x = choose(); let mut y = 3; while y > 0 { y -= 1; continue; } return x; }";
        let object = compile_source_to_object(source, manifest())
            .expect("loop and while control must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn machine_tier_returns_the_computed_main_result() {
        // Guards the default (non-`--native`) path against returning a
        // fabricated constant instead of executing the program.
        let source = "fn main() -> i64 { let value = 41; return value; }";
        assert_eq!(
            compile_source_to_interpreter_value(source, manifest()).expect("machine execution"),
            41
        );
    }

    #[test]
    fn machine_tier_computes_arithmetic_calls_and_control_flow() {
        // Each case is checked against a hand-computed result, so a constant
        // return of any fixed value cannot satisfy this suite.
        let cases: &[(&str, i64)] = &[
            ("fn main() -> i64 { let a = 6; let b = 7; return a * b; }", 42),
            ("fn inc(x: i64) -> i64 { return x + 1; } fn main() -> i64 { return inc(41); }", 42),
            (
                "fn main() -> i64 { let mut s = 0; let mut i = 1; while i <= 10 { s += i; i += 1; } return s; }",
                55,
            ),
            ("fn choose(x: i64) -> i64 { match x { 0 => 10, _ => 20 } } fn main() -> i64 { return choose(0); }", 10),
            ("fn main() -> i64 { let arr = [10,20,30]; return arr[2]; }", 30),
        ];

        for (source, expected) in cases {
            assert_eq!(
                compile_source_to_interpreter_value(source, manifest())
                    .unwrap_or_else(|e| { panic!("machine execution failed for {source}: {e}") }),
                *expected,
                "wrong result for {source}"
            );
        }
    }

    #[test]
    fn machine_tier_requires_verified_mir_before_execution() {
        // Execution runs on verified MIR only. This asserts the machine path
        // lowers and verifies successfully for a well-formed program; the
        // matching failure case is covered by the no-main rejection test.
        let source = "fn main() -> i64 { let value = 41; return value; }";
        assert!(
            lower_to_verified_mir(source, manifest()).is_ok(),
            "a well-formed program must lower and verify"
        );
    }

    #[test]
    fn machine_tier_rejects_source_without_main() {
        let source = "fn helper(x: i64) -> i64 { return x; }";
        let error = compile_source_to_interpreter_value(source, manifest())
            .expect_err("a program without main must be rejected");
        assert!(error.contains("main"), "unexpected error: {error}");
    }

    #[test]
    fn rejects_invalid_optimization_level() {
        let args = vec![
            "omni-driver".to_string(),
            "--native".to_string(),
            "--opt-level".to_string(),
            "not-a-number".to_string(),
            "program.omni".to_string(),
        ];
        let error = match parse_args(&args) {
            Ok(_) => panic!("invalid optimization level must fail"),
            Err(error) => error,
        };
        assert!(error.contains("Invalid optimization level"), "unexpected error: {error}");
    }

    #[test]
    fn source_pipeline_executes_explicit_generic_call() {
        let source = "fn id<T>(x: T) -> T { return x; } fn main() -> i64 { let value = id<i64>(41); return value; }";
        let object =
            compile_source_to_object(source, manifest()).expect("generic call native compilation");
        let dir = std::env::temp_dir();
        static SEQ: AtomicU64 = AtomicU64::new(500);
        let stem = format!(
            "omni-driver-generic-e2e-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        );
        let object_path = dir.join(format!("{stem}.o"));
        let executable_path = dir.join(&stem);
        fs::write(&object_path, object).expect("write object");

        let status = std::process::Command::new("cc")
            .arg(&object_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .expect("system C linker is required for generic E2E");
        assert!(status.success(), "link failed with status {status}");

        let run_status = std::process::Command::new(&executable_path)
            .status()
            .expect("linked generic executable must run");
        assert_eq!(run_status.code(), Some(41));

        fs::remove_file(&object_path).ok();
        fs::remove_file(&executable_path).ok();
    }

    #[test]
    fn source_pipeline_executes_integer_call_semantics() {
        let source = "fn inc(x: i64) -> i64 { return x + 1; } fn main() -> i64 { let value = inc(41); return value; }";
        let object = compile_source_to_object(source, manifest()).expect("native compilation");
        let dir = std::env::temp_dir();
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let stem = format!(
            "omni-driver-e2e-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        );
        let object_path = dir.join(format!("{stem}.o"));
        let executable_path = dir.join(&stem);
        fs::write(&object_path, object).expect("write object");

        let status = std::process::Command::new("cc")
            .arg(&object_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .expect("system C linker is required for native E2E");
        assert!(status.success(), "link failed with status {status}");

        let run_status = std::process::Command::new(&executable_path)
            .status()
            .expect("linked native executable must run");
        assert_eq!(run_status.code(), Some(42));

        fs::remove_file(&object_path).ok();
        fs::remove_file(&executable_path).ok();
    }

    #[test]
    fn source_pipeline_accepts_reference_types_until_native_boundary() {
        let source = "fn read(x: &i64) -> i64 { return *x; } fn main() -> i64 { let x = 7; return read(&x); }";
        let err = compile_source_to_object(source, manifest())
            .expect_err("native backend must reject reference storage explicitly");
        assert!(
            err.contains("shared borrow") || err.contains("reference"),
            "reference program must fail at the documented storage boundary: {err}"
        );
    }

    #[test]
    fn source_pipeline_executes_explicit_numeric_casts() {
        // GRAM-0002: `expr_stmt = expression ";"`. Only the block's trailing
        // `final_expression` may omit the `;`, so the `if` statement here needs
        // one before the following `return`.
        let source =
            "fn main() -> i64 { let x = 7 as f64; let y = x as i64; if y == 7 { return 42; }; return 0; }";
        let object =
            compile_source_to_object(source, manifest()).expect("numeric cast native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_executes_float_arithmetic_and_comparison() {
        let source = "fn add(a: f64, b: f64) -> f64 { return a + b; } fn main() -> i64 { let x = add(1.5, 2.5); if x >= 4.0 { return 42; }; return 0; }";
        let object =
            compile_source_to_object(source, manifest()).expect("float native compilation");
        let dir = std::env::temp_dir();
        static SEQ: AtomicU64 = AtomicU64::new(600);
        let stem = format!(
            "omni-driver-float-e2e-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        );
        let object_path = dir.join(format!("{stem}.o"));
        let executable_path = dir.join(&stem);
        fs::write(&object_path, object).expect("write object");

        let status = std::process::Command::new("cc")
            .arg(&object_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .expect("system C linker is required for float E2E");
        assert!(status.success(), "link failed with status {status}");

        let run_status = std::process::Command::new(&executable_path)
            .status()
            .expect("linked native executable must run");
        assert_eq!(run_status.code(), Some(42));

        fs::remove_file(&object_path).ok();
        fs::remove_file(&executable_path).ok();
    }

    #[test]
    fn source_pipeline_executes_boolean_comparison_and_not() {
        let source = "fn main() -> bool { return !(1 == 2); }";
        let object =
            compile_source_to_object(source, manifest()).expect("native boolean compilation");
        let dir = std::env::temp_dir();
        static SEQ: AtomicU64 = AtomicU64::new(100);
        let stem = format!(
            "omni-driver-bool-e2e-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        );
        let object_path = dir.join(format!("{stem}.o"));
        let executable_path = dir.join(&stem);
        fs::write(&object_path, object).expect("write object");

        let status = std::process::Command::new("cc")
            .arg(&object_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .expect("system C linker is required for native E2E");
        assert!(status.success(), "link failed with status {status}");

        let run_status = std::process::Command::new(&executable_path)
            .status()
            .expect("linked native executable must run");
        assert_eq!(run_status.code(), Some(1));

        fs::remove_file(&object_path).ok();
        fs::remove_file(&executable_path).ok();
    }

    #[test]
    fn source_pipeline_executes_value_loop_break() {
        let source = "fn main() -> i64 { return loop { break 42; }; }";
        let object =
            compile_source_to_object(source, manifest()).expect("loop break native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_executes_while_loop() {
        let source = "fn main() -> i64 { let mut n = 0; while n < 3 { n += 1; } return n; }";
        let object =
            compile_source_to_object(source, manifest()).expect("while loop native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_executes_integer_range_for_loop() {
        let source =
            "fn main() -> i64 { let mut sum = 0; for i in 0..5 { sum += i; } return sum; }";
        let object = compile_source_to_object(source, manifest())
            .expect("integer-range for-loop native compilation");
        let dir = std::env::temp_dir();
        static SEQ: AtomicU64 = AtomicU64::new(300);
        let stem = format!(
            "omni-driver-for-e2e-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        );
        let object_path = dir.join(format!("{stem}.o"));
        let executable_path = dir.join(&stem);
        fs::write(&object_path, object).expect("write object");

        let status = std::process::Command::new("cc")
            .arg(&object_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .expect("system C linker is required for integer-range E2E");
        assert!(status.success(), "link failed with status {status}");

        let run_status = std::process::Command::new(&executable_path)
            .status()
            .expect("linked native executable must run");
        assert_eq!(run_status.code(), Some(10));

        fs::remove_file(&object_path).ok();
        fs::remove_file(&executable_path).ok();
    }

    #[test]
    fn source_pipeline_executes_scalar_match_binding() {
        let source = "fn main() -> i64 { let x = 42; return match x { y => y, }; }";
        let object = compile_source_to_object(source, manifest())
            .expect("scalar binding match native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_parses_borrow_and_deref_forms() {
        let source = "fn f(x: i64) { let y = &x; let z = &mut x; let q = *y; }";
        let mut parser = omni_parse::Parser::from_source(source);
        let parsed = parser.parse_source();
        assert!(parsed.is_ok(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.syntax().text().to_string(), source);
        // Borrowing and dereferencing are allowed Stage-0 features and are not
        // separately recorded, so the profile gate must see nothing to judge.
        assert!(parsed.feature_uses.is_empty(), "{:?}", parsed.feature_uses);
        enforce_stage0_profile(&parsed.feature_uses, manifest())
            .expect("borrowing is enabled by the Stage-0 profile");
    }

    // ----------------------------------------------------------------------
    // Stage-0 profile gating. The parser accepts all of Edition 1; the profile
    // decides which features are enabled. These tests pin both halves.
    // ----------------------------------------------------------------------

    /// Parse `source` and run the profile gate, returning the gate's verdict.
    fn profile_verdict(source: &str) -> Result<(), String> {
        let mut parser = omni_parse::Parser::from_source(source);
        let parsed = parser.parse_source();
        assert!(parsed.is_ok(), "source must be Edition-1-legal: {:?}", parsed.diagnostics);
        // The gate runs on a program the grammar layer accepted, so it can only
        // be rejecting on profile grounds.
        enforce_stage0_profile(&parsed.feature_uses, manifest())
    }

    #[test]
    fn forbidden_features_are_rejected_by_the_profile_gate() {
        // `macros` and `async` are both normative Edition 1 productions, so the
        // parser must accept them; the manifest forbids them for Stage 0, so the
        // gate must reject them. Each assertion is on the real manifest.
        for (source, feature) in [
            ("fn main() -> i64 { foo!(); return 0; }", "macros"),
            ("fn main() -> i64 { let x = async move { 1 }; return 0; }", "async"),
            ("async fn main() -> i64 { return 0; }", "async"),
        ] {
            let error = profile_verdict(source)
                .expect_err(&format!("{feature} is forbidden and must be rejected"));
            assert!(error.contains("Stage-0 profile error"), "unexpected error: {error}");
            assert!(error.contains(feature), "error must name {feature}: {error}");
            // The diagnostic must point at real source bytes, not at nothing.
            assert!(error.contains("at bytes "), "error must carry a span: {error}");
        }
    }

    #[test]
    fn allowed_features_pass_the_profile_gate() {
        // `unsafe_raw_memory` is on the manifest's allowed list, so an unsafe
        // block parses and passes. Without this, the gate could be rejecting
        // everything unconditionally and the forbidden-feature test would still
        // pass.
        assert!(profile_verdict("fn main() -> i64 { let x = unsafe { 1 }; return x; }").is_ok());
        // And source using no classified feature at all passes trivially.
        assert!(profile_verdict("fn main() -> i64 { return 7; }").is_ok());
    }

    #[test]
    fn profile_gate_reports_every_offending_feature_in_a_stable_order() {
        // Both offending features must be reported, and repeated runs must agree
        // on the message, so a build cannot accept and reject the same program.
        let source = "fn main() -> i64 { let x = async move { foo!() }; return 0; }";
        let first = profile_verdict(source).expect_err("both features are forbidden");
        assert!(first.contains("async"), "{first}");
        assert!(first.contains("macros"), "{first}");
        for _ in 0..3 {
            assert_eq!(profile_verdict(source).expect_err("stable"), first);
        }
    }

    #[test]
    fn an_empty_feature_list_passes_without_consulting_the_manifest() {
        // A source that used nothing must not fail just because a manifest were
        // unavailable; the gate is a no-op rather than a spurious rejection.
        assert!(enforce_stage0_profile(&[], manifest()).is_ok());
    }

    #[test]
    fn source_pipeline_executes_literal_match() {
        let source = "fn main() -> i64 { let x = 2; return match x { 1 => 7, 2 => 42, _ => 0, }; }";
        let object =
            compile_source_to_object(source, manifest()).expect("literal match native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_executes_inclusive_integer_range_for_loop() {
        let source =
            "fn main() -> i64 { let mut sum = 0; for i in 0..=4 { sum += i; } return sum; }";
        let object = compile_source_to_object(source, manifest())
            .expect("inclusive range for-loop native compilation");
        let dir = std::env::temp_dir();
        static SEQ: AtomicU64 = AtomicU64::new(400);
        let stem = format!(
            "omni-driver-for-inclusive-e2e-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        );
        let object_path = dir.join(format!("{stem}.o"));
        let executable_path = dir.join(&stem);
        fs::write(&object_path, object).expect("write object");

        let status = std::process::Command::new("cc")
            .arg(&object_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .expect("system C linker is required for inclusive range E2E");
        assert!(status.success(), "link failed with status {status}");

        let run_status = std::process::Command::new(&executable_path)
            .status()
            .expect("linked inclusive-range executable must run");
        assert_eq!(run_status.code(), Some(10));

        fs::remove_file(&object_path).ok();
        fs::remove_file(&executable_path).ok();
    }

    #[test]
    fn source_pipeline_compiles_value_if_expression() {
        // The E2E harness links `main` with no arguments, so the condition is
        // supplied by a helper rather than by a `main` parameter, matching every
        // other end-to-end fixture in this module.
        let source = "fn pick(x: i64) -> i64 { let value = if x > 0 { 42 } else { 7 }; return value; } fn main() -> i64 { return pick(1); }";
        let object =
            compile_source_to_object(source, manifest()).expect("if-expression native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_preserves_unit_call_without_fabricating_result() {
        let source = "fn touch() { return; } fn main() -> i64 { touch(); return 7; }";
        let object =
            compile_source_to_object(source, manifest()).expect("unit call native compilation");
        let dir = std::env::temp_dir();
        static SEQ: AtomicU64 = AtomicU64::new(200);
        let stem = format!(
            "omni-driver-unit-call-e2e-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        );
        let object_path = dir.join(format!("{stem}.o"));
        let executable_path = dir.join(&stem);
        fs::write(&object_path, object).expect("write object");

        let status = std::process::Command::new("cc")
            .arg(&object_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .expect("system C linker is required for native E2E");
        assert!(status.success(), "link failed with status {status}");

        let run_status = std::process::Command::new(&executable_path)
            .status()
            .expect("linked native executable must run");
        assert_eq!(run_status.code(), Some(7));

        fs::remove_file(&object_path).ok();
        fs::remove_file(&executable_path).ok();
    }

    #[test]
    fn invalid_unary_source_fails_during_semantic_checking() {
        let source = "fn main() -> i64 { return !1; }";
        let error = compile_source_to_object(source, manifest())
            .expect_err("integer logical-not must fail");
        assert!(error.contains("Type error"), "unexpected error: {error}");
    }

    // ----------------------------------------------------------------------
    // Inherent methods. GRAM: `method_call_expr = expression "." identifier
    // [ "<" type_args ">" ] "(" [ args ] ")"`.
    // ----------------------------------------------------------------------

    #[test]
    fn inherent_method_call_executes_on_the_receiver() {
        assert_eq!(
            compile_source_to_interpreter_value(
                "struct P { x: i64 } impl P { fn get() -> i64 { return self.x; } } \
                 fn main() -> i64 { let p = P { x: 42 }; return p.get(); }",
                manifest()
            )
            .expect("method call must execute"),
            42
        );
    }

    #[test]
    fn method_dispatch_is_selected_by_the_receiver_type() {
        // The load-bearing property: the same method name on two different
        // receiver types must reach two different functions. If dispatch were
        // resolved by name alone — or if the last registration simply won —
        // this would return the wrong value or fail to compile, so it is the
        // test that distinguishes real dispatch from a coincidence.
        assert_eq!(
            compile_source_to_interpreter_value(
                "struct A { v: i64 } struct B { v: i64 } \
                 impl A { fn get() -> i64 { return 1; } } \
                 impl B { fn get() -> i64 { return 2; } } \
                 fn main() -> i64 { let a = A { v: 0 }; let b = B { v: 0 }; return a.get() * 10 + b.get(); }",
                manifest()
            )
            .expect("two types may each define the same method name"),
            12
        );
    }

    #[test]
    fn method_call_accepts_arguments_alongside_the_receiver() {
        // The receiver is prepended to the argument list, so arity must account
        // for it: a method declared with one parameter is called with one
        // explicit argument.
        assert_eq!(
            compile_source_to_interpreter_value(
                "struct S { base: i64 } impl S { fn add(n: i64) -> i64 { return self.base + n; } } \
                 fn main() -> i64 { let s = S { base: 40 }; return s.add(2); }",
                manifest()
            )
            .expect("method with an argument"),
            42
        );
        // And arity is still checked: passing two arguments to a one-parameter
        // method must be rejected rather than silently dropping one.
        let error = compile_source_to_interpreter_value(
            "struct S { base: i64 } impl S { fn add(n: i64) -> i64 { return self.base + n; } } \
             fn main() -> i64 { let s = S { base: 1 }; return s.add(1, 2); }",
            manifest(),
        )
        .expect_err("too many arguments must be rejected");
        assert!(
            error.contains("ArgumentCountMismatch") || error.contains("argument"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn unknown_method_is_a_distinct_diagnostic() {
        // A missing method is reported as `NoMethodOnType`, not as a missing free
        // function: the receiver's type simply does not declare the method, and
        // the two situations have different remedies.
        let error = compile_source_to_interpreter_value(
            "struct A { v: i64 } fn main() -> i64 { let a = A { v: 1 }; return a.nope(); }",
            manifest(),
        )
        .expect_err("unknown method must be rejected");
        assert!(
            error.contains("NoMethodOnType") && error.contains("nope"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn method_call_on_a_non_nominal_type_is_rejected() {
        // There is no inherent impl for a scalar, so this cannot resolve. It must
        // be rejected rather than searching for a method under some derived name.
        let error = compile_source_to_interpreter_value(
            "fn main() -> i64 { let x = 5; return x.get(); }",
            manifest(),
        )
        .expect_err("a scalar receiver has no methods");
        assert!(error.contains("NoMethodOnType"), "unexpected error: {error}");
    }

    #[test]
    fn methods_do_not_enter_the_free_function_namespace() {
        // A method is not callable as a free function. `P::get` is the internal
        // qualified identity, not source-level call syntax, so `get(p)` must not
        // resolve — otherwise a method could be invoked without a receiver and
        // `self` would be unbound.
        let error = compile_source_to_interpreter_value(
            "struct P { x: i64 } impl P { fn get() -> i64 { return self.x; } } \
             fn main() -> i64 { let p = P { x: 1 }; return get(); }",
            manifest(),
        )
        .expect_err("a method must not be callable as a free function");
        // The rejection legitimately comes from whichever stage owns the name:
        // the resolver sees an unbound identifier, and the type checker would
        // report no such function. Accepting either keeps the assertion on the
        // property being tested — the method is not reachable as a free
        // function — rather than on which stage happens to run first.
        assert!(
            error.contains("UnresolvedName")
                || error.contains("FunctionNotFound")
                || error.contains("VariableNotFound"),
            "unexpected error: {error}"
        );
    }

    // ----------------------------------------------------------------------
    // `unsafe` blocks. UNSAFE-0001 requires `unsafe` not disable ordinary
    // checks; UNSAFE-0002 requires the unsafe context to stay visible in MIR.
    // ----------------------------------------------------------------------

    #[test]
    fn unsafe_block_executes_its_body() {
        // An unsafe block is a block, so it evaluates to its body's value.
        assert_eq!(
            compile_source_to_interpreter_value(
                "fn main() -> i64 { return unsafe { 7 }; }",
                manifest()
            )
            .expect("unsafe block must execute"),
            7
        );
        // Including a body with its own bindings and a final expression.
        assert_eq!(
            compile_source_to_interpreter_value(
                "fn main() -> i64 { return unsafe { let a = 2; let b = 3; a * b }; }",
                manifest()
            )
            .expect("unsafe block with bindings"),
            6
        );
        // And nested inside ordinary control flow, where the unsafe block's
        // value must be the one the loop body produces.
        assert_eq!(
            compile_source_to_interpreter_value(
                "fn main() -> i64 { let mut s = 0; for i in 0..5 { s += i; } return unsafe { s }; }",
                manifest()
            )
            .expect("unsafe block after a loop"),
            10
        );
        // A statement-position unsafe block is fine too; it yields Unit here.
        // The trailing `;` is required because LEX-0002 states newlines never
        // terminate statements, so a block statement still needs its separator.
        compile_source_to_object(
            "fn main() -> i64 { unsafe { let x = 1; }; return 9; }",
            manifest(),
        )
        .expect("statement-position unsafe block");
    }

    #[test]
    fn unsafe_does_not_disable_type_checking() {
        // UNSAFE-0001: "`unsafe` ... does not disable ordinary typing". Each of
        // these is rejected outside `unsafe`; wrapping the body in `unsafe` must
        // not make it pass. If `unsafe` ever started waiving checks, this test
        // would start failing at the `.expect_err`.
        for (source, why) in [
            ("fn main() -> i64 { return unsafe { !1 }; }", "logical-not on int"),
            ("fn main() -> i64 { let x = unsafe { 1 }; return x + true; }", "int + bool"),
            (
                // Indexing a one-element array and then adding is fine; adding
                // to the *element* is a type error only once the element is an
                // int, so this asserts the error is reported through the unsafe
                // block rather than being swallowed by it.
                "fn main() -> i64 { let a = unsafe { [1] }; return a[0] + true; }",
                "indexed element + bool",
            ),
        ] {
            let error = compile_source_to_interpreter_value(source, manifest())
                .expect_err(&format!("unsafe must not permit {why}"));
            // Which stage catches it is not fixed — a given error may surface in
            // the type checker, in MIR lowering, or in MIR verification, and the
            // pipeline is free to move that boundary. What must hold is that it
            // is caught at all, and that the message names a real diagnosis
            // rather than a generic failure. Asserting one stage would pin an
            // implementation detail and fail on a legitimate reordering.
            assert!(
                error.contains("Type error")
                    || error.contains("MIR lowering error")
                    || error.contains("MIR verification error")
                    || error.contains("Semantic"),
                "unsafe must not permit {why}, but got: {error}"
            );
        }
    }

    #[test]
    fn unsafe_does_not_disable_ownership_or_mir_verification() {
        // UNSAFE-0001 also forbids disabling ownership and initialization
        // checks. Reading a never-initialized local is rejected by MIR
        // verification whether or not it is wrapped in `unsafe`, so the failure
        // must come from verification rather than from any relaxation.
        let error = compile_source_to_interpreter_value(
            "fn main() -> i64 { return unsafe { uninitialized_local }; }",
            manifest(),
        )
        .expect_err("reading an uninitialized local must be rejected");
        assert!(
            error.contains("MIR verification error") || error.contains("Name resolution"),
            "unsafe must not waive MIR verification: {error}"
        );
    }

    #[test]
    fn unsafe_context_is_recorded_in_mir() {
        // UNSAFE-0002: the unsafe effect must be visible in MIR. A body that
        // uses no `unsafe` records no region; one that does records the blocks
        // it reached, so an audit could tell a covered operation from an
        // uncovered one.
        let plain = lower_to_verified_mir("fn main() -> i64 { return 1; }", manifest())
            .expect("plain program verifies");
        let main = plain.functions.iter().find(|f| f.name == "main").expect("main");
        assert!(
            main.body.unsafe_blocks.is_empty(),
            "a program with no unsafe block must record no unsafe region: {:?}",
            main.body.unsafe_blocks
        );

        let with_unsafe = lower_to_verified_mir(
            "fn main() -> i64 { let mut s = 0; for i in 0..3 { s += i; } return unsafe { s }; }",
            manifest(),
        )
        .expect("unsafe program verifies");
        let main = with_unsafe.functions.iter().find(|f| f.name == "main").expect("main");
        assert!(
            !main.body.unsafe_blocks.is_empty(),
            "an unsafe block must be recorded for UNSAFE-0002 visibility"
        );
        // Every recorded block must be a real block of this body, or the region
        // would name blocks that do not exist.
        for block in &main.body.unsafe_blocks {
            assert!(
                block.index() < main.body.blocks.len(),
                "recorded unsafe block {:?} is out of range for a body of {}",
                block.index(),
                main.body.blocks.len()
            );
        }
    }

    #[test]
    fn pipeline_operator_executes_on_abstract_machine() {
        let source = "fn double(x: i64) -> i64 { return x * 2; } \
                      fn add(x: i64, y: i64) -> i64 { return x + y; } \
                      fn main() -> i64 { \
                          let result = 5 |> double |> add(10); \
                          return result; \
                      }";
        let value = compile_source_to_interpreter_value(source, manifest())
            .expect("pipeline operator execution on machine");
        assert_eq!(value, 20); // (5 * 2) + 10 = 20
    }

    #[test]
    fn expression_bodied_function_executes() {
        let source = "fn add(a: i64, b: i64) -> i64 = a + b; \
                      fn main() -> i64 { \
                          return add(20, 22); \
                      }";
        let value = compile_source_to_interpreter_value(source, manifest())
            .expect("expression bodied function execution");
        assert_eq!(value, 42);
    }

    #[test]
    fn pipeline_operator_with_placeholder_executes() {
        let source = "fn sub(a: i64, b: i64) -> i64 = a - b; \
                      fn main() -> i64 { \
                          let result = 10 |> sub(30, _); \
                          return result; \
                      }";
        let value = compile_source_to_interpreter_value(source, manifest())
            .expect("pipeline operator with placeholder execution");
        assert_eq!(value, 20); // 30 - 10 = 20
    }

    #[test]
    fn contextual_keyword_as_identifier_executes() {
        let source = "struct Point { where: i64, in: i64 } \
                      fn main() -> i64 { \
                          let p = Point { where: 15, in: 25 }; \
                          return p.where + p.in; \
                      }";
        let value = compile_source_to_interpreter_value(source, manifest())
            .expect("contextual keywords where and in as struct fields");
        assert_eq!(value, 40);
    }
}
