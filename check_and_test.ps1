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
  return (Get-Command $name -ErrorAction SilentlyContinue) -ne $null
}

# 1. Clean
Write-Info "Cleaning: cargo clean"
if (Test-Command -name 'cargo') {
  & cargo clean
} else {
  Write-Err "cargo not found in PATH"
  exit 1
}

# 2. Format check
Write-Info "Format check: cargo fmt --all -- --check"
if (Test-Command -name 'cargo' -and (Try { & cargo fmt --version > $null 2>&1; $true } Catch { $false })) {
  try {
    & cargo fmt --all -- --check
  } catch {
    Write-Warn "Formatting check failed. Running 'cargo fmt --all' to fix."
    & cargo fmt --all
  }
} else {
  Write-Warn "cargo fmt (rustfmt) not available. Skipping format check."
}

# 3. Static checks
Write-Info "Static analysis: cargo clippy (deny warnings) if available"
if (Test-Command -name 'cargo' -and (Try { & cargo clippy --version > $null 2>&1; $true } Catch { $false })) {
  & cargo clippy --all-targets --all-features -- -D warnings
} else {
  Write-Warn "cargo clippy not available. Skipping clippy."
}

Write-Info "Type check / build check: cargo check --all-targets --all-features"
& cargo check --all-targets --all-features

# Optional: cargo-audit
if (Test-Command -name 'cargo' -and (Try { & cargo audit --version > $null 2>&1; $true } Catch { $false })) {
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