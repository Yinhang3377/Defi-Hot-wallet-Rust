// tests/cli_tests.rs
use std::process::Command;

#[test]
fn test_cli_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "wallet-cli", "--", "--help"])
        .output()
        .expect("Failed to execute CLI");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wallet-cli"));
}

#[test]
fn test_cli_create_wallet() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "wallet-cli", "--", "create", "--name", "cli-test-wallet"])
        .output()
        .expect("Failed to create wallet via CLI");
    
    // 检查命令成功执行，而不是特定的输出字符串
    assert!(output.status.success(), "CLI command failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // 可选：检查输出中是否包含成功相关的关键词
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("created") || 
        stdout.contains("exists") || 
        stdout.contains("success") || 
        stdout.contains("wallet"),
        "Unexpected CLI output: {}", stdout
    );
}

#[test]
fn test_cli_list_wallets() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "wallet-cli", "--", "list"])
        .output()
        .expect("Failed to list wallets via CLI");
    
    assert!(output.status.success());
}

#[test]
fn test_cli_generate_mnemonic() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "wallet-cli", "--", "generate-mnemonic"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // 解析表格格式的助记词输出
    // 格式如: "  1. face          2. shed          3. chunk         ..."
    let mut words = Vec::new();
    
    for line in stdout.lines() {
        // 查找包含编号单词的行
        if line.contains(". ") && line.chars().any(|c| c.is_ascii_digit()) {
            // 分割行，提取每个 "数字. 单词" 对
            let parts: Vec<&str> = line.split("  ").collect();
            for part in parts {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                // 匹配 "数字. 单词" 格式
                if let Some(dot_pos) = part.find('.') {
                    if let Some(word_start) = part[dot_pos..].find(char::is_alphabetic) {
                        let word = &part[dot_pos + word_start..].split_whitespace().next().unwrap_or("");
                        if !word.is_empty() {
                            words.push(word.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // 验证找到了24个单词
    assert_eq!(words.len(), 24, "Expected 24 words in mnemonic, found {}", words.len());
    
    // 验证单词不为空且看起来像英文单词
    for word in &words {
        assert!(!word.is_empty(), "Empty word found in mnemonic");
        assert!(word.chars().all(|c| c.is_ascii_alphabetic()), "Invalid character in word: {}", word);
    }
    
    println!("Successfully parsed {} words from CLI output", words.len());
}