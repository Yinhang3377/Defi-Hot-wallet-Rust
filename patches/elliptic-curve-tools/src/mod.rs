﻿//! src/tools/mod.rs
//!
//! Utility functions and tools used across the wallet.
// 如果模块文件是 sop.rs，则导出模块并重导出函数到 crate 根
pub mod sop;

// 便于旧测试 `use elliptic_curve_tools::sum_of_products_impl_relaxed;` 正常工作
pub use crate::sop::sum_of_products_impl_relaxed;
pub mod serdes;
