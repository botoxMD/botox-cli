#!/usr/bin/env bash
set -e

# Botox Installer
# Copies or builds the self-contained single native binary into ~/.local/bin

INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/botox"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

mkdir -p "${INSTALL_DIR}" "${CONFIG_DIR}"

echo "=========================================="
echo "Installing Botox Single Native Binary"
echo "=========================================="

if [ -f "${SCRIPT_DIR}/target/release/botox" ]; then
    echo "Installing compiled release binary..."
    rm -f "${INSTALL_DIR}/botox"
    cp "${SCRIPT_DIR}/target/release/botox" "${INSTALL_DIR}/botox"
elif [ -f "${SCRIPT_DIR}/bin/botox" ]; then
    echo "Installing binary from bin/..."
    cp -f "${SCRIPT_DIR}/bin/botox" "${INSTALL_DIR}/botox"
elif command -v cargo >/dev/null 2>&1; then
    echo "Compiling release binary with cargo..."
    (cd "${SCRIPT_DIR}" && cargo build --release)
    strip "${SCRIPT_DIR}/target/release/botox"
    cp -f "${SCRIPT_DIR}/target/release/botox" "${INSTALL_DIR}/botox"
else
    echo "Error: Neither prebuilt binary nor cargo found."
    echo "Please install Rust (cargo) or place prebuilt botox binary into ${INSTALL_DIR}."
    exit 1
fi

chmod +x "${INSTALL_DIR}/botox"

# Copy default config if none exists
if [ ! -f "${CONFIG_DIR}/config.yaml" ] && [ -f "${SCRIPT_DIR}/botox.yaml" ]; then
    cp "${SCRIPT_DIR}/botox.yaml" "${CONFIG_DIR}/config.yaml"
    echo "Installed default configuration to ${CONFIG_DIR}/config.yaml"
fi

echo ""
echo "Installation complete: ${INSTALL_DIR}/botox"
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo "Notice: Ensure ${INSTALL_DIR} is in your PATH."
    echo "Add this to your ~/.bashrc or ~/.zshrc:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
fi

echo ""
echo "Verify installation:"
echo "  botox --help"
