#!/usr/bin/env bash
#
# Botox Installer & Updater
# Pure Rust Markdown-to-PDF & Slides Typesetting Engine
# https://github.com/botoxMD/botox-cli
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/botoxMD/botox-cli/main/install.sh | bash
#   ./install.sh [--version <vX.Y.Z>] [--dir <path>] [--force]
#

set -euo pipefail

REPO="botoxMD/botox-cli"
BIN_NAME="botox"
TEMP_DIR=""

cleanup() {
    if [ -n "${TEMP_DIR:-}" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT INT TERM

# Formatting helpers
setup_colors() {
    if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
        COLOR_RESET="\033[0m"
        COLOR_BOLD="\033[1m"
        COLOR_GREEN="\033[32m"
        COLOR_BLUE="\033[34m"
        COLOR_YELLOW="\033[33m"
        COLOR_RED="\033[31m"
        COLOR_CYAN="\033[36m"
    else
        COLOR_RESET=""
        COLOR_BOLD=""
        COLOR_GREEN=""
        COLOR_BLUE=""
        COLOR_YELLOW=""
        COLOR_RED=""
        COLOR_CYAN=""
    fi
}

info() {
    printf "%b==>%b %b%s%b\n" "$COLOR_BLUE" "$COLOR_RESET" "$COLOR_BOLD" "$1" "$COLOR_RESET"
}

success() {
    printf "%b==>%b %b%s%b\n" "$COLOR_GREEN" "$COLOR_RESET" "$COLOR_BOLD" "$1" "$COLOR_RESET"
}

warn() {
    printf "%bwarning:%b %s\n" "$COLOR_YELLOW" "$COLOR_RESET" "$1"
}

error() {
    printf "%berror:%b %s\n" "$COLOR_RED" "$COLOR_RESET" "$1" >&2
    exit 1
}

# Check download utilities
fetch() {
    local url="$1"
    local dest="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fSL --progress-bar "$url" -o "$dest"
    elif command -v wget >/dev/null 2>&1; then
        wget -q --show-progress "$url" -O "$dest"
    else
        error "Neither curl nor wget was found. Please install curl or wget to continue."
    fi
}

fetch_header() {
    local url="$1"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSLI -o /dev/null -w "%{url_effective}" "$url" 2>/dev/null || true
    elif command -v wget >/dev/null 2>&1; then
        wget --spider -S "$url" 2>&1 | awk -F': ' '/^  Location: /{print $2}' | tail -n1
    else
        echo ""
    fi
}

# Detect Operating System and Architecture
detect_platform() {
    local os arch libc target

    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$arch" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            error "Unsupported architecture: $arch. Pre-built binaries are available for x86_64 and aarch64."
            ;;
    esac

    case "$os" in
        linux)
            libc="gnu"
            if command -v ldd >/dev/null 2>&1; then
                if ldd --version 2>&1 | grep -qi "musl"; then
                    libc="musl"
                fi
            elif ls /lib/ld-musl* /lib/libc.musl* >/dev/null 2>&1; then
                libc="musl"
            fi

            if [ "$libc" = "musl" ] && [ "$arch" = "x86_64" ]; then
                target="x86_64-unknown-linux-musl"
            else
                target="${arch}-unknown-linux-gnu"
            fi
            ;;
        darwin)
            target="${arch}-apple-darwin"
            ;;
        msys*|mingw*|cygwin*)
            if [ "$arch" = "x86_64" ]; then
                target="x86_64-pc-windows-msvc"
            else
                error "Unsupported Windows architecture: $arch. Only x86_64 is currently provided."
            fi
            ;;
        *)
            error "Unsupported operating system: $os. Botox supports Linux, macOS, and Windows."
            ;;
    esac

    echo "$target"
}

# Resolve latest release tag without API rate limits
resolve_latest_tag() {
    local effective_url tag
    effective_url=$(fetch_header "https://github.com/${REPO}/releases/latest")
    tag=$(echo "$effective_url" | sed -E 's#.*/tag/([^/]+).*#\1#')

    if [ -z "$tag" ] || [ "$tag" = "$effective_url" ]; then
        # Fallback to GitHub API with or without auth token
        local auth_header=()
        if [ -n "${GITHUB_TOKEN:-}" ]; then
            auth_header=(-H "Authorization: token ${GITHUB_TOKEN}")
        fi
        if command -v curl >/dev/null 2>&1; then
            tag=$(curl -fsSL "${auth_header[@]}" "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
                  | grep '"tag_name":' | head -n1 | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' || true)
        fi
    fi

    if [ -z "$tag" ]; then
        error "Could not determine the latest release tag for ${REPO}. Please specify a version with --version <vX.Y.Z>."
    fi

    echo "$tag"
}

usage() {
    cat <<EOF
Botox Installer & Updater

Usage:
  install.sh [options]

Options:
  -v, --version <tag>     Install a specific version (e.g. v0.2.0) [default: latest]
  -d, --dir <path>        Installation directory [default: ~/.local/bin or current binary directory]
  -f, --force             Force install/reinstall even if already at the requested version
  -h, --help              Show this help message

Environment Variables:
  VERSION=<tag>           Target release version
  INSTALL_DIR=<path>      Custom installation directory
  GITHUB_TOKEN=<token>    GitHub personal access token for higher API rate limits
  NO_COLOR=1              Disable ANSI color output
EOF
}

main() {
    setup_colors

    local requested_version="${VERSION:-}"
    local custom_install_dir="${INSTALL_DIR:-}"
    local force_install=false

    # Parse CLI flags
    while [ $# -gt 0 ]; do
        case "$1" in
            -v|--version)
                requested_version="$2"
                shift 2
                ;;
            -d|--dir)
                custom_install_dir="$2"
                shift 2
                ;;
            -f|--force)
                force_install=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                error "Unknown option: $1 (use --help for usage)"
                ;;
        esac
    done

    printf "%b%bBotox Installer & Updater%b\n\n" "$COLOR_BOLD" "$COLOR_CYAN" "$COLOR_RESET"

    # 1. Detect platform target
    info "Detecting system platform..."
    local target
    target=$(detect_platform)
    printf "  Detected target: %b%s%b\n" "$COLOR_CYAN" "$target" "$COLOR_RESET"

    # 2. Determine target version
    local tag
    if [ -n "$requested_version" ]; then
        tag="$requested_version"
        # Ensure 'v' prefix
        case "$tag" in
            v*) ;;
            *) tag="v$tag" ;;
        esac
        info "Requested version: ${tag}"
    else
        info "Checking latest release..."
        tag=$(resolve_latest_tag)
        printf "  Latest version:  %b%s%b\n" "$COLOR_CYAN" "$tag" "$COLOR_RESET"
    fi

    # 3. Detect existing installation for update
    local existing_bin=""
    local existing_version=""
    if command -v "$BIN_NAME" >/dev/null 2>&1; then
        existing_bin=$(command -v "$BIN_NAME")
        if command -v readlink >/dev/null 2>&1; then
            existing_bin=$(readlink -f "$existing_bin" 2>/dev/null || echo "$existing_bin")
        fi
        existing_version=$("$existing_bin" --version 2>/dev/null | awk '{print $2}' || true)
    fi

    local target_clean_version="${tag#v}"
    if [ -n "$existing_version" ] && [ "$existing_version" = "$target_clean_version" ] && [ "$force_install" = false ]; then
        success "Botox is already up to date!"
        printf "  Version:  %b%s%b\n" "$COLOR_GREEN" "$existing_version" "$COLOR_RESET"
        printf "  Location: %s\n" "$existing_bin"
        printf "  (Run with %b--force%b to reinstall)\n" "$COLOR_BOLD" "$COLOR_RESET"
        exit 0
    fi

    # 4. Determine destination directory
    local install_dir=""
    if [ -n "$custom_install_dir" ]; then
        install_dir="$custom_install_dir"
    elif [ -n "$existing_bin" ]; then
        install_dir=$(dirname "$existing_bin")
    else
        # Fresh installation default
        if [ "$(id -u)" -eq 0 ]; then
            install_dir="/usr/local/bin"
        elif [ -d "$HOME/.local/bin" ] || [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
            install_dir="$HOME/.local/bin"
        elif [ -d "$HOME/.cargo/bin" ] && [[ ":$PATH:" == *":$HOME/.cargo/bin:"* ]]; then
            install_dir="$HOME/.cargo/bin"
        else
            install_dir="$HOME/.local/bin"
        fi
    fi

    # Check if we need sudo/root privileges for install_dir
    local use_sudo=false
    mkdir -p "$install_dir" 2>/dev/null || true
    if [ ! -w "$install_dir" ]; then
        if command -v sudo >/dev/null 2>&1 && [ "$(id -u)" -ne 0 ]; then
            use_sudo=true
            warn "Elevated permissions required to write to $install_dir (sudo will be used)."
        else
            error "Destination directory '$install_dir' is not writable and sudo is unavailable."
        fi
    fi

    # 5. Download archive
    local archive_ext="tar.gz"
    case "$target" in
        *windows*) archive_ext="zip" ;;
    esac

    local archive_name="${BIN_NAME}-${tag}-${target}.${archive_ext}"
    local download_url="https://github.com/${REPO}/releases/download/${tag}/${archive_name}"

    TEMP_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'botox-install')
    local archive_path="${TEMP_DIR}/${archive_name}"

    if [ -n "$existing_version" ]; then
        info "Updating Botox (${existing_version} -> ${target_clean_version})..."
    else
        info "Downloading Botox ${tag} for ${target}..."
    fi

    fetch "$download_url" "$archive_path"

    # 6. Extract binary
    info "Extracting executable..."
    if [ "$archive_ext" = "tar.gz" ]; then
        tar -xzf "$archive_path" -C "$TEMP_DIR"
    elif [ "$archive_ext" = "zip" ]; then
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$archive_path" -d "$TEMP_DIR"
        else
            error "Unzip command not found. Please install unzip to extract the Windows archive."
        fi
    fi

    local extracted_bin="${TEMP_DIR}/${BIN_NAME}"
    if [ ! -f "$extracted_bin" ] && [ -f "${TEMP_DIR}/${BIN_NAME}.exe" ]; then
        extracted_bin="${TEMP_DIR}/${BIN_NAME}.exe"
    fi

    if [ ! -f "$extracted_bin" ]; then
        error "Archive extraction failed: executable '$BIN_NAME' was not found in release archive."
    fi

    chmod 755 "$extracted_bin"

    # 7. Atomically Install Binary
    info "Installing to ${install_dir}/${BIN_NAME}..."
    local temp_target="${install_dir}/${BIN_NAME}.tmp.$$"

    if [ "$use_sudo" = true ]; then
        sudo mkdir -p "$install_dir"
        sudo cp -f "$extracted_bin" "$temp_target"
        sudo chmod 755 "$temp_target"
        sudo mv -f "$temp_target" "${install_dir}/${BIN_NAME}"
    else
        mkdir -p "$install_dir"
        cp -f "$extracted_bin" "$temp_target"
        chmod 755 "$temp_target"
        mv -f "$temp_target" "${install_dir}/${BIN_NAME}"
    fi

    # 8. Verify installation
    local installed_path="${install_dir}/${BIN_NAME}"
    local installed_ver
    installed_ver=$("$installed_path" --version 2>/dev/null || echo "installed")

    printf "\n"
    success "Successfully installed ${installed_ver} at ${installed_path}!"

    # 9. PATH Verification & Guidance
    local in_path=false
    case ":$PATH:" in
        *":$install_dir:"*) in_path=true ;;
    esac

    if [ "$in_path" = false ]; then
        printf "\n"
        warn "${install_dir} is not currently in your system PATH."
        printf "To make '${BIN_NAME}' available everywhere, add this to your shell profile:\n\n"
        local user_shell
        user_shell=$(basename "${SHELL:-bash}")
        case "$user_shell" in
            zsh)
                printf "  %becho 'export PATH=\"%s:\$PATH\"' >> ~/.zshrc%b\n" "$COLOR_BOLD" "$install_dir" "$COLOR_RESET"
                printf "  %bsource ~/.zshrc%b\n\n" "$COLOR_BOLD" "$COLOR_RESET"
                ;;
            fish)
                printf "  %bfish_add_path %s%b\n\n" "$COLOR_BOLD" "$install_dir" "$COLOR_RESET"
                ;;
            *)
                printf "  %becho 'export PATH=\"%s:\$PATH\"' >> ~/.bashrc%b\n" "$COLOR_BOLD" "$install_dir" "$COLOR_RESET"
                printf "  %bsource ~/.bashrc%b\n\n" "$COLOR_BOLD" "$COLOR_RESET"
                ;;
        esac
    else
        printf "\nRun '%bbotox --help%b' to get started.\n" "$COLOR_BOLD" "$COLOR_RESET"
    fi
}

main "$@"
