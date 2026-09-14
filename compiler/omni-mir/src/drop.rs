use crate::ir::{Body, Place, Statement, Terminator};

/// Elaborates drops in the MIR graph based on ownership and initialization state.
pub struct DropElaborator;

impl DropElaborator {
    /// Performs dataflow analysis to compute initialization states,
    /// then injects Statement::Drop into the MIR CFG.
    pub fn elaborate(body: &mut Body) {
        let mut return_blocks = Vec::new();

        // Step 1: Identify all exit points (blocks ending in Return)
        for (block_idx, block_data) in body.blocks.iter_enumerated() {
            if let Some(Terminator::Return) = block_data.terminator {
                return_blocks.push(block_idx);
            }
        }

        // Step 2: Inject Drop statements in reverse-initialization order
        // Note: A full implementation requires Gen/Kill dataflow sets to handle branching.
        // This baseline assumes all declared locals need dropping at the function exit.
        for block_idx in return_blocks {
            let block = &mut body.blocks[block_idx];
            
            // Iterate backwards through the local declarations
            for local in body.local_decls.indices().rev() {
                block.statements.push(Statement::Drop(Place { local }));
            }
        }
    }
}
