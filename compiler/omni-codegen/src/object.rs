//! # ELF/PE Object Writer
//! Bundles compiled Cranelift modules into object files and generates runtime wrappers.

use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct ObjectWriter {
    pub output_path: String,
}

impl ObjectWriter {
    pub fn new(output_path: impl Into<String>) -> Self {
        Self {
            output_path: output_path.into(),
        }
    }

    /// Bundles object code bytes into a target file (.o / .obj) and writes a runtime entry stub
    pub fn write_object(&self, object_bytes: &[u8]) -> Result<(), String> {
        let path = Path::new(&self.output_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(object_bytes).map_err(|e| e.to_string())?;
        Ok(())
    }
}
