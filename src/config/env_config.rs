use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AppEnvConfig {
    pub some_field: Option<String>,
    pub another_field: Option<u16>,
}

/// 从环境变量加载配置。
///
/// 期望的环境变量：
/// - `APP_SOME_FIELD`: `some_field` 的字符串值。
/// - `APP_ANOTHER_FIELD`: `another_field` 的 u16 值。
pub fn load() -> Result<AppEnvConfig, Box<dyn std::error::Error>> {
    let some_field = env::var("APP_SOME_FIELD").ok();
    let another_field = env::var("APP_ANOTHER_FIELD").ok().and_then(|s| s.parse().ok());

    Ok(AppEnvConfig { some_field, another_field })
}
