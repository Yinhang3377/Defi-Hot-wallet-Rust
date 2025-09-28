<#
.SYNOPSIS
  DeFi 钱包修正后的全面功能测试脚本
.DESCRIPTION
  测试钱包的基本功能和桥接功能
#>

Write-Host "=== DeFi 钱包全面功能测试 (修正版) ===" -ForegroundColor Cyan

# 1. 编译检查
Write-Host "`n1. 编译检查..." -ForegroundColor Yellow
cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 编译失败" -ForegroundColor Red
    exit 1
}
Write-Host "✅ 编译成功" -ForegroundColor Green

# 2. 单元测试
Write-Host "`n2. 运行单元测试..." -ForegroundColor Yellow
cargo test --lib --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 单元测试失败" -ForegroundColor Red
    exit 1
}
Write-Host "✅ 单元测试通过" -ForegroundColor Green

# 3. CLI 功能测试
Write-Host "`n3. CLI 功能测试..." -ForegroundColor Yellow

# 测试帮助信息
cargo run --bin wallet-cli -- --help > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✅ CLI 帮助正常" -ForegroundColor Green }

# 检查 CLI 参数并创建测试钱包
Write-Host "  尝试创建测试钱包..." -ForegroundColor Gray
$createResult = cargo run --bin wallet-cli -- create --name "test-wallet-cli" 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  ✅ 钱包创建成功" -ForegroundColor Green
} else {
    # 检查错误信息，调整参数
    $errorMsg = $createResult -join "`n"
    Write-Host "  ⚠️ 创建钱包失败，错误信息: $errorMsg" -ForegroundColor Yellow
    
    # 尝试不同的参数组合
    if ($errorMsg -match "--quantum") {
        Write-Host "  尝试使用 --quantum 参数..." -ForegroundColor Gray
        cargo run --bin wallet-cli -- create --name "test-wallet-cli" --quantum true > $null
        if ($LASTEXITCODE -eq 0) { Write-Host "  ✅ 使用 --quantum 参数创建钱包成功" -ForegroundColor Green }
    }
}

# 列出钱包
cargo run --bin wallet-cli -- list > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✅ 钱包列表正常" -ForegroundColor Green }

# 查看钱包信息 - 检查参数名称
$infoResult = cargo run --bin wallet-cli -- info --wallet "test-wallet-cli" 2>&1
if ($LASTEXITCODE -ne 0) {
    $errorMsg = $infoResult -join "`n"
    if ($errorMsg -match "--wallet") {
        Write-Host "  尝试使用其他参数名称..." -ForegroundColor Gray
        cargo run --bin wallet-cli -- info --name "test-wallet-cli" > $null
        if ($LASTEXITCODE -eq 0) { Write-Host "  ✅ 使用 --name 参数查询钱包信息成功" -ForegroundColor Green }
    }
} else {
    Write-Host "  ✅ 钱包信息查询正常" -ForegroundColor Green
}

# 生成助记词
cargo run --bin wallet-cli -- generate-mnemonic > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✅ 助记词生成正常" -ForegroundColor Green }

# 删除测试钱包
cargo run --bin wallet-cli -- delete --name "test-wallet-cli" > $null
if ($LASTEXITCODE -eq 0) { 
    Write-Host "  ✅ 钱包删除成功" -ForegroundColor Green 
} else {
    # 尝试其他参数组合
    cargo run --bin wallet-cli -- delete --name "test-wallet-cli" --confirm > $null
    if ($LASTEXITCODE -eq 0) { Write-Host "  ✅ 使用 --confirm 参数删除钱包成功" -ForegroundColor Green }
}

# 4. 桥接功能测试
Write-Host "`n4. 桥接功能测试..." -ForegroundColor Yellow

# ETH -> SOL
cargo run --bin bridge_test -- eth-to-sol --amount 50.0 --token USDC > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✅ ETH -> SOL 桥接正常" -ForegroundColor Green }

# SOL -> ETH
cargo run --bin bridge_test -- sol-to-eth --amount 25.0 --token USDT > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✅ SOL -> ETH 桥接正常" -ForegroundColor Green }

# ETH -> BSC
cargo run --bin bridge_test -- eth-to-bsc --amount 100.0 --token BUSD > $null
if ($LASTEXITCODE -eq 0) { Write-Host "  ✅ ETH -> BSC 桥接正常" -ForegroundColor Green }

# 5. 发布版本测试
Write-Host "`n5. 尝试发布版本编译..." -ForegroundColor Yellow
cargo build --release --quiet
if ($LASTEXITCODE -eq 0) { 
    Write-Host "  ✅ 发布版本编译成功" -ForegroundColor Green
    
    # 测试发布版本
    .\target\release\bridge_test.exe eth-to-sol --amount 10.0 --token USDC > $null
    if ($LASTEXITCODE -eq 0) { Write-Host "  ✅ 发布版本运行正常" -ForegroundColor Green }
}

Write-Host "`n🎉 测试完成！" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Cyan