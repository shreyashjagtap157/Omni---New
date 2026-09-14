//! Typed LLVM Intermediate Representation Model and Deterministic Serializer.
//! Provides structured types and instructions to replace raw format-string generation.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlvmType {
    Void,
    Integer { bits: u16 },
    Float32,
    Float64,
    Pointer { address_space: u32 },
    Array { len: u64, element: Box<LlvmType> },
    Struct(Vec<LlvmType>),
}

impl LlvmType {
    pub fn emit(&self) -> String {
        match self {
            LlvmType::Void => "void".into(),
            LlvmType::Integer { bits } => format!("i{}", bits),
            LlvmType::Float32 => "float".into(),
            LlvmType::Float64 => "double".into(),
            LlvmType::Pointer { .. } => "ptr".into(),
            LlvmType::Array { len, element } => format!("[{} x {}]", len, element.emit()),
            LlvmType::Struct(fields) => {
                let field_strs: Vec<_> = fields.iter().map(|f| f.emit()).collect();
                format!("{{{}}}", field_strs.join(", "))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlvmInst {
    Add { dest: String, ty: LlvmType, lhs: String, rhs: String },
    Sub { dest: String, ty: LlvmType, lhs: String, rhs: String },
    Load { dest: String, ty: LlvmType, ptr: String },
    Store { ty: LlvmType, val: String, ptr: String },
    Call { dest: Option<String>, ret_ty: LlvmType, func: String, args: Vec<(LlvmType, String)> },
    Branch { target: String },
    CondBranch { cond: String, true_target: String, false_target: String },
    Return { ty: LlvmType, val: Option<String> },
}

impl LlvmInst {
    pub fn emit(&self) -> String {
        match self {
            LlvmInst::Add { dest, ty, lhs, rhs } => {
                format!("  {} = add {} {}, {}", dest, ty.emit(), lhs, rhs)
            }
            LlvmInst::Sub { dest, ty, lhs, rhs } => {
                format!("  {} = sub {} {}, {}", dest, ty.emit(), lhs, rhs)
            }
            LlvmInst::Load { dest, ty, ptr } => {
                format!("  {} = load {}, ptr {}", dest, ty.emit(), ptr)
            }
            LlvmInst::Store { ty, val, ptr } => {
                format!("  store {} {}, ptr {}", ty.emit(), val, ptr)
            }
            LlvmInst::Call { dest, ret_ty, func, args } => {
                let arg_strs: Vec<_> =
                    args.iter().map(|(t, v)| format!("{} {}", t.emit(), v)).collect();
                let call_expr =
                    format!("call {} @{}({})", ret_ty.emit(), func, arg_strs.join(", "));
                match dest {
                    Some(d) => format!("  {} = {}", d, call_expr),
                    None => format!("  {}", call_expr),
                }
            }
            LlvmInst::Branch { target } => {
                format!("  br label %{}", target)
            }
            LlvmInst::CondBranch { cond, true_target, false_target } => {
                format!("  br i1 {}, label %{}, label %{}", cond, true_target, false_target)
            }
            LlvmInst::Return { ty, val } => match val {
                Some(v) => format!("  ret {} {}", ty.emit(), v),
                None => "  ret void".into(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct LlvmBasicBlock {
    pub name: String,
    pub instructions: Vec<LlvmInst>,
}

#[derive(Debug, Clone)]
pub struct LlvmFunction {
    pub name: String,
    pub ret_ty: LlvmType,
    pub params: Vec<(LlvmType, String)>,
    pub blocks: Vec<LlvmBasicBlock>,
}

impl LlvmFunction {
    pub fn emit(&self) -> String {
        let param_strs: Vec<_> =
            self.params.iter().map(|(t, n)| format!("{} %{}", t.emit(), n)).collect();
        let mut out =
            format!("define {} @{}({}) {{\n", self.ret_ty.emit(), self.name, param_strs.join(", "));
        for block in &self.blocks {
            out.push_str(&format!("{}:\n", block.name));
            for inst in &block.instructions {
                out.push_str(&format!("{}\n", inst.emit()));
            }
        }
        out.push_str("}\n");
        out
    }
}

#[derive(Debug, Clone)]
pub struct LlvmModule {
    pub name: String,
    pub datalayout: String,
    pub functions: BTreeMap<String, LlvmFunction>,
}

impl LlvmModule {
    pub fn new(name: impl Into<String>, datalayout: impl Into<String>) -> Self {
        Self { name: name.into(), datalayout: datalayout.into(), functions: BTreeMap::new() }
    }

    pub fn add_function(&mut self, func: LlvmFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    pub fn emit_text(&self) -> String {
        let mut out = format!(
            "; ModuleID = '{}'\nsource_filename = \"{}\"\ntarget datalayout = \"{}\"\n\n",
            self.name, self.name, self.datalayout
        );
        for func in self.functions.values() {
            out.push_str(&func.emit());
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_llvm_model_emission() {
        let mut module = LlvmModule::new("test_mod", "e-m:e");
        let func = LlvmFunction {
            name: "add_test".into(),
            ret_ty: LlvmType::Integer { bits: 64 },
            params: vec![
                (LlvmType::Integer { bits: 64 }, "a".into()),
                (LlvmType::Integer { bits: 64 }, "b".into()),
            ],
            blocks: vec![LlvmBasicBlock {
                name: "entry".into(),
                instructions: vec![
                    LlvmInst::Add {
                        dest: "%res".into(),
                        ty: LlvmType::Integer { bits: 64 },
                        lhs: "%a".into(),
                        rhs: "%b".into(),
                    },
                    LlvmInst::Return {
                        ty: LlvmType::Integer { bits: 64 },
                        val: Some("%res".into()),
                    },
                ],
            }],
        };
        module.add_function(func);
        let text = module.emit_text();
        assert!(text.contains("define i64 @add_test(i64 %a, i64 %b)"));
        assert!(text.contains("%res = add i64 %a, %b"));
    }
}
