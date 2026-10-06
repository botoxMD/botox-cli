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
elif command -v cargo >/dev/null 2>&1 && [ -f "${SCRIPT_DIR}/Cargo.toml" ]; then
    echo "Compiling release binary with cargo..."
    (cd "${SCRIPT_DIR}" && cargo build --release)
    cp -f "${SCRIPT_DIR}/target/release/botox" "${INSTALL_DIR}/botox"
else
    echo "Fetching prebuilt binary from GitHub Releases..."
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    TARGET=""

    case "${OS}" in
        Linux)
            case "${ARCH}" in
                x86_64) TARGET="x86_64-unknown-linux-musl" ;;
                aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
            esac
            ;;
        Darwin)
            case "${ARCH}" in
                x86_64) TARGET="x86_64-apple-darwin" ;;
                arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
            esac
            ;;
    esac

    if [ -n "${TARGET}" ]; then
        LATEST_URL="https://github.com/botoxMD/botox-cli/releases/latest/download/botox-${TARGET}.tar.gz"
        echo "Downloading ${LATEST_URL}..."
        TMP_DIR="$(mktemp -d)"
        if curl -fsSL "${LATEST_URL}" | tar -xz -C "${TMP_DIR}"; then
            cp -f "${TMP_DIR}/botox" "${INSTALL_DIR}/botox"
            rm -rf "${TMP_DIR}"
        else
            rm -rf "${TMP_DIR}"
            echo "Error: Failed to download prebuilt release for ${TARGET}."
            exit 1
        fi
    else
        echo "Error: Unsupported operating system/architecture: ${OS} ${ARCH}"
        exit 1
    fi
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
