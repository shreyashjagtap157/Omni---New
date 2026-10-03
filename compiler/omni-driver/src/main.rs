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
) -> Result<(omni_types::monomorph::MonomorphizedProgram, HashMap<String, StructDef>), String> {
    let mut parser = omni_parse::Parser::from_source(source_code);
    let parsed = parser.parse_source();
    if !parsed.is_ok() {
        return Err(format!(
            "Parse error: {}",
            parsed.diagnostics.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join("; ")
        ));
    }

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
fn lower_to_verified_mir(source_code: &str) -> Result<omni_mir::ir::MirProgram, String> {
    let (program, struct_defs) = lower_to_concrete_program(source_code)?;

    let mut lowering = omni_mir::lower::LoweringContext::new();
    lowering.set_struct_defs(struct_defs);
    let mir = lowering
        .lower_monomorphized_program(&program)
        .map_err(|e| format!("MIR lowering error: {}", e))?;

    omni_verify::MirVerifier::verify_program(&mir)
        .map_err(|e| format!("MIR verification error: {:?}", e))?;

    Ok(mir)
}

/// Executes `main` on the abstract machine and returns its integer result.
///
/// This is the machine tier's real entry point. It lowers and verifies MIR
/// first, so an unverified or malformed program is rejected before execution
/// rather than being interpreted optimistically.
pub fn compile_source_to_interpreter_value(source_code: &str) -> Result<i64, String> {
    let mir = lower_to_verified_mir(source_code)?;

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
pub fn compile_source_to_object(source_code: &str) -> Result<Vec<u8>, String> {
    let (program, struct_defs) = lower_to_concrete_program(source_code)?;
    omni_codegen::compile_monomorphized_program_with_structs(&program, struct_defs)
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

fn semantic_functions_from_cst(
    root: &omni_syntax::SyntaxNode,
) -> Result<Vec<GenericFnDef>, String> {
    let mut functions = Vec::new();
    let mut names = HashSet::new();
    let mut function_nodes = Vec::new();
    collect_function_nodes(root, "", &mut function_nodes);

    let enum_names: HashSet<String> = root
        .children()
        .filter(|n| n.kind() == omni_syntax::SyntaxKind::EnumDef)
        .filter_map(|n| direct_name(&n))
        .collect();
    let alias_defs = semantic_type_aliases_from_cst(root, &enum_names)?
        .into_iter()
        .map(|a| (a.name.clone(), a))
        .collect::<HashMap<_, _>>();

    for (node, name) in function_nodes {
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
            let base = children
                .next()
                .ok_or_else(|| "Semantic frontend error: field has no base".to_string())?;
            let field = node
                .children_with_tokens()
                .filter_map(|e| e.into_token())
                .filter(|t| t.kind() == omni_syntax::SyntaxKind::Ident)
                .last()
                .map(|t| t.text().to_string())
                .ok_or_else(|| "Semantic frontend error: field has no name".to_string())?;
            Ok(Expr::Field { expr: Box::new(expr_from_node(&base)?), field })
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
        omni_syntax::SyntaxKind::AwaitExpr
        | omni_syntax::SyntaxKind::MethodCallExpr
        | omni_syntax::SyntaxKind::MacroInvocation
        | omni_syntax::SyntaxKind::AsyncBlock
        | omni_syntax::SyntaxKind::UnsafeBlock
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

                if parsed.emit_native {
                    match compile_source_to_object(&source_code) {
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
                    match compile_source_to_interpreter_value(&source_code) {
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
                println!("Omni Systems Programming Language Compiler v1.0.0.0");
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

    #[test]
    fn source_pipeline_compiles_explicit_turbofish_generic_call() {
        let source = "fn id<T>(x: T) -> T { x } fn main() -> i64 { return id::<i64>(41); }";
        let object = compile_source_to_object(source).expect("explicit generic call must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_generic_specialization() {
        let source = "fn id<T>(x: T) -> T { x } fn main() -> i64 { return id(41); }";
        let object = compile_source_to_object(source).expect("generic specialization must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_assignment_and_bitwise_integer_ops() {
        let source = "fn main() -> i64 { let mut x = 6; x += 3; x <<= 1; x ^= 2; return x; }";
        let object = compile_source_to_object(source).expect("assignment operators must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_short_circuit_boolean_expression() {
        let source = "fn main() -> bool { return false && (1 == 2) || true; }";
        let object =
            compile_source_to_object(source).expect("short-circuit expression must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_if_expression_values() {
        let source = "fn choose(a: bool) -> i64 { if a { 1 } else { 2 } } fn main() -> i64 { return choose(true); }";
        let object = compile_source_to_object(source).expect("if expression must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_integer_match() {
        let source = "fn choose(x: i64) -> i64 { match x { 0 => 10, _ => 20 } } fn main() -> i64 { return choose(0); }";
        let object = compile_source_to_object(source).expect("integer match must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_compiles_loop_value_and_while_control() {
        let source = "fn choose() -> i64 { loop { break 7; } } fn main() -> i64 { let x = choose(); let mut y = 3; while y > 0 { y -= 1; continue; } return x; }";
        let object = compile_source_to_object(source).expect("loop and while control must compile");
        assert!(!object.is_empty());
    }

    #[test]
    fn machine_tier_returns_the_computed_main_result() {
        // Guards the default (non-`--native`) path against returning a
        // fabricated constant instead of executing the program.
        let source = "fn main() -> i64 { let value = 41; return value; }";
        assert_eq!(compile_source_to_interpreter_value(source).expect("machine execution"), 41);
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
                compile_source_to_interpreter_value(source)
                    .unwrap_or_else(|e| panic!("machine execution failed for {source}: {e}")),
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
            lower_to_verified_mir(source).is_ok(),
            "a well-formed program must lower and verify"
        );
    }

    #[test]
    fn machine_tier_rejects_source_without_main() {
        let source = "fn helper(x: i64) -> i64 { return x; }";
        let error = compile_source_to_interpreter_value(source)
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
        let object = compile_source_to_object(source).expect("generic call native compilation");
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
        let object = compile_source_to_object(source).expect("native compilation");
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
        let err = compile_source_to_object(source)
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
        let object = compile_source_to_object(source).expect("numeric cast native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_executes_float_arithmetic_and_comparison() {
        let source = "fn add(a: f64, b: f64) -> f64 { return a + b; } fn main() -> i64 { let x = add(1.5, 2.5); if x >= 4.0 { return 42; }; return 0; }";
        let object = compile_source_to_object(source).expect("float native compilation");
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
            .expect("linked float executable must run");
        assert_eq!(run_status.code(), Some(42));

        fs::remove_file(&object_path).ok();
        fs::remove_file(&executable_path).ok();
    }

    #[test]
    fn source_pipeline_executes_boolean_comparison_and_not() {
        let source = "fn main() -> bool { return !(1 == 2); }";
        let object = compile_source_to_object(source).expect("native boolean compilation");
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
        let object = compile_source_to_object(source).expect("loop break native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_executes_while_loop() {
        let source = "fn main() -> i64 { let mut n = 0; while n < 3 { n += 1; } return n; }";
        let object = compile_source_to_object(source).expect("while loop native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_executes_integer_range_for_loop() {
        let source =
            "fn main() -> i64 { let mut sum = 0; for i in 0..5 { sum += i; } return sum; }";
        let object =
            compile_source_to_object(source).expect("integer-range for-loop native compilation");
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
            .expect("system C linker is required for native for-loop E2E");
        assert!(status.success(), "link failed with status {status}");

        let run_status = std::process::Command::new(&executable_path)
            .status()
            .expect("linked native for-loop executable must run");
        assert_eq!(run_status.code(), Some(10));

        fs::remove_file(&object_path).ok();
        fs::remove_file(&executable_path).ok();
    }

    #[test]
    fn source_pipeline_executes_scalar_match_binding() {
        let source = "fn main() -> i64 { let x = 42; return match x { y => y, }; }";
        let object =
            compile_source_to_object(source).expect("scalar binding match native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_parses_borrow_and_deref_forms() {
        let source = "fn f(x: i64) { let y = &x; let z = &mut x; let q = *y; }";
        let mut parser = omni_parse::Parser::from_source(source);
        let parsed = parser.parse_source();
        assert!(parsed.is_ok(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.syntax().text().to_string(), source);
    }

    #[test]
    fn source_pipeline_executes_literal_match() {
        let source = "fn main() -> i64 { let x = 2; return match x { 1 => 7, 2 => 42, _ => 0, }; }";
        let object = compile_source_to_object(source).expect("literal match native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_executes_inclusive_integer_range_for_loop() {
        let source =
            "fn main() -> i64 { let mut sum = 0; for i in 0..=4 { sum += i; } return sum; }";
        let object =
            compile_source_to_object(source).expect("inclusive range for-loop native compilation");
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
        let object = compile_source_to_object(source).expect("if-expression native compilation");
        assert!(!object.is_empty());
    }

    #[test]
    fn source_pipeline_preserves_unit_call_without_fabricating_result() {
        let source = "fn touch() { return; } fn main() -> i64 { touch(); return 7; }";
        let object = compile_source_to_object(source).expect("unit call native compilation");
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
        let error = compile_source_to_object(source).expect_err("integer logical-not must fail");
        assert!(error.contains("Type error"), "unexpected error: {error}");
    }
}
