# 6_test_matrix — 测试矩阵

<a id="toc"></a>
目录
- [1. 概要](#overview)
- [2. 目标与范围](#goals-scope)
- [3. 测试类型](#types)
- [4. 测试矩阵（场景表）](#matrix)
- [5. 覆盖率与通过标准](#criteria)
- [6. 工具与环境](#tools-env)
- [7. 时间与里程碑](#schedule)
- [8. 角色与责任](#owners)
- [9. CI 集成](#ci)
- [10. 输出与归档](#artifacts)
- [11. 变更日志](#changelog)

---

<a id="overview"></a>
## 1. 概要
本文件定义热钱包的测试目标、类型与场景矩阵，覆盖功能、安全、性能与兼容性，确保生成/转账/恢复/监控等关键路径稳定可用。

[↑ 返回目录](#toc)

---

<a id="goals-scope"></a>
## 2. 目标与范围
- 功能性：创建/导入/恢复钱包，ETH 与常见 ERC-20 转账，节点切换。
- 安全性：KDF 参数与零化、端到端加密、日志脱敏、多签与回滚、反调试。
- 性能：普通转账端到端延迟可接受；关键算法基准在阈值内。
- 兼容性：Linux/Windows（开发、CI），可扩展到移动端/多链（后续）。
- 不在本轮范围：移动端原生 UI、极端弱网/断网后的大规模重放恢复。

[↑ 返回目录](#toc)

---

<a id="types"></a>
## 3. 测试类型
1) 单元测试 tests/unit  
- 目标：验证函数/模块最小单元逻辑  
- 模块：security/（kdf、memory_protection）、core/domain.rs、audit/*  
- 工具：Rust test、proptest、mockall

2) 集成测试 tests/integration  
- 目标：验证模块间协作（application + security + audit）  
- 场景：签名→序列化→RPC 发送→状态落库  
- 工具：tokio-test、testcontainers（可选）

3) 端到端测试 tests/e2e  
- 目标：模拟真实用户（创建/转账/恢复/多签）  
- 工具：ethers-rs + 本地/测试网 RPC，脚本驱动

4) 安全测试 tests/security  
- 目标：攻击模拟与防护验证（内存、MITM、分片恢复、权限）  
- 工具：cargo audit、semgrep（可选）、Slither（合约）

5) 性能与基准 benches/*  
- 目标：KDF/签名/序列化/RPC 延迟与吞吐  
- 工具：criterion、wrk（HTTP 代理时）

[↑ 返回目录](#toc)

---

<a id="matrix"></a>
## 4. 测试矩阵（场景表）

| 模块 | 测试点 | 场景 | 预期结果 | 工具 |
|---|---|---|---|---|
| security/kdf.rs | 密钥派生 | 使用既定 Argon2id 参数派生种子 | 输出定长 32B；重复输入一致 | Rust test |
| security/memory_protection.rs | 零化 | 私钥用后清理 | 内存被 zeroize，污点扫描无残留 | Rust test + valgrind(可选) |
| security/encryption.rs | AEAD | AES-256-GCM 加解密 | 解密一致；nonce 唯一性测试通过 | Rust test |
| security/anti_debug.rs | 反调试 | 命中调试探针 | 触发退出/告警 | Rust test |
| audit/operation_log.rs | 日志脱敏 | 交易/恢复日志 | 日志无助记词/私钥；PII 掩码 | Rust test |
| audit/rollback.rs | 失败回滚 | 合约执行失败 | 资产回退、余额一致 | integration |
| application/wallet.rs | 钱包创建 | 生成中文助记词 | BIP-39 24 词有效 | bip39 |
| application/transfer.rs | 单签转账 | ETH→地址 | 交易成功；nonce/余额正确 | ethers-rs |
| application/multisig.rs | 多签 | 2-of-3 收集签名 | 门限达成发送；拒签回滚 | integration |
| infra/rpc_manager.rs | 节点切换 | 主节点宕机 | 切换白名单节点，无中断 | integration |
| ops/metrics.rs | 指标 | 1k req/s 压力 | 指标上报、告警无异常 | Prometheus |
| recovery/shamir.rs | 分片恢复 | 2-of-3 恢复 | 成功恢复种子；错误分片失败 | Rust test |
| e2e | 丢失设备 | 分片+云备份恢复 | 新设备完成恢复并可转账 | e2e 脚本 |

[↑ 返回目录](#toc)

---

<a id="criteria"></a>
## 5. 覆盖率与通过标准
- 单元测试：行/分支覆盖率 ≥ 80%（关键安全模块更高）。  
- 集成/E2E：关键用户旅程 100% 通过（创建→转账→恢复→再次转账）。  
- 安全：cargo audit 无高危；日志不泄露敏感数据；分片恢复≥100 次随机化回归全通过。  
- 性能基线（示例，按硬件调参）：  
  - KDF（Argon2id）以团队基线参数为准；基准报告中给出时间/内存/并行配置与结果。  
  - 单笔测试网转账（不含链确认）端到端 ≤ 2s（网络正常时）。  

[↑ 返回目录](#toc)

---

<a id="tools-env"></a>
## 6. 工具与环境
- Rust: cargo test, cargo nextest(可选), tarpaulin(覆盖率), criterion(基准)  
- 安全: cargo audit, semgrep(可选), Slither(如有合约)  
- 测试网: Goerli/Sepolia 或本地 devnet（anvil/hardhat）  
- 依赖: ethers-rs, bip39, zeroize, aes-gcm, argon2  
- 环境: Linux (CI), Windows/WSL (开发), Docker (可选)

[↑ 返回目录](#toc)

---

<a id="schedule"></a>
## 7. 时间与里程碑
- 第1周：核心单元测试（config/domain/security 基础）  
- 第2–3周：安全 + 集成测试（KDF/零化/日志/多签/回滚/节点切换）  
- 第4周：E2E 场景与性能基准，出测试报告  
- 总计：约 30 小时，穿插 40 天开发流程执行

[↑ 返回目录](#toc)

---

<a id="owners"></a>
## 8. 角色与责任
- 开发（你）：编写测试、修复缺陷、提交报告  
- 安全顾问：场景建议、参数基线复核  
- 测试/CI：维护流水线、覆盖率阈值与告警  
- AI 助手：生成边界与攻击用例、回归数据集

[↑ 返回目录](#toc)

---

<a id="ci"></a>
## 9. CI 集成
- 触发：PR 与 main 分支 push  
- 任务：fmt → clippy → unit/integration → tarpaulin(阈值) → cargo audit  
- 工件：测试/覆盖率/基准报告上传（Artifacts）  
- 阻断：覆盖率或安全检查不达标则拒绝合并

[↑ 返回目录](#toc)

---

<a id="artifacts"></a>
## 10. 输出与归档
- 测试报告：覆盖率、失败用例、基准曲线  
- 安全日志：攻击用例与防护结果  
- 版本记录：将关键结果写入 CHANGELOG 与发布说明  
- 归档：docs/archives/tests/YYYY-MM-DD.md

[↑ 返回目录](#toc)

---

<a id="changelog"></a>
## 11. 变更日志
- 2025-09-21：按统一规范重写测试矩阵，补充场景表、阈值与 CI 集成。

[↑ 返回目录](#toc)