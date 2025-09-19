#!/bin/bash

# DeFi Hot Wallet Demo Script
# 演示脚本 - 展示钱包的主要功能

set -e

echo "🔒 DeFi Hot Wallet - Demo Script"
echo "================================"

# 检查二进制文件是否存在
if [ ! -f "./target/release/wallet-cli" ]; then
    echo "❌ wallet-cli not found. Building project..."
    cargo build --release
fi

WALLET_CLI="./target/release/wallet-cli"

echo ""
echo "1. 🔧 Creating a quantum-safe wallet..."
echo "   创建量子安全钱包..."
$WALLET_CLI create --name demo-wallet --quantum true

echo ""
echo "2. 🔑 Generating a new mnemonic phrase..."
echo "   生成新的助记词..."
$WALLET_CLI generate-mnemonic

echo ""
echo "3. 💰 Checking wallet balance on Ethereum..."
echo "   检查以太坊钱包余额..."
$WALLET_CLI balance --wallet demo-wallet --network eth || echo "   (Simulated - would show real balance with proper setup)"

echo ""
echo "4. 💸 Simulating a transaction..."
echo "   模拟发送交易..."
echo "   Sending 0.1 ETH to test address..."
$WALLET_CLI send \
    --wallet demo-wallet \
    --to 0x742d35Cc6635C0532925a3b8D400e8B78fFe4860 \
    --amount 0.1 \
    --network eth || echo "   (Simulated - would send real transaction with proper setup)"

echo ""
echo "5. 🛡️ Checking security status..."
echo "   检查安全状态..."
$WALLET_CLI security

echo ""
echo "6. ℹ️ Wallet information..."
echo "   钱包信息..."
$WALLET_CLI info --wallet demo-wallet

echo ""
echo "7. 💾 Creating backup..."
echo "   创建备份..."
$WALLET_CLI backup --wallet demo-wallet --output ./demo-wallet-backup.json

echo ""
echo "8. 🌍 Testing internationalization..."
echo "   测试国际化..."
echo "   English interface:"
$WALLET_CLI --language en info --wallet demo-wallet
echo ""
echo "   中文界面:"
$WALLET_CLI --language zh info --wallet demo-wallet

echo ""
echo "✅ Demo completed successfully!"
echo "   演示完成！"
echo ""
echo "📚 Next steps:"
echo "   - Start the server: $WALLET_CLI server"
echo "   - Check API: curl http://localhost:8080/api/health"
echo "   - View metrics: curl http://localhost:8080/api/metrics"
echo ""
echo "🔐 Security reminders:"
echo "   - This is a demonstration/development version"
echo "   - Do not use with real funds without proper security audit"
echo "   - Always backup your wallet data"
echo "   - Keep your private keys secure"