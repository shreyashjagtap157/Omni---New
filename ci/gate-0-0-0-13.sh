#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

compute_sha256() {
    local target_file="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "${target_file}" | awk '{print $1}'
    else
        shasum -a 256 "${target_file}" | awk '{print $1}'
    fi
}

fail() {
    echo "[-] FAIL-CLOSED: $*" >&2
    exit 1
}

echo "================================================================"
echo "=== [Omni Gate 0.0.0.13] Pre-0.0.1.0 Foundation Qualification ==="
echo "================================================================"

cd "${WORKSPACE_ROOT}"

echo "[Step 1/9] Verifying Git worktree status and maintainer identity..."
[[ -z "$(git status --porcelain)" ]] || fail "Dirty working tree detected."
CURRENT_USER_NAME="$(git config --local --get user.name || true)"
CURRENT_USER_EMAIL="$(git config --local --get user.email || true)"
[[ "${CURRENT_USER_NAME}" == "shreyashjagtap157" && "${CURRENT_USER_EMAIL}" == "ssjagtap2016@gmail.com" ]] || fail "Repository author configuration mismatch: ${CURRENT_USER_NAME} <${CURRENT_USER_EMAIL}>"
echo "[+] Step 1 PASS: Worktree clean, maintainer identity matched."

echo "[Step 2/9] Validating normative specification SHA-256..."
SPEC_FILE="${WORKSPACE_ROOT}/docs/specification/Omni_Complete_Specification_Edition1_1.0.0-candidate.1_updated.md"
EXPECTED_SPEC_HASH="e3ff0e1b6ef0f1f1d713647cc3c0c04dfd940513d292bac95f7b0d44e6c95f0c"
[[ -f "${SPEC_FILE}" ]] || fail "Missing normative specification file at ${SPEC_FILE}"
ACTUAL_SPEC_HASH="$(compute_sha256 "${SPEC_FILE}")"
[[ "${ACTUAL_SPEC_HASH}" == "${EXPECTED_SPEC_HASH}" ]] || fail "Specification hash mismatch. Expected ${EXPECTED_SPEC_HASH}, got ${ACTUAL_SPEC_HASH}"
echo "[+] Step 2 PASS: Normative specification hash verified."

echo "[Step 3/9] Validating implementation master plan SHA-256..."
PLAN_FILE="${WORKSPACE_ROOT}/docs/implementation/Omni_Implementation_Master_Plan_Edition1_Final_Comprehensive_MicroAtomic_Final.md"
EXPECTED_PLAN_HASH="cdf7ff8cb0eae597b52e295fa43c11f701415f62f2a292f4bd4a2be5012cb506"
[[ -f "${PLAN_FILE}" ]] || fail "Missing implementation plan file at ${PLAN_FILE}"
ACTUAL_PLAN_HASH="$(compute_sha256 "${PLAN_FILE}")"
[[ "${ACTUAL_PLAN_HASH}" == "${EXPECTED_PLAN_HASH}" ]] || fail "Implementation plan hash mismatch. Expected ${EXPECTED_PLAN_HASH}, got ${ACTUAL_PLAN_HASH}"
echo "[+] Step 3 PASS: Implementation plan hash verified."

echo "[Step 4/9] Running assert-toolchain.sh..."
"${WORKSPACE_ROOT}/ci/assert-toolchain.sh"
echo "[+] Step 4 PASS: Toolchain strictly qualified."

echo "[Step 5/9] Auditing workspace unsafe-code lint policy..."
if ! grep -Eq '^[[:space:]]*unsafe_code[[:space:]]*=[[:space:]]*"forbid"[[:space:]]*$' "${WORKSPACE_ROOT}/Cargo.toml"; then
    fail "Cargo.toml does not declare workspace unsafe_code = \"forbid\"."
fi
echo "[+] Step 5 PASS: Workspace unsafe-code forbid policy verified."

echo "[Step 6/9] Executing cargo check across workspace..."
cargo check --workspace --all-targets
echo "[+] Step 6 PASS: Workspace check succeeded."

echo "[Step 7/9] Executing cargo clippy (-D warnings)..."
cargo clippy --workspace --all-targets -- -D warnings
echo "[+] Step 7 PASS: Zero clippy warnings."

echo "[Step 8/9] Executing cargo fmt check..."
cargo fmt --check
echo "[+] Step 8 PASS: Code formatting clean."

echo "[Step 9/9] Executing workspace test runner..."
cargo test --workspace
echo "[+] Step 9 PASS: Test harness execution nominal."

echo "================================================================"
echo "[+] PASS: OMNI-IMP-0.0.0.13 Foundation Gate PASSED."
echo "    The boundary to OMNI-IMP-0.0.1.0 (Lexer Module) is UNLOCKED."
echo "================================================================"
