//! Omni Compiler Driver CLI Entry Point
//! Fully featured argument parser supporting input files, optimization levels, and output targets.

use omni_types::ast::{BinOp, Expr, GenericFnDef, Lit, TypeSpec, UnOp};
use omni_types::checker::SubstEnv;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Compile source through the authoritative frontend, semantic, monomorphization,
/// typed-MIR, verification, and Cranelift native stages.
pub fn compile_source_to_object(source_code: &str) -> Result<Vec<u8>, String> {
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

    omni_codegen::compile_monomorphized_program(&program)
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
            .flat_map(|g| g.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::GenericParam))
            .filter_map(|g| g.children().find(|n| n.kind() == omni_syntax::SyntaxKind::TypeParam))
            .filter_map(|p| direct_name(&p))
            .collect::<Vec<_>>();
        let generic_names = type_params.iter().cloned().collect::<HashSet<_>>();
        let mut fields = Vec::new();
        for field in node.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::StructField) {
            let field_name = direct_name(&field)
                .ok_or_else(|| format!("Semantic frontend error: struct '{}' has unnamed field", name))?;
            let field_ty = direct_type(&field)
                .ok_or_else(|| format!("Semantic frontend error: field '{}' has no type", field_name))
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
            .flat_map(|g| g.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::GenericParam))
            .filter_map(|g| g.children().find(|n| n.kind() == omni_syntax::SyntaxKind::TypeParam))
            .filter_map(|p| direct_name(&p))
            .collect::<Vec<_>>();
        let generic_names = type_params.iter().cloned().collect::<HashSet<_>>();
        let mut variants = Vec::new();
        for variant in node.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::EnumVariant) {
            let variant_name = direct_name(&variant)
                .ok_or_else(|| format!("Semantic frontend error: enum '{}' has an unnamed variant", name))?;
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

fn semantic_functions_from_cst(
    root: &omni_syntax::SyntaxNode,
) -> Result<Vec<GenericFnDef>, String> {
    let mut functions = Vec::new();
    let mut names = HashSet::new();

    let enum_names: HashSet<String> = root
        .children()
        .filter(|n| n.kind() == omni_syntax::SyntaxKind::EnumDef)
        .filter_map(|n| direct_name(&n))
        .collect();



    for node in root.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::FnDef) {
        let name = direct_name(&node)
            .ok_or_else(|| "Semantic frontend error: function is missing a name".to_string())?;
        if !names.insert(name.clone()) {
            return Err(format!("Semantic frontend error: duplicate function '{}'", name));
        }

        let type_params = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::GenericParams)
            .into_iter()
            .flat_map(|g| g.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::GenericParam))
            .filter_map(|g| g.children().find(|n| n.kind() == omni_syntax::SyntaxKind::TypeParam))
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
            params.push((param_name, type_spec_from_cst_with_context(param_type, &generic_names, &enum_names)?));
        }

        let return_type = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::Type)
            .map(|n| type_spec_from_cst_with_context(n, &generic_names, &enum_names))
            .transpose()?
            .unwrap_or(TypeSpec::Unit);

        let body = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::Block)
            .ok_or_else(|| format!("Semantic frontend error: function '{}' has no body", name))?;

        functions.push(GenericFnDef {
            name,
            type_params,
            bounds: Vec::new(),
            params,
            return_type,
            effects: omni_effects::EffectRow::pure(),
            capabilities: Vec::new(),
            body: expr_from_block(&body)?,
        });
    }

    Ok(functions)
}

fn direct_name(node: &omni_syntax::SyntaxNode) -> Option<String> {
    if node.kind() == omni_syntax::SyntaxKind::NameRef {
        let name = node.text().to_string().trim().to_string();
        return (!name.is_empty()).then_some(name);
    }
    node.children()
        .find(|n| n.kind() == omni_syntax::SyntaxKind::NameRef)
        .map(|n| n.text().to_string().trim().to_string())
        .filter(|s| !s.is_empty())
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
                other => {
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
            Ok(TypeSpec::Tuple(node.children().map(|n| type_spec_from_cst_with_generics(n, generic_names)).collect::<Result<Vec<_>, _>>()?))
        }
        omni_syntax::SyntaxKind::ArrayType => {
            let mut children = node.children();
            let elem = children.next().ok_or_else(|| "Semantic frontend error: array type has no element".to_string()).and_then(|n| type_spec_from_cst_with_generics(n, generic_names))?;
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
        omni_syntax::SyntaxKind::SliceType
        | omni_syntax::SyntaxKind::ReferenceType
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
                let binding_name = direct_name(statement).ok_or_else(|| {
                    "Semantic frontend error: let statement is missing a binding name".to_string()
                })?;
                let binding_type = direct_type(statement).map(type_spec_from_cst).transpose()?;
                let initializer = statement
                    .children()
                    .filter(|n| n.kind() != omni_syntax::SyntaxKind::PatternField)
                    .rev()
                    .find(|n| {
                        !matches!(
                            n.kind(),
                            omni_syntax::SyntaxKind::NameRef
                                | omni_syntax::SyntaxKind::Type
                                | omni_syntax::SyntaxKind::BindingPattern
                                | omni_syntax::SyntaxKind::IdentifierPattern
                        )
                    })
                    .ok_or_else(|| {
                        format!("Semantic frontend error: let '{}' has no initializer", binding_name)
                    })?;
                let init = expr_from_node(&initializer)?;
                let remaining = block_statements_to_expr(&statements[index + 1..])?;
                return Ok(Expr::Let {
                    name: binding_name,
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
            | omni_syntax::SyntaxKind::YieldExpr => {
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
            let token = node.first_token().ok_or_else(|| "Semantic frontend error: literal node has no token".to_string())?;
            Ok(Expr::Literal(lit_from_text(token.text())?))
        }
        omni_syntax::SyntaxKind::NameRef => {
            let name = node.text().to_string().trim().to_string();
            (!name.is_empty()).then_some(Expr::Var(name))
                .ok_or_else(|| "Semantic frontend error: empty name reference".to_string())
        }
        omni_syntax::SyntaxKind::PathExpr => Ok(Expr::Var(node.text().to_string().trim().to_string())),
        omni_syntax::SyntaxKind::BinaryExpr
        | omni_syntax::SyntaxKind::AssignExpr
        | omni_syntax::SyntaxKind::RangeExpr => {
            let parts = node.children().collect::<Vec<_>>();
            if parts.len() != 2 {
                return Err(format!("Semantic frontend error: {:?} must contain two operand nodes", node.kind()));
            }
            let op = node.children_with_tokens()
                .filter_map(|e| e.into_token())
                .find(|t| t.kind() == omni_syntax::SyntaxKind::Punct)
                .map(|t| t.text().to_string())
                .ok_or_else(|| "Semantic frontend error: expression has no operator token".to_string())?;
            match node.kind() {
                omni_syntax::SyntaxKind::AssignExpr => {
                    let assign = node
                        .children_with_tokens()
                        .filter_map(|e| e.into_token())
                        .find(|t| t.kind() == omni_syntax::SyntaxKind::Punct)
                        .map(|t| t.text().to_string())
                        .ok_or_else(|| "Semantic frontend error: assignment has no operator token".to_string())?;
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
                            other => return Err(format!("Semantic frontend error: unsupported assignment operator '{}'", other)),
                        };
                        Ok(Expr::CompoundAssign {
                            op,
                            target: Box::new(expr_from_node(&parts[0])?),
                            value: Box::new(expr_from_node(&parts[1])?),
                        })
                    }
                },
                omni_syntax::SyntaxKind::RangeExpr => Ok(Expr::Range {
                    start: Box::new(expr_from_node(&parts[0])?),
                    end: Box::new(expr_from_node(&parts[1])?),
                }),
                _ => Ok(Expr::Binary {
                    op: bin_op_from_text(&op)?,
                    lhs: Box::new(expr_from_node(&parts[0])?),
                    rhs: Box::new(expr_from_node(&parts[1])?),
                }),
            }
        }
        omni_syntax::SyntaxKind::UnaryExpr => {
            let token = node.first_token().ok_or_else(|| "Semantic frontend error: unary expression has no token".to_string())?;
            let operand = node.children().last().ok_or_else(|| "Semantic frontend error: unary expression has no operand".to_string())?;
            match token.text() {
                "-" => Ok(Expr::Unary { op: UnOp::Neg, expr: Box::new(expr_from_node(&operand)?) }),
                "!" => Ok(Expr::Unary { op: UnOp::Not, expr: Box::new(expr_from_node(&operand)?) }),
                other => Err(format!("Semantic frontend error: unsupported unary operator '{}'", other)),
            }
        }
        omni_syntax::SyntaxKind::CallExpr => {
            let mut children = node.children();
            let callee = children.next().ok_or_else(|| "Semantic frontend error: call has no callee".to_string())?;
            let (func, generic_args) = call_target_from_cst(&callee)?;
            let args = children.map(|n| expr_from_node(&n)).collect::<Result<Vec<_>, _>>()?;
            Ok(Expr::Call { func, generic_args, args })
        }
        omni_syntax::SyntaxKind::FieldExpr => {
            let mut children = node.children();
            let base = children.next().ok_or_else(|| "Semantic frontend error: field has no base".to_string())?;
            let field = node.children_with_tokens()
                .filter_map(|e| e.into_token())
                .rev()
                .find(|t| t.kind() == omni_syntax::SyntaxKind::Ident)
                .map(|t| t.text().to_string())
                .ok_or_else(|| "Semantic frontend error: field has no name".to_string())?;
            Ok(Expr::Field { expr: Box::new(expr_from_node(&base)?), field })
        }
        omni_syntax::SyntaxKind::IndexExpr => {
            let mut children = node.children();
            let base = children.next().ok_or_else(|| "Semantic frontend error: index has no base".to_string())?;
            let index = children.next().ok_or_else(|| "Semantic frontend error: index has no expression".to_string())?;
            Ok(Expr::Index { expr: Box::new(expr_from_node(&base)?), index: Box::new(expr_from_node(&index)?) })
        }
        omni_syntax::SyntaxKind::ArrayExpr => Ok(Expr::Array(node.children().map(|n| expr_from_node(&n)).collect::<Result<Vec<_>, _>>()?)),
        omni_syntax::SyntaxKind::TupleExpr => Ok(Expr::Tuple(node.children().map(|n| expr_from_node(&n)).collect::<Result<Vec<_>, _>>()?)),
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
            let expr = children.next().ok_or_else(|| "Semantic frontend error: cast has no source expression".to_string())?;
            let ty = children.next().ok_or_else(|| "Semantic frontend error: cast has no target type".to_string())?;
            Ok(Expr::Cast {
                expr: Box::new(expr_from_node(&expr)?),
                ty: type_spec_from_cst(ty)?,
            })
        }
        omni_syntax::SyntaxKind::LoopExpr => {
            let label = node
                .children()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Lifetime)
                .map(|n| n.text().to_string().trim().trim_start_matches('\'').to_string());
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
            let children = node.children().filter(|n| n.kind() != omni_syntax::SyntaxKind::Lifetime).collect::<Vec<_>>();
            let condition = children
                .get(1)
                .cloned()
                .ok_or_else(|| "Semantic frontend error: while has no condition".to_string())?;
            let body = children
                .iter()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Block)
                .cloned()
                .ok_or_else(|| "Semantic frontend error: while has no body".to_string())?;
            Ok(Expr::While {
                label,
                condition: Box::new(expr_from_node(&condition)?),
                body: Box::new(expr_from_node(&body)?),
            })
        }
        omni_syntax::SyntaxKind::BreakExpr => {
            let mut children = node.children();
            let label = children
                .next()
                .filter(|n| n.kind() == omni_syntax::SyntaxKind::Lifetime)
                .map(|n| n.text().to_string().trim().trim_start_matches('\'').to_string());
            let value = if label.is_some() {
                children.next()
            } else {
                label = None;
                node.children().next()
            }
            .map(|n| expr_from_node(&n))
            .transpose()?
            .map(Box::new);
            Ok(Expr::Break { label, value })
        }
        omni_syntax::SyntaxKind::ContinueExpr => {
            let label = node
                .children()
                .find(|n| n.kind() == omni_syntax::SyntaxKind::Lifetime)
                .map(|n| n.text().to_string().trim().trim_start_matches('\'').to_string());
            Ok(Expr::Continue { label })
        },
        omni_syntax::SyntaxKind::IfExpr => {
            let mut children = node.children();
            let condition = children.next().ok_or_else(|| "Semantic frontend error: if has no condition".to_string())?;
            let then_branch = children.next().ok_or_else(|| "Semantic frontend error: if has no then branch".to_string())?;
            let else_branch = children.next().map(|n| expr_from_node(&n)).transpose()?.map(Box::new);
            Ok(Expr::If {
                condition: Box::new(expr_from_node(&condition)?),
                then_branch: Box::new(expr_from_node(&then_branch)?),
                else_branch,
            })
        }
        omni_syntax::SyntaxKind::CastExpr => {
            let mut children = node.children();
            let expr = children.next().ok_or_else(|| "Semantic frontend error: cast has no source expression".to_string())?;
            let ty = children.next().ok_or_else(|| "Semantic frontend error: cast has no target type".to_string())
                .and_then(type_spec_from_cst)?;
            Ok(Expr::Cast { expr: Box::new(expr_from_node(&expr)?), ty })
        }
        omni_syntax::SyntaxKind::MatchExpr => {
            let mut children = node.children();
            let scrutinee = children.next().ok_or_else(|| "Semantic frontend error: match has no scrutinee".to_string())?;
            let arms = children.filter(|n| n.kind() == omni_syntax::SyntaxKind::MatchArm)
                .map(|n| match_arm_from_cst(&n))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Expr::Match { expr: Box::new(expr_from_node(&scrutinee)?), arms })
        }
        omni_syntax::SyntaxKind::ClosureExpr => {
            let params = node.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::ClosureParam)
                .map(|n| {
                    let name = direct_name(&n).ok_or_else(|| "Semantic frontend error: closure parameter has no name".to_string())?;
                    let ty = direct_type(&n).map(type_spec_from_cst).transpose()?.ok_or_else(|| format!("Semantic frontend error: closure parameter '{}' requires a type", name))?;
                    Ok((name, ty))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let body = node.children().rev().find(|n| n.kind() != omni_syntax::SyntaxKind::ClosureParam)
                .ok_or_else(|| "Semantic frontend error: closure has no body".to_string())?;
            Ok(Expr::Lambda { params, body: Box::new(expr_from_node(&body)?) })
        }
        omni_syntax::SyntaxKind::AwaitExpr
        | omni_syntax::SyntaxKind::MethodCallExpr
        | omni_syntax::SyntaxKind::MacroInvocation
        | omni_syntax::SyntaxKind::AsyncBlock
        | omni_syntax::SyntaxKind::UnsafeBlock
        | omni_syntax::SyntaxKind::TryBlock
        | omni_syntax::SyntaxKind::TryExpr
        | omni_syntax::SyntaxKind::ForExpr
        => Err(format!("Semantic frontend error: native AST lowering does not yet support {:?}", node.kind())),
        other => Err(format!("Semantic frontend error: unsupported expression node {:?}", other)),
    }
}

fn call_target_from_cst(node: &omni_syntax::SyntaxNode) -> Result<(String, Vec<TypeSpec>), String> {
    let segment = node
        .descendants()
        .filter(|n| n.kind() == omni_syntax::SyntaxKind::PathSegment)
        .last()
        .ok_or_else(|| "Semantic frontend error: call target has no path segment".to_string())?;
    let name = direct_name(&segment)
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
    let pattern = children.next().ok_or_else(|| "Semantic frontend error: match arm has no pattern".to_string())?;
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
            let name = direct_name(node).ok_or_else(|| "Semantic frontend error: pattern has no binding name".to_string())?;
            Ok(omni_types::ast::Pattern::Binding(name))
        }
        omni_syntax::SyntaxKind::LiteralPattern => {
            let lit_node = node.children().next().ok_or_else(|| "Semantic frontend error: literal pattern is empty".to_string())?;
            let token = lit_node.first_token().ok_or_else(|| "Semantic frontend error: literal pattern has no token".to_string())?;
            Ok(omni_types::ast::Pattern::Lit(lit_from_text(token.text())?))
        }
        omni_syntax::SyntaxKind::TuplePattern => Ok(omni_types::ast::Pattern::Tuple(node.children().map(|n| pattern_from_cst(&n)).collect::<Result<Vec<_>, _>>()?)),
        omni_syntax::SyntaxKind::OrPattern => Ok(omni_types::ast::Pattern::Or(node.children().map(|n| pattern_from_cst(&n)).collect::<Result<Vec<_>, _>>()?)),
        omni_syntax::SyntaxKind::RangePattern => {
            let parts = node.children().collect::<Vec<_>>();
            let start = parts.first().ok_or_else(|| "Semantic frontend error: range pattern has no start".to_string())?;
            let end = parts.get(1).ok_or_else(|| "Semantic frontend error: range pattern has no end".to_string())?;
            let start_lit = match expr_from_node(start)? { Expr::Literal(l) => l, _ => return Err("Semantic frontend error: non-literal range pattern unsupported".into()) };
            let end_lit = match expr_from_node(end)? { Expr::Literal(l) => l, _ => return Err("Semantic frontend error: non-literal range pattern unsupported".into()) };
            Ok(omni_types::ast::Pattern::Range {
                start: omni_types::ast::PatternRangeBoundary::Inclusive(start_lit),
                end: omni_types::ast::PatternRangeBoundary::Exclusive(end_lit),
            })
        }
        omni_syntax::SyntaxKind::StructPattern => {
            let mut parts = node.children();
            let path = parts.next().ok_or_else(|| "Semantic frontend error: struct pattern has no path".to_string())?;
            let fields = parts.map(|f| {
                let name = direct_name(&f).ok_or_else(|| "Semantic frontend error: pattern field has no name".to_string())?;
                let sub = f.children().nth(1).map(|n| pattern_from_cst(&n)).transpose()?.unwrap_or(omni_types::ast::Pattern::Binding(name.clone()));
                Ok((name, sub))
            }).collect::<Result<Vec<_>, String>>()?;
            Ok(omni_types::ast::Pattern::Struct { name: path.text().to_string().trim().to_string(), fields })
        }
        omni_syntax::SyntaxKind::EnumPattern => {
            let mut parts = node.children();
            let path = parts
                .next()
                .ok_or_else(|| "Semantic frontend error: enum pattern has no path".to_string())?;
            let path_text = path.text().to_string().trim().to_string();
            let (enum_name, variant) = path_text
                .rsplit_once("::")
                .map(|(e, v)| (e.to_string(), v.to_string()))
                .unwrap_or_else(|| (path_text.clone(), path_text.clone()));
            let subpatterns = parts
                .map(|n| pattern_from_cst(&n))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(omni_types::ast::Pattern::Variant {
                enum_name,
                variant,
                subpatterns,
            })
        },
        omni_syntax::SyntaxKind::ReferencePattern
        | omni_syntax::SyntaxKind::SlicePattern
        | omni_syntax::SyntaxKind::GuardPattern => Err(format!("Semantic frontend error: pattern lowering does not yet support {:?}", node.kind())),
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
                    match omni_machine::bridge::execute_source(&source_code) {
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
        let object = compile_source_to_object(source).expect("short-circuit expression must compile");
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
