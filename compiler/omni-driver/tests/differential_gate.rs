//! Real Stage-0 differential gate.
//!
//! The reference machine and native backend consume the same compiler-produced
//! verified MIR. The native object is renamed only in this test so that a small
//! Rust harness can provide the host process entry point without changing Omni's
//! compiler-level `main` convention.
+
#[cfg(target_os = "linux")]
mod linux {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    use omni_driver::{compile_source_to_interpreter_value, compile_source_to_object};
    use omni_registry::load_specification;

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }

    fn unique_stem() -> String {
        format!(
            "omni-diff-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        )
    }

    fn llvm_objcopy() -> PathBuf {
        let rustc = Command::new("rustc")
            .args(["--print", "sysroot"])
            .output()
            .expect("rustc must be available");
        assert!(rustc.status.success(), "rustc --print sysroot failed");
        let sysroot = PathBuf::from(String::from_utf8(rustc.stdout).expect("utf8 sysroot").trim());
        let path = sysroot.join("bin").join("llvm-objcopy");
        assert!(path.is_file(), "pinned Rust toolchain must provide llvm-objcopy: {}", path.display());
        path
    }

    fn run_native_object(object: &[u8]) -> i64 {
        let stem = unique_stem();
        let dir = std::env::temp_dir();
        let object_path = dir.join(format!("{stem}.o"));
        let shim_path = dir.join(format!("{stem}_shim.rs"));
        let exe_path = dir.join(format!("{stem}.bin"));

        std::fs::write(&object_path, object).expect("object write");

        let objcopy = llvm_objcopy();
        let rename = Command::new(&objcopy)
            .arg("--redefine-sym")
            .arg("main=omni_main")
            .arg(&object_path)
            .output()
            .expect("llvm-objcopy must execute");
        assert!(
            rename.status.success(),
            "llvm-objcopy failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&rename.stdout),
            String::from_utf8_lossy(&rename.stderr)
        );

        std::fs::write(
            &shim_path,
            r#"unsafe extern "C" { fn omni_main() -> i64; }
fn main() {
    println!("{}", unsafe { omni_main() });
}
"#,
        )
        .expect("shim write");

        let link = Command::new("rustc")
            .arg("--edition=2021")
            .arg(&shim_path)
            .arg("-o")
            .arg(&exe_path)
            .arg("-C")
            .arg(format!("link-arg={}", object_path.display()))
            .output()
            .expect("rustc must be available for native linking");
        assert!(
            link.status.success(),
            "native link failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&link.stdout),
            String::from_utf8_lossy(&link.stderr)
        );

        let run = Command::new(&exe_path).output().expect("native executable must run");
        assert!(
            run.status.success(),
            "native executable failed: stdout={} stderr={}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let output = String::from_utf8(run.stdout).expect("native stdout must be UTF-8");
        let value = output.trim().parse::<i64>().expect("native result must be an i64");

        std::fs::remove_file(object_path).ok();
        std::fs::remove_file(shim_path).ok();
        std::fs::remove_file(exe_path).ok();

        value
    }

    #[test]
    fn reference_machine_and_native_backend_agree_on_real_source() {
        let source = "fn main() -> i64 { let x: i64 = 40; x + 2 }";
        let manifest = load_specification(workspace_root().join("spec"))
            .expect("authoritative specification must load")
            .manifest()
            .clone();

        let reference = compile_source_to_interpreter_value(source, &manifest)
            .expect("reference-machine compilation/execution must succeed");
        let object =
            compile_source_to_object(source, &manifest).expect("native compilation must succeed");
        let native = run_native_object(&object);

        assert_eq!(
            reference, native,
            "reference-machine and native execution diverged for the same source"
        );
    }
}
