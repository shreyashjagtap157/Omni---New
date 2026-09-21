use std::env;
use std::path::PathBuf;

use omni_canon::{canonical_hash_json, spec_tree};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 && args[1] == "--spec-tree" {
        let spec_root = PathBuf::from(&args[2]);
        let (digest, files) = spec_tree::spec_tree_digest(&spec_root)?;
        println!("{digest}");
        for file in files {
            eprintln!("  tree: {file}");
        }
        return Ok(());
    }
    if args.len() != 3 {
        eprintln!("Usage: omni-canon <file.json> <workspace_root>");
        eprintln!("       omni-canon --spec-tree <spec_root>");
        std::process::exit(1);
    }
    let target_file = PathBuf::from(&args[1]);
    let workspace_root = PathBuf::from(&args[2]);
    println!("{}", canonical_hash_json(&target_file, &workspace_root)?);
    Ok(())
}
