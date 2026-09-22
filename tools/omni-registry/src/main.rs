use std::path::PathBuf;

use omni_registry::load_specification;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let spec_root = args.get(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("spec"));
    match load_specification(spec_root) {
        Ok(loaded) => {
            println!(
                "REGISTRY LOAD PASS: tree={} artifacts={} rules={} schemas={} models=empty data=empty loader={}",
                loaded.identity().tree_digest,
                loaded.identity().artifacts.len(),
                loaded.registry().rules.len(),
                loaded.schemas().len(),
                omni_registry::LOADER_VERSION,
            );
        }
        Err(e) => {
            eprintln!("REGISTRY LOAD ERROR: {e}");
            std::process::exit(101);
        }
    }
}
