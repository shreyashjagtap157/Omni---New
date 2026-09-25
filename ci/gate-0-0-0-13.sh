#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

fail() {
    echo "[-] FAIL-CLOSED: $*" >&2
    exit 1
}

echo "================================================================"
echo "=== [Omni Gate 0.0.0.13] Foundation Qualification ==="
echo "================================================================"

cd "${WORKSPACE_ROOT}"

echo "[Step 1/10] Verifying clean Git worktree and exact HEAD..."
[[ -z "$(git status --porcelain)" ]] || fail "Dirty working tree detected."
EXPECTED_HEAD="$(git rev-parse HEAD)"
[[ -n "${EXPECTED_HEAD}" ]] || fail "Unable to resolve repository HEAD."
echo "[+] Step 1 PASS: Worktree clean at ${EXPECTED_HEAD}."

echo "[Step 2/10] Verifying pinned Rust toolchain..."
"${WORKSPACE_ROOT}/ci/assert-toolchain.sh"
echo "[+] Step 2 PASS: Toolchain strictly qualified."

echo "[Step 3/10] Checking formatting..."
cargo fmt --all -- --check
echo "[+] Step 3 PASS: Formatting clean."

echo "[Step 4/10] Checking workspace..."
cargo check -j 1 --workspace --all-targets --locked
echo "[+] Step 4 PASS: Workspace check succeeded."

echo "[Step 5/10] Running strict Clippy..."
cargo clippy -j 1 --workspace --all-targets --locked -- -D warnings
echo "[+] Step 5 PASS: Zero Clippy warnings."

echo "[Step 6/10] Running the complete workspace test suite..."
cargo test -j 1 --workspace --locked -- --test-threads=1
echo "[+] Step 6 PASS: Workspace tests succeeded."

echo "[Step 7/10] Verifying locked metadata and dependency tree..."
cargo metadata --format-version 1 --locked > /dev/null
cargo tree --locked > /dev/null
echo "[+] Step 7 PASS: Metadata and dependency tree verified."

echo "[Step 8/10] Loading the normative specification through omni-registry..."
cargo run -j 1 -p omni-registry --locked -- "${WORKSPACE_ROOT}/spec"
echo "[+] Step 8 PASS: Specification loader verified."

echo "[Step 9/10] Running release/conformance/linkage/topology gates..."
cargo run -j 1 -p omni-conform --locked
cargo run -j 1 -p omni-audit --locked
cargo run -j 1 -p omni-topology --locked
echo "[+] Step 9 PASS: Conformance, linkage, and topology gates passed."

echo "[Step 10/10] Verifying canonical specification-tree identity..."
EXPECTED_DIGEST="$(sed -n 's/^[[:space:]]*"spec_tree_sha256":[[:space:]]*"\([0-9a-f]\{64\}\)",.*$/\1/p' "${WORKSPACE_ROOT}/spec/manifest/omni-edition1.manifest.json" | head -n 1)"
[[ -n "${EXPECTED_DIGEST}" ]] || fail "Unable to read manifest spec_tree_sha256."
ACTUAL_DIGEST="$(cargo run -j 1 -p omni-canon --locked -- --spec-tree "${WORKSPACE_ROOT}/spec")"
[[ "${ACTUAL_DIGEST}" == "${EXPECTED_DIGEST}" ]] || fail "Specification-tree digest mismatch: expected ${EXPECTED_DIGEST}, got ${ACTUAL_DIGEST}"
echo "[+] Step 10 PASS: Spec-tree digest ${ACTUAL_DIGEST} verified against the manifest."

echo "================================================================"
echo "[+] PASS: OMNI-IMP-0.0.0.13 Foundation Gate PASSED."
echo "    The boundary to OMNI-IMP-0.0.1.0 (Lexer Module) is UNLOCKED."
echo "================================================================"
