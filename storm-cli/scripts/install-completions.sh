#!/usr/bin/env bash
# Script to install storm-cli completions for various shells

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BINARY_NAME="storm-cli"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Storm CLI Completion Installer"
echo "=============================="
echo

# Detect the shell
detect_shell() {
    if [ -n "$BASH_VERSION" ]; then
        echo "bash"
    elif [ -n "$ZSH_VERSION" ]; then
        echo "zsh"
    elif [ -n "$FISH_VERSION" ]; then
        echo "fish"
    else
        # Try to detect from SHELL variable
        case "$SHELL" in
            */bash) echo "bash" ;;
            */zsh) echo "zsh" ;;
            */fish) echo "fish" ;;
            *) echo "unknown" ;;
        esac
    fi
}

# Install Bash completions
install_bash() {
    echo "Installing Bash completions..."

    # Common completion directories
    local dirs=(
        "/etc/bash_completion.d"
        "/usr/local/etc/bash_completion.d"
        "$HOME/.local/share/bash-completion/completions"
        "$HOME/.bash_completion.d"
    )

    local install_dir=""
    for dir in "${dirs[@]}"; do
        if [ -d "$dir" ] || [ -w "$(dirname "$dir")" ]; then
            install_dir="$dir"
            break
        fi
    done

    if [ -z "$install_dir" ]; then
        install_dir="$HOME/.bash_completion.d"
        mkdir -p "$install_dir"
    fi

    echo -e "${GREEN}Generating completions...${NC}"
    $BINARY_NAME completion bash > "$install_dir/$BINARY_NAME.bash"

    echo -e "${GREEN}✓ Installed to: $install_dir/$BINARY_NAME.bash${NC}"
    echo
    echo "To use immediately, run:"
    echo -e "${YELLOW}  source $install_dir/$BINARY_NAME.bash${NC}"
    echo
    echo "To use in all future sessions, add to your ~/.bashrc:"
    echo -e "${YELLOW}  source $install_dir/$BINARY_NAME.bash${NC}"
}

# Install Zsh completions
install_zsh() {
    echo "Installing Zsh completions..."

    # Common completion directories
    local dirs=(
        "$HOME/.zsh/completions"
        "$HOME/.config/zsh/completions"
        "/usr/local/share/zsh/site-functions"
        "/usr/share/zsh/site-functions"
    )

    local install_dir=""
    for dir in "${dirs[@]}"; do
        if [ -d "$dir" ] || [ -w "$(dirname "$dir")" ]; then
            install_dir="$dir"
            break
        fi
    done

    if [ -z "$install_dir" ]; then
        install_dir="$HOME/.zsh/completions"
        mkdir -p "$install_dir"
    fi

    echo -e "${GREEN}Generating completions...${NC}"
    $BINARY_NAME completion zsh > "$install_dir/_$BINARY_NAME"

    echo -e "${GREEN}✓ Installed to: $install_dir/_$BINARY_NAME${NC}"
    echo
    echo "Make sure your ~/.zshrc includes the completions directory:"
    echo -e "${YELLOW}  fpath=($install_dir \$fpath)${NC}"
    echo -e "${YELLOW}  autoload -U compinit && compinit${NC}"
    echo
    echo "Then reload your shell or run:"
    echo -e "${YELLOW}  exec zsh${NC}"
}

# Install Fish completions
install_fish() {
    echo "Installing Fish completions..."

    local install_dir="$HOME/.config/fish/completions"
    mkdir -p "$install_dir"

    echo -e "${GREEN}Generating completions...${NC}"
    $BINARY_NAME completion fish > "$install_dir/$BINARY_NAME.fish"

    echo -e "${GREEN}✓ Installed to: $install_dir/$BINARY_NAME.fish${NC}"
    echo
    echo "Completions will be available immediately in new Fish sessions."
}

# Install PowerShell completions
install_powershell() {
    echo "Installing PowerShell completions..."

    local profile_dir
    if [ -n "$USERPROFILE" ]; then
        # Windows
        profile_dir="$USERPROFILE/Documents/PowerShell"
    else
        # Linux/macOS
        profile_dir="$HOME/.config/powershell"
    fi

    mkdir -p "$profile_dir"

    echo -e "${GREEN}Generating completions...${NC}"
    $BINARY_NAME completion powershell > "$profile_dir/$BINARY_NAME.ps1"

    echo -e "${GREEN}✓ Installed to: $profile_dir/$BINARY_NAME.ps1${NC}"
    echo
    echo "Add to your PowerShell profile:"
    echo -e "${YELLOW}  . $profile_dir/$BINARY_NAME.ps1${NC}"
}

# Main installation logic
main() {
    # Check if storm-cli is in PATH
    if ! command -v $BINARY_NAME &> /dev/null; then
        echo -e "${RED}Error: $BINARY_NAME not found in PATH${NC}"
        echo "Please ensure storm-cli is installed and in your PATH"
        exit 1
    fi

    # If shell specified as argument, use it
    if [ -n "$1" ]; then
        case "$1" in
            bash) install_bash ;;
            zsh) install_zsh ;;
            fish) install_fish ;;
            powershell) install_powershell ;;
            *)
                echo -e "${RED}Unknown shell: $1${NC}"
                echo "Supported shells: bash, zsh, fish, powershell"
                exit 1
                ;;
        esac
    else
        # Auto-detect shell
        local detected_shell=$(detect_shell)

        if [ "$detected_shell" = "unknown" ]; then
            echo -e "${YELLOW}Could not detect shell${NC}"
            echo "Please specify shell: $0 [bash|zsh|fish|powershell]"
            exit 1
        fi

        echo "Detected shell: $detected_shell"
        echo

        case "$detected_shell" in
            bash) install_bash ;;
            zsh) install_zsh ;;
            fish) install_fish ;;
            *)
                echo -e "${RED}Unsupported shell: $detected_shell${NC}"
                echo "Please manually specify: $0 [bash|zsh|fish|powershell]"
                exit 1
                ;;
        esac
    fi

    echo
    echo -e "${GREEN}Installation complete!${NC}"
}

main "$@"
