# Toolchain qualification record

Declared toolchain: Rust `1.80.0` with `rustfmt`, `clippy`, `llvm-tools-preview`.

Required targets:

- `riscv64-unknown-none-elf`
- `x86_64-unknown-linux-gnu`

Current host state: Rust/Cargo/Rustup are not installed in the execution environment. Therefore
`ci/assert-toolchain.sh` is intentionally expected to fail closed and no Rust qualification is
claimed by this milestone.
