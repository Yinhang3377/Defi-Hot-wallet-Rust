<<<<<<< HEAD
﻿#[allow(clippy::module_inception)]
pub mod config;
pub mod env_config;
=======
//! 配置模块：负责加载和管理环境变量（如 ENCRYPTION_KEY、NETWORK、SALT）。
#![allow(clippy::module_inception)]

pub mod config; // keep existing structure; allow the clippy lint instead of refactoring now.
pub use config::WalletConfig;
>>>>>>> be35db3d094cb6edd3c63585f33fdcb299a57158
