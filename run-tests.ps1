<#
.SYNOPSIS
  DeFi Wallet comprehensive test script
.DESCRIPTION
  Tests basic wallet functionality and bridge features
#>

Write-Host "=== DeFi Wallet Comprehensive Test ===" -ForegroundColor Cyan

# 1. Compilation check
Write-Host "`n1. Compilation check..." -ForegroundColor Yellow
cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Compilation failed" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Compilation successful" -ForegroundColor Green

# 2. Unit tests
Write-Host "`n2. Running unit tests..." -ForegroundColor Yellow
cargo test --lib --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Unit tests failed" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Unit tests passed" -ForegroundColor Green

# 3. CLI functionality tests
Write-Host "`n3. CLI functionality tests..." -ForegroundColor Yellow

# Test help information
cargo run --bin wallet-cli -- --help > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✓ CLI help works" -ForegroundColor Green }

# Check CLI parameters and create test wallet
Write-Host "  Trying to create test wallet..." -ForegroundColor Gray
$createResult = cargo run --bin wallet-cli -- create --name "test-wallet-cli" 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  ✓ Wallet creation successful" -ForegroundColor Green
} else {
    # Check error message, adjust parameters
    $errorMsg = $createResult -join "`n"
    Write-Host "  ⚠️ Wallet creation failed, error: $errorMsg" -ForegroundColor Yellow
    
    # Try different parameter combinations
    if ($errorMsg -match "--quantum") {
        Write-Host "  Trying with --quantum parameter..." -ForegroundColor Gray
        cargo run --bin wallet-cli -- create --name "test-wallet-cli" --quantum true > $null
        if ($LASTEXITCODE -eq 0) { Write-Host "  ✓ Created wallet with --quantum parameter" -ForegroundColor Green }
    }
}

# List wallets
cargo run --bin wallet-cli -- list > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✓ Wallet listing works" -ForegroundColor Green }

# View wallet info - check parameter names
$infoResult = cargo run --bin wallet-cli -- info --wallet "test-wallet-cli" 2>&1
if ($LASTEXITCODE -ne 0) {
    $errorMsg = $infoResult -join "`n"
    if ($errorMsg -match "--wallet") {
        Write-Host "  Trying other parameter names..." -ForegroundColor Gray
        cargo run --bin wallet-cli -- info --name "test-wallet-cli" > $null
        if ($LASTEXITCODE -eq 0) { Write-Host "  ✓ Wallet info query works with --name" -ForegroundColor Green }
    }
} else {
    Write-Host "  ✓ Wallet info query works" -ForegroundColor Green
}

# Generate mnemonic
cargo run --bin wallet-cli -- generate-mnemonic > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✓ Mnemonic generation works" -ForegroundColor Green }

# Delete test wallet
cargo run --bin wallet-cli -- delete --name "test-wallet-cli" > $null
if ($LASTEXITCODE -eq 0) { 
    Write-Host "  ✓ Wallet deletion works" -ForegroundColor Green 
} else {
    # Try other parameter combinations
    cargo run --bin wallet-cli -- delete --name "test-wallet-cli" --confirm > $null
    if ($LASTEXITCODE -eq 0) { Write-Host "  ✓ Wallet deletion with --confirm works" -ForegroundColor Green }
}

# 4. Bridge functionality tests
Write-Host "`n4. Bridge functionality tests..." -ForegroundColor Yellow

# ETH -> SOL
cargo run --bin bridge_test -- eth-to-sol --amount 50.0 --token USDC > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✓ ETH -> SOL bridge works" -ForegroundColor Green }

# SOL -> ETH
cargo run --bin bridge_test -- sol-to-eth --amount 25.0 --token USDT > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✓ SOL -> ETH bridge works" -ForegroundColor Green }

# ETH -> BSC
cargo run --bin bridge_test -- eth-to-bsc --amount 100.0 --token BUSD > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✓ ETH -> BSC bridge works" -ForegroundColor Green }

# 5. Release version test
Write-Host "`n5. Testing release build..." -ForegroundColor Yellow
cargo build --release --quiet
if ($LASTEXITCODE -eq 0) { 
    Write-Host "  ✓ Release build successful" -ForegroundColor Green
    
    # Test release version
    .\target\release\bridge_test.exe eth-to-sol --amount 10.0 --token USDC > $null
    if ($LASTEXITCODE -eq 0) { Write-Host "  ✓ Release version runs correctly" -ForegroundColor Green }
}

Write-Host "`n🎉 Tests completed!" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Cyan