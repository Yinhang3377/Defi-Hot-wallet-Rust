use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AppEnvConfig {
    pub some_field: Option<String>,
    pub another_field: Option<u16>,
}

/// 浠庣幆澧冨彉閲忓姞杞介厤缃€?///
/// 鏈熸湜鐨勭幆澧冨彉閲忥細
/// - `APP_SOME_FIELD`: `some_field` 鐨勫瓧绗︿覆鍊笺€?/// - `APP_ANOTHER_FIELD`: `another_field` 鐨?u16 鍊笺€?pub fn load() -> Result<AppEnvConfig, Box<dyn std::error::Error>> {
    let some_field = env::var("APP_SOME_FIELD").ok();
    let another_field = env::var("APP_ANOTHER_FIELD").ok().and_then(|s| s.parse().ok());

    Ok(AppEnvConfig { some_field, another_field })
}
