mod config;
mod security;
mod tools;
use crate::config::WalletConfig;
use crate::security::encryption::EncryptionService;
use crate::security::memory_protection::SensitiveData;
use crate::tools::error::WalletError;
use clap::{ Parser, Subcommand };
use secp256k1::Secp256k1;
use secp256k1::All;
use serde::{ Serialize, Deserialize };
use std::error::Error;
use std::fs;
use std::io::{ self, Write };
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "A secure, multi-chain hot wallet framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Create {
        /// Optional associated data to bind to the encryption
        #[arg(long)]
        aad: Option<String>,

        /// Path to save the wallet file
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[derive(Serialize, Deserialize)]
struct WalletFile {
    public_key: String,
    encrypted_private_key: String,
    network: String,
    aad: String,
}

#[derive(Debug)]
struct AppError(String);

impl From<Box<dyn Error>> for AppError {
    fn from(e: Box<dyn Error>) -> Self {
        AppError(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError(err.to_string())
    }
}

impl From<WalletError> for AppError {
    fn from(err: WalletError) -> Self {
        AppError(err.to_string())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {}

impl Cli {
    pub fn prompt_password() -> Result<String, io::Error> {
        print!("请输入加密密钥: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        Ok(password.trim().to_string())
    }
}

fn load_config() -> Result<WalletConfig, AppError> {
    WalletConfig::from_env().map_err(|e| AppError(format!("[配置错误] {}", e)))
}

fn initialize_services() -> Result<(Secp256k1<All>, SensitiveData<Vec<u8>>), AppError> {
    let secp = Secp256k1::new();
    let (secret_key, _) = secp.generate_keypair(&mut rand::thread_rng());
    let sensitive_sk = SensitiveData::new(secret_key.secret_bytes().to_vec());
    Ok((secp, sensitive_sk))
}

fn execute_command(cli: Cli, config: WalletConfig) -> Result<(), AppError> {
    match cli.command {
        Commands::Create { aad, output } => {
            // 提示用户输入加密密钥
            let encryption_key = Cli::prompt_password()?;

            let (secp, sensitive_sk) = initialize_services()?;
            // 准备关联数据 (AAD)
            let aad_bytes = aad.as_deref().unwrap_or("").as_bytes();
            println!("[加密] 使用关联数据 (AAD): '{}'", aad.as_deref().unwrap_or("<无>"));

            // 用用户输入的加密密钥加密私钥
            let encryption_service = EncryptionService::new(aad_bytes.to_vec(), vec![]);
            let encrypted = encryption_service.encrypt(&sensitive_sk.data, &encryption_key)?;

            println!("[加密] 加密私钥(hex): {}", hex::encode(&encrypted));

            // 创建并保存钱包文件
            let wallet_file = WalletFile {
                public_key: secp.generate_keypair(&mut rand::thread_rng()).1.to_string(),
                encrypted_private_key: hex::encode(&encrypted),
                network: config.network.clone(),
                aad: aad.unwrap_or_default(),
            };

            let wallet_json = serde_json::to_string_pretty(&wallet_file)?;

            fs::write(&output, wallet_json)?;
            println!("✅ 钱包已成功创建并保存至: {}", output.display());
            Ok(())
        }
    }
}

/// 程序主入口
fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志记录器，以便在加密等模块中打印错误日志
    env_logger::init();

    // 1. 使用 clap 解析命令行参数
    let cli = Cli::parse();

    // 2. 加载环境变量配置
    let config = load_config()?;
    // 3. 根据解析的命令执行相应操作
    execute_command(cli, config).map_err(|e| Box::new(e) as Box<dyn Error>)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use tokio_test::block_on;

    mock! {
        WalletConfig {};
        impl WalletConfig {
            pub fn from_env() -> Result<Self, Box<dyn Error>>;
        }
    }

    #[test]
    fn test_load_config_failure() {
        let result = load_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_initialize_services() {
        let result = initialize_services();
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_command_create() {
        let cli = Cli {
            command: Commands::Create {
                aad: Some("test_aad".to_string()),
                output: PathBuf::from("test_wallet.json"),
            },
        };
        let config = WalletConfig {
            network: "testnet".to_string(),
        };
        let result = execute_command(cli, config);
        assert!(result.is_ok());
    }
}
