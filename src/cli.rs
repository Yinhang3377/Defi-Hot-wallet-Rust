use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wallet-cli")]
#[command(about = "DeFi Hot Wallet CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 鍒涘缓鏂伴挶鍖?    Create {
        /// 閽卞寘鍚嶇О
        #[arg(short, long)]
        name: String,
        /// 杈撳嚭鏂囦欢璺緞
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 鏄剧ず閽卞寘淇℃伅
    Info {
        /// 閽卞寘鍚嶇О
        #[arg(short, long)]
        name: String,
    },
    /// 杞处
    Transfer {
        /// 閽卞寘鍚嶇О
        #[arg(short, long)]
        name: String,
        /// 鎺ユ敹鍦板潃
        #[arg(short, long)]
        to: String,
        /// 閲戦
        #[arg(short, long)]
        amount: String,
    },
    /// 鏌ヨ浣欓
    Balance {
        /// 閽卞寘鍚嶇О
        #[arg(short, long)]
        name: String,
    },
    /// 妗ユ帴杞处
    Bridge {
        /// 閽卞寘鍚嶇О
        #[arg(short, long)]
        name: String,
        /// 婧愰摼
        #[arg(long)]
        from_chain: String,
        /// 鐩爣閾?        #[arg(long)]
        to_chain: String,
        /// 浠ｅ竵
        #[arg(short, long)]
        token: String,
        /// 閲戦
        #[arg(short, long)]
        amount: String,
    },
    /// 鍒楀嚭鎵€鏈夐挶鍖?    List,
    /// 鐢熸垚鍔╄璇?    GenerateMnemonic,
}

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { name, output } => {
            println!("馃敀 鍒涘缓閽卞寘: {}", name);
            if let Some(path) = output {
                println!("杈撳嚭鍒? {}", path.display());
            }
            // TODO: 瀹炵幇閽卞寘鍒涘缓閫昏緫
        }
        Commands::Info { name } => {
            println!("馃搵 鏄剧ず閽卞寘淇℃伅: {}", name);
            // TODO: 瀹炵幇閽卞寘淇℃伅鏄剧ず閫昏緫
        }
        Commands::Transfer { name, to, amount } => {
            println!("馃捀 杞处: {} -> {} 閲戦: {}", name, to, amount);
            // TODO: 瀹炵幇杞处閫昏緫
        }
        Commands::Balance { name } => {
            println!("馃挵 鏌ヨ浣欓: {}", name);
            // TODO: 瀹炵幇浣欓鏌ヨ閫昏緫
        }
        Commands::Bridge { name, from_chain, to_chain, token, amount } => {
            println!(
                "馃寜 妗ユ帴杞处: {} 浠?{} 鍒?{} 浠ｅ竵: {} 閲戦: {}",
                name, from_chain, to_chain, token, amount
            );
            // TODO: 瀹炵幇妗ユ帴閫昏緫
        }
        Commands::List => {
            println!("馃搵 鍒楀嚭鎵€鏈夐挶鍖?);
            // TODO: 瀹炵幇鍒楀嚭閫昏緫
        }
        Commands::GenerateMnemonic => {
            // 鐢熸垚 24 瀛楀姪璁拌瘝锛堟ā鎷燂級
            let mnemonic =
                "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"; // 绀轰緥 12 瀛楋紝瀹為檯搴旂敓鎴?24 瀛?            println!("{}", mnemonic);
        }
    }

    Ok(())
}
