# 5 — Code Review Plan（代码审查计划）

目录
- [概述](#概述)
- [审查目标](#审查目标)
- [审查范围](#审查范围)
- [审查流程](#审查流程)
- [工具与自动化](#工具与自动化)
- [审查清单（Checklist）](#审查清单checklist)
- [通过标准](#通过标准)
- [时间安排](#时间安排)
- [角色与责任](#角色与责任)
- [输出与归档](#输出与归档)

## 概述
目标是以最低成本发现高风险缺陷，确保热钱包在安全、功能、性能与可维护性上的可靠交付。

## 审查目标
- 安全性：KDF 参数安全、内存零化、私钥与敏感数据不可泄露；多签/权限正确。
- 功能性：创建/恢复/签名/转账等核心流程符合需求与边界。
- 可维护性：模块边界清晰、trait/接口合理、错误处理一致、文档/注释清楚。
- 性能：加密/签名与网络发送不成为瓶颈；普通转账端到端 < 2s（测试链）。

## 审查范围
- security/：encryption.rs、memory_protection.rs、kdf.rs、access_control.rs、anti_debug.rs、compliance.rs
- audit/：operation_log.rs、alert.rs、confirmation.rs、rollback.rs
- core/ 与 application/：domain.rs、application.rs（钱包/交易/多签逻辑）
- 重点：Shamir 分片与恢复、多签门限逻辑、AES-GCM/TLS 的密钥与 nonce 管理

## 审查流程
1. 提交前自检（开发）
   - cargo fmt、cargo clippy 无阻断
   - 单元测试通过，新增代码覆盖率≥80%（tarpaulin 报告）
   - 敏感数据路径标注并验证 zeroize
2. AI 辅助审查
   - 算法/参数：Argon2、Shamir、多签流程
   - 生成边界/攻击场景用例并跑测试
3. 同行评审（PR）
   - ≥1 名安全角色 + 1 名 Rust 工程师签核
   - 关注接口一致性、错误处理、并发与边界
4. 结果处理
   - 标注严重级别（Critical/High/Medium/Low），高危修复后再合并

## 工具与自动化
- cargo fmt、clippy、test、audit（CVE）
- 覆盖率：cargo tarpaulin
- 依赖治理：dependabot 或 cargo-deny
- 合约（如有）：Slither / mythril
- 规则扫描（可选）：semgrep
- CI：GitHub Actions 在 PR 上强制执行以上检查

## 审查清单（Checklist）
- 安全
  - Argon2 参数文档化（内存/并行/迭代）
  - 私钥/种子/会话密钥受控内存，zeroize 可靠触发
  - AEAD 随机数/nonce 生成与重用防护
  - 多签门限、超时、回退策略无死锁；签名材料不泄露
  - 分片/恢复脚本≥100 次随机回归通过
- 功能
  - 创建/导入/恢复/签名/转账端到端可重放
  - 失败路径（网络/余额/拒签）行为一致
- 可维护
  - 错误类型统一（thiserror/anyhow）；日志不泄露敏感数据
  - trait/模块边界清晰，公共 API 文档注释完整
- 性能
  - KDF、签名、序列化、网络发送基准在阈值内
  - 日志/监控异步化，不阻塞交易主路径

## 通过标准
- Critical/High 为 0（未解决不得合并）
- 覆盖率 ≥80%，关键安全模块更高
- cargo audit 无高危依赖；CI 全绿
- PR 评审意见已关闭并记录

## 时间安排
- 第 1 周：框架结构（config/domain/application）
- 第 2–3 周：security/ 与 audit/ 深度审查
- 第 4 周：插件/运维/i18n 等
- 每模块 2–4 小时，累计约 20 小时（贯穿 40 天开发）
  
## 角色与责任
- 你：提交 PR、自检、修复
- 协作：安全/性能建议与复核
- AI：用例生成与自动化建议
- 参与者（若有）：最终签核与归档

## 输出与归档
- GitHub PR 记录（讨论、变更、结论）
- 问题清单与修复跟踪（Issues/Projects）
- 覆盖率/基准测试报告、cargo audit 报告
- 审查通过的提交编号（Tag/Release Note）