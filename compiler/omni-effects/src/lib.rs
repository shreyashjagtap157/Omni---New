//! Effect Row Solver for Omni (EFF-0001).
//! Implements open and closed effect rows with row polymorphism and effect containment checking.

#[macro_export]
macro_rules! implements {
    ($tag:literal) => {};
}

implements!("EFF-0001");

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    IO,
    State,
    Async,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectRow {
    Closed(Vec<Effect>),
    Open(Vec<Effect>, String), // Effects present + row variable tail (e.g., {IO} | ρ)
}

impl EffectRow {
    pub fn closed(effects: Vec<Effect>) -> Self {
        Self::Closed(effects)
    }

    pub fn open(effects: Vec<Effect>, tail: impl Into<String>) -> Self {
        Self::Open(effects, tail.into())
    }

    /// Check if this effect row can satisfy or be subtyped by a required effect row
    pub fn satisfies(&self, required: &EffectRow) -> bool {
        match (self, required) {
            (EffectRow::Closed(provided), EffectRow::Closed(req)) => {
                req.iter().all(|e| provided.contains(e))
            }
            (EffectRow::Open(provided, _), EffectRow::Closed(req)) => {
                req.iter().all(|e| provided.contains(e))
            }
            (EffectRow::Closed(_), EffectRow::Open(_, _)) => {
                false // Closed cannot satisfy an open polymorphic requirement directly without instantiation
            }
            (EffectRow::Open(p1, _), EffectRow::Open(p2, _)) => p2.iter().all(|e| p1.contains(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_row_solving() {
        let provided = EffectRow::closed(vec![Effect::IO, Effect::State]);
        let required = EffectRow::closed(vec![Effect::IO]);
        assert!(provided.satisfies(&required));

        let unprovided = EffectRow::closed(vec![Effect::State]);
        assert!(!unprovided.satisfies(&required));

        let open_provided = EffectRow::open(vec![Effect::IO], "rho");
        assert!(open_provided.satisfies(&required));
    }
}
