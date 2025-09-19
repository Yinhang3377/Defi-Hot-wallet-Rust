/// 主入口：集成配置、安全、错误等模块，实现 wallet create 命令生成加密账户
use clap::{ Parser, Subcommand };
use hex;
use hot_wallet::config::WalletConfig; // 钱包配置加载
use hot_wallet::security::encryption::WalletSecurity; // 加密/解密操作
use hot_wallet::security::memory_protection::SensitiveData;
use serde::{ Deserialize, Serialize };
use env_logger;
use secp256k1::{ PublicKey, Secp256k1, SecretKey };
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{ self, Write };
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use rand::rngs::OsRng;
use rand::RngCore;

#[derive(Parser, Debug)]
#[command(author, version, about = "A secure, multi-chain hot wallet framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new encrypted wallet keypair
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

impl Cli {
    pub fn prompt_password() -> Result<String, io::Error> {
        print!("请输入加密密钥: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        Ok(password.trim().to_string())
    }
}

/// 程序主入口
fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志记录器，以便在加密等模块中打印错误日志
    env_logger::init();

    // 1. 使用 clap 解析命令行参数
    let cli = Cli::parse();

    // 2. 加载环境变量配置
    let config = match WalletConfig::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("[配置错误] {}", e);
            return Err(Box::from(e));
        }
    };
    println!("[配置] network: {}", config.network);

    // 3. 根据解析的命令执行相应操作
    match &cli.command {
        Commands::Create { aad, output } => {
            println!("正在创建新的加密钱包...");

            // 提示用户输入加密密钥
            let encryption_key = Cli::prompt_password()?;

            // 生成 secp256k1 密钥对
            let secp = Secp256k1::new();
            // Updated secret key generation to use a random 32-byte array
            let mut rng = OsRng;
            let mut secret_key_bytes = [0u8; 32];
            rng.fill_bytes(&mut secret_key_bytes);
            let secret_key = SecretKey::from_slice(&secret_key_bytes).expect(
                "Failed to create secret key"
            );
            let public_key = PublicKey::from_secret_key(&secp, &secret_key);
            println!("[生成] 公钥: {}", public_key);

            // 用 SensitiveData 包裹私钥并锁定内存
            let sensitive_sk = SensitiveData::new(secret_key.secret_bytes());

            // 准备关联数据 (AAD)
            let aad_bytes = aad.as_deref().unwrap_or("").as_bytes();
            println!("[加密] 使用关联数据 (AAD): '{}'", aad.as_deref().unwrap_or("<无>"));

            // 用用户输入的加密密钥加密私钥
            let encrypted = WalletSecurity::encrypt_private_key(
                &sensitive_sk.data,
                &encryption_key,
                aad_bytes
            )?;

            println!("[加密] 加密私钥(hex): {}", hex::encode(&encrypted));

            // 创建并保存钱包文件
            let wallet_file = WalletFile {
                public_key: public_key.to_string(),
                encrypted_private_key: hex::encode(&encrypted),
                network: config.network.clone(),
                aad: aad.as_deref().unwrap_or("").to_string(),
            };

            let wallet_json = serde_json::to_string_pretty(&wallet_file)?;

            let mut open_options = OpenOptions::new();
            open_options.write(true).create_new(true); // 防止覆盖已存在的文件

            // 在 Unix 系统上，设置文件权限为 600 (仅所有者可读写)
            #[cfg(unix)]
            open_options.mode(0o600);

            open_options.open(output)?.write_all(wallet_json.as_bytes())?;

            println!("✅ 钱包已成功创建并保存至: {}", output.display());
        }
    }

    Ok(())
}
