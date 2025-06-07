#!/usr/bin/env bash

set -e

# Configuration
APP_NAME="tide"
APP_VERSION="0.1.0"
GITHUB_REPO="builtbyjb/tide"
INSTALL_DIR="$HOME/.local/bin"

# Detect os and architecture
detect_platform() {
  local os arch
  case "$(uname -s)" in
    Linux*) os="linux" ;;
    Darwin*) os="macos" ;;
    CYGWIN*|MINGW*|MSYS*) os="windows" ;;
    *)
    echo "Unsupported operating system: $(uname -s)"
    exit 1
    ;;
  esac

  case "$(uname -m)" in
    x86_64|amd64) arch="x86_64" ;;
    arm64|aarch64) arch="arm64" ;;
    arm7l) arch="armv7" ;;
    *)
    echo "Unsupported architecture: $(uname -m)"
    exit 1
    ;;
  esac 

  echo "${os}-${arch}"
}

install_binary() {
  local platform="$1"
  local download_url="https://github.com/builtbyjb/tide/releases/download/v$APP_VERSION/$APP_NAME-$platform"

  if [ ! -d "$INSTALL_DIR" ]; then
    mkdir -p "$INSTALL_DIR"
  fi

  # Download binary
  curl -LsSf -o "$APP_NAME" "$download_url"

  chmod +x "$APP_NAME"

  if [ "$platform" = "macos-arm64" ]; then
    # Remove com.app.quarantine attribute if it exists
    if xattr -p com.app.quarantine "$APP_NAME" >/dev/null 2>&1; then
      xattr -d "com.apple.quarantine" "$APP_NAME"
    fi
  fi

  mv "$APP_NAME" "$INSTALL_DIR"
}

verify_installation() {
  if [ -x "$INSTALL_DIR/$APP_NAME" ]; then
    echo "Installation successful!"
    echo "Check current version with: $APP_NAME --version"
  else
    echo "Installation failed: binary not found at $INSTALL_DIR/$APP_NAME"
    exit 1
  fi
}

main() {
  local platform=$(detect_platform)
  # Check if the binary exists before installing
  if [ -f "$INSTALL_DIR/$APP_NAME" ]; then
    local current_version
    current_version=$("$INSTALL_DIR/$APP_NAME" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")

    if [ "$current_version" = "$APP_VERSION" ]; then 
      echo "Application is up-to-date (version $APP_VERSION)"
      exit 1
    elif [ "$current_version" = "unknown" ]; then
      echo "Could not determine current version. Proceeding with installation"
    fi
    rm -rf "$INSTALL_DIR/$APP_NAME"
  fi

  install_binary "$platform"
  verify_installation
}

main