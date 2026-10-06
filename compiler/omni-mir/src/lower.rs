//! AST-to-MIR Lowering Engine for Omni.
//! Translates validated, monomorphized expressions into canonical Mid-Level IR.

use crate::ir::Body;
use index_vec::IndexVec;
use omni_types::intern::{Ty, TyCtxt, TyKind};
use omni_types::monomorph::MonomorphizedProgram;
use std::collections::HashMap;

/// True when an expression transfers control away instead of falling through.
///
/// `Expr::Return` is typed as its operand's type rather than as `Never`, so the
/// inferred type alone cannot tell a diverging branch from a value-producing
/// one. This structural check is what lets `if c { return 1; }` be lowered as
/// the Unit statement it is.
fn expr_diverges(expr: &omni_types::ast::Expr) -> bool {
    use omni_types::ast::Expr;
    match expr {
        Expr::Return(_) | Expr::Break { .. } | Expr::Continue { .. } => true,
        Expr::Block(stmts) => stmts.last().is_some_and(expr_diverges),
        Expr::UnsafeBlock { body } => expr_diverges(body),
        _ => false,
    }
}

pub struct LoweringContext {
    body: Body,
    struct_defs: HashMap<String, omni_types::ast::StructDef>,
}

impl LoweringContext {
    pub fn new() -> Self {
        Self { body: Body::default(), struct_defs: HashMap::new() }
    }

    /// Lowers the canonical semantic HIR into typed MIR.
    ///
    /// HIR is consumed at the MIR boundary so its TyCtxt becomes the exact
    /// type context owned by the resulting MIR program. This prevents MIR from
    /// rebuilding a second, semantically independent type arena.
    pub fn lower_hir_program(
        &mut self,
        hir: omni_hir::HirProgram,
    ) -> Result<crate::ir::MirProgram, String> {
        hir.validate()?;
        self.struct_defs = hir.struct_defs.clone();
        let tcx = hir.tcx;
        self.lower_hir_functions(&hir.functions, tcx)
    }

    /// Supplies struct declarations so field projections can be typed during lowering.
    pub fn set_struct_defs(&mut self, struct_defs: HashMap<String, omni_types::ast::StructDef>) {
        self.struct_defs = struct_defs;
    }

    /// Lowers a concrete monomorphized program through the canonical HIR boundary.
    ///
    /// This compatibility entry point is retained for existing callers. It
    /// constructs canonical HIR first and then uses exactly the same HIR-to-MIR
    /// implementation as the explicit HIR entry point.
    pub fn lower_monomorphized_program(
        &mut self,
        prog: &MonomorphizedProgram,
    ) -> Result<crate::ir::MirProgram, String> {
        let hir = omni_hir::HirProgram::from_monomorphized(
            prog,
            self.struct_defs.clone(),
            HashMap::new(),
        )?;
        self.lower_hir_program(hir)
    }

    /// Lower the canonical HIR representation into typed MIR without
    /// reconstructing or re-lowering its type specifications.
    ///
    /// HIR owns the canonical TyCtxt and concrete function type handles. Those
    /// handles become the MIR type authority directly. The declarative
    /// TypeSpec fields retained by HIR are consumed only where MIR must
    /// preserve source-level declarations for diagnostics/compatibility.
    fn lower_hir_functions(
        &mut self,
        functions: &[omni_hir::HirFunction],
        mut tcx: TyCtxt,
    ) -> Result<crate::ir::MirProgram, String> {
        let mut fn_sigs: HashMap<String, (Vec<Ty>, Ty)> = HashMap::new();

        for func in functions {
            if func.param_tys.len() != func.params.len() {
                return Err(format!(
                    "MIR lowering error: HIR parameter metadata mismatch in '{}'",
                    func.name
                ));
            }
            if fn_sigs.insert(func.name.clone(), (func.param_tys.clone(), func.return_ty)).is_some()
            {
                return Err(format!("MIR lowering error: duplicate HIR function '{}'", func.name));
            }
        }

        let mut mir_functions = Vec::with_capacity(functions.len());

        for func in functions {
            let mut blocks = IndexVec::new();
            let mut local_decls = IndexVec::new();
            let mut scope = HashMap::new();

            let ret_ty = func.return_ty;
            let return_place = local_decls
                .push(crate::ir::LocalDecl { name: Some("_return".to_string()), ty: Some(ret_ty) });

            let mut param_locals = Vec::with_capacity(func.params.len());
            for ((p_name, _), &p_ty) in func.params.iter().zip(&func.param_tys) {
                let p_local = local_decls
                    .push(crate::ir::LocalDecl { name: Some(p_name.clone()), ty: Some(p_ty) });
                param_locals.push(p_local);
                scope.insert(p_name.clone(), p_local);
            }

            let subst = omni_types::checker::SubstEnv::new();
            let mut builder = FnMirBuilder {
                tcx: &mut tcx,
                subst: &subst,
                local_decls: &mut local_decls,
                blocks: &mut blocks,
                scope,
                fn_sigs: &fn_sigs,
                struct_defs: &self.struct_defs,
                return_ty: ret_ty,
                current_block: None,
                loops: Vec::new(),
                diverged: false,
                unsafe_blocks: Vec::new(),
            };

            let entry_block = builder.new_block();
            builder.current_block = Some(entry_block);

            let result = builder.lower_expr(&func.body)?;

            if let Some(curr) = builder.current_block {
                match result {
                    Some((operand, value_ty)) => {
                        if value_ty != ret_ty {
                            return Err(format!(
                                "MIR lowering error in '{}': function body has type {:?}, expected {:?}",
                                func.name, value_ty, ret_ty
                            ));
                        }
                        let ret_p = crate::ir::Place::local(return_place);
                        builder.blocks[curr].statements.push(crate::ir::Statement::Assign(
                            ret_p,
                            crate::ir::Rvalue::Use(operand),
                        ));
                    }
                    None => {
                        let unit_ty = builder.tcx.intern(TyKind::Unit);
                        let never_ty = builder.tcx.intern(TyKind::Never);
                        if ret_ty != unit_ty && ret_ty != never_ty {
                            return Err(format!(
                                "MIR lowering error in '{}': function body has no value, expected {:?}",
                                func.name, ret_ty
                            ));
                        }
                    }
                }

                if builder.blocks[curr].terminator.is_none() {
                    builder.blocks[curr].terminator = Some(crate::ir::Terminator::Return);
                }
            }

            let unsafe_blocks = builder.unsafe_blocks;
            let body = Body { blocks, local_decls, unsafe_blocks };
            mir_functions.push(crate::ir::MirFunction {
                name: func.name.clone(),
                params: param_locals,
                return_place,
                return_type: func.return_type.clone(),
                body,
            });
        }

        let mir_prog = crate::ir::MirProgram {
            tcx,
            functions: mir_functions,
            struct_defs: self.struct_defs.clone(),
        };
        if let Some(first_fn) = mir_prog.functions.first() {
            self.body = first_fn.body.clone();
        }
        Ok(mir_prog)
    }
}

fn simple_pattern_binding(pattern: &omni_types::ast::Pattern) -> Option<String> {
    match pattern {
        omni_types::ast::Pattern::Binding(name) => Some(name.clone()),
        _ => None,
    }
}

struct LoopContext {
    label: Option<String>,
    continue_block: crate::ir::BasicBlock,
    break_block: crate::ir::BasicBlock,
    result_local: Option<crate::ir::Local>,
    result_ty: Option<Ty>,
}

struct FnMirBuilder<'a> {
    tcx: &'a mut TyCtxt,
    subst: &'a omni_types::checker::SubstEnv,
    local_decls: &'a mut IndexVec<crate::ir::Local, crate::ir::LocalDecl>,
    blocks: &'a mut IndexVec<crate::ir::BasicBlock, crate::ir::BlockData>,
    scope: HashMap<String, crate::ir::Local>,
    fn_sigs: &'a HashMap<String, (Vec<Ty>, Ty)>,
    struct_defs: &'a HashMap<String, omni_types::ast::StructDef>,
    return_ty: Ty,
    current_block: Option<crate::ir::BasicBlock>,
    loops: Vec<LoopContext>,
    /// Set once the enclosing *function* has transferred control to its return
    /// place. `current_block` is also cleared by `break`/`continue`, but those
    /// only leave the current basic block inside a loop, so a block statement
    /// list must keep lowering after them; only a real `return` makes the
    /// remaining statements unreachable.
    diverged: bool,
    /// Basic blocks reached from inside an `unsafe` region, carried out to
    /// `Body::unsafe_blocks` for UNSAFE-0002 visibility. Metadata only: it
    /// relaxes nothing.
    unsafe_blocks: Vec<crate::ir::BasicBlock>,
}

impl<'a> FnMirBuilder<'a> {
    fn new_block(&mut self) -> crate::ir::BasicBlock {
        self.blocks.push(crate::ir::BlockData { statements: Vec::new(), terminator: None })
    }

    fn new_temp(&mut self, name: Option<String>, ty: Ty) -> crate::ir::Local {
        self.local_decls.push(crate::ir::LocalDecl { name, ty: Some(ty) })
    }

    fn local_ty(&self, local: crate::ir::Local) -> Result<Ty, String> {
        self.local_decls[local]
            .ty
            .ok_or_else(|| format!("MIR lowering error: local {:?} has no type", local))
    }

    /// Resolves the type of a projected place by walking the projection chain
    /// against the type of the local it starts from.
    ///
    /// This is the single authority for what a projection *means*, shared by
    /// assignment targets and by read lowering, so `p.x` cannot be typed one
    /// way when read and a different way when written.
    fn projected_ty(
        &mut self,
        base_ty: Ty,
        projections: &[crate::ir::Projection],
    ) -> Result<Ty, String> {
        let mut current = base_ty;
        for projection in projections {
            current = match projection {
                crate::ir::Projection::Field(name) => self.field_ty(current, name)?,
                crate::ir::Projection::ConstantIndex(index) => {
                    self.constant_index_ty(current, *index)?
                }
                // A runtime index carries no constant, so the element type of an
                // array is known but a tuple position is not. Tuples are
                // required to use constant subscripts, which is what
                // `ConstantIndex` exists for.
                crate::ir::Projection::Index(_) => match self.tcx.get(current).clone() {
                    TyKind::Array(elem, _) => elem,
                    other => {
                        return Err(format!(
                            "MIR lowering error: index projection requires an array, found {:?}",
                            other
                        ))
                    }
                },
                crate::ir::Projection::Deref => match self.tcx.get(current).clone() {
                    TyKind::Reference { inner, .. } => inner,
                    other => {
                        return Err(format!(
                            "MIR lowering error: deref projection requires a reference, found {:?}",
                            other
                        ))
                    }
                },
            };
        }
        Ok(current)
    }

    /// Resolves the type of a named field projection on a struct or tuple.
    fn field_ty(&mut self, base_ty: Ty, field: &str) -> Result<Ty, String> {
        match self.tcx.get(base_ty).clone() {
            TyKind::Struct(name, args) => {
                let def = self.struct_defs.get(&name).cloned().ok_or_else(|| {
                    format!("MIR lowering error: unknown struct '{}' in field '{}'", name, field)
                })?;
                let mut field_env = omni_types::checker::SubstEnv::new();
                for (param, arg) in def.type_params.iter().zip(args.iter()) {
                    field_env.insert(param.clone(), *arg);
                }
                def.fields
                    .iter()
                    .find(|f| f.name == field)
                    .map(|f| self.tcx.lower_type_spec(&f.ty, &field_env))
                    .ok_or_else(|| {
                        format!("MIR lowering error: field '{}' not found on '{}'", field, name)
                    })
            }
            TyKind::Tuple(types) => {
                let index: usize = field.parse().map_err(|_| {
                    "MIR lowering error: tuple field must be a numeric index".to_string()
                })?;
                types.get(index).copied().ok_or_else(|| {
                    format!("MIR lowering error: tuple field index {} out of bounds", index)
                })
            }
            other => Err(format!(
                "MIR lowering error: field projection requires struct or tuple, found {:?}",
                other
            )),
        }
    }

    /// Resolves the type of a constant index projection on an array or tuple.
    fn constant_index_ty(&self, base_ty: Ty, index: usize) -> Result<Ty, String> {
        match self.tcx.get(base_ty).clone() {
            TyKind::Array(elem, length) => {
                if index >= length {
                    return Err(format!(
                        "MIR lowering error: array index {} is out of bounds for length {}",
                        index, length
                    ));
                }
                Ok(elem)
            }
            TyKind::Tuple(types) => types
                .get(index)
                .copied()
                .ok_or_else(|| format!("MIR lowering error: tuple index {} out of bounds", index)),
            other => Err(format!(
                "MIR lowering error: index projection requires array or tuple, found {:?}",
                other
            )),
        }
    }

    /// Lowers an assignment target to a projected place and its type.
    ///
    /// A bare name yields a whole-local place. A field or index target yields
    /// that place with a projection appended, so the destination of `p.x = v`
    /// names `p.x` rather than the whole of `p`. The result type is computed by
    /// `projected_ty`, the same authority the read path uses, so a projection
    /// cannot be typed one way when read and another way when written.
    fn lower_assign_place(
        &mut self,
        target: &omni_types::ast::Expr,
    ) -> Result<(crate::ir::Place, Ty), String> {
        match target {
            omni_types::ast::Expr::Var(name) => {
                let local = *self.scope.get(name).ok_or_else(|| {
                    format!("MIR lowering error: assignment target '{}' is not bound", name)
                })?;
                let ty = self.local_ty(local)?;
                Ok((crate::ir::Place::local(local), ty))
            }
            omni_types::ast::Expr::Field { expr, field } => {
                let (base_place, base_ty) = self.lower_assign_place(expr)?;
                let place = base_place.project(crate::ir::Projection::Field(field.clone()));
                let ty = self.projected_ty(base_ty, &place.projections)?;
                Ok((place, ty))
            }
            omni_types::ast::Expr::Index { expr, index } => {
                let (base_place, base_ty) = self.lower_assign_place(expr)?;
                // A literal subscript folds to a constant projection, which keeps
                // the destination statically checkable against the array length.
                if let omni_types::ast::Expr::Literal(omni_types::ast::Lit::Int(n)) = index.as_ref()
                {
                    if *n < 0 {
                        return Err("MIR lowering error: index must not be negative".to_string());
                    }
                    let place =
                        base_place.project(crate::ir::Projection::ConstantIndex(*n as usize));
                    let ty = self.projected_ty(base_ty, &place.projections)?;
                    return Ok((place, ty));
                }
                // A computed subscript is bound to a temporary local, and the
                // projection refers to that local. The index expression is
                // evaluated exactly once, before the store.
                let int_ty = self.tcx.intern(TyKind::Int);
                let (index_op, index_ty) = self
                    .lower_expr(index)?
                    .ok_or_else(|| "MIR lowering error: index expression is Unit".to_string())?;
                if index_ty != int_ty {
                    return Err(format!(
                        "MIR lowering error: index expression must be Int, found {:?}",
                        index_ty
                    ));
                }
                let element_ty = match self.tcx.get(base_ty).clone() {
                    TyKind::Array(elem, _) => elem,
                    other => {
                        return Err(format!(
                            "MIR lowering error: index assignment requires an array, found {:?}",
                            other
                        ))
                    }
                };
                let block = self.current_block.ok_or_else(|| {
                    "MIR lowering error: index assignment has no live block".to_string()
                })?;
                let index_local = self.new_temp(Some("_index_tmp".to_string()), int_ty);
                let index_place = crate::ir::Place::local(index_local);
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    index_place.clone(),
                    crate::ir::Rvalue::Use(index_op),
                ));
                let place = base_place.project(crate::ir::Projection::Index(index_local));
                Ok((place, element_ty))
            }
            other => Err(format!("MIR lowering error: unsupported assignment target {:?}", other)),
        }
    }

    fn literal_ty(&mut self, lit: &omni_types::ast::Lit) -> Ty {
        match lit {
            omni_types::ast::Lit::Int(_) => self.tcx.intern(TyKind::Int),
            omni_types::ast::Lit::Float(_) => self.tcx.intern(TyKind::Float),
            omni_types::ast::Lit::Bool(_) => self.tcx.intern(TyKind::Bool),
            omni_types::ast::Lit::Char(_) => self.tcx.intern(TyKind::Char),
            omni_types::ast::Lit::Byte(_) => self.tcx.intern(TyKind::Byte),
            omni_types::ast::Lit::String(_) => self.tcx.intern(TyKind::String),
        }
    }

    fn bind_let_pattern(
        &mut self,
        pattern: &omni_types::ast::Pattern,
        operand: crate::ir::Operand,
        ty: Ty,
    ) -> Result<Vec<(String, crate::ir::Local)>, String> {
        match pattern {
            omni_types::ast::Pattern::Binding(name) => {
                let block = self.current_block.ok_or_else(|| "MIR lowering error: let pattern has no live block".to_string())?;
                let local = self.new_temp(Some(name.clone()), ty);
                let place = crate::ir::Place::local(local);
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Use(operand),
                ));
                Ok(vec![(name.clone(), local)])
            }
            omni_types::ast::Pattern::Wildcard => Ok(Vec::new()),
            omni_types::ast::Pattern::Tuple(patterns) => {
                let TyKind::Tuple(types) = self.tcx.get(ty).clone() else {
                    return Err("MIR lowering error: tuple destructuring requires a tuple initializer".into());
                };
                if patterns.len() != types.len() {
                    return Err(format!(
                        "MIR lowering error: tuple pattern has {} elements, initializer has {}",
                        patterns.len(),
                        types.len()
                    ));
                }
                let mut bindings = Vec::new();
                for (index, (subpattern, sub_ty)) in patterns.iter().zip(types.iter()).enumerate() {
                    let block = self.current_block.ok_or_else(|| "MIR lowering error: tuple destructuring has no live block".to_string())?;
                    let local = self.new_temp(Some(format!("_tuple_field_{index}")), *sub_ty);
                    let place = crate::ir::Place::local(local);
                    let projection = crate::ir::Rvalue::Field {
                        base: operand.clone(),
                        field: index.to_string(),
                        ty: *sub_ty,
                    };
                    self.blocks[block].statements.push(crate::ir::Statement::Assign(
                        place.clone(),
                        projection,
                    ));
                    bindings.extend(self.bind_let_pattern(
                        subpattern,
                        crate::ir::Operand::Copy(place),
                        *sub_ty,
                    )?);
                }
                Ok(bindings)
            }
            omni_types::ast::Pattern::Reference { .. } => {
                Err("MIR lowering error: reference-pattern let bindings require reference storage".into())
            }
            _ => Err("MIR lowering error: let destructuring pattern is not representable by current MIR projections".into()),
        }
    }

    fn lower_loop_expression(
        &mut self,
        label: Option<&str>,
        body: &omni_types::ast::Expr,
    ) -> Result<Option<(crate::ir::Operand, Ty)>, String> {
        let entry = self
            .current_block
            .ok_or_else(|| "MIR lowering error: loop has no live entry block".to_string())?;
        let header = self.new_block();
        let break_block = self.new_block();
        self.blocks[entry].terminator = Some(crate::ir::Terminator::Goto(header));

        self.current_block = Some(header);
        self.loops.push(LoopContext {
            label: label.map(str::to_owned),
            continue_block: header,
            break_block,
            result_local: None,
            result_ty: None,
        });
        // A `return` in the loop body diverges that path only; the loop's own
        // back edge keeps the body reachable, so the divergence flag does not
        // escape the loop.
        let outer_diverged = self.diverged;
        self.diverged = false;

        self.lower_expr(body)?;
        let _body_diverged = self.diverged;
        self.diverged = outer_diverged;
        if let Some(block) = self.current_block {
            if self.blocks[block].terminator.is_none() {
                self.blocks[block].terminator = Some(crate::ir::Terminator::Goto(header));
            }
        }
        let context = self.loops.pop().expect("loop context balanced");
        self.current_block = Some(break_block);

        if let Some(local) = context.result_local {
            let ty = context.result_ty.expect("result type accompanies result local");
            Ok(Some((crate::ir::Operand::Copy(crate::ir::Place::local(local)), ty)))
        } else if let Some(ty) = context.result_ty {
            if matches!(self.tcx.get(ty), TyKind::Unit) {
                Ok(None)
            } else {
                Err("MIR lowering error: loop break value has no result storage".into())
            }
        } else {
            Ok(None)
        }
    }

    fn lower_while_expression(
        &mut self,
        label: Option<&str>,
        condition: &omni_types::ast::Expr,
        body: &omni_types::ast::Expr,
    ) -> Result<Option<(crate::ir::Operand, Ty)>, String> {
        let entry = self
            .current_block
            .ok_or_else(|| "MIR lowering error: while has no live entry block".to_string())?;
        let condition_block = self.new_block();
        let body_block = self.new_block();
        let break_block = self.new_block();
        self.blocks[entry].terminator = Some(crate::ir::Terminator::Goto(condition_block));

        self.current_block = Some(condition_block);
        let (condition_op, condition_ty) = self
            .lower_expr(condition)?
            .ok_or_else(|| "MIR lowering error: while condition is Unit".to_string())?;
        let bool_ty = self.tcx.intern(TyKind::Bool);
        if condition_ty != bool_ty {
            return Err(format!(
                "MIR lowering error: while condition has type {:?}, expected Bool",
                condition_ty
            ));
        }
        self.blocks[condition_block].terminator = Some(crate::ir::Terminator::SwitchInt {
            discr: condition_op,
            targets: vec![(1, body_block)],
            otherwise: break_block,
        });

        self.current_block = Some(body_block);
        self.loops.push(LoopContext {
            label: label.map(str::to_owned),
            continue_block: condition_block,
            break_block,
            result_local: None,
            result_ty: Some(self.tcx.intern(TyKind::Unit)),
        });
        self.lower_expr(body)?;
        if let Some(block) = self.current_block {
            if self.blocks[block].terminator.is_none() {
                self.blocks[block].terminator = Some(crate::ir::Terminator::Goto(condition_block));
            }
        }
        let context = self.loops.pop().expect("while loop context balanced");
        if context.result_ty != Some(self.tcx.intern(TyKind::Unit))
            || context.result_local.is_some()
        {
            return Err("MIR lowering error: while loop break values are unsupported".into());
        }
        self.current_block = Some(break_block);
        Ok(None)
    }

    fn lower_break_expression(
        &mut self,
        label: Option<&str>,
        value: Option<&omni_types::ast::Expr>,
    ) -> Result<Option<(crate::ir::Operand, Ty)>, String> {
        let value_result = if let Some(expr) = value {
            Some(
                self.lower_expr(expr)?
                    .ok_or_else(|| "MIR lowering error: break value is Unit".to_string())?,
            )
        } else {
            None
        };
        let block = self
            .current_block
            .ok_or_else(|| "MIR lowering error: break has no live block".to_string())?;
        let loop_index = if let Some(label) = label {
            self.loops
                .iter()
                .rposition(|ctx| ctx.label.as_deref() == Some(label))
                .ok_or_else(|| format!("MIR lowering error: unknown break label '{}'", label))?
        } else {
            self.loops
                .len()
                .checked_sub(1)
                .ok_or_else(|| "MIR lowering error: break outside loop".to_string())?
        };
        let value_ty = value_result
            .as_ref()
            .map(|(_, ty)| *ty)
            .unwrap_or_else(|| self.tcx.intern(TyKind::Unit));

        let break_target = self.loops[loop_index].break_block;
        if let Some(existing) = self.loops[loop_index].result_ty {
            if existing != value_ty {
                return Err(format!(
                    "MIR lowering error: break type {:?} does not match loop break type {:?}",
                    value_ty, existing
                ));
            }
        } else {
            self.loops[loop_index].result_ty = Some(value_ty);
        }

        if let Some((operand, _)) = value_result {
            let result_local = if let Some(local) = self.loops[loop_index].result_local {
                local
            } else {
                let local = self.new_temp(Some("_loop_result".to_string()), value_ty);
                self.loops[loop_index].result_local = Some(local);
                local
            };
            let place = crate::ir::Place::local(result_local);
            self.blocks[block]
                .statements
                .push(crate::ir::Statement::Assign(place, crate::ir::Rvalue::Use(operand)));
        }

        self.blocks[block].terminator = Some(crate::ir::Terminator::Goto(break_target));
        self.current_block = None;
        Ok(None)
    }

    fn lower_continue_expression(
        &mut self,
        label: Option<&str>,
    ) -> Result<Option<(crate::ir::Operand, Ty)>, String> {
        let loop_index = if let Some(label) = label {
            self.loops
                .iter()
                .rposition(|ctx| ctx.label.as_deref() == Some(label))
                .ok_or_else(|| format!("MIR lowering error: unknown continue label '{}'", label))?
        } else {
            self.loops
                .len()
                .checked_sub(1)
                .ok_or_else(|| "MIR lowering error: continue outside loop".to_string())?
        };
        let continue_block = self.loops[loop_index].continue_block;
        let current = self
            .current_block
            .ok_or_else(|| "MIR lowering error: continue has no live block".to_string())?;
        self.blocks[current].terminator = Some(crate::ir::Terminator::Goto(continue_block));
        self.current_block = None;
        Ok(None)
    }

    fn lower_for_expression(
        &mut self,
        label: Option<&str>,
        pattern: &omni_types::ast::Pattern,
        iterable: &omni_types::ast::Expr,
        body: &omni_types::ast::Expr,
    ) -> Result<Option<(crate::ir::Operand, Ty)>, String> {
        let omni_types::ast::Expr::Range { start, end, inclusive } = iterable else {
            return Err(
                "MIR lowering error: only integer ranges are currently supported by for iteration"
                    .into(),
            );
        };
        let (start_op, start_ty) = self
            .lower_expr(start)?
            .ok_or_else(|| "MIR lowering error: for-range start is Unit".to_string())?;
        let (end_op, end_ty) = self
            .lower_expr(end)?
            .ok_or_else(|| "MIR lowering error: for-range end is Unit".to_string())?;
        let int_ty = self.tcx.intern(TyKind::Int);
        if start_ty != int_ty || end_ty != int_ty {
            return Err("MIR lowering error: integer range iteration requires Int endpoints".into());
        }

        let entry = self
            .current_block
            .ok_or_else(|| "MIR lowering error: for loop has no live entry block".to_string())?;
        let header = self.new_block();
        let body_block = self.new_block();
        let step_block = self.new_block();
        let exit = self.new_block();
        let index_local = self.new_temp(Some("_for_index".to_string()), int_ty);
        let end_local = self.new_temp(Some("_for_end".to_string()), int_ty);

        let index_place = crate::ir::Place::local(index_local);
        let end_place = crate::ir::Place::local(end_local);
        self.blocks[entry].statements.push(crate::ir::Statement::Assign(
            index_place.clone(),
            crate::ir::Rvalue::Use(start_op),
        ));
        self.blocks[entry]
            .statements
            .push(crate::ir::Statement::Assign(end_place.clone(), crate::ir::Rvalue::Use(end_op)));
        self.blocks[entry].terminator = Some(crate::ir::Terminator::Goto(header));

        self.current_block = Some(header);
        let bool_ty = self.tcx.intern(TyKind::Bool);
        let cmp_temp = self.new_temp(Some("_for_cond".to_string()), bool_ty);
        let cmp_place = crate::ir::Place::local(cmp_temp);
        let cmp_op = if *inclusive { crate::ir::BinOp::Le } else { crate::ir::BinOp::Lt };
        self.blocks[header].statements.push(crate::ir::Statement::Assign(
            cmp_place.clone(),
            crate::ir::Rvalue::BinaryOp(
                cmp_op,
                crate::ir::Operand::Copy(index_place.clone()),
                crate::ir::Operand::Copy(end_place.clone()),
            ),
        ));
        self.blocks[header].terminator = Some(crate::ir::Terminator::SwitchInt {
            discr: crate::ir::Operand::Copy(cmp_place),
            targets: vec![(1, body_block)],
            otherwise: exit,
        });

        let bound_name = match pattern {
            omni_types::ast::Pattern::Binding(name) => Some(name.clone()),
            omni_types::ast::Pattern::Wildcard => None,
            _ => {
                return Err(
                    "MIR lowering error: for-range pattern must be a binding or wildcard".into()
                )
            }
        };

        self.current_block = Some(body_block);
        self.loops.push(LoopContext {
            label: label.map(str::to_owned),
            continue_block: step_block,
            break_block: exit,
            result_local: None,
            result_ty: Some(self.tcx.intern(TyKind::Unit)),
        });
        let saved_scope = self.scope.clone();
        if let Some(name) = bound_name {
            self.scope.insert(name, index_local);
        }
        self.lower_expr(body)?;
        self.scope = saved_scope;

        if let Some(block) = self.current_block {
            if self.blocks[block].terminator.is_none() {
                self.blocks[block].terminator = Some(crate::ir::Terminator::Goto(step_block));
            }
        }
        self.loops.pop().expect("for loop context balanced");

        self.current_block = Some(step_block);
        self.blocks[step_block].statements.push(crate::ir::Statement::Assign(
            index_place.clone(),
            crate::ir::Rvalue::BinaryOp(
                crate::ir::BinOp::Add,
                crate::ir::Operand::Copy(index_place.clone()),
                crate::ir::Operand::Constant(crate::ir::Constant::Lit(omni_types::ast::Lit::Int(
                    1,
                ))),
            ),
        ));
        self.blocks[step_block].terminator = Some(crate::ir::Terminator::Goto(header));

        self.current_block = Some(exit);
        Ok(None)
    }

    fn lower_if_expression(
        &mut self,
        condition: &omni_types::ast::Expr,
        then_branch: &omni_types::ast::Expr,
        else_branch: Option<&omni_types::ast::Expr>,
    ) -> Result<Option<(crate::ir::Operand, Ty)>, String> {
        let (cond_op, cond_ty) = self
            .lower_expr(condition)?
            .ok_or_else(|| "MIR lowering error: if condition is Unit".to_string())?;
        let bool_ty = self.tcx.intern(TyKind::Bool);
        if cond_ty != bool_ty {
            return Err(format!(
                "MIR lowering error: if condition has type {:?}, expected Bool",
                cond_ty
            ));
        }
        let entry = self
            .current_block
            .ok_or_else(|| "MIR lowering error: if has no live entry block".to_string())?;
        let then_block = self.new_block();
        let else_block = self.new_block();
        let join_block = self.new_block();
        self.blocks[entry].terminator = Some(crate::ir::Terminator::SwitchInt {
            discr: cond_op,
            targets: vec![(1, then_block)],
            otherwise: else_block,
        });

        self.current_block = Some(then_block);
        let then_result = self.lower_expr(then_branch)?;
        let then_end = self.current_block;
        // A `return` inside the then-branch diverges only that branch; the else
        // branch and the code after the `if` are still reachable, so the
        // function-wide divergence flag is restored rather than left set.
        let then_diverged = self.diverged;
        self.diverged = false;

        let else_result = if let Some(else_expr) = else_branch {
            self.current_block = Some(else_block);
            let result = self.lower_expr(else_expr)?;
            let end = self.current_block;
            (result, end)
        } else {
            (None, Some(else_block))
        };
        let else_diverged = self.diverged;
        // The `if` itself diverges only when *both* arms transfer control away.
        // With no `else` arm the empty else path always falls through to the
        // join block, so the `if` is not a diverging statement.
        self.diverged = then_diverged && else_diverged && else_branch.is_some();

        let result_ty = match (&then_result, &else_result.0) {
            (Some((_, then_ty)), Some((_, else_ty))) if then_ty == else_ty => *then_ty,
            (None, None) => self.tcx.intern(TyKind::Unit),
            // A branch that transfers control away (`return`, `break`,
            // `continue`) yields no value even though `lower_expr` reports the
            // operand's type for a `return`. Treating it as value-producing
            // would reject the ordinary `if c { return 1; }` statement form.
            (Some(_), None) if expr_diverges(then_branch) => self.tcx.intern(TyKind::Unit),
            (Some(_), None) | (None, Some(_)) => {
                return Err("MIR lowering error: non-unit if branch requires an else value".into());
            }
            (Some((_, a)), Some((_, b))) => {
                return Err(format!(
                    "MIR lowering error: if branches have types {:?} and {:?}",
                    a, b
                ));
            }
        };

        let result_local = if result_ty == self.tcx.intern(TyKind::Unit) {
            None
        } else {
            Some(self.new_temp(Some("_if_tmp".to_string()), result_ty))
        };

        if let (Some((operand, _)), Some(end)) = (&then_result, then_end) {
            let place = result_local.map(crate::ir::Place::local);
            if let Some(place) = place {
                self.blocks[end].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Use(operand.clone()),
                ));
            }
            if self.blocks[end].terminator.is_none() {
                self.blocks[end].terminator = Some(crate::ir::Terminator::Goto(join_block));
            }
        } else if let Some(end) = then_end {
            if self.blocks[end].terminator.is_none() {
                self.blocks[end].terminator = Some(crate::ir::Terminator::Goto(join_block));
            }
        }

        if let (Some((operand, _)), Some(end)) = (&else_result.0, else_result.1) {
            let place = result_local.map(crate::ir::Place::local);
            if let Some(place) = place {
                self.blocks[end].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Use(operand.clone()),
                ));
            }
            if self.blocks[end].terminator.is_none() {
                self.blocks[end].terminator = Some(crate::ir::Terminator::Goto(join_block));
            }
        } else if let Some(end) = else_result.1 {
            if self.blocks[end].terminator.is_none() {
                self.blocks[end].terminator = Some(crate::ir::Terminator::Goto(join_block));
            }
        }

        self.current_block = Some(join_block);
        if let Some(local) = result_local {
            let place = crate::ir::Place::local(local);
            Ok(Some((crate::ir::Operand::Copy(place), result_ty)))
        } else {
            Ok(None)
        }
    }

    fn lower_match_expression(
        &mut self,
        scrutinee: &omni_types::ast::Expr,
        arms: &[omni_types::ast::MatchArm],
    ) -> Result<Option<(crate::ir::Operand, Ty)>, String> {
        if arms.is_empty() {
            return Ok(None);
        }
        if arms.iter().any(|arm| arm.guard.is_some()) {
            return Err(
                "MIR lowering error: guarded match arms require guard-aware dispatch".into()
            );
        }

        let (scrutinee_op, scrutinee_ty) = self
            .lower_expr(scrutinee)?
            .ok_or_else(|| "MIR lowering error: match scrutinee is Unit".to_string())?;

        let entry = self
            .current_block
            .ok_or_else(|| "MIR lowering error: match has no live entry block".to_string())?;

        let scrutinee_place = match scrutinee_op {
            crate::ir::Operand::Copy(place) | crate::ir::Operand::Move(place) => place,
            operand => {
                let local = self.new_temp(Some("_match_scrutinee".to_string()), scrutinee_ty);
                let place = crate::ir::Place::local(local);
                self.blocks[entry].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Use(operand),
                ));
                place
            }
        };

        let arm_blocks = (0..arms.len()).map(|_| self.new_block()).collect::<Vec<_>>();
        let otherwise = self.new_block();
        let join = self.new_block();

        let mut targets = Vec::new();
        let mut wildcard_target = None;
        for (index, arm) in arms.iter().enumerate() {
            self.collect_match_targets(
                &arm.pattern,
                arm_blocks[index],
                &mut targets,
                &mut wildcard_target,
            )?;
        }
        let otherwise_block = wildcard_target.unwrap_or(otherwise);
        self.blocks[entry].terminator = Some(crate::ir::Terminator::SwitchInt {
            discr: crate::ir::Operand::Copy(scrutinee_place.clone()),
            targets,
            otherwise: otherwise_block,
        });

        let mut result_ty = None;
        let mut result_local = None;
        let mut any_arm_diverged = false;

        for (index, arm) in arms.iter().enumerate() {
            self.current_block = Some(arm_blocks[index]);
            let saved_scope = self.scope.clone();
            if let Some(name) = simple_pattern_binding(&arm.pattern) {
                self.scope.insert(name, scrutinee_place.local);
            }

            let body = self.lower_expr(&arm.body)?;
            self.scope = saved_scope;
            // Each arm is an independent path: a `return` in one arm must not
            // mark the other arms, or the code after the `match`, as diverged.
            let arm_diverged = self.diverged;
            self.diverged = false;
            any_arm_diverged |= arm_diverged;

            if let Some((_, ty)) = &body {
                if let Some(expected) = result_ty {
                    if expected != *ty {
                        return Err(format!(
                            "MIR lowering error: match arm {} has type {:?}, expected {:?}",
                            index, ty, expected
                        ));
                    }
                } else {
                    result_ty = Some(*ty);
                    let local = self.new_temp(Some("_match_tmp".to_string()), *ty);
                    result_local = Some(local);
                }
            } else if result_ty.is_none() {
                result_ty = Some(self.tcx.intern(TyKind::Unit));
            }

            if let (Some((operand, _)), Some(local)) = (body, result_local) {
                if let Some(block) = self.current_block {
                    let place = crate::ir::Place::local(local);
                    self.blocks[block]
                        .statements
                        .push(crate::ir::Statement::Assign(place, crate::ir::Rvalue::Use(operand)));
                    if self.blocks[block].terminator.is_none() {
                        self.blocks[block].terminator = Some(crate::ir::Terminator::Goto(join));
                    }
                }
            } else if let Some(block) = self.current_block {
                if self.blocks[block].terminator.is_none() {
                    self.blocks[block].terminator = Some(crate::ir::Terminator::Goto(join));
                }
            }
        }

        // The implicit `otherwise` block only needs an `Unreachable` terminator
        // when it is still the switch's fallthrough target. When an arm supplies
        // the wildcard, `otherwise_block` points at that arm instead and `otherwise`
        // is left with no predecessor edge; it must still be terminated, or the
        // verifier rejects the function with "BasicBlock N lacks a valid
        // terminator".
        if self.blocks[otherwise].terminator.is_none() {
            self.blocks[otherwise].terminator = Some(crate::ir::Terminator::Unreachable);
        }

        // The `match` itself diverges only when *every* arm transfers control
        // away and there is no fallthrough target; otherwise the join block is
        // reachable and execution continues after the `match`.
        self.diverged = any_arm_diverged && wildcard_target.is_none();
        self.current_block = Some(join);
        match result_local {
            Some(local) => Ok(Some((
                crate::ir::Operand::Copy(crate::ir::Place::local(local)),
                result_ty.expect("result type exists"),
            ))),
            None => Ok(None),
        }
    }

    fn collect_match_targets(
        &self,
        pattern: &omni_types::ast::Pattern,
        target: crate::ir::BasicBlock,
        targets: &mut Vec<(u64, crate::ir::BasicBlock)>,
        wildcard_target: &mut Option<crate::ir::BasicBlock>,
    ) -> Result<(), String> {
        match pattern {
            omni_types::ast::Pattern::Wildcard | omni_types::ast::Pattern::Binding(_) => {
                if wildcard_target.replace(target).is_some() {
                    return Err("MIR lowering error: multiple wildcard match arms".into());
                }
            }
            omni_types::ast::Pattern::Or(patterns) => {
                for pattern in patterns {
                    self.collect_match_targets(pattern, target, targets, wildcard_target)?;
                }
            }
            omni_types::ast::Pattern::Lit(lit) => {
                let value = match lit {
                    omni_types::ast::Lit::Bool(v) => u64::from(*v),
                    omni_types::ast::Lit::Int(v) => *v as u64,
                    omni_types::ast::Lit::Byte(v) => u64::from(*v),
                    omni_types::ast::Lit::Char(v) => *v as u64,
                    _ => return Err(format!(
                        "MIR lowering error: literal pattern {:?} is not representable by SwitchInt",
                        lit
                    )),
                };
                targets.push((value, target));
            }
            other => {
                return Err(format!(
                    "MIR lowering error: match pattern {:?} requires aggregate/discriminant lowering",
                    other
                ));
            }
        }
        Ok(())
    }

    fn lower_short_circuit(
        &mut self,
        op: omni_types::ast::BinOp,
        lhs: &omni_types::ast::Expr,
        rhs: &omni_types::ast::Expr,
    ) -> Result<Option<(crate::ir::Operand, Ty)>, String> {
        let (lhs_op, lhs_ty) = self
            .lower_expr(lhs)?
            .ok_or_else(|| "MIR lowering error: logical lhs is Unit".to_string())?;
        let bool_ty = self.tcx.intern(TyKind::Bool);
        if lhs_ty != bool_ty {
            return Err("MIR lowering error: logical operators require Bool lhs".into());
        }

        let entry = self
            .current_block
            .ok_or_else(|| "MIR lowering error: logical lhs terminated control flow".to_string())?;
        let rhs_block = self.new_block();
        let short_block = self.new_block();
        let join_block = self.new_block();
        let result_local = self.new_temp(Some("_logic_tmp".to_string()), bool_ty);
        let result_place = crate::ir::Place::local(result_local);

        let short_value = match op {
            omni_types::ast::BinOp::LogicalAnd => 0,
            omni_types::ast::BinOp::LogicalOr => 1,
            _ => unreachable!("lower_short_circuit only receives logical operators"),
        };
        let branch_value = 1u64 - short_value;

        self.blocks[entry].terminator = Some(crate::ir::Terminator::SwitchInt {
            discr: lhs_op,
            targets: vec![(branch_value, rhs_block)],
            otherwise: short_block,
        });

        self.current_block = Some(rhs_block);
        let (rhs_op, rhs_ty) = self
            .lower_expr(rhs)?
            .ok_or_else(|| "MIR lowering error: logical rhs is Unit".to_string())?;
        if rhs_ty != bool_ty {
            return Err("MIR lowering error: logical operators require Bool rhs".into());
        }
        let rhs_end = self
            .current_block
            .ok_or_else(|| "MIR lowering error: logical rhs terminated control flow".to_string())?;
        self.blocks[rhs_end].statements.push(crate::ir::Statement::Assign(
            result_place.clone(),
            crate::ir::Rvalue::Use(rhs_op),
        ));
        if self.blocks[rhs_end].terminator.is_none() {
            self.blocks[rhs_end].terminator = Some(crate::ir::Terminator::Goto(join_block));
        }

        self.current_block = Some(short_block);
        self.blocks[short_block].statements.push(crate::ir::Statement::Assign(
            result_place.clone(),
            crate::ir::Rvalue::Use(crate::ir::Operand::Constant(crate::ir::Constant::Lit(
                omni_types::ast::Lit::Bool(short_value == 1),
            ))),
        ));
        self.blocks[short_block].terminator = Some(crate::ir::Terminator::Goto(join_block));

        self.current_block = Some(join_block);
        Ok(Some((crate::ir::Operand::Copy(result_place), bool_ty)))
    }

    fn lower_expr(
        &mut self,
        expr: &omni_types::ast::Expr,
    ) -> Result<Option<(crate::ir::Operand, Ty)>, String> {
        if self.current_block.is_none() {
            return Err("MIR lowering error: expression evaluated after control flow terminated"
                .to_string());
        }

        match expr {
            omni_types::ast::Expr::Literal(lit) => {
                let ty = self.literal_ty(lit);
                Ok(Some((crate::ir::Operand::Constant(crate::ir::Constant::Lit(lit.clone())), ty)))
            }
            omni_types::ast::Expr::Var(name) => {
                let local =
                    self.scope.get(name).copied().ok_or_else(|| {
                        format!("MIR lowering error: undefined variable '{}'", name)
                    })?;
                let ty = self.local_ty(local)?;
                Ok(Some((crate::ir::Operand::Copy(crate::ir::Place::local(local)), ty)))
            }
            omni_types::ast::Expr::Binary { op, lhs, rhs }
                if matches!(
                    op,
                    omni_types::ast::BinOp::LogicalAnd | omni_types::ast::BinOp::LogicalOr
                ) =>
            {
                self.lower_short_circuit(op.clone(), lhs, rhs)
            }
            omni_types::ast::Expr::Binary { op, lhs, rhs } => {
                let (lhs_op, lhs_ty) = self
                    .lower_expr(lhs)?
                    .ok_or_else(|| "MIR lowering error: left operand is Unit".to_string())?;
                let (rhs_op, rhs_ty) = self
                    .lower_expr(rhs)?
                    .ok_or_else(|| "MIR lowering error: right operand is Unit".to_string())?;
                if lhs_ty != rhs_ty {
                    return Err(format!(
                        "MIR lowering error: binary operands have incompatible types {:?} and {:?}",
                        lhs_ty, rhs_ty
                    ));
                }
                let float_ty = self.tcx.intern(TyKind::Float);
                if lhs_ty == float_ty
                    && matches!(
                        op,
                        omni_types::ast::BinOp::Rem
                            | omni_types::ast::BinOp::BitAnd
                            | omni_types::ast::BinOp::BitOr
                            | omni_types::ast::BinOp::BitXor
                            | omni_types::ast::BinOp::Shl
                            | omni_types::ast::BinOp::Shr
                    )
                {
                    return Err(format!(
                        "MIR lowering error: operator {:?} is not defined for Float",
                        op
                    ));
                }
                let mir_op = match op {
                    omni_types::ast::BinOp::Add => crate::ir::BinOp::Add,
                    omni_types::ast::BinOp::Sub => crate::ir::BinOp::Sub,
                    omni_types::ast::BinOp::Mul => crate::ir::BinOp::Mul,
                    omni_types::ast::BinOp::Div => crate::ir::BinOp::Div,
                    omni_types::ast::BinOp::Rem => crate::ir::BinOp::Rem,
                    omni_types::ast::BinOp::BitAnd => crate::ir::BinOp::BitAnd,
                    omni_types::ast::BinOp::BitOr => crate::ir::BinOp::BitOr,
                    omni_types::ast::BinOp::BitXor => crate::ir::BinOp::BitXor,
                    omni_types::ast::BinOp::Shl => crate::ir::BinOp::Shl,
                    omni_types::ast::BinOp::Shr => crate::ir::BinOp::Shr,
                    omni_types::ast::BinOp::Eq => crate::ir::BinOp::Eq,
                    omni_types::ast::BinOp::Ne => crate::ir::BinOp::Ne,
                    omni_types::ast::BinOp::Lt => crate::ir::BinOp::Lt,
                    omni_types::ast::BinOp::Le => crate::ir::BinOp::Le,
                    omni_types::ast::BinOp::Gt => crate::ir::BinOp::Gt,
                    omni_types::ast::BinOp::Ge => crate::ir::BinOp::Ge,
                    omni_types::ast::BinOp::LogicalAnd | omni_types::ast::BinOp::LogicalOr => {
                        return Err(
                            "MIR lowering error: logical operators require short-circuit lowering"
                                .into(),
                        );
                    }
                };
                let result_ty = match op {
                    omni_types::ast::BinOp::Eq
                    | omni_types::ast::BinOp::Ne
                    | omni_types::ast::BinOp::Lt
                    | omni_types::ast::BinOp::Le
                    | omni_types::ast::BinOp::Gt
                    | omni_types::ast::BinOp::Ge
                    | omni_types::ast::BinOp::LogicalAnd
                    | omni_types::ast::BinOp::LogicalOr => self.tcx.intern(TyKind::Bool),
                    omni_types::ast::BinOp::Add
                    | omni_types::ast::BinOp::Sub
                    | omni_types::ast::BinOp::Mul
                    | omni_types::ast::BinOp::Div
                    | omni_types::ast::BinOp::Rem
                    | omni_types::ast::BinOp::BitAnd
                    | omni_types::ast::BinOp::BitOr
                    | omni_types::ast::BinOp::BitXor
                    | omni_types::ast::BinOp::Shl
                    | omni_types::ast::BinOp::Shr => lhs_ty,
                };
                let curr_block = self.current_block.ok_or_else(|| {
                    "MIR lowering error: binary expression has no live continuation block"
                        .to_string()
                })?;
                let temp_local = self.new_temp(Some("_bin_tmp".to_string()), result_ty);
                let place = crate::ir::Place::local(temp_local);
                self.blocks[curr_block].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::BinaryOp(mir_op, lhs_op, rhs_op),
                ));
                Ok(Some((crate::ir::Operand::Copy(place.clone()), result_ty)))
            }
            omni_types::ast::Expr::Unary { op, expr } => {
                let (inner_op, inner_ty) = self
                    .lower_expr(expr)?
                    .ok_or_else(|| "MIR lowering error: unary operand is Unit".to_string())?;
                match op {
                    omni_types::ast::UnOp::Deref => {
                        let inner_place = match inner_op {
                            crate::ir::Operand::Copy(place) | crate::ir::Operand::Move(place) => place,
                            other => {
                                let block = self.current_block.ok_or_else(|| {
                                    "MIR lowering error: dereference has no live continuation block"
                                        .to_string()
                                })?;
                                let temp = self.new_temp(Some("_deref_ref".to_string()), inner_ty);
                                let place = crate::ir::Place::local(temp);
                                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                                    place.clone(),
                                    crate::ir::Rvalue::Use(other),
                                ));
                                place
                            }
                        };
                        let ty = self.projected_ty(
                            inner_ty,
                            &[crate::ir::Projection::Deref],
                        )?;
                        let place = inner_place.project(crate::ir::Projection::Deref);
                        Ok(Some((crate::ir::Operand::Copy(place), ty)))
                    }
                    omni_types::ast::UnOp::BorrowShared | omni_types::ast::UnOp::BorrowMut => {
                        let (target, target_ty) = self.lower_assign_place(expr)?;
                        let ref_ty = self.tcx.intern(TyKind::Reference {
                            lifetime: None,
                            mutable: matches!(op, omni_types::ast::UnOp::BorrowMut),
                            inner: target_ty,
                        });
                        let block = self.current_block.ok_or_else(|| {
                            "MIR lowering error: borrow has no live continuation block".to_string()
                        })?;
                        let temp = self.new_temp(Some("_ref_tmp".to_string()), ref_ty);
                        let place = crate::ir::Place::local(temp);
                        self.blocks[block].statements.push(crate::ir::Statement::Assign(
                            place.clone(),
                            crate::ir::Rvalue::Reference {
                                place: target,
                                mutable: matches!(op, omni_types::ast::UnOp::BorrowMut),
                                ty: ref_ty,
                            },
                        ));
                        Ok(Some((crate::ir::Operand::Copy(place), ref_ty)))
                    }
                    omni_types::ast::UnOp::Neg
                    | omni_types::ast::UnOp::Not
                    | omni_types::ast::UnOp::BitNot => {
                        let bool_ty = self.tcx.intern(TyKind::Bool);
                        let int_ty = self.tcx.intern(TyKind::Int);
                        match op {
                            omni_types::ast::UnOp::Neg
                                if inner_ty != int_ty
                                    && inner_ty != self.tcx.intern(TyKind::Float) =>
                            {
                                return Err(format!(
                                    "MIR lowering error: unary operator requires Int or Float for '-', found {:?}",
                                    inner_ty
                                ));
                            }
                            omni_types::ast::UnOp::Not if inner_ty != bool_ty => {
                                return Err(format!(
                                    "MIR lowering error: unary '!' requires Bool, found {:?}",
                                    inner_ty
                                ));
                            }
                            omni_types::ast::UnOp::BitNot if inner_ty != int_ty => {
                                return Err(format!(
                                    "MIR lowering error: unary '~' requires Int, found {:?}",
                                    inner_ty
                                ));
                            }
                            _ => {}
                        }
                        let mir_op = match op {
                            omni_types::ast::UnOp::Neg => crate::ir::UnOp::Neg,
                            omni_types::ast::UnOp::Not => crate::ir::UnOp::Not,
                            omni_types::ast::UnOp::BitNot => crate::ir::UnOp::BitNot,
                            _ => unreachable!(),
                        };
                        let curr_block = self.current_block.ok_or_else(|| {
                            "MIR lowering error: unary expression has no live continuation block"
                                .to_string()
                        })?;
                        let temp_local = self.new_temp(Some("_un_tmp".to_string()), inner_ty);
                        let place = crate::ir::Place::local(temp_local);
                        self.blocks[curr_block].statements.push(crate::ir::Statement::Assign(
                            place.clone(),
                            crate::ir::Rvalue::UnaryOp(mir_op, inner_op),
                        ));
                        Ok(Some((crate::ir::Operand::Copy(place), inner_ty)))
                    }
                }
            }
            omni_types::ast::Expr::Let { pattern, ty, init, body } => {
                let (init_op, init_ty) = self.lower_expr(init)?.ok_or_else(|| {
                    "MIR lowering error: Unit-valued let initializers are not materialized"
                        .to_string()
                })?;
                let var_ty = if let Some(spec) = ty {
                    self.tcx.lower_type_spec(spec, self.subst)
                } else {
                    init_ty
                };
                if var_ty != init_ty {
                    return Err(format!(
                        "MIR lowering error: let initializer has type {:?}, declared type is {:?}",
                        init_ty, var_ty
                    ));
                }
                let saved_scope = self.scope.clone();
                let bindings = self.bind_let_pattern(pattern, init_op, var_ty)?;
                for (name, local) in bindings {
                    self.scope.insert(name, local);
                }
                let result = self.lower_expr(body);
                self.scope = saved_scope;
                result
            }
            omni_types::ast::Expr::Block(exprs) => {
                let mut last = None;
                for expr in exprs {
                    if self.diverged {
                        // The whole function has left through a `return`, so no
                        // later statement in this block is reachable.
                        break;
                    }
                    if self.current_block.is_none() {
                        // `break`/`continue` only end the current basic block
                        // *inside a loop*; control still has somewhere to go, so
                        // the remaining statements are lowered into a fresh
                        // block rather than dropped. Outside a loop the block
                        // would be unreachable, which is left to the verifier.
                        let resume = self.new_block();
                        self.current_block = Some(resume);
                    }
                    last = self.lower_expr(expr)?;
                }
                Ok(last)
            }
            omni_types::ast::Expr::UnsafeBlock { body } => {
                // The body is lowered exactly as an ordinary block, so UNSAFE-0001
                // holds at the MIR level too: `unsafe` waives no check, and the
                // body must still satisfy the verifier and ownership rules.
                //
                // Every block the body reaches is recorded, which is what makes
                // the unsafe context visible for a UNSAFE-0002 audit. The region
                // is tracked by block *range*: lowering the body can create new
                // blocks, and those are inside the region exactly as much as the
                // block it was entered from.
                let region_start = self.blocks.len();
                if let Some(entry) = self.current_block {
                    self.unsafe_blocks.push(entry);
                }
                let result = self.lower_expr(body)?;
                for raw in region_start..self.blocks.len() {
                    let block = crate::ir::BasicBlock::from_usize(raw);
                    if !self.unsafe_blocks.contains(&block) {
                        self.unsafe_blocks.push(block);
                    }
                }
                Ok(result)
            }
            omni_types::ast::Expr::Tuple(elements) => {
                let mut operands = Vec::with_capacity(elements.len());
                let mut types = Vec::with_capacity(elements.len());
                for element in elements {
                    let (operand, ty) = self
                        .lower_expr(element)?
                        .ok_or_else(|| "MIR lowering error: tuple element is Unit".to_string())?;
                    operands.push(operand);
                    types.push(ty);
                }
                let tuple_ty = self.tcx.intern(TyKind::Tuple(types));
                let block = self
                    .current_block
                    .ok_or_else(|| "MIR lowering error: tuple has no live block".to_string())?;
                let local = self.new_temp(Some("_tuple_tmp".to_string()), tuple_ty);
                let place = crate::ir::Place::local(local);
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Aggregate {
                        kind: crate::ir::AggregateKind::Tuple,
                        operands,
                        ty: tuple_ty,
                    },
                ));
                Ok(Some((crate::ir::Operand::Copy(place.clone()), tuple_ty)))
            }
            omni_types::ast::Expr::Array(elements) => {
                if elements.is_empty() {
                    return Err(
                        "MIR lowering error: empty array requires contextual element type".into()
                    );
                }
                let mut operands = Vec::with_capacity(elements.len());
                let mut elem_ty = None;
                for element in elements {
                    let (operand, ty) = self
                        .lower_expr(element)?
                        .ok_or_else(|| "MIR lowering error: array element is Unit".to_string())?;
                    if let Some(expected) = elem_ty {
                        if expected != ty {
                            return Err(format!(
                                "MIR lowering error: array elements have incompatible types {:?} and {:?}",
                                expected, ty
                            ));
                        }
                    } else {
                        elem_ty = Some(ty);
                    }
                    operands.push(operand);
                }
                let elem_ty = elem_ty.expect("non-empty array has an element type");
                let array_ty = self.tcx.intern(TyKind::Array(elem_ty, elements.len()));
                let block = self
                    .current_block
                    .ok_or_else(|| "MIR lowering error: array has no live block".to_string())?;
                let local = self.new_temp(Some("_array_tmp".to_string()), array_ty);
                let place = crate::ir::Place::local(local);
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Aggregate {
                        kind: crate::ir::AggregateKind::Array,
                        operands,
                        ty: array_ty,
                    },
                ));
                Ok(Some((crate::ir::Operand::Copy(place.clone()), array_ty)))
            }
            omni_types::ast::Expr::Field { expr, field } => {
                let (base, base_ty) = self
                    .lower_expr(expr)?
                    .ok_or_else(|| "MIR lowering error: field base is Unit".to_string())?;
                let result_ty = match self.tcx.get(base_ty).clone() {
                    TyKind::Struct(name, args) => {
                        let def = self.struct_defs.get(&name).ok_or_else(|| {
                            format!("MIR lowering error: unknown struct '{}'", name)
                        })?;
                        let mut field_env = omni_types::checker::SubstEnv::new();
                        for (param, arg) in def.type_params.iter().zip(args.iter()) {
                            field_env.insert(param.clone(), *arg);
                        }
                        let mut found = None;
                        for f in &def.fields {
                            if f.name == *field {
                                found = Some(self.tcx.lower_type_spec(&f.ty, &field_env));
                                break;
                            }
                        }
                        found.ok_or_else(|| {
                            format!("MIR lowering error: field '{}' not found on '{}'", field, name)
                        })?
                    }
                    TyKind::Tuple(types) => {
                        let index = field.parse::<usize>().map_err(|_| {
                            "MIR lowering error: tuple field must be a numeric index".to_string()
                        })?;
                        *types.get(index).ok_or_else(|| {
                            format!("MIR lowering error: tuple field index {} out of bounds", index)
                        })?
                    }
                    _ => {
                        return Err(format!(
                        "MIR lowering error: field projection requires struct or tuple, found {:?}",
                        self.tcx.get(base_ty)
                    ))
                    }
                };
                let block = self
                    .current_block
                    .ok_or_else(|| "MIR lowering error: field has no live block".to_string())?;
                let local = self.new_temp(Some("_field_tmp".to_string()), result_ty);
                let place = crate::ir::Place::local(local);
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Field { base, field: field.clone(), ty: result_ty },
                ));
                Ok(Some((crate::ir::Operand::Copy(place.clone()), result_ty)))
            }
            omni_types::ast::Expr::Index { expr, index } => {
                let (base, base_ty) = self
                    .lower_expr(expr)?
                    .ok_or_else(|| "MIR lowering error: index base is Unit".to_string())?;
                let (index_op, index_ty) = self
                    .lower_expr(index)?
                    .ok_or_else(|| "MIR lowering error: index is Unit".to_string())?;
                let int_ty = self.tcx.intern(TyKind::Int);
                if index_ty != int_ty {
                    return Err(format!(
                        "MIR lowering error: index expression must be Int, found {:?}",
                        index_ty
                    ));
                }
                let (result_ty, index_local_for_final) = match self.tcx.get(base_ty).clone() {
                    TyKind::Array(elem, length) => {
                        // Check if this is a constant or dynamic index
                        let index_local = match index.as_ref() {
                            omni_types::ast::Expr::Literal(omni_types::ast::Lit::Int(n))
                                if *n >= 0 =>
                            {
                                // Constant index: check bounds at compile time
                                if *n >= length as i64 {
                                    return Err(format!(
                                        "MIR lowering error: array index {} out of bounds for length {}",
                                        n, length
                                    ));
                                }
                                None
                            }
                            _ => {
                                // Dynamic index: insert runtime bounds check
                                let index_local =
                                    self.new_temp(Some("_index_local".to_string()), index_ty);
                                let block = self.current_block.ok_or_else(|| {
                                    "MIR lowering error: index has no live block".to_string()
                                })?;
                                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                                    crate::ir::Place::local(index_local),
                                    crate::ir::Rvalue::Use(index_op.clone()),
                                ));
                                // Insert bounds check for dynamic index
                                self.blocks[block].statements.push(
                                    crate::ir::Statement::BoundsCheck {
                                        index: index_local,
                                        length,
                                    },
                                );
                                Some(index_local)
                            }
                        };
                        (elem, index_local)
                    }
                    TyKind::Tuple(types) => {
                        let position = match index.as_ref() {
                            omni_types::ast::Expr::Literal(omni_types::ast::Lit::Int(n))
                                if *n >= 0 =>
                            {
                                *n as usize
                            }
                            _ => {
                                return Err(
                                    "MIR lowering error: tuple index must be a constant Int"
                                        .to_string(),
                                )
                            }
                        };
                        (
                            *types.get(position).ok_or_else(|| {
                                format!(
                                    "MIR lowering error: tuple index {} out of bounds",
                                    position
                                )
                            })?,
                            None,
                        )
                    }
                    other => {
                        return Err(format!(
                            "MIR lowering error: indexing requires Array or Tuple, found {:?}",
                            other
                        ))
                    }
                };
                let block = self
                    .current_block
                    .ok_or_else(|| "MIR lowering error: index has no live block".to_string())?;
                let local = self.new_temp(Some("_index_tmp".to_string()), result_ty);
                let place = crate::ir::Place::local(local);

                // Use the index local if it was created for dynamic indexing, otherwise use the original operand
                let final_index_op = match index_local_for_final {
                    None => {
                        // Constant index: use original operand
                        index_op
                    }
                    Some(index_local) => {
                        // Dynamic index: use the local that was assigned and bounds-checked
                        crate::ir::Operand::Copy(crate::ir::Place::local(index_local))
                    }
                };

                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Index { base, index: final_index_op, ty: result_ty },
                ));
                Ok(Some((crate::ir::Operand::Copy(place.clone()), result_ty)))
            }
            omni_types::ast::Expr::Struct { name, generic_args, fields } => {
                let args = generic_args
                    .iter()
                    .map(|spec| self.tcx.lower_type_spec(spec, self.subst))
                    .collect::<Vec<_>>();
                let ty = self.tcx.intern(TyKind::Struct(name.clone(), args));
                let mut lowered_fields = Vec::with_capacity(fields.len());
                for (field_name, value) in fields {
                    let (operand, _) = self.lower_expr(value)?.ok_or_else(|| {
                        format!("MIR lowering error: struct field '{}' is Unit", field_name)
                    })?;
                    lowered_fields.push((field_name.clone(), operand));
                }
                let block = self.current_block.ok_or_else(|| {
                    "MIR lowering error: struct literal has no live block".to_string()
                })?;
                let local = self.new_temp(Some("_struct_tmp".to_string()), ty);
                let place = crate::ir::Place::local(local);
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Struct { name: name.clone(), fields: lowered_fields, ty },
                ));
                Ok(Some((crate::ir::Operand::Copy(place.clone()), ty)))
            }
            omni_types::ast::Expr::EnumVariant { enum_name, variant, generic_args, args } => {
                let generic_tys = generic_args
                    .iter()
                    .map(|spec| self.tcx.lower_type_spec(spec, self.subst))
                    .collect::<Vec<_>>();
                let ty = self.tcx.intern(TyKind::Enum(enum_name.clone(), generic_tys));
                let mut operands = Vec::with_capacity(args.len());
                for arg in args {
                    let (operand, _) = self.lower_expr(arg)?.ok_or_else(|| {
                        format!(
                            "MIR lowering error: enum constructor '{}::{}' contains Unit payload",
                            enum_name, variant
                        )
                    })?;
                    operands.push(operand);
                }
                let block = self.current_block.ok_or_else(|| {
                    "MIR lowering error: enum constructor has no live block".to_string()
                })?;
                let local = self.new_temp(Some("_enum_tmp".to_string()), ty);
                let place = crate::ir::Place::local(local);
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::EnumVariant {
                        enum_name: enum_name.clone(),
                        variant: variant.clone(),
                        operands,
                        ty,
                    },
                ));
                Ok(Some((crate::ir::Operand::Copy(place.clone()), ty)))
            }
            omni_types::ast::Expr::Call { func, generic_args: _, args } => {
                let (param_tys, ret_ty) = self.fn_sigs.get(func).cloned().ok_or_else(|| {
                    format!(
                        "MIR lowering error: call target '{}' is not present in monomorphized program",
                        func
                    )
                })?;
                if args.len() != param_tys.len() {
                    return Err(format!(
                        "MIR lowering error in call '{}': expected {} arguments, found {}",
                        func,
                        param_tys.len(),
                        args.len()
                    ));
                }

                let mut arg_ops = Vec::with_capacity(args.len());
                for (arg, expected_ty) in args.iter().zip(param_tys.iter()) {
                    let (arg_op, arg_ty) = self.lower_expr(arg)?.ok_or_else(|| {
                        "MIR lowering error: Unit-valued call argument".to_string()
                    })?;
                    if arg_ty != *expected_ty {
                        return Err(format!(
                            "MIR lowering error in call '{}': expected argument type {:?}, found {:?}",
                            func, expected_ty, arg_ty
                        ));
                    }
                    arg_ops.push(arg_op);
                }

                let curr_block = self.current_block.ok_or_else(|| {
                    "MIR lowering error: call has no live continuation block".to_string()
                })?;
                let next_block = self.new_block();
                let destination = if ret_ty == self.tcx.intern(TyKind::Unit) {
                    None
                } else {
                    let dest_local = self.new_temp(Some("_call_dest".to_string()), ret_ty);
                    Some(crate::ir::Place::local(dest_local))
                };

                self.blocks[curr_block].terminator = Some(crate::ir::Terminator::Call {
                    func: crate::ir::Operand::Constant(crate::ir::Constant::FnRef(func.clone())),
                    args: arg_ops,
                    destination: destination.clone(),
                    target: next_block,
                    cleanup: None,
                });
                self.current_block = Some(next_block);

                match destination {
                    Some(place) => Ok(Some((crate::ir::Operand::Copy(place.clone()), ret_ty))),
                    None => Ok(None),
                }
            }
            omni_types::ast::Expr::Assign { target, value } => {
                let (target_place, target_ty) = self.lower_assign_place(target)?;
                let (value_op, value_ty) = self
                    .lower_expr(value)?
                    .ok_or_else(|| "MIR lowering error: assignment value is Unit".to_string())?;
                if target_ty != value_ty {
                    return Err(format!(
                        "MIR lowering error: assignment to {} expects {:?}, found {:?}",
                        target_place, target_ty, value_ty
                    ));
                }
                let block = self.current_block.ok_or_else(|| {
                    "MIR lowering error: assignment has no live block".to_string()
                })?;
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    target_place.clone(),
                    crate::ir::Rvalue::Use(value_op),
                ));
                Ok(Some((crate::ir::Operand::Copy(target_place.clone()), target_ty)))
            }
            omni_types::ast::Expr::Range { start, end, inclusive } => {
                let (start_op, start_ty) = self
                    .lower_expr(start)?
                    .ok_or_else(|| "MIR lowering error: range start is Unit".to_string())?;
                let (end_op, end_ty) = self
                    .lower_expr(end)?
                    .ok_or_else(|| "MIR lowering error: range end is Unit".to_string())?;
                if start_ty != end_ty {
                    return Err(format!(
                        "MIR lowering error: range endpoints have incompatible types {:?} and {:?}",
                        start_ty, end_ty
                    ));
                }
                if !matches!(self.tcx.get(start_ty), TyKind::Int | TyKind::Byte | TyKind::Char) {
                    return Err(
                        "MIR lowering error: only scalar ranges are representable by current MIR"
                            .into(),
                    );
                }
                let range_ty = self.tcx.intern(TyKind::Range(start_ty));
                let block = self
                    .current_block
                    .ok_or_else(|| "MIR lowering error: range has no live block".to_string())?;
                let local = self.new_temp(Some("_range_tmp".to_string()), range_ty);
                let place = crate::ir::Place::local(local);
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Range {
                        start: start_op,
                        end: end_op,
                        inclusive: *inclusive,
                        ty: range_ty,
                    },
                ));
                Ok(Some((crate::ir::Operand::Copy(place.clone()), range_ty)))
            }
            omni_types::ast::Expr::Cast { expr, ty } => {
                let (operand, from_ty) = self
                    .lower_expr(expr)?
                    .ok_or_else(|| "MIR lowering error: cast source is Unit".to_string())?;
                let to_ty = self.tcx.lower_type_spec(ty, self.subst);
                let scalar = |t: Ty, tcx: &TyCtxt| {
                    matches!(tcx.get(t), TyKind::Int | TyKind::Byte | TyKind::Char | TyKind::Float)
                };
                if !scalar(from_ty, self.tcx) || !scalar(to_ty, self.tcx) {
                    return Err(format!(
                        "MIR lowering error: unsupported non-scalar cast {:?} -> {:?}",
                        from_ty, to_ty
                    ));
                }
                let block = self
                    .current_block
                    .ok_or_else(|| "MIR lowering error: cast has no live block".to_string())?;
                let temp = self.new_temp(Some("_cast_tmp".to_string()), to_ty);
                let place = crate::ir::Place::local(temp);
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    place.clone(),
                    crate::ir::Rvalue::Cast { operand, from: from_ty, to: to_ty },
                ));
                Ok(Some((crate::ir::Operand::Copy(place.clone()), to_ty)))
            }
            omni_types::ast::Expr::CompoundAssign { op, target, value } => {
                let omni_types::ast::Expr::Var(name) = target.as_ref() else {
                    return Err("MIR lowering error: compound assignment target must be a local variable in the native backend".into());
                };
                let target_local = *self.scope.get(name).ok_or_else(|| {
                    format!(
                        "MIR lowering error: compound assignment target '{}' is not bound",
                        name
                    )
                })?;
                let target_place = crate::ir::Place::local(target_local);
                let target_ty = self.local_ty(target_local)?;
                let target_op = crate::ir::Operand::Copy(target_place.clone());
                let (value_op, value_ty) = self.lower_expr(value)?.ok_or_else(|| {
                    "MIR lowering error: compound assignment value is Unit".to_string()
                })?;
                if target_ty != value_ty {
                    return Err(format!(
                        "MIR lowering error: compound assignment operands have incompatible types {:?} and {:?}",
                        target_ty, value_ty
                    ));
                }
                let mir_op = match op {
                    omni_types::ast::AssignOp::Add => crate::ir::BinOp::Add,
                    omni_types::ast::AssignOp::Sub => crate::ir::BinOp::Sub,
                    omni_types::ast::AssignOp::Mul => crate::ir::BinOp::Mul,
                    omni_types::ast::AssignOp::Div => crate::ir::BinOp::Div,
                    omni_types::ast::AssignOp::Rem => crate::ir::BinOp::Rem,
                    omni_types::ast::AssignOp::BitAnd => crate::ir::BinOp::BitAnd,
                    omni_types::ast::AssignOp::BitOr => crate::ir::BinOp::BitOr,
                    omni_types::ast::AssignOp::BitXor => crate::ir::BinOp::BitXor,
                    omni_types::ast::AssignOp::Shl => crate::ir::BinOp::Shl,
                    omni_types::ast::AssignOp::Shr => crate::ir::BinOp::Shr,
                    omni_types::ast::AssignOp::Assign => {
                        return Err(
                            "MIR lowering error: plain assignment is not a compound operation"
                                .into(),
                        );
                    }
                };
                let block = self.current_block.ok_or_else(|| {
                    "MIR lowering error: compound assignment has no live block".to_string()
                })?;
                self.blocks[block].statements.push(crate::ir::Statement::Assign(
                    target_place.clone(),
                    crate::ir::Rvalue::BinaryOp(mir_op, target_op, value_op),
                ));
                Ok(Some((crate::ir::Operand::Copy(target_place), target_ty)))
            }
            omni_types::ast::Expr::If { condition, then_branch, else_branch } => {
                self.lower_if_expression(condition, then_branch, else_branch.as_deref())
            }
            omni_types::ast::Expr::Loop { label, body } => {
                self.lower_loop_expression(label.as_deref(), body)
            }
            omni_types::ast::Expr::While { label, condition, body } => {
                self.lower_while_expression(label.as_deref(), condition, body)
            }
            omni_types::ast::Expr::For { label, pattern, iterable, body } => {
                self.lower_for_expression(label.as_deref(), pattern, iterable, body)
            }
            omni_types::ast::Expr::Match { expr, arms } => self.lower_match_expression(expr, arms),
            omni_types::ast::Expr::Return(opt_expr) => {
                let ret_result = if let Some(inner) = opt_expr {
                    let value = self.lower_expr(inner)?.ok_or_else(|| {
                        "MIR lowering error: explicit return expression evaluates to Unit"
                            .to_string()
                    })?;
                    if value.1 != self.return_ty {
                        return Err(format!(
                            "MIR lowering error: return expression has type {:?}, expected {:?}",
                            value.1, self.return_ty
                        ));
                    }
                    Some(value)
                } else {
                    let unit_ty = self.tcx.intern(TyKind::Unit);
                    if self.return_ty != unit_ty {
                        return Err(format!(
                            "MIR lowering error: bare return requires Unit return type, found {:?}",
                            self.return_ty
                        ));
                    }
                    None
                };

                let curr_block = self.current_block.ok_or_else(|| {
                    "MIR lowering error: return has no live continuation block".to_string()
                })?;
                if let Some((operand, _)) = &ret_result {
                    let ret_p = crate::ir::Place::local(crate::ir::Local::from_usize(0));
                    self.blocks[curr_block].statements.push(crate::ir::Statement::Assign(
                        ret_p,
                        crate::ir::Rvalue::Use(operand.clone()),
                    ));
                }
                self.blocks[curr_block].terminator = Some(crate::ir::Terminator::Return);
                self.current_block = None;
                self.diverged = true;
                Ok(ret_result)
            }
            omni_types::ast::Expr::Break { label, value } => {
                self.lower_break_expression(label.as_deref(), value.as_deref())
            }
            omni_types::ast::Expr::Continue { label } => {
                self.lower_continue_expression(label.as_deref())
            }
            unsupported => {
                Err(format!("Unsupported AST expression form for MIR lowering: {:?}", unsupported))
            }
        }
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omni_types::ast::{Expr, GenericFnDef, Lit, TypeSpec};

    /// Collects every assignment destination place in a lowered function.
    fn assigned_places(func: &crate::ir::MirFunction) -> Vec<crate::ir::Place> {
        func.body
            .blocks
            .iter()
            .flat_map(|b| b.statements.iter())
            .filter_map(|s| match s {
                crate::ir::Statement::Assign(place, _) => Some(place.clone()),
                _ => None,
            })
            .collect()
    }

    /// Lowers a function whose body is `body`, with `params` as the bound names.
    fn lower_with(
        params: Vec<(String, TypeSpec)>,
        body: Expr,
    ) -> Result<crate::ir::MirFunction, String> {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "f".to_string(),
                type_params: vec![],
                bounds: vec![],
                params,
                return_type: TypeSpec::Int,
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body,
            }],
        };
        ctx.lower_monomorphized_program(&prog).map(|mut mir| mir.functions.remove(0))
    }

    fn lower(body: Expr) -> crate::ir::MirFunction {
        lower_with(vec![("x".to_string(), TypeSpec::Int)], body).expect("lowering should succeed")
    }

    #[test]
    fn tuple_field_assignment_targets_a_projected_place() {
        // `p.0 = 30` must name p.0, not the whole of p. Before the place model
        // carried a projection chain this destination was not expressible.
        let func = lower_with(
            vec![("p".to_string(), TypeSpec::Tuple(vec![TypeSpec::Int]))],
            Expr::Assign {
                target: Box::new(Expr::Field {
                    expr: Box::new(Expr::Var("p".to_string())),
                    field: "0".to_string(),
                }),
                value: Box::new(Expr::Literal(Lit::Int(30))),
            },
        )
        .expect("lowering should succeed");
        let projected = assigned_places(&func)
            .into_iter()
            .find(|p| p.projections.iter().any(|pr| matches!(pr, crate::ir::Projection::Field(_))))
            .expect("assignment destination must carry a field projection");
        assert_eq!(projected.projections, vec![crate::ir::Projection::Field("0".to_string())]);
    }

    #[test]
    fn array_constant_index_assignment_uses_a_constant_projection() {
        let func = lower_with(
            vec![("a".to_string(), TypeSpec::Array(Box::new(TypeSpec::Int), 3))],
            Expr::Assign {
                target: Box::new(Expr::Index {
                    expr: Box::new(Expr::Var("a".to_string())),
                    index: Box::new(Expr::Literal(Lit::Int(1))),
                }),
                value: Box::new(Expr::Literal(Lit::Int(9))),
            },
        )
        .expect("lowering should succeed");
        assert!(assigned_places(&func)
            .iter()
            .any(|p| p.projections.contains(&crate::ir::Projection::ConstantIndex(1))));
    }

    #[test]
    fn a_whole_local_assignment_produces_no_projection() {
        // The ordinary `x = v` case must not regress into a projected place.
        let func = lower(Expr::Assign {
            target: Box::new(Expr::Var("x".to_string())),
            value: Box::new(Expr::Literal(Lit::Int(1))),
        });
        assert!(assigned_places(&func).iter().all(|p| p.is_local()));
    }

    #[test]
    fn tuple_field_assignment_beyond_the_tuple_is_rejected() {
        // The destination type comes from the same projection authority the read
        // path uses, so an out-of-range position is a lowering error rather than
        // a silently accepted store.
        let err = lower_with(
            vec![("p".to_string(), TypeSpec::Tuple(vec![TypeSpec::Int]))],
            Expr::Assign {
                target: Box::new(Expr::Field {
                    expr: Box::new(Expr::Var("p".to_string())),
                    field: "5".to_string(),
                }),
                value: Box::new(Expr::Literal(Lit::Int(1))),
            },
        )
        .unwrap_err();
        assert!(err.contains("out of bounds"), "unexpected error: {err}");
    }

    #[test]
    fn assignment_type_mismatch_names_the_projected_destination() {
        let err = lower_with(
            vec![("p".to_string(), TypeSpec::Tuple(vec![TypeSpec::Int]))],
            Expr::Assign {
                target: Box::new(Expr::Field {
                    expr: Box::new(Expr::Var("p".to_string())),
                    field: "0".to_string(),
                }),
                value: Box::new(Expr::Literal(Lit::Bool(true))),
            },
        )
        .unwrap_err();
        assert!(err.contains("expects"), "unexpected error: {err}");
    }

    #[test]
    fn constant_array_index_outside_the_array_is_rejected() {
        // A literal subscript is bounds-checked against the array length, so a
        // provably out-of-range store is refused at lowering time.
        let err = lower_with(
            vec![("a".to_string(), TypeSpec::Array(Box::new(TypeSpec::Int), 2))],
            Expr::Assign {
                target: Box::new(Expr::Index {
                    expr: Box::new(Expr::Var("a".to_string())),
                    index: Box::new(Expr::Literal(Lit::Int(7))),
                }),
                value: Box::new(Expr::Literal(Lit::Int(1))),
            },
        )
        .unwrap_err();
        assert!(err.contains("out of bounds"), "unexpected error: {err}");
    }

    #[test]
    fn short_circuit_boolean_lowering_creates_branch_and_join() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "logic".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![("a".to_string(), TypeSpec::Bool), ("b".to_string(), TypeSpec::Bool)],
                return_type: TypeSpec::Bool,
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::Binary {
                    op: omni_types::ast::BinOp::LogicalAnd,
                    lhs: Box::new(Expr::Var("a".to_string())),
                    rhs: Box::new(Expr::Var("b".to_string())),
                },
            }],
        };
        let mir = ctx.lower_monomorphized_program(&prog).expect("logical lowering");
        assert!(mir.functions[0].body.blocks.len() >= 4);
        assert!(mir.functions[0]
            .body
            .blocks
            .iter()
            .any(|b| { matches!(b.terminator, Some(crate::ir::Terminator::SwitchInt { .. })) }));
        assert!(mir.functions[0]
            .body
            .blocks
            .iter()
            .any(|b| { matches!(b.terminator, Some(crate::ir::Terminator::Goto(_))) }));
    }

    #[test]
    fn scalar_match_lowering_creates_switch_and_join() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "choose".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![("x".to_string(), TypeSpec::Int)],
                return_type: TypeSpec::Int,
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::Match {
                    expr: Box::new(Expr::Var("x".to_string())),
                    arms: vec![
                        omni_types::ast::MatchArm {
                            pattern: omni_types::ast::Pattern::Lit(omni_types::ast::Lit::Int(0)),
                            guard: None,
                            body: Expr::Literal(omni_types::ast::Lit::Int(10)),
                        },
                        omni_types::ast::MatchArm {
                            pattern: omni_types::ast::Pattern::Wildcard,
                            guard: None,
                            body: Expr::Literal(omni_types::ast::Lit::Int(20)),
                        },
                    ],
                },
            }],
        };
        let mir = ctx.lower_monomorphized_program(&prog).expect("scalar match lowering");
        assert!(mir.functions[0]
            .body
            .blocks
            .iter()
            .any(|b| { matches!(b.terminator, Some(crate::ir::Terminator::SwitchInt { .. })) }));
        assert!(
            mir.functions[0]
                .body
                .blocks
                .iter()
                .filter(|b| { matches!(b.terminator, Some(crate::ir::Terminator::Goto(_))) })
                .count()
                >= 2
        );
    }

    #[test]
    fn loop_lowering_creates_back_edge_and_break_exit() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "count".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Int,
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::Loop {
                    label: None,
                    body: Box::new(Expr::Break {
                        label: None,
                        value: Some(Box::new(Expr::Literal(omni_types::ast::Lit::Int(7)))),
                    }),
                },
            }],
        };
        let mir = ctx.lower_monomorphized_program(&prog).expect("loop lowering");
        assert!(mir.functions[0]
            .body
            .blocks
            .iter()
            .any(|b| { matches!(b.terminator, Some(crate::ir::Terminator::Goto(_))) }));
        assert!(mir.functions[0]
            .body
            .local_decls
            .iter()
            .any(|l| l.name.as_deref() == Some("_loop_result")));
    }

    #[test]
    fn infinite_no_break_loop_has_no_live_continuation() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "spin_forever".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Never,
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::Loop {
                    label: Some("spin".to_string()),
                    body: Box::new(Expr::Continue { label: Some("spin".to_string()) }),
                },
            }],
        };
        let mir = ctx.lower_monomorphized_program(&prog).expect("infinite loop lowering");
        assert!(mir.functions[0]
            .body
            .blocks
            .iter()
            .any(|b| { matches!(b.terminator, Some(crate::ir::Terminator::Goto(_))) }));
    }

    #[test]
    fn continue_lowering_targets_current_loop_header() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "spin".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::While {
                    label: None,
                    condition: Box::new(Expr::Literal(omni_types::ast::Lit::Bool(true))),
                    body: Box::new(Expr::Continue { label: None }),
                },
            }],
        };
        let mir = ctx.lower_monomorphized_program(&prog).expect("continue lowering");
        assert!(mir.functions[0].body.blocks.len() >= 3);
    }

    #[test]
    fn loop_break_and_continue_lower_to_cfg_edges() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "loop_fn".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![("x".to_string(), TypeSpec::Int)],
                return_type: TypeSpec::Int,
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::Loop {
                    label: None,
                    body: Box::new(Expr::Block(vec![
                        Expr::Continue { label: None },
                        Expr::Break {
                            label: None,
                            value: Some(Box::new(Expr::Var("x".to_string()))),
                        },
                    ])),
                },
            }],
        };
        let mir = ctx.lower_monomorphized_program(&prog).expect("loop lowering");
        assert!(mir.functions[0]
            .body
            .blocks
            .iter()
            .any(|b| { matches!(b.terminator, Some(crate::ir::Terminator::Goto(_))) }));
        assert!(mir.functions[0]
            .body
            .local_decls
            .iter()
            .any(|d| d.name.as_deref() == Some("_loop_result")));
    }

    #[test]
    fn integer_range_for_lowering_creates_condition_step_and_exit() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "sum".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![("sum".to_string(), TypeSpec::Int)],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::For {
                    label: None,
                    pattern: omni_types::ast::Pattern::Binding("i".to_string()),
                    iterable: Box::new(Expr::Range {
                        start: Box::new(Expr::Literal(omni_types::ast::Lit::Int(0))),
                        end: Box::new(Expr::Literal(omni_types::ast::Lit::Int(3))),
                        inclusive: false,
                    }),
                    body: Box::new(Expr::CompoundAssign {
                        op: omni_types::ast::AssignOp::Add,
                        target: Box::new(Expr::Var("sum".to_string())),
                        value: Box::new(Expr::Var("i".to_string())),
                    }),
                },
            }],
        };
        let mir = ctx.lower_monomorphized_program(&prog).expect("for lowering");
        assert!(mir.functions[0]
            .body
            .blocks
            .iter()
            .any(|b| { matches!(b.terminator, Some(crate::ir::Terminator::SwitchInt { .. })) }));
        assert!(
            mir.functions[0]
                .body
                .blocks
                .iter()
                .filter(|b| { matches!(b.terminator, Some(crate::ir::Terminator::Goto(_))) })
                .count()
                >= 3
        );
        assert!(mir.functions[0]
            .body
            .local_decls
            .iter()
            .any(|d| d.name.as_deref() == Some("_for_index")));
    }

    #[test]
    fn while_lowering_creates_condition_branch_and_exit() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "while_fn".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![("x".to_string(), TypeSpec::Bool)],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::While {
                    label: Some("outer".to_string()),
                    condition: Box::new(Expr::Var("x".to_string())),
                    body: Box::new(Expr::Break { label: Some("outer".to_string()), value: None }),
                },
            }],
        };
        let mir = ctx.lower_monomorphized_program(&prog).expect("while lowering");
        assert!(mir.functions[0]
            .body
            .blocks
            .iter()
            .any(|b| { matches!(b.terminator, Some(crate::ir::Terminator::SwitchInt { .. })) }));
        assert!(
            mir.functions[0]
                .body
                .blocks
                .iter()
                .filter(|b| { matches!(b.terminator, Some(crate::ir::Terminator::Goto(_))) })
                .count()
                >= 2
        );
    }

    /// Struct construction lowers to typed MIR rather than failing in the
    /// lowering pass. Commit `1f0eaee` ("lower struct and enum constructors
    /// into typed MIR") deliberately replaced the previous fail-closed arm here,
    /// so the old expectation that lowering rejects a struct literal is a stale
    /// assumption and no longer describes this compiler.
    ///
    /// The aggregate layout boundary is real, but it lives one layer down: the
    /// IR carries the constructor and its declared type, and `omni-codegen`
    /// refuses to emit native code for it until a target layout contract exists.
    /// Asserting the boundary here would be asserting a rule this layer does not
    /// own.
    #[test]
    fn aggregate_construction_lowers_to_typed_mir() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "struct_ctor".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Struct("Pair".to_string(), vec![]),
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::Struct {
                    name: "Pair".to_string(),
                    generic_args: vec![],
                    fields: vec![
                        ("first".to_string(), Expr::Literal(Lit::Int(1))),
                        ("second".to_string(), Expr::Literal(Lit::Int(2))),
                    ],
                },
            }],
        };
        let mut mir =
            ctx.lower_monomorphized_program(&prog).expect("struct construction must lower to MIR");
        let f = &mir.functions[0];
        assert_eq!(f.name, "struct_ctor");
        let ty = mir.tcx.intern(TyKind::Struct("Pair".to_string(), vec![]));
        assert!(
            f.body.blocks.iter().any(|b| b.statements.iter().any(|s| matches!(
                s,
                crate::ir::Statement::Assign(_, crate::ir::Rvalue::Struct { name, ty: t, .. })
                    if name == "Pair" && *t == ty
            ))),
            "struct construction must emit a typed Rvalue::Struct"
        );
    }

    #[test]
    fn enum_construction_lowers_to_typed_mir() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "enum_ctor".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Enum("Option".to_string(), vec![TypeSpec::Int]),
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::EnumVariant {
                    enum_name: "Option".to_string(),
                    variant: "None".to_string(),
                    generic_args: vec![TypeSpec::Int],
                    args: vec![],
                },
            }],
        };
        let mut mir =
            ctx.lower_monomorphized_program(&prog).expect("enum construction must lower to MIR");
        let f = &mir.functions[0];
        let int_ty = mir.tcx.intern(TyKind::Int);
        let ty = mir.tcx.intern(TyKind::Enum("Option".to_string(), vec![int_ty]));
        assert!(
            f.body.blocks.iter().any(|b| b.statements.iter().any(|s| matches!(
                s,
                crate::ir::Statement::Assign(
                    _,
                    crate::ir::Rvalue::EnumVariant { enum_name, variant, ty: t, .. }
                ) if enum_name == "Option" && variant == "None" && *t == ty
            ))),
            "enum construction must emit a typed Rvalue::EnumVariant carrying its arguments"
        );
    }

    #[test]
    fn test_mir_lowering_concrete_gate_pass() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "main_spec".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Block(vec![]),
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_ok(), "Concrete Unit program must lower successfully");
    }

    #[test]
    fn test_mir_lowering_concrete_gate_fails_on_unresolved_generic() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "unspecialized".to_string(),
                type_params: vec!["T".to_string()],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Literal(Lit::Int(42)),
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(
            res.is_err(),
            "Program with unresolved type parameters must fail closed at MIR lowering gate"
        );
        assert!(res.unwrap_err().contains("unresolved type parameters"));
    }

    #[test]
    fn test_mir_lowering_concrete_gate_fails_on_undischarged_bounds() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "unresolved_bound".to_string(),
                type_params: vec![],
                bounds: vec![(
                    "T".to_string(),
                    omni_types::ast::TraitBound::Positive("Display".to_string()),
                )],
                params: vec![],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Literal(Lit::Int(42)),
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(
            res.is_err(),
            "Program with undischarged trait bounds must fail closed at MIR lowering gate"
        );
        assert!(res.unwrap_err().contains("unresolved trait bounds"));
    }

    #[test]
    fn test_mir_lowering_range_preserves_inclusive_endpoint() {
        let mut ctx = LoweringContext::new();
        let program = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "range".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Range(Box::new(TypeSpec::Int)),
                effects: omni_effects::EffectRow::pure(),
                capabilities: vec![],
                body: Expr::Range {
                    start: Box::new(Expr::Literal(Lit::Int(1))),
                    end: Box::new(Expr::Literal(Lit::Int(4))),
                    inclusive: true,
                },
            }],
        };
        let mir = ctx.lower_monomorphized_program(&program).expect("range lowering");
        assert!(mir.functions[0].body.blocks.iter().any(|block| {
            block.statements.iter().any(|statement| {
                matches!(
                    statement,
                    crate::ir::Statement::Assign(
                        _,
                        crate::ir::Rvalue::Range { inclusive: true, .. }
                    )
                )
            })
        }));
    }

    #[test]
    fn test_mir_lowering_tuple_and_array_aggregates() {
        let mut ctx = LoweringContext::new();
        let program = MonomorphizedProgram {
            functions: vec![
                GenericFnDef {
                    name: "tuple_value".to_string(),
                    type_params: vec![],
                    bounds: vec![],
                    params: vec![],
                    return_type: TypeSpec::Tuple(vec![TypeSpec::Int, TypeSpec::Int]),
                    effects: omni_effects::EffectRow::default(),
                    capabilities: vec![],
                    body: Expr::Tuple(vec![Expr::Literal(Lit::Int(1)), Expr::Literal(Lit::Int(2))]),
                },
                GenericFnDef {
                    name: "array_value".to_string(),
                    type_params: vec![],
                    bounds: vec![],
                    params: vec![],
                    return_type: TypeSpec::Array(Box::new(TypeSpec::Int), 2),
                    effects: omni_effects::EffectRow::default(),
                    capabilities: vec![],
                    body: Expr::Array(vec![Expr::Literal(Lit::Int(3)), Expr::Literal(Lit::Int(4))]),
                },
            ],
        };
        let mir = ctx.lower_monomorphized_program(&program).expect("aggregate lowering");
        assert!(mir.functions.iter().all(|f| {
            f.body.blocks.iter().any(|b| {
                b.statements.iter().any(|s| {
                    matches!(
                        s,
                        crate::ir::Statement::Assign(_, crate::ir::Rvalue::Aggregate { .. })
                    )
                })
            })
        }));
    }

    #[test]
    fn test_mir_lowering_semantic_binary_and_let_expressions() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "add_let".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![("x".to_string(), TypeSpec::Int)],
                return_type: TypeSpec::Int,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Let {
                    pattern: omni_types::ast::Pattern::Binding("y".to_string()),
                    ty: None,
                    init: Box::new(Expr::Binary {
                        op: omni_types::ast::BinOp::Add,
                        lhs: Box::new(Expr::Var("x".to_string())),
                        rhs: Box::new(Expr::Literal(Lit::Int(2))),
                    }),
                    body: Box::new(Expr::Var("y".to_string())),
                },
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog).unwrap();
        let func = &res.functions[0];
        assert_eq!(func.params.len(), 1);
        assert!(!func.body.blocks.is_empty());
        assert!(func.body.local_decls.iter().all(|decl| decl.ty.is_some()));
    }

    #[test]
    fn test_mir_lowering_call_uses_callee_signature() {
        let mut ctx = LoweringContext::new();
        let callee = GenericFnDef {
            name: "inc".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![("x".to_string(), TypeSpec::Int)],
            return_type: TypeSpec::Int,
            effects: omni_effects::EffectRow::default(),
            capabilities: vec![],
            body: Expr::Var("x".to_string()),
        };
        let caller = GenericFnDef {
            name: "main_spec".to_string(),
            type_params: vec![],
            bounds: vec![],
            params: vec![],
            return_type: TypeSpec::Int,
            effects: omni_effects::EffectRow::default(),
            capabilities: vec![],
            body: Expr::Call {
                func: "inc".to_string(),
                generic_args: vec![],
                args: vec![Expr::Literal(Lit::Int(41))],
            },
        };
        let prog = MonomorphizedProgram { functions: vec![callee, caller] };

        let res = ctx.lower_monomorphized_program(&prog).unwrap();
        let caller_mir = res.functions.iter().find(|f| f.name == "main_spec").unwrap();
        assert!(matches!(
            caller_mir.body.blocks[0].terminator,
            Some(crate::ir::Terminator::Call { destination: Some(_), .. })
        ));
    }

    #[test]
    fn test_mir_lowering_comparison_produces_bool() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "cmp".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Bool,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Binary {
                    op: omni_types::ast::BinOp::Eq,
                    lhs: Box::new(Expr::Literal(Lit::Int(1))),
                    rhs: Box::new(Expr::Literal(Lit::Int(1))),
                },
            }],
        };

        let mir = ctx.lower_monomorphized_program(&prog).expect("comparison should lower");
        let function = &mir.functions[0];
        let result_local = function
            .body
            .local_decls
            .iter()
            .find(|decl| decl.name.as_deref() == Some("_bin_tmp"))
            .expect("comparison temporary should exist");
        assert_eq!(result_local.ty, function.body.local_decls[function.return_place].ty);
    }

    #[test]
    fn test_mir_lowering_rejects_return_type_mismatch() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "bad".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Int,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Literal(Lit::Bool(true)),
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("function body has type"));
    }

    #[test]
    fn test_mir_lowering_bare_return_requires_unit() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "unit_return".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Unit,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Return(None),
            }],
        };

        assert!(ctx.lower_monomorphized_program(&prog).is_ok());
    }

    #[test]
    fn test_mir_lowering_unsupported_expression_fails_explicitly() {
        let mut ctx = LoweringContext::new();
        let prog = MonomorphizedProgram {
            functions: vec![GenericFnDef {
                name: "unsupported".to_string(),
                type_params: vec![],
                bounds: vec![],
                params: vec![],
                return_type: TypeSpec::Int,
                effects: omni_effects::EffectRow::default(),
                capabilities: vec![],
                body: Expr::Lambda { params: vec![], body: Box::new(Expr::Literal(Lit::Int(1))) },
            }],
        };

        let res = ctx.lower_monomorphized_program(&prog);
        assert!(res.is_err(), "Unsupported AST expressions must fail lowering explicitly");
        assert!(res.unwrap_err().contains("Unsupported AST expression form"));
    }

    #[test]
    fn test_call_result_let_binding_lands_in_continuation_block() {
        let mut ctx = LoweringContext::new();
        let program = MonomorphizedProgram {
            functions: vec![
                GenericFnDef {
                    name: "inc".to_string(),
                    type_params: vec![],
                    bounds: vec![],
                    params: vec![("x".to_string(), TypeSpec::Int)],
                    return_type: TypeSpec::Int,
                    effects: omni_effects::EffectRow::default(),
                    capabilities: vec![],
                    body: Expr::Return(Some(Box::new(Expr::Binary {
                        op: omni_types::ast::BinOp::Add,
                        lhs: Box::new(Expr::Var("x".to_string())),
                        rhs: Box::new(Expr::Literal(Lit::Int(1))),
                    }))),
                },
                GenericFnDef {
                    name: "main".to_string(),
                    type_params: vec![],
                    bounds: vec![],
                    params: vec![],
                    return_type: TypeSpec::Int,
                    effects: omni_effects::EffectRow::default(),
                    capabilities: vec![],
                    body: Expr::Let {
                        pattern: omni_types::ast::Pattern::Binding("value".to_string()),
                        ty: None,
                        init: Box::new(Expr::Call {
                            func: "inc".to_string(),
                            generic_args: vec![],
                            args: vec![Expr::Literal(Lit::Int(41))],
                        }),
                        body: Box::new(Expr::Return(Some(Box::new(Expr::Var(
                            "value".to_string(),
                        ))))),
                    },
                },
            ],
        };

        let mir = ctx
            .lower_monomorphized_program(&program)
            .expect("call result must lower to a continuation block");
        let main = mir.functions.iter().find(|f| f.name == "main").expect("main MIR");
        assert!(main.body.blocks.len() >= 2);
        assert!(main.body.blocks.iter().any(|block| {
            block.statements.iter().any(|statement| {
                matches!(
                    statement,
                    crate::ir::Statement::Assign(
                        _,
                        crate::ir::Rvalue::Use(crate::ir::Operand::Copy(_))
                    )
                )
            })
        }));
    }

    #[test]
    fn test_mir_lowering_materializes_literal_var_unary_and_block() {
        let mut ctx = LoweringContext::new();
        let program = MonomorphizedProgram {
            functions: vec![
                GenericFnDef {
                    name: "literal".to_string(),
                    type_params: vec![],
                    bounds: vec![],
                    params: vec![],
                    return_type: TypeSpec::Int,
                    effects: omni_effects::EffectRow::default(),
                    capabilities: vec![],
                    body: Expr::Literal(Lit::Int(7)),
                },
                GenericFnDef {
                    name: "identity".to_string(),
                    type_params: vec![],
                    bounds: vec![],
                    params: vec![("x".to_string(), TypeSpec::Int)],
                    return_type: TypeSpec::Int,
                    effects: omni_effects::EffectRow::default(),
                    capabilities: vec![],
                    body: Expr::Var("x".to_string()),
                },
                GenericFnDef {
                    name: "negate".to_string(),
                    type_params: vec![],
                    bounds: vec![],
                    params: vec![("x".to_string(), TypeSpec::Int)],
                    return_type: TypeSpec::Int,
                    effects: omni_effects::EffectRow::default(),
                    capabilities: vec![],
                    body: Expr::Unary {
                        op: omni_types::ast::UnOp::Neg,
                        expr: Box::new(Expr::Var("x".to_string())),
                    },
                },
                GenericFnDef {
                    name: "block_value".to_string(),
                    type_params: vec![],
                    bounds: vec![],
                    params: vec![],
                    return_type: TypeSpec::Int,
                    effects: omni_effects::EffectRow::default(),
                    capabilities: vec![],
                    body: Expr::Block(vec![Expr::Literal(Lit::Int(1)), Expr::Literal(Lit::Int(2))]),
                },
            ],
        };

        let mir = ctx.lower_monomorphized_program(&program).expect("expression corpus must lower");
        assert_eq!(mir.functions.len(), 4);
        for function in &mir.functions {
            assert!(
                function.body.blocks.iter().any(|block| !block.statements.is_empty()),
                "function '{}' must materialize at least one MIR statement",
                function.name
            );
            assert!(
                function.body.local_decls.iter().all(|decl| decl.ty.is_some()),
                "function '{}' must type every MIR local",
                function.name
            );
        }
        let negate = mir.functions.iter().find(|f| f.name == "negate").unwrap();
        assert!(negate.body.blocks.iter().any(|block| {
            block.statements.iter().any(|statement| {
                matches!(
                    statement,
                    crate::ir::Statement::Assign(
                        _,
                        crate::ir::Rvalue::UnaryOp(crate::ir::UnOp::Neg, _)
                    )
                )
            })
        }));
    }
}
