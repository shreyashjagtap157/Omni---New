#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMP_ROOT="$(mktemp -d /tmp/omni-memory-models.XXXXXX)"
trap 'rm -rf -- "$TEMP_ROOT"' EXIT
cd "$REPO_ROOT"

SPIKE="research/memory-model/spike.rs"
DOMAIN="research/memory-model/domain_lifetimes.rs"

for source in "$SPIKE" "$DOMAIN"; do
    [[ -f "$source" ]] || { echo "Missing research model: $source" >&2; exit 1; }
done

echo "[memory-model] Formatting isolated research models with Rust 1.95.0"
rustfmt +1.95.0 --edition 2021 --check "$SPIKE" "$DOMAIN"

for model in spike domain_lifetimes; do
    source="research/memory-model/$model.rs"
    binary="$TEMP_ROOT/$model"
    echo "[memory-model] Compiling $source"
    rustc +1.95.0 --edition=2021 --test -D warnings "$source" -o "$binary"
    echo "[memory-model] Executing $model"
    "$binary" --test-threads=1
done

echo "[memory-model] PASS: both isolated research models formatted, compiled and passed."
