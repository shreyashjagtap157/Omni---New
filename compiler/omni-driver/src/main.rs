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

fn semantic_functions_from_cst(
    root: &omni_syntax::SyntaxNode,
) -> Result<Vec<GenericFnDef>, String> {
    let mut functions = Vec::new();
    let mut names = HashSet::new();

    for node in root.children().filter(|n| n.kind() == omni_syntax::SyntaxKind::FnDef) {
        let name = direct_name(&node)
            .ok_or_else(|| "Semantic frontend error: function is missing a name".to_string())?;
        if !names.insert(name.clone()) {
            return Err(format!("Semantic frontend error: duplicate function '{}'", name));
        }

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
            params.push((param_name, type_spec_from_cst(param_type)?));
        }

        let return_type = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::Type)
            .map(type_spec_from_cst)
            .transpose()?
            .unwrap_or(TypeSpec::Unit);

        let body = node
            .children()
            .find(|n| n.kind() == omni_syntax::SyntaxKind::Block)
            .ok_or_else(|| format!("Semantic frontend error: function '{}' has no body", name))?;

        functions.push(GenericFnDef {
            name,
            type_params: Vec::new(),
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
    let token = node
        .first_token()
        .ok_or_else(|| "Semantic frontend error: type node has no token".to_string())?;
    match token.text() {
        "i64" | "Int" => Ok(TypeSpec::Int),
        "f64" | "Float" => Ok(TypeSpec::Float),
        "bool" | "Bool" => Ok(TypeSpec::Bool),
        "char" | "Char" => Ok(TypeSpec::Char),
        "u8" | "byte" | "Byte" => Ok(TypeSpec::Byte),
        "str" | "String" => Ok(TypeSpec::String),
        "unit" | "Unit" => Ok(TypeSpec::Unit),
        "never" | "Never" => Ok(TypeSpec::Never),
        other => Err(format!("Semantic frontend error: unsupported type spelling '{}'", other)),
    }
}

fn expr_from_block(node: &omni_syntax::SyntaxNode) -> Result<Expr, String> {
    let statements: Vec<_> = node.children().collect();
    block_statements_to_expr(&statements)
}

fn block_statements_to_expr(statements: &[omni_syntax::SyntaxNode]) -> Result<Expr, String> {
    if statements.is_empty() {
        return Ok(Expr::Block(Vec::new()));
    }

    let first = &statements[0];
    match first.kind() {
        omni_syntax::SyntaxKind::LetStmt => {
            let binding_name = direct_name(first).ok_or_else(|| {
                "Semantic frontend error: let statement is missing a binding name".to_string()
            })?;
            let binding_type = direct_type(first).map(type_spec_from_cst).transpose()?;
            let initializer = first
                .children()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .find(|n| {
                    matches!(
                        n.kind(),
                        omni_syntax::SyntaxKind::ExprStmt
                            | omni_syntax::SyntaxKind::BinaryExpr
                            | omni_syntax::SyntaxKind::UnaryExpr
                            | omni_syntax::SyntaxKind::LiteralExpr
                            | omni_syntax::SyntaxKind::NameRef
                            | omni_syntax::SyntaxKind::CallExpr
                    )
                })
                .ok_or_else(|| {
                    format!("Semantic frontend error: let '{}' has no initializer", binding_name)
                })?;
            let init = expr_from_node(&initializer)?;
            let body = block_statements_to_expr(&statements[1..])?;
            Ok(Expr::Let {
                name: binding_name,
                ty: binding_type,
                init: Box::new(init),
                body: Box::new(body),
            })
        }
        omni_syntax::SyntaxKind::ReturnExpr => {
            if statements.len() > 1 {
                return Err(
                    "Semantic frontend error: statements after return are not yet supported"
                        .to_string(),
                );
            }
            expr_from_node(first)
        }
        omni_syntax::SyntaxKind::ExprStmt => {
            let first_expr = expr_from_node(first)?;
            if statements.len() == 1 {
                Ok(first_expr)
            } else {
                let rest = block_statements_to_expr(&statements[1..])?;
                match rest {
                    Expr::Block(mut exprs) => {
                        exprs.insert(0, first_expr);
                        Ok(Expr::Block(exprs))
                    }
                    other => Ok(Expr::Block(vec![first_expr, other])),
                }
            }
        }
        other => Err(format!("Semantic frontend error: unsupported block statement {:?}", other)),
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
                omni_syntax::SyntaxKind::AssignExpr => Ok(Expr::Assign {
                    target: Box::new(expr_from_node(&parts[0])?),
                    value: Box::new(expr_from_node(&parts[1])?),
                }),
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
            let func = direct_name(&callee)
                .or_else(|| (callee.kind() == omni_syntax::SyntaxKind::PathExpr).then(|| callee.text().to_string().trim().to_string()))
                .ok_or_else(|| "Semantic frontend error: only direct named calls are supported".to_string())?;
            let args = children.map(|n| expr_from_node(&n)).collect::<Result<Vec<_>, _>>()?;
            Ok(Expr::Call { func, generic_args: Vec::new(), args })
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
        | omni_syntax::SyntaxKind::CastExpr
        | omni_syntax::SyntaxKind::MacroInvocation
        | omni_syntax::SyntaxKind::AsyncBlock
        | omni_syntax::SyntaxKind::UnsafeBlock
        | omni_syntax::SyntaxKind::TryBlock
        | omni_syntax::SyntaxKind::TryExpr
        | omni_syntax::SyntaxKind::IfExpr
        | omni_syntax::SyntaxKind::WhileExpr
        | omni_syntax::SyntaxKind::ForExpr
        | omni_syntax::SyntaxKind::LoopExpr
        => Err(format!("Semantic frontend error: native AST lowering does not yet support {:?}", node.kind())),
        other => Err(format!("Semantic frontend error: unsupported expression node {:?}", other)),
    }
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
        omni_syntax::SyntaxKind::EnumPattern => Err("Semantic frontend error: enum pattern lowering is not yet defined for the current AST shape".into()),
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
        "==" => Ok(BinOp::Eq),
        "!=" => Ok(BinOp::Ne),
        "<" => Ok(BinOp::Lt),
        ">" => Ok(BinOp::Gt),
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
