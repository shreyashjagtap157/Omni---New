#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TOOLCHAIN_FILE="${WORKSPACE_ROOT}/rust-toolchain.toml"

EXPECTED_COMPONENTS=("rustfmt" "clippy" "rust-src" "llvm-tools-preview")
# Foundation contract targets per spec/release/foundation-gate.json:
# x86_64-unknown-linux-gnu + riscv64-unknown-none-elf.
# Rust 1.95.0 renamed riscv64-unknown-none-elf -> riscv64gc-unknown-none-elf;
# assert the canonical 1.95.0 name.
REQUIRED_TARGETS=("x86_64-unknown-linux-gnu" "riscv64gc-unknown-none-elf")

fail() {
    echo "[-] FAIL-CLOSED: $*" >&2
    exit 101
}

echo "=== [Omni CI] Toolchain Qualification Gate ==="

[[ -f "${TOOLCHAIN_FILE}" ]] || fail "Toolchain definition file missing at ${TOOLCHAIN_FILE}"

EXPECTED_CHANNEL="$(awk -F'"' '/^[[:space:]]*channel[[:space:]]*=/ {print $2}' "${TOOLCHAIN_FILE}" | tr -d '\r')"
[[ -n "${EXPECTED_CHANNEL}" ]] || fail "Failed to parse 'channel' from ${TOOLCHAIN_FILE}"
echo "[+] Pinned toolchain channel: ${EXPECTED_CHANNEL}"

for bin in rustup rustc cargo rustfmt cargo-clippy; do
    command -v "${bin}" >/dev/null 2>&1 || fail "Required binary '${bin}' not found in PATH."
done

ACTIVE_TOOLCHAIN="$(rustup show active-toolchain | tr -d '\r' | awk '{print $1}')"
case "${ACTIVE_TOOLCHAIN}" in
    "${EXPECTED_CHANNEL}"|"${EXPECTED_CHANNEL}-"*) ;;
    *) fail "Active rustup toolchain (${ACTIVE_TOOLCHAIN}) does not match pinned channel (${EXPECTED_CHANNEL})." ;;
esac

ACTIVE_RUSTC_VERSION="$(rustc --version | tr -d '\r')"
echo "[+] Detected rustc version: ${ACTIVE_RUSTC_VERSION}"
grep -F -q " ${EXPECTED_CHANNEL} " <<<" ${ACTIVE_RUSTC_VERSION} " || fail "Active rustc (${ACTIVE_RUSTC_VERSION}) does not match pinned channel (${EXPECTED_CHANNEL})."

INSTALLED_COMPONENTS="$(rustup component list --installed --toolchain "${EXPECTED_CHANNEL}" 2>/dev/null | tr -d '\r' || true)"
for component in "${EXPECTED_COMPONENTS[@]}"; do
    base_component="${component%-preview}"
    grep -E -q "^(${component}|${base_component})(-|$)" <<<"${INSTALLED_COMPONENTS}" || fail "Required component '${component}' is not installed for ${EXPECTED_CHANNEL}."
    echo "[+] Verified component: ${component}"
done

INSTALLED_TARGETS="$(rustup target list --installed --toolchain "${EXPECTED_CHANNEL}" | tr -d '\r')"
for target in "${REQUIRED_TARGETS[@]}"; do
    grep -F -x -q "${target}" <<<"${INSTALLED_TARGETS}" || fail "Required target '${target}' is not installed for ${EXPECTED_CHANNEL}."
    echo "[+] Verified target: ${target}"
done

rustfmt --version >/dev/null 2>&1 || fail "rustfmt execution failed."
cargo clippy --version >/dev/null 2>&1 || fail "cargo-clippy execution failed."

if [[ "$(rustup run "${EXPECTED_CHANNEL}" rustc --version | tr -d '\r')" != "${ACTIVE_RUSTC_VERSION}" ]]; then
    fail "Active rustc differs from rustup toolchain ${EXPECTED_CHANNEL} rustc."
fi

echo "[+] PASS: Toolchain qualification verified successfully against ${EXPECTED_CHANNEL}."
exit 0