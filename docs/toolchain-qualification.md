# Toolchain qualification record

Declared toolchain: Rust `1.95.0` with `rustfmt`, `clippy`, `rust-src`, `llvm-tools-preview`.

Required targets (foundation contract):

- `x86_64-unknown-linux-gnu`
- `riscv64-unknown-none-elf` (provided by Rust 1.95.0 as canonical `riscv64gc-unknown-none-elf`)

Qualification (verified 2026-09-21, Windows host + CI Linux contract):

- `rust-toolchain.toml` pins `channel = "1.95.0"` with explicit `components` and `targets`.
- `rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0`.
- `ci/assert-toolchain.sh` PASS: pinned channel, all components, both targets verified.
- `cargo fmt --all -- --check`, `cargo check --workspace --locked`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --locked`, `cargo metadata --format-version 1 --locked`,
  `cargo tree --workspace --locked` all pass under 1.95.0 with the committed `Cargo.lock`.
- Dependency graph retained at `cranelift-* 0.110`, `object 0.36`,
  `target-lexicon 0.12`, `indexmap 2.2.6`, `sha2 0.10` — verified compatible with
  1.95.0; no blind upgrade performed per smallest-change rule.
