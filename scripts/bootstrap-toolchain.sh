#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TOOLCHAIN_FILE="${WORKSPACE_ROOT}/rust-toolchain.toml"

fail() {
    echo "[-] FAIL-CLOSED: $*" >&2
    exit 1
}

echo "=== [Omni Bootstrap] Toolchain Provisioning ==="
[[ -f "${TOOLCHAIN_FILE}" ]] || fail "${TOOLCHAIN_FILE} not found."

export PATH="${HOME}/.cargo/bin:${PATH}"

if ! command -v rustup >/dev/null 2>&1; then
    echo "[*] rustup not detected. Fetching via official standalone installer..."
    command -v curl >/dev/null 2>&1 || fail "'curl' is required to install rustup."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y
    export PATH="${HOME}/.cargo/bin:${PATH}"
fi

command -v rustup >/dev/null 2>&1 || fail "rustup installation did not make rustup available in PATH."

cd "${WORKSPACE_ROOT}"
echo "[*] Installing pinned toolchain components from rust-toolchain.toml..."
rustup toolchain install

EXPECTED_CHANNEL="$(awk -F'"' '/^[[:space:]]*channel[[:space:]]*=/ {print $2}' "${TOOLCHAIN_FILE}")"
[[ -n "${EXPECTED_CHANNEL}" ]] || fail "Unable to determine pinned channel."
rustup target add x86_64-unknown-linux-gnu --toolchain "${EXPECTED_CHANNEL}"
rustup target add aarch64-unknown-linux-gnu --toolchain "${EXPECTED_CHANNEL}"

echo "[*] Running toolchain qualification gate..."
chmod +x "${WORKSPACE_ROOT}/ci/assert-toolchain.sh"
"${WORKSPACE_ROOT}/ci/assert-toolchain.sh"

echo "[+] Bootstrap completed. Source \$HOME/.cargo/env in your current shell if rustup installed it."
