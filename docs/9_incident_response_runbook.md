# 9 — Incident Response Runbook（安全事件响应）

状态: Stable
适用范围: Defi-Hot-wallet-Rust 全组件（核心库、CLI、节点集成、CI/CD）

索引
- [1. 目标与范围](#purpose)
- [2. 角色与联系人](#roles)
- [3. 分级与 SLA](#severity)
- [4. 响应流程（标准作业程序）](#process)
- [5. 通信与对外通报](#comms)
- [6. 证据保全与取证清单](#forensics)
- [7. 常见场景 Playbooks](#playbooks)
- [8. 工具与检查清单](#tools)
- [9. 指标与演练](#metrics)
- [10. 附录：模板](#templates)
- [11. 变更日志](#changelog)

---

<a id="purpose"></a>
## 1. 目标与范围
- 快速识别、遏制与修复安全事件，保护资金与密钥，满足对外合规通报。
- 覆盖代码仓库、构建与发布、依赖供应链、运行时节点/钱包服务、密钥管理与备份。

---

<a id="roles"></a>
## 2. 角色与联系人
- Incident Commander (IC): 安全负责人（备用：后备 IC）
- Engineering Lead: 核心模块负责人
- Communications: 对外公告、法律与合规接口
- On-call SRE: 运维与节点管理员
- Artifact Owner: 发布与签名负责人
提示：将实际姓名/电话/IM频道放入内部通讯录，不在公开仓库暴露。

---

<a id="severity"></a>
## 3. 分级与 SLA
- SEV-0 Critical（密钥泄露/资金损失/大规模入侵）
  - 响应 ≤ 15 分钟，1 小时内初步通报，24 小时内阶段报告
- SEV-1 High（核心漏洞可被利用、资产高风险）
  - 响应 ≤ 1 小时，当天出补丁/缓解
- SEV-2 Medium（权限越权、信息泄露、DoS 风险）
  - 响应 ≤ 4 小时，1 个迭代内修复
- SEV-3 Low（低风险错配、日志暴露等）
  - 纳入常规版本修复

---

<a id="process"></a>
## 4. 响应流程（标准作业程序）
1) 发现与分级
   - 来源：告警/用户报告/代码审计/依赖通告/链上监控
   - 创建事件编号：IR-YYYYMMDD-序号；记录时间线
2) 立即遏制
   - 冻结高风险操作（提现/转账），切换只读模式
   - 轮换或撤销可疑 API/密钥/令牌
   - 隔离受影响主机或服务实例
3) 调查与取证
   - 收集系统/应用/链上证据，保持时间同步与证据链
4) 修复与缓解
   - 打补丁、配置变更、依赖回滚/升级、规则下发
5) 验证与恢复
   - 灰度发布 → 验证关键路径 → 逐步恢复
6) 复盘与改进
   - 根因分析、指标复盘、行动项与责任人/截止日期
   - 更新文档：威胁模型、补丁清单、安全基线

---

<a id="comms"></a>
## 5. 通信与对外通报
- 内部渠道：专用安全频道（最小知情原则），仅 IC 可以对外发声
- 对外公告内容：事件摘要、影响范围、时间线、已执行措施、用户措施建议、后续计划
- 法规：按地域监管要求（如有）提交通报

---

<a id="forensics"></a>
## 6. 证据保全与取证清单
- 时间同步：统一使用 UTC，校验 NTP
- 应用与系统日志：最近 7/30/90 天（按等级）留存
- 关键取证项
  - 配置与环境：.env、启动参数、权限与策略
  - 账户与访问：最近登录、令牌使用记录、远程会话
  - 构建与发布：CI 运行记录、签名产物、SBOM
  - 链上证据：地址、交易、事件日志
- Windows 采集示例（管理员 PowerShell）
  - 事件日志导出：
    - Application：`wevtutil epl Application C:\ir\Application.evtx`
    - System：`wevtutil epl System C:\ir\System.evtx`
    - OpenSSH：`wevtutil epl "OpenSSH/Operational" C:\ir\OpenSSH.evtx`
  - 网络连接：`Get-NetTCPConnection | Export-Csv C:\ir\net.csv -NoTypeInformation`
  - 进程与服务：`Get-Process | Export-Csv C:\ir\proc.csv -NoTypeInformation`
  - 用户与组：`Get-LocalUser; Get-LocalGroupMember administrators`
- 保全策略：只读挂载/镜像快照；每一步操作记录执行人与时间戳

---

<a id="playbooks"></a>
## 7. 常见场景 Playbooks
### 7.1 热钱包密钥疑似泄露（SEV-0）
- 立即
  - 暂停提现/转账，设置风控阈值为阻断
  - 通过离线环境生成新密钥（参考 docs/8_key_management_policy.md）
  - 将资金迁移至新地址（多签/硬件保护）
- 2 小时内
  - 全面轮换相关 API/令牌；撤销旧密钥权限
  - 更新签名与白名单，发布热修复版本
- 恢复
  - 小流量灰度恢复 → 全量
  - 公告说明并指导用户检查授权/地址

### 7.2 节点/RPC 被劫持或不稳定（SEV-1）
- 切换至受信第三方或自建备节点；启用只读模式
- 验证区块头与余额一致性；对比两家以上数据源
- 恢复后补做一致性校验与重放审计

### 7.3 依赖供应链高危漏洞（SEV-1）
- 执行 `cargo audit` 和 `cargo-deny`；确定受影响范围
- 升级/回滚；必要时打补丁版并加签发布
- 发布后在 CI 强化 deny 规则与最小版本锁定

### 7.4 CI/CD 机密泄露（SEV-0/1）
- 立即吊销泄露凭据，冻结流水线
- 重新生成所有敏感凭据，轮换签名密钥
- 审计最近发布产物与签名校验，必要时撤回

---

<a id="tools"></a>
## 8. 工具与检查清单
- 代码与依赖
  - `cargo fmt`, `cargo clippy -D warnings`, `cargo test`
  - `cargo audit`, `cargo deny check`, 生成 SBOM（`cargo sbom` 或 CycloneDX）
- 发布与签名
  - 产物签名与校验、发布记录与可追溯性
- 访问与权限
  - 最小权限、双人复核、关键操作审计开启
- 清单（事件开启必做）
  - [ ] 创建 IR 工单与时间线
  - [ ] 分配 IC/沟通负责人/技术负责人
  - [ ] 证据保全开始与完成时间
  - [ ] 变更冻结与例外审批记录
  - [ ] 复盘行动项登记到 Issue/Project

---

<a id="metrics"></a>
## 9. 指标与演练
- 指标：MTTA、MTTR、RTO、RPO、遏制时长、影响用户数
- 演练：季度至少 1 次全链路演练；演练后 1 周内完成改进项

---

<a id="templates"></a>
## 10. 附录：模板
### 10.1 对外公告模板
- 标题：关于[事件类型]的安全通告
- 时间线：发现/处置/恢复关键时间点
- 影响范围：版本、组件、是否影响资金
- 已采取措施：遏制、修复、轮换
- 用户需执行：升级版本/撤销授权/更换密钥
- 责任与计划：后续工作与时间点

### 10.2 事后复盘报告（Postmortem）
- 摘要、根因、影响评估、时间线、处置过程、改进项（负责人/DDL）、附件（证据清单）

---

<a id="changelog"></a>
## 11. 变更日志
- 2025-09-22：完善分级/SOP/取证清单/Playbooks/模板与指标。
- 2025-09-21：初稿。