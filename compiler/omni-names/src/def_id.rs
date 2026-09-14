/// A canonical, globally unique identifier for any definition in the Omni ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefId {
    pub package: u32,
    pub module: u32,
    pub index: u32,
}

impl DefId {
    pub const fn new(package: u32, module: u32, index: u32) -> Self {
        Self { package, module, index }
    }
}

#[cfg(any())]
#[implements("NAME-0001")]
fn _audit_def_id() {}
