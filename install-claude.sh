#!/usr/bin/env bash
set -euo pipefail

# Claude one-click installer for macOS/Linux
# Supports both interactive prompts and CLI params.
YES=0
TOKEN=""
BASE_URL=""
OS_NAME=""
ARCH_NAME=""
VOLTA_HOME="${VOLTA_HOME:-$HOME/.volta}"
export VOLTA_HOME

with_volta_path() {
  local volta_bin="$VOLTA_HOME/bin"
  if [[ -d "$volta_bin" && ":$PATH:" != *":$volta_bin:"* ]]; then
    export PATH="$volta_bin:$PATH"
  fi
}

require_command() {
  local binary="$1"
  local guidance="$2"
  if ! command -v "$binary" >/dev/null 2>&1; then
    echo "Error: required command '$binary' not found. $guidance" >&2
    exit 10
  fi
}

detect_platform() {
  local uname_out
  local arch_out
  uname_out=$(uname -s 2>/dev/null || echo unknown)
  arch_out=$(uname -m 2>/dev/null || echo unknown)
  case "$uname_out" in
    Darwin) OS_NAME="macOS" ;;
    Linux) OS_NAME="Linux" ;;
    *)
      echo "Error: unsupported operating system '$uname_out'. Only macOS and Linux are supported." >&2
      exit 12
      ;;
  esac
  case "$arch_out" in
    x86_64|amd64) ARCH_NAME="x86_64" ;;
    arm64|aarch64) ARCH_NAME="arm64" ;;
    *)
      ARCH_NAME="$arch_out"
      echo "Warning: unverified architecture '$arch_out'." >&2
      ;;
  esac
  echo "Detected $OS_NAME ($ARCH_NAME) environment."
}

ensure_curl() {
  require_command "curl" "Install curl to download Volta (see https://curl.se)."
}

normalize_base_url() {
  if [[ -z "${BASE_URL:-}" ]]; then
    return
  fi
  local original="$BASE_URL"
  local normalized="$BASE_URL"
  while [[ "$normalized" == */ ]]; do
    normalized="${normalized%/}"
  done
  BASE_URL="$normalized"
  if [[ "$BASE_URL" != "$original" ]]; then
    echo "Normalized BASE_URL to $BASE_URL"
  fi
}

confirm_action() {
  local prompt="$1"
  if [[ $YES -eq 1 ]]; then
    return 0
  fi
  read -r -p "$prompt [y/N]: " reply
  case "$reply" in
    y|Y|yes|YES) return 0 ;;
    *) return 1 ;;
  esac
}

install_volta() {
  with_volta_path
  if command -v volta >/dev/null 2>&1; then
    return
  fi
  ensure_curl
  echo "Volta not detected. It will be used to manage the required Node.js version."
  if ! confirm_action "Install Volta via curl https://get.volta.sh | bash?"; then
    echo "Volta is required to provision Node.js automatically. Please install it manually and re-run." >&2
    exit 13
  fi
  curl -sSf https://get.volta.sh | bash
  with_volta_path
  if ! command -v volta >/dev/null 2>&1; then
    echo "Error: Volta installation failed or PATH not configured (expected $VOLTA_HOME/bin)." >&2
    exit 13
  fi
}

install_node_via_volta() {
  install_volta
  if ! confirm_action "Install Node.js 20.12.0 (pinned by Volta)?"; then
    echo "Node.js 20.12.0 is required. Aborting as per user request." >&2
    exit 11
  fi
  echo "Installing Node.js 20.12.0 via Volta..."
  volta install node@20.12.0
  with_volta_path
}

ensure_node() {
  local version=""
  local need_install=0
  local major=0
  local minor=0

  if command -v node >/dev/null 2>&1; then
    version=$(node -v 2>/dev/null || true)
  fi

  if [[ -z "$version" ]]; then
    echo "Node.js not detected."
    need_install=1
  elif [[ $version =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+) ]]; then
    major=${BASH_REMATCH[1]}
    minor=${BASH_REMATCH[2]}
    if (( major < 18 )); then
      echo "Node.js $version detected but >= 18 is required."
      need_install=1
    elif (( major < 20 || (major == 20 && minor < 12) )); then
      echo "Node.js $version detected; installing recommended 20.12.0 via Volta."
      need_install=1
    fi
  else
    echo "Unable to parse Node.js version '$version'. Installing Node.js 20.12.0 via Volta for consistency."
    need_install=1
  fi

  if (( need_install == 1 )); then
    install_node_via_volta
  fi

  with_volta_path
  version=$(node -v 2>/dev/null || true)
  if [[ ! $version =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+) ]]; then
    echo "Error: failed to verify Node.js version after installation (got '$version')." >&2
    exit 11
  fi
  major=${BASH_REMATCH[1]}
  minor=${BASH_REMATCH[2]}
  if (( major < 18 )); then
    echo "Error: Node.js version $version is below required 18." >&2
    exit 11
  fi
  if (( major < 20 || (major == 20 && minor < 12) )); then
    echo "Warning: Node.js $version detected. Version 20.12.0 is recommended (Volta install)." >&2
  else
    echo "Detected Node.js $version."
  fi
}

ensure_npm() {
  if ! command -v npm >/dev/null 2>&1; then
    echo "Error: npm not found. Please ensure your Node.js installation provides npm." >&2
    exit 17
  fi
}

install_claude_cli() {
  local original_registry
  original_registry=$(npm config get registry 2>/dev/null || echo "")
  npm config set registry https://registry.npmmirror.com >/dev/null 2>&1 || true
  echo "Installing @anthropic-ai/claude-code globally via npm..."
  if npm install -g @anthropic-ai/claude-code >/dev/null 2>&1; then
    echo "✅ Installed '@anthropic-ai/claude-code'"
  else
    echo "⚠️ Global install failed, retrying with output..."
    if ! npm install -g @anthropic-ai/claude-code; then
      if [[ -n "$original_registry" ]]; then
        npm config set registry "$original_registry" >/dev/null 2>&1 || true
      fi
      return 1
    fi
  fi
  if [[ -n "$original_registry" ]]; then
    npm config set registry "$original_registry" >/dev/null 2>&1 || true
  fi
  return 0
}

ensure_claude_cli() {
  if command -v claude >/dev/null 2>&1; then
    echo "Detected 'claude' CLI: $(claude --version 2>/dev/null || echo present)"
    return
  fi
  ensure_npm
  install_claude_cli || {
    echo "Error: failed to install '@anthropic-ai/claude-code'. Consider using a Node version manager or adjusting npm permissions." >&2
    exit 18
  }
  if ! command -v claude >/dev/null 2>&1; then
    echo "Error: 'claude' CLI still not found after installation." >&2
    exit 18
  fi
}


print_help() {
  cat <<USAGE
Usage: install-claude.sh [--token <token>] [--base-url <url>] [-y|--yes]

Options:
  --token, -t       ANTHROPIC_AUTH_TOKEN. If omitted, prompt interactively.
  --base-url, -u    ANTHROPIC_BASE_URL. REQUIRED. No default.
  --yes, -y         Non-interactive; skip confirmation.
  --help, -h        Show this help.

You can also set env vars before running:
  ANTHROPIC_AUTH_TOKEN, ANTHROPIC_BASE_URL
USAGE
}

# Parse args
while [[ ${#} -gt 0 ]]; do
  case "$1" in
    --token|-t) TOKEN="$2"; shift 2;;
    --base-url|-u) BASE_URL="$2"; shift 2;;
    --yes|-y) YES=1; shift;;
    --help|-h) print_help; exit 0;;
    *) echo "Unknown option: $1" >&2; print_help; exit 1;;
  esac
done

# Fallback to env vars (no default)
TOKEN="${TOKEN:-${ANTHROPIC_AUTH_TOKEN:-}}"
BASE_URL="${BASE_URL:-${ANTHROPIC_BASE_URL:-}}"

detect_platform
ensure_node
ensure_claude_cli

prompt_if_needed() {
  if [[ -z "$TOKEN" ]]; then
    read -r -s -p "Enter ANTHROPIC_AUTH_TOKEN (input hidden): " TOKEN; echo
    if [[ -z "$TOKEN" ]]; then
      echo "Error: token is required." >&2; exit 2
    fi
  fi
  if [[ -z "$BASE_URL" ]]; then
    read -r -p "Enter ANTHROPIC_BASE_URL (e.g. https://api.anthropic.com): " BASE_URL || true
    if [[ -z "$BASE_URL" ]]; then
      echo "Error: base URL is required." >&2; exit 2
    fi
  fi
}

if [[ $YES -eq 0 ]]; then
  prompt_if_needed
  normalize_base_url
  echo "About to write settings to "$HOME/.claude/settings.json" with:"
  echo "  ANTHROPIC_AUTH_TOKEN=********"
  echo "  ANTHROPIC_BASE_URL=$BASE_URL"
  read -r -p "Proceed? [y/N]: " CONFIRM
  case "$CONFIRM" in
    y|Y|yes|YES) ;;
    *) echo "Aborted."; exit 3;;
  esac
else
  # Non-interactive mode: still ensure we have values
  if [[ -z "$TOKEN" ]]; then
    echo "Error: --token or ANTHROPIC_AUTH_TOKEN required in non-interactive mode." >&2; exit 2
  fi
  if [[ -z "$BASE_URL" ]]; then
    echo "Error: --base-url or ANTHROPIC_BASE_URL is required in non-interactive mode." >&2; exit 2
  fi
  normalize_base_url
fi

TARGET_DIR="$HOME/.claude"
TARGET_FILE="$TARGET_DIR/settings.json"
CONFIG_FILE="$TARGET_DIR/config.json"
mkdir -p "$TARGET_DIR"

# Write JSON
cat > "$TARGET_FILE" <<JSON
{
  "env": {
    "ANTHROPIC_AUTH_TOKEN": "$TOKEN",
    "ANTHROPIC_BASE_URL": "$BASE_URL",
    "CLAUDE_CODE_MAX_OUTPUT_TOKENS": "64000",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
    "API_TIMEOUT_MS": "600000",
    "BASH_DEFAULT_TIMEOUT_MS": "600000",
    "BASH_MAX_TIMEOUT_MS": "600000",
    "MCP_TIMEOUT": "30000",
    "MCP_TOOL_TIMEOUT": "600000",
    "CLAUDE_API_TIMEOUT": "600000"
  },
  "permissions": {
    "allow": [],
    "deny": []
  }
}
JSON

# Restrict file perms on Unix
chmod 600 "$TARGET_FILE" 2>/dev/null || true

echo "✅ Wrote $TARGET_FILE"
echo "   ANTHROPIC_AUTH_TOKEN=$TOKEN"
echo "   ANTHROPIC_BASE_URL=$BASE_URL"

cat > "$CONFIG_FILE" <<'JSON'
{
  "primaryApiKey": "default"
}
JSON

chmod 600 "$CONFIG_FILE" 2>/dev/null || true
echo "✅ Wrote $CONFIG_FILE"
