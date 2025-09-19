//! generator.rs
//! 代码生成/脚手架工具模块

/// 生成指定类型的模板代码（示例）
#[allow(dead_code)]
pub fn generate_wallet_template(name: &str) -> String {
    format!("// 钱包模板: {}\nstruct {}Wallet {{\n    // ...字段定义\n}}", name, name)
}

// 可扩展：生成密钥、配置、测试等脚手架代码
