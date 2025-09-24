//! # DeFi Hot Wallet
//! 
//! 一个用Rust编写的多链支持、安全性优先的DeFi热钱包实现。
//! 支持多区块链网络、量子安全加密和Shamir秘密共享等高级功能。

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::upper_case_acronyms)]
#[allow(clippy::module_inception)]

/// 适配器模块 - 处理不同区块链接口的适配
pub mod adapter;

/// 应用程序逻辑模块
#[allow(clippy::module_inception)]
pub mod application;

/// 审计和日志记录模块
pub mod audit;

/// 区块链集成模块 - 以太坊、Solana等多链支持
pub mod blockchain;

/// 配置管理模块
#[allow(clippy::module_inception)]
pub mod config;

/// 核心功能模块 - 包含钱包、交易等核心功能
pub mod core;

/// 加密和安全模块 - 包含量子安全实现、多重签名等
pub mod crypto;

/// 国际化和本地化支持
pub mod i18n;

/// 基础设施模块 - 处理底层系统资源
#[allow(clippy::module_inception)]
pub mod infrastructure;

/// 接口模块 - 定义系统内外部接口
#[allow(clippy::module_inception)]
pub mod interface;

/// 监控和指标收集模块
pub mod monitoring;

/// 最小可行产品实现
pub mod mvp;

/// 网络通信模块
pub mod network;

/// 操作和管理功能
pub mod ops;

/// 插件系统
pub mod plugin;

/// 安全功能
pub mod security;

/// 服务层 - 提供各种业务服务
#[allow(clippy::module_inception)]
pub mod service;

/// 存储模块 - 处理数据持久化
pub mod storage;

/// 工具和实用函数
pub mod tools;

/// 钱包版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 应用程序名称
pub const APP_NAME: &str = "DeFi Hot Wallet";