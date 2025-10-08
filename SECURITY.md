# DeFi Hot Wallet 安全策略

## 支持版本

| 版本   | 支持状态            |
| ------ | ------------------ |
| 0.1.x  | :white_check_mark: |

## 报告安全漏洞

如果您发现安全漏洞，请发送邮件至 [security@example.com](mailto:security@example.com)，而非创建公开 issue。

## 安全状态

### 已解决的安全问题

- ✅ **RUSTSEC-2025-0009**: ring 0.16.20 中的 AES 函数在启用溢出检查时可能崩溃
  - 解决方案：升级到 ring 0.17.14 和 jsonwebtoken 9.3.1

### 已缓解但未完全解决的安全问题

- ⚠️ **RUSTSEC-2023-0071**: RSA 0.9.8 中的 Marvin 攻击（中等严重性 - 5.9）
  - 状态：目前无可用升级
  - 缓解措施：已从 SQLx 配置中排除 MySQL 特性，避免此依赖路径
  - 影响：仅限于 MySQL 数据库连接（默认配置未使用）
  - 后续计划：持续监控 sqlx-mysql 上游依赖的修复

### 未维护依赖

项目依赖树中存在以下未维护的包，但安全影响有限：
- async-std：仅用于测试代码（httpmock）
- atomic-polyfill：用于 postcard 序列化（非关键路径）
- fxhash：通过 ethers-providers 间接依赖
- instant：通过 ethers 间接依赖

## 安全审计流程

- CI 流程中使用 `cargo audit` 自动检查依赖安全
- 所有 PR 的代码审查过程中包含安全检查
- 遵循项目安全编码规范（详见 README）

## 上次安全更新：2025 年 10 月