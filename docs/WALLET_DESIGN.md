<!-- TOC start -->
# 热钱包框架蓝图 | Hot Wallet Architecture Blueprint

## 目录 | Table of Contents
1. [完整项目布局（目录树） | Project Structure (Directory Tree)](#完整项目布局目录树--project-structure-directory-tree)
2. [各模块功能说明 | Module Descriptions](#各模块功能说明--module-descriptions)
3. [示例与用例 | Examples](#示例与用例--examples)
<!-- TOC end -->

## 完整项目布局（目录树） | Project Structure (Directory Tree)
使用 cargo new hot_wallet --lib 初始化，然后按如下结构组织：

```
hot_wallet/
├── Cargo.toml                  # 依赖管理和配置 | Dependency management & config
├── README.md                   # 项目概述、安装指南 | Project overview, installation
├── CONTRIBUTING.md             # 贡献指南 | Contribution guide
├── CODEOWNERS                  # 代码所有者文件（GitHub集成）| Code owners (GitHub)
├── docs/                       # 文档目录 | Documentation
│   ├── architecture.md         # 架构设计 | Architecture
│   ├── compliance.md           # 安全合规文档 | Compliance
│   └── api.md                  # API参考 | API Reference
├── src/                        # 源代码 | Source code
│   ├── lib.rs                  # 入口模块（pub use 所有子模块）| Entry module (pub use)
│   ├── security/               # 1. 安全模块 | Security
│   │   ├── mod.rs
│   │   ├── encryption.rs       # 多层加密（e.g., AES + Argon2）| Multi-layer encryption
│   │   ├── memory_protection.rs # 内存保护与清理（e.g., zeroize）| Memory protection/zeroize
│   │   ├── anti_debug.rs       # 防调试/反序列化（e.g., detect debugger）| Anti-debug/anti-serialization
│   │   ├── access_control.rs   # 权限管理（e.g., RBAC）| Access control (RBAC)
│   │   └── compliance.rs       # 安全合规接口（e.g., GDPR/AML hooks）| Compliance hooks
│   ├── audit/                  # 2. 审计与异常模块 | Audit & Exception
│   │   ├── mod.rs
│   │   ├── operation_log.rs    # 操作日志 | Operation log
│   │   ├── alert.rs            # 异常报警（e.g., email/Slack）| Alerting (email/Slack)
│   │   ├── confirmation.rs     # 多重确认（e.g., 2FA for tx）| Multi-factor confirmation
│   │   ├── rollback.rs         # 回滚机制（e.g., tx revert）| Rollback
│   │   └── logging.rs          # 结构化日志（e.g., tracing/slog）| Structured logging
│   ├── config/                 # 3. 配置与依赖模块 | Config & Dependency
│   │   ├── mod.rs
│   │   ├── config.rs           # 统一配置（e.g., TOML/JSON）| Unified config
│   │   └── env_config.rs       # 多环境配置隔离（dev/prod）| Env separation
│   ├── service/                # 3. 配置与依赖模块（续）| Service (cont.)
│   │   ├── mod.rs
│   │   ├── service.rs          # 依赖注入/服务注册 | DI/service registration
│   │   └── di_container.rs     # 依赖注入容器（e.g., shaku）| DI container
│   ├── plugin/                 # 4. 插件与扩展模块 | Plugin & Extension
│   │   ├── mod.rs
│   │   ├── plugin.rs           # 插件接口（trait定义）| Plugin trait
│   │   ├── plugin_manager.rs   # 插件管理（动态加载）| Plugin manager
│   │   ├── middleware.rs       # 统一中间件（e.g., auth/rate limit）| Middleware
│   │   └── event_bus.rs        # 事件驱动/消息总线（e.g., tokio::sync）| Event bus
│   ├── network/                # 5. 网络与高可用模块 | Network & HA
│   │   ├── mod.rs
│   │   ├── node_manager.rs     # 节点管理/自动切换（e.g., RPC failover）| Node manager
│   │   └── rate_limit.rs       # API限流（e.g., governor）| Rate limiting
│   ├── core/                   # 6. 业务分层模块（核心/领域层）| Core/domain
│   │   ├── mod.rs
│   │   └── domain.rs           # 领域层/核心业务（e.g., Wallet, Tx实体）| Domain logic
│   ├── application/            # 6. 应用层 | Application
│   │   ├── mod.rs
│   │   └── application.rs      # 应用层/服务编排（e.g., WalletService）| Application logic
│   ├── infrastructure/         # 6. 基础设施层 | Infrastructure
│   │   ├── mod.rs
│   │   └── infrastructure.rs   # 基础设施层/外部集成（e.g., DB/Blockchain RPC）| Integration
│   ├── interface/              # 6. 接口适配层 | Interface
│   │   ├── mod.rs
│   │   └── interface.rs        # 接口适配层（e.g., HTTP/CLI adapters）| Interface adapters
│   ├── adapter/                # 6. 多平台适配 | Adapter
│   │   ├── mod.rs
│   │   └── adapter.rs          # 多平台适配（e.g., Web/Mobile）| Multi-platform adapter
│   ├── ops/                    # 7. 运维与监控模块 | Ops & Monitoring
│   │   ├── mod.rs
│   │   ├── health.rs           # 健康检查（e.g., /health endpoint）| Health check
│   │   ├── metrics.rs          # 运行时监控（e.g., Prometheus）| Metrics
│   │   └── backup.rs           # 自动化备份（e.g., cron jobs）| Backup
│   ├── i18n/                   # 8. 国际化与本地化模块 | i18n & l10n
│   │   ├── mod.rs
│   │   └── localization.rs     # 多语言/多币种支持（e.g., fluent-rs）| Localization
│   └── tools/                  # 9. 工程化与工具模块 | Tooling
│       ├── mod.rs
│       ├── generator.rs        # 代码生成/脚手架（e.g., build.rs hooks）| Codegen/scaffold
│       ├── async_support.rs    # 异步/多线程支持（e.g., tokio traits）| Async/multithread
│       └── error.rs            # 统一错误处理（e.g., anyhow/thiserror）| Error handling
├── tests/                      # 10. 测试模块 | Tests
│   ├── unit/                   # 单元测试 | Unit tests
│   ├── integration/            # 集成测试 | Integration tests
│   └── e2e/                    # 端到端测试 | E2E tests
├── ci/                         # 10. CI/CD配置 | CI/CD
│   ├── .github/workflows/      # GitHub Actions
│   │   ├── ci.yml              # CI管道 | CI pipeline
│   │   └── cd.yml              # CD部署 | CD deploy
│   └── dockerfile              # Docker构建 | Dockerfile
└── examples/                   # 11. 示例与用例 | Examples
    ├── basic_wallet.rs         # 基本用法：生成/转账 | Basic usage: create/transfer
    └── advanced_tx.rs          # 高级：多签 + 回滚 | Advanced: multisig + rollback
```

## 各模块功能说明 | Module Descriptions

### 安全模块 | Security
- **加密模块**：提供多层加密（如AES + Argon2），确保数据传输和存储的安全性。
- **内存保护模块**：实现内存清理（如zeroize），防止敏感数据泄露。
- **防调试模块**：检测调试器并阻止反序列化攻击，增强运行时安全性。
- **权限管理模块**：基于RBAC模型，提供细粒度的访问控制。
- **合规模块**：集成GDPR和AML相关接口，确保符合国际安全标准。

### 审计与异常模块 | Audit & Exception
- **操作日志模块**：记录所有关键操作，支持结构化日志分析。
- **异常报警模块**：通过邮件或Slack发送实时警报，快速响应异常事件。
- **多重确认模块**：为交易提供2FA支持，确保操作安全。
- **回滚机制模块**：支持事务回滚，减少错误操作的影响。
- **日志模块**：采用tracing或slog库，提供高效的日志记录和查询功能。

### 配置与依赖模块 | Config & Dependency
- **统一配置模块**：支持TOML或JSON格式的配置文件，简化管理。
- **环境配置模块**：隔离开发和生产环境，确保配置的灵活性。
- **服务注册模块**：通过依赖注入实现模块间的解耦和动态加载。

### 插件与扩展模块 | Plugin & Extension
- **插件接口模块**：定义通用trait，支持插件的动态扩展。
- **插件管理模块**：提供插件的加载、卸载和生命周期管理。
- **中间件模块**：统一处理认证和限流逻辑，提升系统性能。
- **事件驱动模块**：基于消息总线实现模块间的异步通信。

### 网络与高可用模块 | Network & HA
- **节点管理模块**：支持RPC故障切换，确保网络的高可用性。
- **限流模块**：通过governor库实现API的动态限流。

### 业务分层模块 | Core/Domain
- **领域层模块**：定义核心业务逻辑（如钱包和交易实体）。
- **应用层模块**：负责服务编排（如WalletService），连接领域层和基础设施层。
- **基础设施层模块**：集成外部服务（如数据库和区块链RPC）。
- **接口适配层模块**：提供HTTP和CLI的适配器，简化用户交互。
- **多平台适配模块**：支持Web和移动端的多平台适配。

### 运维与监控模块 | Ops & Monitoring
- **健康检查模块**：提供/health端点，监控服务状态。
- **监控模块**：集成Prometheus，实时跟踪系统性能。
- **备份模块**：通过定时任务实现数据的自动化备份。

### 国际化与本地化模块 | i18n & l10n
- **多语言支持模块**：基于fluent-rs库，提供中英双语支持。
- **多币种支持模块**：支持ETH、SOL和ERC-20代币的本地化。

### 工程化与工具模块 | Tooling
- **代码生成模块**：通过build.rs钩子实现代码的自动生成。
- **异步支持模块**：基于tokio库，优化多线程性能。
- **错误处理模块**：统一错误处理逻辑，提升调试效率。

## 示例与用例 | Examples

### 示例 1: 基本用法 | Example 1: Basic Usage
- **生成钱包**：调用`Wallet::new()`方法，生成一个支持多币种的钱包。
- **转账操作**：使用`Wallet::transfer()`方法，完成ETH或SOL的转账。

### 示例 2: 高级用法 | Example 2: Advanced Usage
- **多重签名**：通过`Multisig::create()`方法，生成多重签名交易。
- **事务回滚**：调用`Transaction::rollback()`方法，恢复到安全状态。

### 示例 3: API调用 | Example 3: API Integration
- **创建钱包**：发送POST请求到`/api/v1/wallet/create`，返回钱包ID。
- **查询余额**：发送GET请求到`/api/v1/wallet/balance`，返回账户余额。
- **转账操作**：发送POST请求到`/api/v1/wallet/transfer`，完成转账并返回交易ID。

