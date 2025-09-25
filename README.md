<<<<<<< HEAD
# DeFi Hot Wallet - Rust Edition

🔒 **DeFi热钱包，Rust打造，安全如堡垒！** 35天自研MVP，为DeFi玩家量身定制。

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Security](https://img.shields.io/badge/security-quantum--safe-green.svg)](#security-features)

## 🌟 核心特性 / Core Features

### 🛡️ 安全性 / Security
- **量子安全加密** - Quantum-safe encryption (Kyber算法模拟)
- **助记词分片** - Mnemonic phrase sharding (Shamir 2-of-3)
- **多重签名** - Multi-signature support (2-of-3 threshold)
- **HSM内存隔离** - Hardware Security Module simulation
- **零化清栈** - Memory zeroization on drop
- **审计日志** - Comprehensive audit logging

### ⚡ 性能 / Performance
- **Rust零开销** - Zero-cost abstractions
- **异步架构** - Async/await throughout
- **交易<2秒** - Sub-2-second transactions
- **并发安全** - Thread-safe operations

### 🌍 区块链支持 / Blockchain Support
- **以太坊** - Ethereum (ETH) - Full support
- **Solana** - Solana (SOL) - Simulated support
- **可扩展** - Extensible architecture for more chains

### 🌐 国际化 / Internationalization
- **中文** - Chinese (简体中文)
- **英文** - English
- **可扩展** - Extensible i18n framework

## 🚀 快速开始 / Quick Start

### 环境要求 / Prerequisites

```bash
# Rust 1.70+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 依赖项 / Dependencies
sudo apt-get install build-essential pkg-config libssl-dev
```

### 安装 / Installation

```bash
# 克隆仓库 / Clone repository
git clone https://github.com/Yinhang3377/Defi-Hot-wallet-Rust.git
cd Defi-Hot-wallet-Rust

# 构建项目 / Build project
cargo build --release

# 运行测试 / Run tests
cargo test
```

### 使用示例 / Usage Examples

#### 1. 命令行界面 / CLI Interface

```bash
# 创建新钱包 / Create new wallet
./target/release/wallet-cli create --name my-wallet --quantum true

# 查看余额 / Check balance
./target/release/wallet-cli balance --wallet my-wallet --network eth

# 发送交易 / Send transaction
./target/release/wallet-cli send \
  --wallet my-wallet \
  --to 0x742d35Cc6635C0532925a3b8D400e8B78fFe4860 \
  --amount 0.1 \
  --network eth

# 生成助记词 / Generate mnemonic
./target/release/wallet-cli generate-mnemonic

# 安全状态 / Security status
./target/release/wallet-cli security
```

#### 2. 服务器模式 / Server Mode

```bash
# 启动钱包服务器 / Start wallet server
./target/release/defi-wallet server --port 8080 --host 0.0.0.0

# API 端点 / API Endpoints
curl -X POST http://localhost:8080/api/wallets \
  -H "Content-Type: application/json" \
  -d '{"name": "my-wallet", "quantum_safe": true}'

curl http://localhost:8080/api/wallets/my-wallet/balance?network=eth

curl http://localhost:8080/api/health
curl http://localhost:8080/api/metrics
```

#### 3. 中文界面 / Chinese Interface

```bash
# 使用中文界面 / Use Chinese interface
./target/release/wallet-cli --language zh create --name 我的钱包

# 查看帮助 / Show help
./target/release/wallet-cli --help
```

## 🏗️ 架构设计 / Architecture

### 分层架构 / Layered Architecture

```
┌─────────────────────────────────────────┐
│               API Layer                 │  ← REST API / CLI
├─────────────────────────────────────────┤
│             Core Services               │  ← Wallet Management
├─────────────────────────────────────────┤
│          Security Modules               │  ← Crypto / HSM / Multi-sig
├─────────────────────────────────────────┤
│         Blockchain Clients              │  ← ETH / Solana / Others
├─────────────────────────────────────────┤
│           Storage Layer                 │  ← SQLite / Audit Logs
└─────────────────────────────────────────┘
```

### 核心模块 / Core Modules

- **`src/core/`** - 钱包核心逻辑 / Core wallet logic
- **`src/crypto/`** - 加密模块 / Cryptographic modules
- **`src/blockchain/`** - 区块链集成 / Blockchain integrations
- **`src/storage/`** - 存储层 / Storage layer
- **`src/monitoring/`** - 监控指标 / Monitoring & metrics
- **`src/api/`** - API服务器 / API server
- **`src/i18n/`** - 国际化 / Internationalization

## 🔐 安全特性详解 / Security Features

### 量子安全加密 / Quantum-Safe Encryption

```rust
// 使用模拟的Kyber算法
let mut crypto = QuantumSafeEncryption::new()?;
let keypair = crypto.generate_keypair()?;
let encrypted = crypto.encrypt(sensitive_data)?;
```

### Shamir密钥分片 / Shamir Secret Sharing

```rust
// 2-of-3阈值分片
let shamir = ShamirSecretSharing::new();
let shares = shamir.create_shares(master_key, 3, 2)?;
let recovered = shamir.reconstruct_secret(&shares[..2])?;
```

### 多重签名 / Multi-Signature

```rust
// 创建多签配置
let config = MultiSignature::create_multisig_config(2, signers)?;

// 提议交易
multisig.propose_transaction(tx_id, to_addr, amount, network, 2)?;

// 签名交易
let complete = multisig.sign_transaction(tx_id, signer, signature)?;
```

### HSM内存隔离 / HSM Memory Isolation

```rust
// 分配安全内存
let region_id = hsm.allocate_secure_memory(64).await?;

// 写入敏感数据
hsm.write_secure_memory(region_id, sensitive_data).await?;

// 自动零化清理
hsm.free_secure_memory(region_id).await?;
```

## 📊 监控指标 / Monitoring

### Prometheus 指标 / Prometheus Metrics

```bash
# 查看指标 / View metrics
curl http://localhost:8080/api/metrics

# 关键指标 / Key metrics
- wallets_created_total
- transactions_sent_total
- quantum_encryptions_total
- failed_logins_total
- active_connections
- response_time_seconds
```

### 安全监控 / Security Monitoring

- 🚨 **异常检测** - Anomaly detection
- 📝 **审计日志** - Audit logging
- 🛡️ **入侵检测** - Intrusion detection
- ⚠️ **告警系统** - Alert system

## 🧪 测试 / Testing

```bash
# 运行所有测试 / Run all tests
cargo test

# 运行特定模块测试 / Run specific module tests
cargo test crypto::tests
cargo test blockchain::tests

# 性能测试 / Benchmark tests
cargo bench

# 测试覆盖率 / Test coverage
cargo tarpaulin --out Html
```

### 测试覆盖率目标 / Test Coverage Goals

- ✅ **核心逻辑**: 95%+
- ✅ **加密模块**: 90%+
- ✅ **API接口**: 85%+
- ✅ **总体覆盖**: 80%+

## 🚀 部署 / Deployment

### Docker 部署 / Docker Deployment

```dockerfile
FROM rust:1.70-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/defi-wallet /usr/local/bin/
EXPOSE 8080
CMD ["defi-wallet", "server"]
```

### 生产环境配置 / Production Configuration

```toml
# config.toml
[server]
host = "0.0.0.0"
port = 8080
tls_enabled = true

[security]
quantum_safe_default = true
hsm_enabled = true
session_timeout_minutes = 30

[monitoring]
metrics_enabled = true
log_level = "info"
alert_webhook_url = "https://your-webhook-url"
```

## 🔧 配置 / Configuration

### 环境变量 / Environment Variables

```bash
# 数据库配置 / Database configuration
export WALLET_DATABASE_URL="sqlite:./wallet.db"

# 网络配置 / Network configuration
export WALLET_ETHEREUM_RPC_URL="https://mainnet.infura.io/v3/YOUR-PROJECT-ID"
export WALLET_SOLANA_RPC_URL="https://api.mainnet-beta.solana.com"

# 安全配置 / Security configuration
export WALLET_ENCRYPTION_KEY_PATH="./keys/master.key"
export WALLET_HSM_ENABLED="false"
```

## 🤝 贡献指南 / Contributing

### 开发流程 / Development Workflow

1. **Fork** 项目 / Fork the project
2. **创建分支** / Create feature branch (`git checkout -b feature/amazing-feature`)
3. **提交更改** / Commit changes (`git commit -m 'Add amazing feature'`)
4. **推送分支** / Push branch (`git push origin feature/amazing-feature`)
5. **提交PR** / Create Pull Request

### 代码规范 / Code Standards

```bash
# 格式化代码 / Format code
cargo fmt

# 代码检查 / Lint code
cargo clippy -- -D warnings

# 安全审计 / Security audit
cargo audit
```

## 📜 许可证 / License

本项目基于 [MIT 许可证](LICENSE) - 详见 LICENSE 文件

This project is licensed under the [MIT License](LICENSE) - see the LICENSE file for details

## 🙏 致谢 / Acknowledgments

- **Rust Foundation** - 卓越的系统编程语言
- **Ethereum Foundation** - 去中心化金融基础设施
- **Solana Labs** - 高性能区块链平台
- **开源社区** - 无私的贡献和支持

## 📞 联系方式 / Contact

- **GitHub**: [@Yinhang3377](https://github.com/Yinhang3377)
- **Issues**: [GitHub Issues](https://github.com/Yinhang3377/Defi-Hot-wallet-Rust/issues)

## ⚠️ 免责声明 / Disclaimer

**重要安全提示 / Important Security Notice:**

此项目仅供教育和研究目的。在生产环境中使用加密货币钱包之前，请进行全面的安全审计。作者不对任何资金损失承担责任。

This project is for educational and research purposes only. Please conduct thorough security audits before using any cryptocurrency wallet in production. The authors are not responsible for any financial losses.

**风险提示 / Risk Warning:**
- 🔐 妥善保管私钥和助记词
- 🛡️ 定期备份钱包数据  
- ⚡ 小额测试后再使用
- 🔍 验证所有交易详情

---

**Made with ❤️ in Rust** | **用Rust制造，充满❤️**
=======
# Secure-Hot-Wallet-in-Rust-

生产级 Rust 热钱包框架，支持多链多资产，安全、高性能、可扩展、易维护。专为以太坊和 Solana 生态系统设计，模块化架构，适用于私钥管理、交易签名和安全存储。Rust 的内存安全性、零成本抽象和并发原语使其成为热钱包的理想选择，有效预防 C/C++ 实现中常见的缓冲区溢出、数据竞争和内存泄漏等漏洞。

## 主要特性
- 多层安全机制
- 插件式架构
- 统一配置与错误处理
- 事件驱动与依赖注入
- 结构化日志与监控
- 完善测试与文档

## 🌟 为什么选择 Rust 开发热钱包？

热钱包处理实时交易签名和私钥加密等敏感操作，因此安全性和性能至关重要。Rust 在这方面表现出色：

### 🔒 无垃圾回收的内存安全
Rust 的所有权模型确保私钥在不再使用时能自动归零并释放，从而消除悬垂指针或 "use-after-free" 等错误。不再需要手动处理内存管理的风险！

### ⚡ 线程安全与并发
内置的 "无畏并发" 特性支持多线程操作（例如并行交易签名）而不会引发数据竞争，这对于高吞吐量的钱包至关重要。

### 🚀 与 C 语言相当的性能
零开销抽象为加密操作（例如通过 secp256k1 进行 ECDSA 签名）提供了原生速度，性能优于 Python 或 JavaScript 等解释型语言。

### 🔐 密码学原语
强大的生态系统提供了像 aes-gcm、zeroize 和 secp256k1 等库，支持抗量子加密和安全密钥生成。

### 🛡️ 可审计性与可组合性
编译时保障和模块化设计使代码更易于审计，减少了区块链环境中的攻击面。

## 快速开始
```sh
# 构建
cargo build
# 运行示例
cargo run --example basic_usage
```

## 目录结构
- src/         主代码
- examples/    用法示例
- tests/       单元/集成测试
- ci/          CI/CD配置
- docs/        开发文档

## 🛠️ 近期变更

### 2025-09-19
- 移除未使用的 `encryption_key` 字段，改为使用 `salt` 动态生成密钥。
- 删除未集成的 `derive_encryption_key` 函数。
- 移除 `MemoryProtector` 结构体及其方法。
- 测试覆盖率优化，确保核心功能稳定。
>>>>>>> be35db3d094cb6edd3c63585f33fdcb299a57158
