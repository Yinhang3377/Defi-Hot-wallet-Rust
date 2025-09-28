<#
.SYNOPSIS
  测试所有桥接功能。
.DESCRIPTION
  此脚本按顺序执行所有定义的跨链桥接测试，并报告每个步骤。
  如果任何步骤失败，脚本将立即停止。
#>
$ErrorActionPreference = 'Stop'

# 设置控制台输出编码为 UTF-8，以正确显示中文字符
$OutputEncoding = [System.Text.Encoding]::UTF8

function Invoke-Test {
    param(
        [string]$TestName,
        [scriptblock]$TestCommand
    )
    Write-Host "========== $TestName ==========" -ForegroundColor Green
    & $TestCommand
    Write-Host ""
}

Write-Host "🚀 开始执行所有桥接功能测试..." -ForegroundColor Cyan
Write-Host ""

Invoke-Test "ETH -> SOL 测试" { cargo run --bin bridge_test -- eth-to-sol --amount 50.0 --token USDC }

Invoke-Test "SOL -> ETH 测试" { cargo run --bin bridge_test -- sol-to-eth --amount 25.0 --token USDT }

Invoke-Test "ETH -> BSC 测试" { cargo run --bin bridge_test -- eth-to-bsc --amount 100.0 --token BUSD }

Write-Host "✅ 所有桥接测试完成！" -ForegroundColor Cyan