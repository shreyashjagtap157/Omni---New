//! Parser implementation for Omni.

pub struct Parser {
    _tokens: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<String>) -> Self {
        Self { _tokens: tokens }
    }

    pub fn parse(&mut self) -> Result<(), String> {
        Ok(())
    }
}
