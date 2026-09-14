//! # Abstract Store (The Interpreter)
//! Implements byte-level memory tracking, allocation provenance, and execution state.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllocId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
    pub bytes: Vec<u8>,
    pub initialized: Vec<bool>,
    pub align: u32,
    pub mutuable: bool,
}

/// Represents the abstract machine's memory store with pointer provenance tracking.
#[derive(Debug, Clone, Default)]
pub struct Memory {
    pub allocations: HashMap<AllocId, Allocation>,
    next_id: u64,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn allocate(&mut self, size: usize, align: u32, mutuable: bool) -> AllocId {
        let id = AllocId(self.next_id);
        self.next_id += 1;
        
        let alloc = Allocation {
            bytes: vec![0; size],
            initialized: vec![false; size],
            align,
            mutuable,
        };
        
        self.allocations.insert(id, alloc);
        id
    }

    pub fn read(&self, id: AllocId, offset: usize, size: usize) -> Result<&[u8], &'static str> {
        let alloc = self.allocations.get(&id).ok_or("Invalid pointer provenance: dangling allocation ID")?;
        if offset + size > alloc.bytes.len() {
            return Err("Out of bounds read trap");
        }
        for i in offset..(offset + size) {
            if !alloc.initialized[i] {
                return Err("Read from uninitialized memory trap");
            }
        }
        Ok(&alloc.bytes[offset..(offset + size)])
    }
}
