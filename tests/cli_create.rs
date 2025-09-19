use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::tempdir;
use std::fs;
use serde_json::Value;
use std::path::PathBuf;
use base64::Engine; // bring encode into scope

// Helper to build a valid 64-hex encryption key (32 bytes)
fn sample_key() -> String {
    // Deterministic 32 distinct bytes: 00..1f -> 64 hex characters; avoids weak key heuristics.
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string()
}

#[test]
fn cli_create_generates_wallet_file() {
    let dir = tempdir().expect("temp dir");
    let wallet_path: PathBuf = dir.path().join("wallet.json");

    // Prepare command
    let mut cmd = Command::cargo_bin("hot_wallet").expect("binary exists");
    cmd.arg("create")
        .arg("--output")
        .arg(&wallet_path)
        .env("ENCRYPTION_KEY", sample_key())
        .env("NETWORK", "testnet")
        .env("SALT", base64::engine::general_purpose::STANDARD.encode("testsalt"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().expect("spawn hot_wallet binary");
    {
        use std::io::Write as _;
        let stdin = child.stdin.as_mut().expect("stdin available");
        // Provide a valid 64-hex encryption key password (same as ENCRYPTION_KEY)
        let pass = format!("{}\n", sample_key());
        stdin.write_all(pass.as_bytes()).expect("write password");
    }
    let output = child.wait_with_output().expect("wait output");
    assert!(
        output.status.success(),
        "process failed: status={:?} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout_str.contains("钱包已成功创建") || stdout_str.contains("成功创建"),
        "stdout was: {}",
        stdout_str
    );

    // Verify file created
    assert!(wallet_path.exists(), "wallet file should exist");
    let data = fs::read_to_string(&wallet_path).expect("read wallet file");
    let v: Value = serde_json::from_str(&data).expect("valid json");

    // Fields
    assert!(v.get("public_key").is_some(), "public_key field");
    assert!(v.get("encrypted_private_key").is_some(), "encrypted_private_key field");
    assert_eq!(
        v.get("network").and_then(|n| n.as_str()),
        Some("testnet")
    );
    assert!(v.get("aad").is_some(), "aad field present");

    // encrypted_private_key should be hex and length > 12 (nonce) *2 due to hex
    if let Some(enc) = v.get("encrypted_private_key").and_then(|e| e.as_str()) {
        assert!(enc.len() > 24, "encrypted_private_key length seems too small");
        assert!(
            enc.chars().all(|c| c.is_ascii_hexdigit()),
            "encrypted_private_key should be hex"
        );
    }
}
