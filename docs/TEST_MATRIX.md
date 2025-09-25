<!-- TOC start -->
# 测试矩阵 | Test Matrix

## 目录 | Table of Contents
1. [测试目标 | Test Objectives](#测试目标--test-objectives)
2. [测试分类 | Test Types](#测试分类--test-types)
3. [测试场景 | Test Scenarios](#测试场景--test-scenarios)
4. [测试时间安排 | Timeline](#测试时间安排--timeline)
5. [责任人 | Responsible Parties](#责任人--responsible-parties)
6. [通过标准 | Acceptance Criteria](#通过标准--acceptance-criteria)
7. [输出 | Deliverables](#输出--deliverables)
8. [实施建议 | Implementation Advice](#实施建议--implementation-advice)
9. [路线图说明 | Roadmap Description](#路线图说明--roadmap-description)
<!-- TOC end -->


## 测试目标 | Test Objectives
- **功能性**：钱包生成、存币、转账、恢复无错误。
- **安全性**：防内存泄露、防中间人攻击、防合约漏洞。
- **性能**：交易延迟<2秒，恢复时间<10秒。
- **兼容性**：支持ETH/Solana，兼容安卓/iOS。

## 测试分类 | Test Types
1. **单元测试**（tests/unit/）
	- 目标：验证单个函数/模块逻辑。
	- 模块：security/（KDF、零化）、core/domain.rs（Wallet/Tx）。
	- 工具：Rust test、mockall。
2. **集成测试**（tests/integration/）
	- 目标：验证模块间交互（e.g., WalletService调用encryption.rs）。
	- 模块：application/ + security/ + audit/。
	- 工具：tokio-test。
3. **端到端测试**（tests/e2e/）
	- 目标：模拟真实用户场景（生成、转账、恢复）。
	- 场景：用户丢失设备，通过分片恢复；多签转账。
	- 工具：cargo test + 自定义脚本。
4. **安全测试**（tests/security/）
	- 目标：模拟攻击（内存转储、中间人、钓鱼）。
	- 工具：Burp Suite、Slither。

## 测试场景 | Test Scenarios
| **模块** | **测试点** | **场景** | **预期结果** | **工具** |
|----------|------------|----------|---------------|----------|
| security/encryption.rs | KDF派生 | 生成10万密钥，验证一致性 | 派生时间<10ms，密钥一致 | Rust test |
| security/memory_protection.rs | 零化 | 私钥使用后检查内存 | 内存全零，无残留 | zeroize + gdb |
| security/anti_debug.rs | 防调试 | 运行调试器，检测触发 | 程序退出，报警 | ptrace模拟 |
| audit/confirmation.rs | 多重签名 | 2-of-3签名，1人拒绝 | 交易失败，日志记录 | Rust test |
| audit/rollback.rs | 回滚 | 模拟合约失败，触发回滚 | 资产回退，无损失 | tokio-test |
| core/domain.rs | 钱包生成 | 生成中文助记词 | 符合BIP-39，24词 | bip39 crate |
| application/application.rs | 转账 | 向Uniswap转ETH | 交易成功，余额更新 | ethers-rs |
| infrastructure/infrastructure.rs | RPC切换 | 主节点断开，切换备份 | 无中断，延迟<1秒 | reqwest mock |
| ops/metrics.rs | 监控 | 模拟1000次/秒请求 | 指标正常，延迟<2秒 | Prometheus |

## 测试时间安排 | Timeline
- **框架阶段**（第1周）：单元测试config.rs、domain.rs（5小时）。
- **安全阶段**（第2-3周）：单元+集成测试security/、audit/（15小时）。
- **扩展阶段**（第4周）：端到端+安全测试（10小时）。
- **总时间**：30小时（分散在40天开发中）。

## 责任人 | Responsible Parties
- **刘德华**：编写测试用例，运行测试。
- **工具**：cargo test、tarpaulin（覆盖率）、Slither（合约）。

## 通过标准 | Acceptance Criteria
- 单元测试覆盖率>80%（tarpaulin报告）。
- 集成测试100%通过，无死锁。
- 端到端测试模拟100次用户场景，成功率100%。
- 安全测试无高危漏洞（e.g., 未零化、合约重入）。

## 输出 | Deliverables
- 测试报告（覆盖率、失败用例）。
- 安全测试日志（攻击防御记录）。
- CI管道集成（GitHub Actions）。

## 路线图说明 | Roadmap Description

1. **总工期**：5周（50天，每周5天，每天10小时），共500小时。分为：
   - **核心阶段**（1-3周，30天）：完成高优先级功能（KDF、助记词分片、HSM、2FA、端到端加密、多签），产出MVP（可生成钱包、存币、转账）。
   - **增强阶段**（第4周，10天）：实现中优先级功能（量子安全加密、动态密钥轮换、去中心化身份、分布式存储），提升安全性和可用性。
   - **高级阶段**（第5周，10天）：完成低优先级功能（备份恢复、日志审计、行为分析、合约审计与升级、威胁情报、实时监控），优化防护。

2. **职责分配**：
   - **您**：主编码和集成，负责核心开发、UI设计、测试执行。
   - **Grok**：提供算法逻辑、优化建议、调试支持，查找最新技术资源。
   - **GPT-5**：生成测试用例、模拟攻击场景、验证功能可靠性。

3. **关联性与依赖**：
   - **核心阶段**：KDF和HSM是基础，助记词分片依赖KDF，2FA和端到端加密依赖HSM，多签依赖密钥安全。
   - **增强阶段**：量子安全加密和动态密钥轮换依赖KDF和HSM，去中心化身份依赖密钥安全，分布式存储支持备份恢复。
   - **高级阶段**：行为分析依赖日志审计，合约升级依赖审计，实时监控和威胁情报依赖行为分析和日志。

4. **风险管理**：
   - **技术风险**：KDF参数错误、HSM兼容性差、量子算法性能低。应对：提前测试，优选成熟库（如OpenSSL、liboqs）。
   - **用户体验**：助记词分片和多签流程复杂可能劝退用户。应对：设计直观界面，提供引导教程。
   - **性能风险**：实时监控和量子加密可能拖慢交易。应对：异步处理，优化算法，测试低端设备。
   - **安全风险**：合约审计遗漏漏洞、日志泄露隐私。应对：多层审计，加密存储，限制访问。

## 时间估算依据 | Time Estimation Basis

- **核心阶段**（30天）：6个高优先级功能，每项3-4天，包含编码、测试、调试。MVP需快速迭代，优先功能简单但稳定。
- **增强阶段**（10天）：4个中优先级功能，每项2-3天，功能复杂性增加，需更多测试。
- **高级阶段**（10天）：7个低优先级功能，每项2天，重点在优化和集成，依赖前期功能稳定。
- **缓冲时间**：每阶段预留1-2天用于意外问题（如调试失败、兼容性调整）。

## 实施建议 | Implementation Advice

1. **分阶段验收**：
   - **核心阶段**：第3周末，验证MVP能生成钱包、存币、转账，助记词可恢复。
   - **增强阶段**：第4周末，测试量子加密和分布式存储的性能。
   - **高级阶段**：第5周末，模拟攻击场景（如异常交易、密钥丢失），验证防护效果。

2. **工具与技术**：
   - **开发**：使用Rust或JavaScript（ethers.js）开发钱包，兼容以太坊/Solana。
   - **库**：KDF用Argon2，HSM用OpenHSM，助记词分片用Shamir秘密共享，2FA用Google Authenticator，加密用OpenSSL。
   - **测试**：Grok和GPT-5生成测试用例，覆盖边缘场景（如分片丢失、设备离线）。

3. **用户体验**：
   - **助记词分片备份**：设计“选择3位好友，2人即可恢复”的引导界面。
   - **多签**：简化授权流程，如“一键请求好友签名”。
   - **2FA**：支持指纹/短信/TOTP，降低用户门槛。

4. **迭代优化**：
   - 每周复盘，检查功能完成度和稳定性。
   - 收集早期用户反馈，优化助记词分片和多签的交互体验。
   - 定期更新威胁情报和行为分析模型，保持防御能力。

## 注意事项 | Notes

- **不要跳步骤**：核心阶段未稳定（如KDF或HSM未测试），不要急于开发高级功能（如行为分析），否则可能返工。
- **测试驱动开发**：每个功能完成后，立即由GPT-5生成测试用例，Grok验证逻辑，您执行测试。
- **区块链兼容性**：初期锁定一个主流链（如以太坊）测试，确保KDF、多签、DID兼容，之后扩展多链支持。
- **资源限制**：HSM和量子加密成本高，初期可用软件模拟HSM（如手机安全区），量子加密用轻量算法。
