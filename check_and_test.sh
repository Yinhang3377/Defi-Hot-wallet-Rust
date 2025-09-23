#!/usr/bin/env bash
# check_and_test.sh
# Cross-project helper: clean, format check, static checks, and tests.
# Also emits a companion PowerShell script (check_and_test.ps1) for Windows.

set -euo pipefail

info() { echo -e "\033[1;34m[INFO]\033[0m $*"; }
warn() { echo -e "\033[1;33m[WARN]\033[0m $*"; }
err()  { echo -e "\033[1;31m[ERROR]\033[0m $*"; }

check_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    return 1
  fi
  return 0
}

# 1. Clean
info "Cleaning: cargo clean"
if check_cmd cargo; then
  cargo clean
else
  err "cargo not found in PATH"
  exit 1
fi

# 2. Format check (rustfmt / cargo fmt)
info "Format check: cargo fmt --all -- --check"
if cargo fmt --version >/dev/null 2>&1; then
  if ! cargo fmt --all -- --check; then
    err "Formatting check failed. Run 'cargo fmt --all' to fix."
    exit 1
  fi
else
  warn "cargo fmt (rustfmt) not available. Skipping format check."
fi

# 3. Static checks: clippy + cargo check
info "Static analysis: cargo clippy (deny warnings) if available"
if cargo clippy --version >/dev/null 2>&1; then
  # -D warnings to fail on warnings; adjust as needed
  cargo clippy --all-targets --all-features -- -D warnings
else
  warn "cargo clippy not available. Skipping clippy."
fi

info "Type check / build check: cargo check --all-targets --all-features"
cargo check --all-targets --all-features

# Optional: cargo-audit if installed
if cargo audit --version >/dev/null 2>&1; then
  info "Security audit: cargo audit"
  cargo audit || warn "cargo-audit reported issues"
else
  warn "cargo-audit not installed. Skipping dependency audit."
fi

# 4. Tests
info "Running tests: cargo test --all -- --nocapture"
cargo test --all -- --nocapture

# 5. Generate Windows PowerShell companion script
info "Generating companion PowerShell script: check_and_test.ps1"
cat > check_and_test.ps1 <<'PS1'
<#
.SYNOPSIS
  check_and_test.ps1 - Clean, format check, static checks and run tests on Windows PowerShell.

.NOTES
  Designed for PowerShell (Windows / PowerShell Core on Linux).
#>

$ErrorActionPreference = 'Stop'

function Write-Info { Write-Host ("[INFO]  " + ($args -join ' ')) -ForegroundColor Cyan }
function Write-Warn { Write-Host ("[WARN]  " + ($args -join ' ')) -ForegroundColor Yellow }
function Write-Err  { Write-Host ("[ERROR] " + ($args -join ' ')) -ForegroundColor Red }

# 1. Clean
Write-Info "Cleaning: cargo clean"
if (Get-Command 'cargo' -ErrorAction SilentlyContinue) {
  & cargo clean
} else {
  Write-Err "cargo not found in PATH"
  exit 1
}

# 2. Format check
Write-Info "Format check: cargo fmt --all -- --check"
if ((Get-Command 'cargo' -ErrorAction SilentlyContinue) -and (Get-Command 'cargo-fmt' -ErrorAction SilentlyContinue)) {
  try {
    & cargo fmt --all -- --check
  } catch {
    Write-Err "Formatting check failed. Run 'cargo fmt --all' to fix."
    exit 1
  }
} else {
  Write-Warn "cargo fmt (rustfmt) not available. Skipping format check."
}

# 3. Static checks
Write-Info "Static analysis: cargo clippy (deny warnings) if available"
if ((Get-Command 'cargo' -ErrorAction SilentlyContinue) -and (Get-Command 'cargo-clippy' -ErrorAction SilentlyContinue)) {
  & cargo clippy --all-targets --all-features -- --deny warnings
} else {
  Write-Warn "cargo clippy not available. Skipping clippy."
}

Write-Info "Type check / build check: cargo check --all-targets --all-features"
& cargo check --all-targets --all-features

# Optional: cargo-audit
if ((Get-Command 'cargo' -ErrorAction SilentlyContinue) -and (Get-Command 'cargo-audit' -ErrorAction SilentlyContinue)) {
  Write-Info "Security audit: cargo audit"
  try {
    & cargo audit
  } catch {
    Write-Warn "cargo-audit reported issues."
  }
} else {
  Write-Warn "cargo-audit not installed. Skipping dependency audit."
}

# 4. Tests
Write-Info "Running tests: cargo test --all -- --nocapture"
& cargo test --all -- --nocapture

Write-Info "All done."
PS1

chmod +x check_and_test.ps1 >/dev/null 2>&1 || true
info "Done. Use './check_and_test.sh' on Linux/WSL or '.\check_and_test.ps1' on PowerShell."