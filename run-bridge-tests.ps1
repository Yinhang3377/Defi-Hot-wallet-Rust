Write-Host "=== 逐一运行桥接测试 ===" -ForegroundColor Cyan

$tests = @(
    "test_bridge_eth_to_solana",
    "test_bridge_security", 
    "test_bridge_different_chains",
    "test_bridge_edge_cases"
)

foreach ($test in $tests) {
    Write-Host "`n运行测试: $test" -ForegroundColor Yellow
    cargo test $test --lib --quiet

    if ($LASTEXITCODE -eq 0) {
        Write-Host " $test 通过" -ForegroundColor Green
    } else {
        Write-Host " $test 失败" -ForegroundColor Red
        exit 1
    }
}

Write-Host "`n 所有桥接测试通过！" -ForegroundColor Green
