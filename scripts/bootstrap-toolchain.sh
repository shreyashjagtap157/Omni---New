#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TOOLCHAIN_FILE="${WORKSPACE_ROOT}/rust-toolchain.toml"

fail() {
    echo "[-] FAIL-CLOSED: $*" >&2
    exit 101
}

echo "=== [Omni Bootstrap] Toolchain Provisioning ==="

[[ -f "${TOOLCHAIN_FILE}" ]] || fail "${TOOLCHAIN_FILE} not found."

export PATH="${HOME}/.cargo/bin:${PATH}"
EXPECTED_CHANNEL="$(awk -F'"' '/^[[:space:]]*channel[[:space:]]*=/ {print $2}' "${TOOLCHAIN_FILE}")"
[[ -n "${EXPECTED_CHANNEL}" ]] || fail "Unable to determine pinned channel."

# 1. Reuse an already provisioned matching compiler when available.
if command -v rustc >/dev/null 2>&1 && rustc --version | grep -q "${EXPECTED_CHANNEL}"; then
    echo "[+] Rustc channel ${EXPECTED_CHANNEL} already present in PATH."
fi

# 2. Provision rustup from an explicitly mounted offline installer when possible.
if ! command -v rustup >/dev/null 2>&1; then
    CACHE_DIR="${OMNI_TOOLCHAIN_CACHE:-${WORKSPACE_ROOT}/.cache/toolchain}"
    OFFLINE_RUSTUP="${CACHE_DIR}/rustup-init"

    if [[ -f "${OFFLINE_RUSTUP}" ]]; then
        echo "[*] Found pre-staged rustup installer at ${OFFLINE_RUSTUP}."
        chmod +x "${OFFLINE_RUSTUP}"
        "${OFFLINE_RUSTUP}" -y --default-toolchain none
        export PATH="${HOME}/.cargo/bin:${PATH}"
    else
        echo "[*] rustup not detected and no offline installer was found at ${CACHE_DIR}."
        if [[ "${OMNI_AIRGAPPED:-0}" == "1" ]]; then
            fail "Air-gapped provisioning requested, but ${OFFLINE_RUSTUP} is unavailable."
        fi
        command -v curl >/dev/null 2>&1 || fail "'curl' is required for network installation."
        echo "[*] Attempting official rustup installation over the network..."
        if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y; then
            fail "Network installation failed and no offline rustup installer was available at ${CACHE_DIR}."
        fi
        export PATH="${HOME}/.cargo/bin:${PATH}"
    fi
fi

command -v rustup >/dev/null 2>&1 || fail "rustup is unavailable after provisioning."

# 3. Install the exact declarative toolchain and components.
cd "${WORKSPACE_ROOT}"
echo "[*] Installing pinned toolchain from rust-toolchain.toml..."
rustup toolchain install

# 4. Ensure the declaratively pinned targets exist for this toolchain.
rustup target add x86_64-unknown-linux-gnu --toolchain "${EXPECTED_CHANNEL}"
rustup target add aarch64-unknown-linux-gnu --toolchain "${EXPECTED_CHANNEL}"

# 5. Execute the immutable fail-closed qualification assertion.
echo "[*] Running toolchain qualification gate..."
chmod +x "${WORKSPACE_ROOT}/ci/assert-toolchain.sh"
"${WORKSPACE_ROOT}/ci/assert-toolchain.sh"

echo "[+] Bootstrap completed successfully."
echo "    Source \"\$HOME/.cargo/env\" in your interactive shell when rustup installed it."
