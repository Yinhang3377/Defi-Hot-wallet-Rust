// ...existing code...
# Defi-Hot-wallet-Rust 架构蓝图与实现指南
> 面向高安全场景的热钱包架构蓝图。采用 DDD + 分层架构，强调安全、审计、可观测、可扩展与工程化最佳实践。  
> Last updated: 2025-09-21

---

<a id="toc"></a>
## 目录
- [1. 完整项目布局（目录树）](#project-layout)
- [2. Cargo.toml 配置模板](#cargo-toml)
- [3. 实施顺序与路线图](#roadmap)
- [4. 关键模块代码模板（Rust样例）](#code-templates)
- [5. 工具链建议与下一步](#tooling)
- [6. README 快速开始（建议落地）](#readme-quickstart)
- [7. FAQ（常见问题）](#faq)
- [8. 安全与合规要点](#security-compliance)
- [9. 版本与变更日志指引](#versioning-changelog)
- [10. 术语表（附录）](#glossary)

---

<a id="project-layout"></a>

// ...existing code...
## 1. 完整项目布局（目录树）
说明：基于实际代码库结构重新生成，已与本地文件对齐。移除理想化占位符，添加实际模块和文件（如区块链客户端、测试文件、CI配置）。若有新增文件，请同步更新。

```text
Defi-Hot-wallet-Rust/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── CONTRIBUTING.md
├── CODEOWNERS
├── SECURITY.md
├── CHANGELOG.md
├── .gitignore
├── .vscode/
├── wallet.json
├── target/
├── docs/
│   ├── 1_architecture_blueprint.md
│   ├── 2_security_patch_list.md
│   ├── 3_priority_table.md
│   ├── 4_analysis_doc.md
│   ├── 5_code_review_plan.md
│   ├── 6_test_matrix.md
│   ├── 7_threat_model.md
│   ├── 8_key_management_policy.md
│   └── 9_incident_response_runbook.md
├── resources/
├── examples/
│   ├── basic_wallet.rs
│   └── advanced_tx.rs
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── cli.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── server.rs
│   │   ├── routes.rs
│   │   └── bridge.rs
│   ├── application/
│   │   ├── mod.rs
│   │   ├── bridge.rs
│   │   └── transaction.rs
│   ├── audit/
│   │   ├── mod.rs
│   │   ├── logging.rs
│   │   ├── alert.rs
│   │   ├── confirmation.rs
│   │   └── rollback.rs
│   ├── blockchain/
│   │   ├── mod.rs
│   │   ├── ethereum.rs
│   │   └── solana.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   └── env_config.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── wallet_manager.rs
│   │   ├── errors.rs
│   │   └── domain.rs
│   ├── crypto/
│   │   ├── mod.rs
│   │   ├── shamir.rs
│   │   ├── quantum.rs
│   │   └── kdf.rs
│   ├── monitoring/
│   │   ├── mod.rs
│   │   ├── metrics.rs
│   │   └── health.rs
│   ├── mvp/
│   │   ├── mod.rs
│   │   ├── wallet.rs
│   │   ├── balance.rs
│   │   └── transaction.rs
│   ├── ops/
│   │   ├── mod.rs
│   │   ├── backup.rs
│   │   ├── metrics.rs
│   │   └── health.rs
│   ├── plugin/
│   │   ├── mod.rs
│   │   ├── plugin.rs
│   │   ├── plugin_manager.rs
│   │   ├── middleware.rs
│   │   └── event_bus.rs
│   ├── security/
│   │   ├── mod.rs
│   │   ├── encryption.rs
│   │   ├── memory_protection.rs
│   │   ├── anti_debug.rs
│   │   ├── access_control.rs
│   │   └── compliance.rs
│   ├── service/
│   │   ├── mod.rs
│   │   ├── service.rs
│   │   └── di_container.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── database.rs
│   │   └── migration.rs
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── generator.rs
│   │   ├── async_support.rs
│   │   └── error.rs
│   ├── i18n/
│   │   ├── mod.rs
│   │   └── localization.rs
│   └── network/
│       ├── mod.rs
│       ├── node_manager.rs
│       └── rate_limit.rs
├── tests/
│   ├── unit/
│   │   ├── crypto_shamir_tests.rs
│   │   ├── shamir_tests.rs
│   │   └── wallet_tests.rs
│   ├── integration/
│   │   ├── bridge_tests.rs
│   │   └── api_tests.rs
│   ├── e2e/
│   │   └── full_flow_tests.rs
│   └── security/
│       └── attack_tests.rs
├── benches/
│   └── performance_benches.rs
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── cd.yml
├── scripts/
│   ├── build.ps1
│   ├── test.ps1
│   └── deploy.sh
└── patches/
    └── elliptic-curve-tools/
```

[↑ 返回目录](#toc)
---
// ...existing code...

[↑ 返回目录](#toc)
---

<a id="cargo-toml"></a>
## 2. Cargo.toml 配置模板
说明：建议只加入实际使用的依赖并定期锁定版本，下面为推荐起点。

```toml
[package]
name = "defi-hot-wallet-rust"
version = "0.1.0"
edition = "2021"

[dependencies]
# 安全
argon2 = "0.5"
zeroize = "1.8"

# 工具
rand = "0.8"
hex = "0.4"

# 异步与日志
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"

# 网络/限流/DI
reqwest = { version = "0.12", features = ["json"] }
governor = "0.6"
shaku = "0.6"

# 序列化与 i18n / 指标
serde = { version = "1.0", features = ["derive"] }
prometheus = "0.13"
fluent-rs = "0.1"

# 错误处理
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.13"
k256 = { version = "0.13", features = ["arithmetic", "serde"] }
p256 = { version = "0.13", features = ["arithmetic", "serde"] }
p384 = { version = "0.13", features = ["arithmetic", "serde"] }
```

[↑ 返回目录](#toc)

---

<a id="roadmap"></a>
## 3. 实施顺序与路线图（精简）
- 阶段 1：config/ + service/（环境隔离、DI 容器）  
- 阶段 2：core/domain + application（Wallet、Tx、WalletService）  
- 阶段 3：infrastructure/interface/adapter（RPC、HTTP、CLI）  
- 阶段 4：security/ + audit/（加密、内存保护、审计、回滚、确认）  
- 阶段 5：network/ + plugin/（节点管理、限流、事件总线）  
- 阶段 6：ops/ + i18n/（健康检查、监控、备份、多语言）  
- 阶段 7：tests/ + ci/（单元/集成/E2E、CI/CD）

每阶段写明验收标准与最小实现（MVP）。将路线图细化为任务卡并放入 3_priority_table.md。

[↑ 返回目录](#toc)

---

<a id="code-templates"></a>
## 4. 关键模块代码模板（Rust样例）
说明：示例展示接口与实现方向，生产环境需完善错误处理、参数与安全细节。

```rust
// src/audit/logging.rs
use tracing::{info, error};
use tracing_subscriber::fmt;

pub fn init_logger() {
    fmt::init();
}

pub fn log_operation(op: &str, user_id: &str, success: bool) {
    if success { info!(operation=%op, user_id=%user_id, "ok"); }
    else { error!(operation=%op, user_id=%user_id, "fail"); }
}
```

```rust
// src/security/encryption.rs
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use zeroize::Zeroize;

pub struct Encryptor { pub salt: SaltString }

impl Encryptor {
    pub fn new() -> Self { Self { salt: SaltString::generate(&mut rand::thread_rng()) } }

    pub fn derive_key(&self, password: &str) -> anyhow::Result<Vec<u8>> {
        let hash = Argon2::default().hash_password(password.as_bytes(), &self.salt)?;
        Ok(hash.to_string().as_bytes().to_vec())
    }

    pub fn encrypt_data(&self, data: &[u8], password: &str) -> anyhow::Result<Vec<u8>> {
        let mut key = self.derive_key(password)?;
        // TODO: 使用 aes-gcm 或 chacha20poly1305 实现真正加密
        key.zeroize();
        Ok(data.to_vec())
    }
}
```

```rust
// src/application/application.rs
use crate::{audit::logging::log_operation, core::domain::{Wallet, Tx}, security::encryption::Encryptor};

pub struct WalletService { enc: Encryptor }

impl Default for WalletService { fn default() -> Self { Self { enc: Encryptor::new() } } }

impl WalletService {
    pub async fn create_wallet(&self, mnemonic: &str) -> anyhow::Result<Wallet> {
        let w = Wallet::from_mnemonic(mnemonic)?;
        log_operation("create_wallet", &w.id, true);
        Ok(w)
    }

    pub async fn send_tx(&self, w: &Wallet, to: &str, amount: u64) -> anyhow::Result<Tx> {
        let tx = Tx::new(w, to, amount);
        log_operation("send_tx", &w.id, true);
        Ok(tx)
    }
}
```

[↑ 返回目录](#toc)

---

<a id="tooling"></a>
## 5. 工具链建议与下一步
- 代码格式与质量：cargo fmt / cargo clippy；启用 pre-commit hooks。  
- 测试与覆盖：cargo test、tarpaulin（覆盖率）。  
- 性能：criterion 基准。  
- 可观测：tracing + prometheus_exporter，暴露 /metrics。  
- CI/CD：GitHub Actions 示例（ci.yml、cd.yml）。  
- 安全：密钥短时驻留内存，zeroize、敏感日志脱敏、KDF 参数化。

[↑ 返回目录](#toc)

---

<a id="readme-quickstart"></a>
## 6. README 快速开始（建议落地）
示例命令（Windows PowerShell）：

```powershell
rustup default stable
cd .\Defi-Hot-wallet-Rust\
cargo build --release
cargo test
cargo run --example basic_wallet
```

README 应包含：项目简介、安装、快速示例、运行与测试、贡献指南、已知限制、许可证。

[↑ 返回目录](#toc)

---

<a id="faq"></a>
## 7. FAQ（常见问题）
- 目录跳转失效？  
  - 本文件使用显式锚点（id）并与 TOC 对齐，VS Code & GitHub 渲染测试通过。  
- target/ 是否提交？  
  - 否，添加到 .gitignore。  
- 示例运行失败？  
  - 检查 Cargo.toml 依赖、模块 pub/export，以及示例是否调用未实现的功能。

[↑ 返回目录](#toc)

---

<a id="security-compliance"></a>
## 8. 安全与合规要点
- KDF：使用 Argon2，生产环境按内存/时间参数调优。  
- 内存安全：zeroize 清理敏感数据。  
- 日志：敏感字段脱敏或不记录。  
- 访问控制：RBAC、关键操作多重确认与可回滚。  
- 合规：为 GDPR/AML 提供钩子与审计记录。

[↑ 返回目录](#toc)

---

<a id="versioning-changelog"></a>
## 9. 版本与变更日志指引
- 版本化：遵循 SemVer。  
- 变更日志：创建 CHANGELOG.md，遵循 Keep a Changelog。  
- 发布：CI 通过 → 生成 Release Notes → 打 tag → 发布二进制/镜像。

[↑ 返回目录](#toc)

---

<a id="glossary"></a>
## 10. 术语表（附录）
- DDD：领域驱动设计。  
- DI：依赖注入。  
- KDF：密钥派生函数（例如 Argon2）。  
- E2E：端到端测试。

[↑ 返回目录](#toc)

---