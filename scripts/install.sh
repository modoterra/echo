#!/usr/bin/env bash
# Echo / xo user install (XDG layout, ADR 0014).
#
# Usage:
#   ./scripts/install.sh              # install (default)
#   ./scripts/install.sh install
#   ./scripts/install.sh upgrade      # install new version, flip current
#   ./scripts/install.sh uninstall [--purge]
#   ./scripts/install.sh doctor
#   ./scripts/install.sh paths
#
# Layout (defaults; override via env):
#   Toolchain:  ${XDG_DATA_HOME:-~/.local/share}/xo/toolchains/<version>/{bin,std}
#   Current:    ${XDG_DATA_HOME:-~/.local/share}/xo/current -> toolchains/<version>
#   PATH link:  ${XO_BIN_DIR:-~/.local/bin}/xo
#   Packages:   ${XO_HOME:-${XDG_CACHE_HOME:-~/.cache}/.xo}/packages
#   State:      ${XDG_STATE_HOME:-~/.local/state}/xo   (REPL history, …)
#   Config:     ${XDG_CONFIG_HOME:-~/.config}/xo
#
# Upgrade is atomic: build into a version dir, then repoint `current`.
# Previous toolchains remain until uninstall --purge or manual prune.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

die() {
  echo "install.sh: $*" >&2
  exit 1
}

info() {
  echo "install.sh: $*" >&2
}

# --- path resolution (mirrors ADR 0014 + docs/install.md) -------------------

xdg_data_home() {
  if [[ -n "${XDG_DATA_HOME:-}" ]]; then
    printf '%s' "$XDG_DATA_HOME"
  else
    printf '%s' "${HOME}/.local/share"
  fi
}

xdg_cache_home() {
  if [[ -n "${XDG_CACHE_HOME:-}" ]]; then
    printf '%s' "$XDG_CACHE_HOME"
  else
    printf '%s' "${HOME}/.cache"
  fi
}

xdg_state_home() {
  if [[ -n "${XDG_STATE_HOME:-}" ]]; then
    printf '%s' "$XDG_STATE_HOME"
  else
    printf '%s' "${HOME}/.local/state"
  fi
}

xdg_config_home() {
  if [[ -n "${XDG_CONFIG_HOME:-}" ]]; then
    printf '%s' "$XDG_CONFIG_HOME"
  else
    printf '%s' "${HOME}/.config"
  fi
}

# User package / tool cache root (same order as echo_resolver::xo_home).
xo_home() {
  if [[ -n "${XO_HOME:-}" ]]; then
    printf '%s' "$XO_HOME"
  else
    printf '%s' "$(xdg_cache_home)/.xo"
  fi
}

xo_data_root() {
  printf '%s' "$(xdg_data_home)/xo"
}

xo_bin_dir() {
  printf '%s' "${XO_BIN_DIR:-${HOME}/.local/bin}"
}

toolchains_dir() {
  printf '%s' "$(xo_data_root)/toolchains"
}

current_link() {
  printf '%s' "$(xo_data_root)/current"
}

manifest_path() {
  printf '%s' "$(xo_data_root)/install.manifest"
}

resolve_version() {
  if [[ -n "${ECHO_VERSION:-}" ]]; then
    printf '%s' "$ECHO_VERSION"
    return
  fi
  if [[ -n "${XO_VERSION:-}" ]]; then
    printf '%s' "$XO_VERSION"
    return
  fi
  if command -v git >/dev/null 2>&1 && [[ -d "${REPO_ROOT}/.git" ]]; then
    local desc
    desc="$(git -C "$REPO_ROOT" describe --tags --always --dirty 2>/dev/null || true)"
    if [[ -n "$desc" ]]; then
      # Sanitize for directory names.
      printf '%s' "${desc//\//-}"
      return
    fi
  fi
  # Fallback: crates/xo/Cargo.toml version
  if [[ -f "${REPO_ROOT}/crates/xo/Cargo.toml" ]]; then
    local ver
    ver="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "${REPO_ROOT}/crates/xo/Cargo.toml" | head -1)"
    if [[ -n "$ver" ]]; then
      printf '%s' "v${ver}"
      return
    fi
  fi
  printf '%s' "dev"
}

# --- layout -----------------------------------------------------------------

ensure_xdg_layout() {
  local xo home_pkg state cfg data
  xo="$(xo_home)"
  home_pkg="${xo}/packages"
  state="$(xdg_state_home)/xo"
  cfg="$(xdg_config_home)/xo"
  data="$(xo_data_root)"
  mkdir -p \
    "$home_pkg" \
    "$state" \
    "$cfg" \
    "${data}/toolchains" \
    "$(xo_bin_dir)"
  # Lightweight markers so doctor/uninstall know these are ours.
  [[ -f "${xo}/.echo-user-root" ]] || printf 'xo user root (ADR 0014)\n' >"${xo}/.echo-user-root"
  [[ -f "${data}/.echo-install-root" ]] || printf 'xo toolchain install root\n' >"${data}/.echo-install-root"
  [[ -f "${state}/.keep" ]] || : >"${state}/.keep"
  [[ -f "${cfg}/.keep" ]] || : >"${cfg}/.keep"
}

print_paths() {
  cat <<EOF
XO_HOME=$(xo_home)
XO_PACKAGES=$(xo_home)/packages
XO_DATA=$(xo_data_root)
XO_TOOLCHAINS=$(toolchains_dir)
XO_CURRENT=$(current_link)
XO_BIN_DIR=$(xo_bin_dir)
XO_BIN=$(xo_bin_dir)/xo
XDG_STATE_HOME/xo=$(xdg_state_home)/xo
XDG_CONFIG_HOME/xo=$(xdg_config_home)/xo
XO_INSTALL_ROOT (runtime)=$(if [[ -L "$(current_link)" || -d "$(current_link)" ]]; then readlink -f "$(current_link)" 2>/dev/null || printf '%s' "$(current_link)"; else echo "(not installed)"; fi)
VERSION=$(resolve_version)
REPO_ROOT=${REPO_ROOT}
EOF
}

doctor() {
  print_paths
  echo
  local bin cur
  bin="$(xo_bin_dir)/xo"
  cur="$(current_link)"
  if [[ -e "$bin" ]]; then
    if [[ -L "$bin" ]]; then
      info "PATH link: $bin -> $(readlink "$bin" 2>/dev/null || echo '?')"
    else
      info "PATH link: $bin (not a symlink)"
    fi
    if command -v "$bin" >/dev/null 2>&1 || [[ -x "$bin" ]]; then
      info "xo --help:"
      "$bin" --help 2>&1 | head -3 || true
    fi
  else
    info "PATH link missing: $bin"
  fi
  if [[ -L "$cur" || -d "$cur" ]]; then
    info "current toolchain: $(readlink -f "$cur" 2>/dev/null || echo "$cur")"
    if [[ -d "$(readlink -f "$cur" 2>/dev/null || echo "$cur")/std" ]]; then
      info "std present under current"
    else
      info "warning: std/ missing under current"
    fi
  else
    info "no current toolchain at $cur"
  fi
  if [[ -d "$(toolchains_dir)" ]]; then
    info "installed toolchains:"
    # shellcheck disable=SC2012
    ls -1 "$(toolchains_dir)" 2>/dev/null | sed 's/^/  /' || true
  fi
  case ":${PATH}:" in
  *":$(xo_bin_dir):"*) info "PATH contains $(xo_bin_dir)" ;;
  *) info "note: add $(xo_bin_dir) to PATH if xo is not found" ;;
  esac
}

# --- build / install --------------------------------------------------------

build_xo() {
  local profile="${CARGO_PROFILE:-release}"
  info "building xo (${profile}) in ${REPO_ROOT}"
  if [[ ! -f "${REPO_ROOT}/Cargo.toml" ]]; then
    die "not an Echo checkout (missing Cargo.toml at ${REPO_ROOT})"
  fi
  if ! command -v cargo >/dev/null 2>&1; then
    die "cargo not found (install Rust: https://rustup.rs)"
  fi
  (
    cd "$REPO_ROOT"
    if [[ "$profile" == "release" ]]; then
      cargo build -p xo --release
    else
      cargo build -p xo
    fi
  )
  if [[ "$profile" == "release" ]]; then
    printf '%s' "${REPO_ROOT}/target/release/xo"
  else
    printf '%s' "${REPO_ROOT}/target/debug/xo"
  fi
}

write_manifest() {
  local version="$1"
  local prefix="$2"
  local built_from="$3"
  cat >"$(manifest_path)" <<EOF
# Generated by scripts/install.sh — do not edit by hand unless you know why.
version=${version}
prefix=${prefix}
bin=$(xo_bin_dir)/xo
built_from=${built_from}
installed_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
EOF
}

install_version() {
  local version
  version="$(resolve_version)"
  local tc_dir
  tc_dir="$(toolchains_dir)/${version}"
  local staging
  staging="$(toolchains_dir)/.${version}.staging.$$"
  local built
  built="$(build_xo)"
  [[ -x "$built" ]] || die "built binary not executable: $built"

  ensure_xdg_layout
  rm -rf "$staging"
  mkdir -p "${staging}/bin" "${staging}/std"

  info "installing toolchain ${version} → ${tc_dir}"
  install -m 0755 "$built" "${staging}/bin/xo"
  if [[ -d "${REPO_ROOT}/std" ]]; then
    # Prefer rsync when available (preserves structure cleanly).
    if command -v rsync >/dev/null 2>&1; then
      rsync -a --delete "${REPO_ROOT}/std/" "${staging}/std/"
    else
      rm -rf "${staging}/std"
      cp -a "${REPO_ROOT}/std" "${staging}/std"
    fi
  else
    die "missing std/ at ${REPO_ROOT}/std"
  fi
  printf '%s\n' "$version" >"${staging}/version"
  printf '%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >"${staging}/installed_at"

  # Atomic replace of version dir.
  rm -rf "$tc_dir"
  mv "$staging" "$tc_dir"

  # Flip current + PATH entry.
  ln -sfn "toolchains/${version}" "$(current_link)"
  # Prefer relative link from bin dir when possible; absolute is fine and clearer.
  ln -sfn "$(readlink -f "$(current_link)")/bin/xo" "$(xo_bin_dir)/xo"

  write_manifest "$version" "$tc_dir" "$REPO_ROOT"
  info "installed xo ${version}"
  info "  binary: $(xo_bin_dir)/xo"
  info "  root:   $tc_dir"
  info "  XO_HOME packages: $(xo_home)/packages"
  case ":${PATH}:" in
  *":$(xo_bin_dir):"*) ;;
  *)
    info "add to shell profile if needed:"
    info "  export PATH=\"$(xo_bin_dir):\$PATH\""
    ;;
  esac
}

cmd_install() {
  ensure_xdg_layout
  install_version
  doctor
}

cmd_upgrade() {
  ensure_xdg_layout
  local prev=""
  if [[ -L "$(current_link)" || -d "$(current_link)" ]]; then
    prev="$(readlink -f "$(current_link)" 2>/dev/null || true)"
  fi
  install_version
  if [[ -n "$prev" ]]; then
    info "previous toolchain kept at: $prev"
    info "remove old toolchains under $(toolchains_dir) when ready"
  fi
}

cmd_uninstall() {
  local purge=0
  for a in "$@"; do
    case "$a" in
    --purge) purge=1 ;;
    -h | --help)
      cat <<'EOF'
Usage: install.sh uninstall [--purge]

  Removes the PATH link and toolchain install under XDG data.
  --purge  also removes $XO_HOME (packages), state, and config.
EOF
      return 0
      ;;
    *) die "unknown uninstall option: $a" ;;
    esac
  done

  local bin cur data
  bin="$(xo_bin_dir)/xo"
  cur="$(current_link)"
  data="$(xo_data_root)"

  if [[ -L "$bin" ]]; then
    local target
    target="$(readlink -f "$bin" 2>/dev/null || true)"
    if [[ -n "$target" && "$target" == "$(xo_data_root)"/* ]]; then
      rm -f "$bin"
      info "removed $bin"
    else
      info "left $bin (does not point into $(xo_data_root))"
    fi
  elif [[ -e "$bin" ]]; then
    info "left $bin (not a symlink managed by install.sh)"
  fi

  if [[ -d "$data" ]]; then
    rm -rf "$data"
    info "removed toolchain data $data"
  fi

  if [[ "$purge" -eq 1 ]]; then
    local xo state cfg
    xo="$(xo_home)"
    state="$(xdg_state_home)/xo"
    cfg="$(xdg_config_home)/xo"
    if [[ -d "$xo" ]]; then
      rm -rf "$xo"
      info "purged XO_HOME $xo"
    fi
    if [[ -d "$state" ]]; then
      rm -rf "$state"
      info "purged state $state"
    fi
    if [[ -d "$cfg" ]]; then
      rm -rf "$cfg"
      info "purged config $cfg"
    fi
  else
    info "kept package cache $(xo_home)/packages (use --purge to remove)"
    info "kept state $(xdg_state_home)/xo and config $(xdg_config_home)/xo"
  fi
}

usage() {
  cat <<'EOF'
Echo / xo installer (XDG)

Usage:
  scripts/install.sh [install]          Build release xo + install (default)
  scripts/install.sh upgrade            Install a new version and switch current
  scripts/install.sh uninstall [--purge]
  scripts/install.sh doctor             Show paths and install status
  scripts/install.sh paths              Print path assignments only

Environment:
  XO_HOME           User .xo root (packages); default $XDG_CACHE_HOME/.xo
  XDG_DATA_HOME     Toolchain root parent (…/xo/toolchains)
  XDG_STATE_HOME    State (REPL history under …/xo)
  XDG_CONFIG_HOME   Config (…/xo)
  XO_BIN_DIR        Where to place the xo PATH link (default ~/.local/bin)
  ECHO_VERSION / XO_VERSION   Force toolchain version directory name
  CARGO_PROFILE     release (default) or debug

Upgrade keeps prior toolchains under $XDG_DATA_HOME/xo/toolchains/.
EOF
}

main() {
  local cmd="${1:-install}"
  shift || true
  case "$cmd" in
  install | i) cmd_install "$@" ;;
  upgrade | u) cmd_upgrade "$@" ;;
  uninstall | remove | rm) cmd_uninstall "$@" ;;
  doctor | status) doctor "$@" ;;
  paths) print_paths ;;
  -h | --help | help) usage ;;
  *)
    usage >&2
    die "unknown command: $cmd"
    ;;
  esac
}

main "$@"
