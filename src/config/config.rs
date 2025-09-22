use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub env: String,
}
