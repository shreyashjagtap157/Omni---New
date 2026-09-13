#!/bin/sh
set -eu

PINNED_CHANNEL="1.80.0"

if ! command -v rustup >/dev/null 2>&1; then
    echo "ERROR: rustup required to install pinned toolchain." >&2
    exit 1
fi

rustup toolchain install "$PINNED_CHANNEL" --profile minimal --component rustfmt clippy llvm-tools-preview
rustup target add riscv64-unknown-none-elf --toolchain "$PINNED_CHANNEL"
rustup target add x86_64-unknown-linux-gnu --toolchain "$PINNED_CHANNEL"
rustup target add x86_64-unknown-linux-gnu --toolchain "$PINNED_CHANNEL"

echo "OK: pinned Rust toolchain installation requested."
