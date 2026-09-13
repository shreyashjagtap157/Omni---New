#!/bin/sh
set -eu

PINNED_CHANNEL="1.80.0"
PINNED_TARGET="riscv64-unknown-none-elf"

if ! command -v rustc >/dev/null 2>&1; then
    echo "FATAL: rustc is not installed or not in PATH." >&2
    echo "BLOCKER: OMNI-IMP-0.0.0.2 qualification requires an active Rust toolchain." >&2
    exit 101
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "FATAL: cargo is not installed or not in PATH." >&2
    exit 102
fi

if ! command -v rustup >/dev/null 2>&1; then
    echo "FATAL: rustup is required for target verification." >&2
    exit 105
fi

ACTIVE_VERSION=$(rustc --version | cut -d' ' -f2)
if [ "$ACTIVE_VERSION" != "$PINNED_CHANNEL" ]; then
    echo "FATAL: Toolchain version mismatch. Expected $PINNED_CHANNEL, found $ACTIVE_VERSION" >&2
    exit 103
fi

if ! rustup target list --installed | grep -F -x -q "$PINNED_TARGET"; then
    echo "FATAL: Required cross-compilation target $PINNED_TARGET is not installed." >&2
    exit 104
fi

echo "OK: Rust toolchain $ACTIVE_VERSION ($PINNED_TARGET) verified."
