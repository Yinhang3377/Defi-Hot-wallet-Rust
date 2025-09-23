# ...existing code...
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

function Test-Command($name) {
  return $null -ne (Get-Command $name -ErrorAction SilentlyContinue)
}

# 1. Clean
Write-Info "Cleaning: cargo clean"
if (Test-Command 'cargo') {
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
# ...existing code...