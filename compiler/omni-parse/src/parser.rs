//! Parser implementation for Omni.

pub struct Parser {
    tokens: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<String>) -> Self {
        Self { tokens }
    }

    pub fn parse(&mut self) -> Result<(), String> {
        Ok(())
    }
}
