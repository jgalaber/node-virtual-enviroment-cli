#!/bin/bash

# NVE (Node Version Environment) Universal Installer
# A fast, cross-platform Node.js version manager written in Rust
# Repository: https://github.com/jgalaber/node-virtual-enviroment-cli

set -e  # Exit on any error

# Configuration
readonly NVE_VERSION="${NVE_VERSION:-latest}"
readonly NVE_HOME="${NVE_HOME:-$HOME/.nve}"
readonly NVE_BIN_DIR="$NVE_HOME/bin"
readonly NVE_CACHE_DIR="$NVE_HOME/cache"
readonly REPO_URL="https://github.com/jgalaber/node-virtual-enviroment-cli"
readonly BINARY_NAME="nve"

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly BOLD='\033[1m'
readonly NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_bold() {
    echo -e "${BOLD}$1${NC}"
}

# Print banner
print_banner() {
    echo -e "${BLUE}"
    cat << 'EOF'
    ███╗   ██╗██╗   ██╗███████╗
    ████╗  ██║██║   ██║██╔════╝
    ██╔██╗ ██║██║   ██║█████╗  
    ██║╚██╗██║╚██╗ ██╔╝██╔══╝  
    ██║ ╚████║ ╚████╔╝ ███████╗
    ╚═╝  ╚═══╝  ╚═══╝  ╚══════╝
    
    Node Version Environment
    Fast, Cross-platform Node.js Version Manager
EOF
    echo -e "${NC}"
}

# Detect OS and architecture
detect_platform() {
    local os arch

    # Detect OS
    case "$(uname -s)" in
        Linux*)
            os="unknown-linux-gnu"
            ;;
        Darwin*)
            os="apple-darwin"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            log_error "Windows detected. Please use the PowerShell installer instead:"
            log_error "Invoke-WebRequest -Uri \"$REPO_URL/raw/main/install.ps1\" -OutFile \"install.ps1\""
            log_error "PowerShell -ExecutionPolicy Bypass -File \"install.ps1\""
            exit 1
            ;;
        *)
            log_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        armv7*)
            arch="armv7"
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac

    PLATFORM="${arch}-${os}"
    log_info "Detected platform: $PLATFORM"
}

# Get the latest release version from GitHub API
get_latest_version() {
    if [ "$NVE_VERSION" = "latest" ]; then
        log_info "Fetching latest version from GitHub..."
        
        # Try multiple methods to get the latest version
        if command -v curl >/dev/null 2>&1; then
            VERSION=$(curl -s "https://api.github.com/repos/jgalaber/node-virtual-enviroment-cli/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/' || echo "")
        elif command -v wget >/dev/null 2>&1; then
            VERSION=$(wget -qO- "https://api.github.com/repos/jgalaber/node-virtual-enviroment-cli/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/' || echo "")
        else
            log_error "Neither curl nor wget found. Please install one of them."
            exit 1
        fi

        # Fallback to default version if API fails
        if [ -z "$VERSION" ] || [ "$VERSION" = "null" ]; then
            log_warn "Could not fetch latest version from GitHub API. Using fallback version v0.1.0"
            VERSION="v0.1.0"
        fi
    else
        VERSION="$NVE_VERSION"
    fi

    # Ensure version starts with 'v'
    if [[ ! "$VERSION" =~ ^v ]]; then
        VERSION="v$VERSION"
    fi

    log_info "Installing NVE version: $VERSION"
}

# Check if required tools are available
check_dependencies() {
    log_info "Checking dependencies..."

    if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
        log_error "Either curl or wget is required for downloading"
        exit 1
    fi

    if ! command -v tar >/dev/null 2>&1; then
        log_error "tar is required for extracting archives"
        exit 1
    fi

    log_success "All dependencies are available"
}

# Create necessary directories
create_directories() {
    log_info "Creating directories..."
    
    mkdir -p "$NVE_BIN_DIR"
    mkdir -p "$NVE_CACHE_DIR"
    mkdir -p "$NVE_HOME/versions"
    
    log_success "Created NVE directories in $NVE_HOME"
}

# Download and extract the binary
download_and_install() {
    local download_url binary_name temp_dir

    binary_name="${BINARY_NAME}-${PLATFORM}"
    download_url="$REPO_URL/releases/download/$VERSION/$binary_name"
    temp_dir=$(mktemp -d)

    log_info "Downloading NVE binary from: $download_url"

    # Download binary
    if command -v curl >/dev/null 2>&1; then
        if ! curl -L --fail --progress-bar "$download_url" -o "$temp_dir/$BINARY_NAME"; then
            log_error "Failed to download NVE binary"
            log_error "URL: $download_url"
            exit 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -q --show-progress "$download_url" -O "$temp_dir/$BINARY_NAME"; then
            log_error "Failed to download NVE binary"
            log_error "URL: $download_url"
            exit 1
        fi
    fi

    log_success "Downloaded NVE binary successfully"

    # Move binary to bin directory and make executable
    mv "$temp_dir/$BINARY_NAME" "$NVE_BIN_DIR/$BINARY_NAME"
    chmod +x "$NVE_BIN_DIR/$BINARY_NAME"

    # Clean up
    rm -rf "$temp_dir"

    log_success "Installed NVE binary to $NVE_BIN_DIR/$BINARY_NAME"
}

# Detect shell and provide setup instructions
setup_shell() {
    local shell_name shell_config

    # Detect current shell
    if [ -n "$BASH_VERSION" ]; then
        shell_name="bash"
        shell_config="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        shell_name="zsh"
        shell_config="$HOME/.zshrc"
    elif [ -n "$FISH_VERSION" ]; then
        shell_name="fish"
        shell_config="$HOME/.config/fish/config.fish"
    else
        # Fallback: detect from $SHELL
        case "$SHELL" in
            */bash)
                shell_name="bash"
                shell_config="$HOME/.bashrc"
                ;;
            */zsh)
                shell_name="zsh"
                shell_config="$HOME/.zshrc"
                ;;
            */fish)
                shell_name="fish"
                shell_config="$HOME/.config/fish/config.fish"
                ;;
            *)
                shell_name="unknown"
                shell_config="$HOME/.profile"
                ;;
        esac
    fi

    log_info "Detected shell: $shell_name"

    # Check if PATH is already configured
    if echo "$PATH" | grep -q "$NVE_BIN_DIR"; then
        log_success "NVE is already in your PATH"
        return 0
    fi

    # Provide shell-specific setup instructions
    case "$shell_name" in
        fish)
            log_warn "Please add the following to your $shell_config:"
            echo
            echo "    set -gx NVE_HOME \"$NVE_HOME\""
            echo "    set -gx PATH \"\$NVE_HOME/current/bin\" \$PATH"
            echo
            log_info "Then reload your shell with: source $shell_config"
            ;;
        *)
            log_warn "Please add the following to your $shell_config:"
            echo
            echo "    export NVE_HOME=\"$NVE_HOME\""
            echo "    export PATH=\"\$NVE_HOME/current/bin:\$PATH\""
            echo
            log_info "Then reload your shell with: source $shell_config"
            ;;
    esac

    # Attempt to add to shell config automatically (with user permission)
    if [ "$SKIP_SHELL_CONFIG" != "1" ]; then
        echo
        read -p "Would you like me to add NVE to your $shell_config automatically? [y/N]: " -r
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            add_to_shell_config "$shell_name" "$shell_config"
        fi
    fi
}

# Add NVE to shell configuration
add_to_shell_config() {
    local shell_name="$1"
    local shell_config="$2"

    # Create config file if it doesn't exist
    if [ ! -f "$shell_config" ]; then
        touch "$shell_config"
    fi

    # Check if already configured
    if grep -q "NVE_HOME" "$shell_config" 2>/dev/null; then
        log_warn "NVE configuration already exists in $shell_config"
        return 0
    fi

    # Add configuration based on shell type
    case "$shell_name" in
        fish)
            {
                echo ""
                echo "# NVE (Node Version Environment)"
                echo "set -gx NVE_HOME \"$NVE_HOME\""
                echo "set -gx PATH \"\$NVE_HOME/current/bin\" \$PATH"
            } >> "$shell_config"
            ;;
        *)
            {
                echo ""
                echo "# NVE (Node Version Environment)"
                echo "export NVE_HOME=\"$NVE_HOME\""
                echo "export PATH=\"\$NVE_HOME/current/bin:\$PATH\""
            } >> "$shell_config"
            ;;
    esac

    log_success "Added NVE configuration to $shell_config"
    log_info "Please restart your terminal or run: source $shell_config"
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."

    if [ ! -f "$NVE_BIN_DIR/$BINARY_NAME" ]; then
        log_error "NVE binary not found at $NVE_BIN_DIR/$BINARY_NAME"
        exit 1
    fi

    if [ ! -x "$NVE_BIN_DIR/$BINARY_NAME" ]; then
        log_error "NVE binary is not executable"
        exit 1
    fi

    # Test if binary works
    if ! "$NVE_BIN_DIR/$BINARY_NAME" --version >/dev/null 2>&1; then
        log_error "NVE binary test failed"
        exit 1
    fi

    log_success "Installation verified successfully"
}

# Print post-installation instructions
print_post_install() {
    echo
    log_bold "🎉 NVE has been installed successfully!"
    echo
    log_info "Installation location: $NVE_HOME"
    log_info "Binary location: $NVE_BIN_DIR/$BINARY_NAME"
    echo
    log_bold "Next steps:"
    echo "  1. Restart your terminal or reload your shell configuration"
    echo "  2. Install a Node.js version: nve install 20"
    echo "  3. Use the installed version: nve use 20"
    echo "  4. Verify Node.js: node -v"
    echo
    log_bold "Quick commands:"
    echo "  nve install 20     # Install Node.js 20.x.x"
    echo "  nve install lts    # Install latest LTS"
    echo "  nve list           # List installed versions"
    echo "  nve use <version>  # Switch to a version"
    echo
    log_info "For more information, visit: $REPO_URL"
    echo
}

# Handle cleanup on exit
cleanup() {
    if [ -n "$temp_dir" ] && [ -d "$temp_dir" ]; then
        rm -rf "$temp_dir"
    fi
}

# Handle interruption
handle_interrupt() {
    log_error "Installation interrupted"
    cleanup
    exit 1
}

# Main installation function
main() {
    # Set up signal handlers
    trap cleanup EXIT
    trap handle_interrupt INT TERM

    print_banner
    
    log_info "Starting NVE installation..."
    echo

    # Perform installation steps
    detect_platform
    get_latest_version
    check_dependencies
    create_directories
    download_and_install
    verify_installation
    setup_shell
    print_post_install

    log_success "Installation completed successfully! 🚀"
}

# Run main function
main "$@"