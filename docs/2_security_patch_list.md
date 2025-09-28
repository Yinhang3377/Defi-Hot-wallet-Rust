// ...existing code...
# 2_security_patch_list — 安全补丁清单

> 目的：集中管理已识别安全问题、补丁状态、影响范围与验证步骤。与 1_architecture_blueprint.md 保持一致的锚点/目录格式，便于跳转与审计记录。  
> Last updated: 2025-09-21

<a id="toc"></a>
## 目录
- [1. 概要](#overview)
- [2. 补丁清单（表格）](#patch-inventory)
- [3. 补丁优先级与风险评估](#risk-assessment)
- [4. 补丁处理流程](#patch-process)
- [5. 部署与回滚计划](#deployment-rollback)
- [6. 验证与测试矩阵](#verification-testing)
- [7. 监控与审计要求](#monitoring-audit)
- [8. 责任与沟通](#owners-communication)
- [9. 参考与资源](#references)
- [10. 变更日志](#changelog)

---

<a id="overview"></a>
## 1. 概要
- 该文档用于记录项目中发现的安全漏洞、补丁状态与处置记录。  
- 每条记录须包含：唯一 ID、发现日期、严重级别、受影响模块、描述、修复措施、状态、负责人、验证步骤与备注。

[↑ 返回目录](#toc)

---

<a id="patch-inventory"></a>
## 2. 补丁清单（表格）
说明：按发现时间倒序排列。状态候选值：Open / In Progress / Patched / Verified / Deferred / WontFix

| ID | 发现日期 | 严重级别 | 受影响模块 | 漏洞简述 | 当前状态 | 负责人 | 修复/减轻措施摘要 | 验证状态 |
|---:|:--------:|:--------:|:----------|:--------|:--------:|:------:|:------------------|:--------:|
| SP-2025-001 | 2025-09-15 | High | src/security/encryption.rs | KDF 未产出固定长度密钥、使用不当导致衍生密钥不可控 | In Progress | @dev1 | 用 Argon2id 生成定长 32B 密钥并使用 aes-gcm / chacha20poly1305 | Pending |
| SP-2025-002 | 2025-09-12 | Medium | src/audit/logging.rs | 日志可能包含敏感字段（助记词/私钥） | Patched | @dev2 | 结构化日志脱敏、移除助记词日志、加入审计白名单 | Verified |
| SP-2025-003 | 2025-09-10 | High | src/network/node_manager.rs | RPC 节点故障切换不安全，可能向恶意节点发送交易 | Open | @dev3 | 增加节点白名单、签名验证与请求限流 | Open |
| SP-2025-004 | 2025-09-08 | Low | examples/basic_wallet.rs | 示例中将明文助记词写入文件（示例危险） | Patched | @doc | 修改示例为交互输入并提示安全存储 | Verified |
| SP-2025-005 | 2025-09-05 | Critical | CI (workflow) | CI 中显示敏感变量日志，潜在泄露 | In Progress | @ops | 隐藏 Secrets、移除敏感输出、审计工作流 | Pending |

> 注：将所有已关闭/已验证条目归档到 docs/archives/security/YYYY-MM-DD.md（建议）。

[↑ 返回目录](#toc)

---

<a id="risk-assessment"></a>
## 3. 补丁优先级与风险评估
- 严重级别定义（建议）：
  - Critical：可直接导致密钥泄露/资金丢失或远程代码执行。立即响应（Triage 0-4h）。
  - High：高风险，影响关键功能（例如签名、KDF、网络层），快速修复（24-72h）。
  - Medium：中等风险，可缓解后合并到下个发布周期（7天内）。
  - Low：信息性或示例代码，安排文档/示例修正。

- 风险评估字段：
  - 可利用性（Exploitability）
  - 影响范围（资产/用户/系统）
  - 暴露面（本地/网络/CI）

[↑ 返回目录](#toc)

---

<a id="patch-process"></a>
## 4. 补丁处理流程
1. 报告与登记：任何人发现问题通过 Issue/邮件/Slack 提交，创建安全 Issue（前缀 security/ 或标签 security）。
2. 初步评估（Triage）：安全负责人 4 小时内判断影响与优先级。
3. 分配：指派开发负责人与 QA 验证人。
4. 修复实现：在 feature/security/* 分支完成修复并编写单元/集成测试。
5. 内部评审：代码审查 + SCA（依赖审计）+ 静态扫描（cargo clippy / Grype 等）。
6. 测试与验证：走 CI（不泄露 secrets），QA 验证补丁。
7. 发布：合并到主分支并在下次 release 中声明修复，若 Critical 可单独发布 hotfix。
8. 关闭与归档：更新补丁表格、CHANGELOG、并保留审计日志。

[↑ 返回目录](#toc)

---

<a id="deployment-rollback"></a>
## 5. 部署与回滚计划
- 部署流程：
  - 在 staging 环境先部署并执行完整回归 + 安全回归测试。
  - 监控关键指标（错误率、异常 RPC 响应、交易失败率）。
  - 若指标正常，则按 Canary / 蓝绿策略逐步切换到生产。

- 回滚策略：
  - 若发现严重回归或新安全问题，执行回滚到上一个已知良好版本（记录回滚理由与时间）。
  - 回滚前通知相关负责人并暂停对外交易流量（若适用）。

[↑ 返回目录](#toc)

---

<a id="verification-testing"></a>
## 6. 验证与测试矩阵
- 单元测试：覆盖补丁代码路径（目标覆盖率 ≥ 80%）。
- 集成测试：模拟 RPC/节点故障、重放攻击、异常输入。
- E2E：使用隔离账户在测试网执行交易，验证签名/nonce/回滚行为。
- 模拟演练：季度进行红队演练和故障注入（chaos testing）。
- 自动化安全扫描：依赖漏洞扫描（Dependabot/Grype）、静态分析（cargo audit）。

表：验证项示例

| 验证项 | 类型 | 负责人 | 通过标准 |
|---|---:|:---:|:---|
| KDF 密钥长度与随机性 | 单元+统计测试 | @dev1 | 生成密钥 32B 且熵符合阈值 |
| 日志脱敏 | 单元+人工 | @dev2 | 日志中无完整助记词/私钥 |
| RPC 节点切换安全 | 集成 | @dev3 | 不将交易发送到未验证节点 |

[↑ 返回目录](#toc)

---

<a id="monitoring-audit"></a>
## 7. 监控与审计要求
- 必须记录的审计事件：创建/导入助记词、导出私钥、发起交易、回滚操作、权限变更。
- 审计日志要求：结构化、可检索、敏感字段掩码、保留期（例如 90 天），并可按需导出（合规要求）。
- 运行时监控：Prometheus 指标 + AlertRules（如 tx failure rate > X、RPC error rate > Y）。
- 安全告警：配置 Slack/Email 告警渠道，并指定 SLO 与响应时限。

[↑ 返回目录](#toc)

---

<a id="owners-communication"></a>
## 8. 责任与沟通
- 安全负责人（Security Lead）：总体 Triage、对外通报（例如合规/法律团队）。
- 开发负责人（Assigned Dev）：实现补丁、编写测试、PR 合并。
- QA / 验证负责人：执行验证矩阵并关闭补丁条目。
- 运维（Ops）：部署、监控告警联动、回滚执行。

沟通渠道：安全事件通过 private channel 报告（避免泄露），对外公告由 Security Lead 与 PM 协同发布。

[↑ 返回目录](#toc)

---

<a id="references"></a>
## 9. 参考与资源
- 项目内部：docs/1_architecture_blueprint.md、CHANGELOG.md、SECURITY.md（建议新增）
- 外部：OWASP Top 10、CWE、Rust Sec Advisory Database、Cargo Audit、Grype

[↑ 返回目录](#toc)

---

<a id="changelog"></a>
## 10. 变更日志
- 2025-09-21：初始版本，包含补丁表格示例、流程与验证矩阵。  
- 建议：每次补丁状态变更在此追加记录（条目 ID、变更人、时间、摘要）。

[↑ 返回目录](#toc)
  
// ...existing code...