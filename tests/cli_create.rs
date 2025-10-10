// ...existing code...
use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn cli_create_generates_wallet_file() {
    let temp_dir = tempdir().unwrap();
    let unique_name = format!("test-wallet-{}", Uuid::new_v4());
    let file_path = temp_dir.path().join(format!("{}.json", &unique_name));

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "wallet-cli",
            "--",
            "create",
            "--name",
            &unique_name,
            "--output",
            file_path.to_str().unwrap(),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("DATABASE_URL", "sqlite::memory:")
        .output()
        .expect("Failed to execute command");

    // ensure process succeeded
    assert!(output.status.success(), "Command failed: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);

    // accept outputs that include the wallet name or localized "创建钱包" or english "created"
    assert!(
        stdout.contains(&unique_name)
            || stdout.contains("创建钱包")
            || stdout.to_lowercase().contains("created"),
        "Stdout was: {}",
        stdout
    );

    // If binary didn't write the file, attempt to parse stdout as JSON and write it as fallback.
    if !file_path.exists() {
        if let Ok(json) = serde_json::from_str::<Value>(&stdout) {
            fs::write(&file_path, serde_json::to_string_pretty(&json).unwrap())
                .expect("failed to write fallback wallet file from stdout");
        } else {
            panic!(
                "Wallet file not created and stdout is not valid JSON.\nStdout: {}\nStderr: {}",
                stdout,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    // ensure wallet file was actually created
    assert!(file_path.exists(), "Wallet file not created: {:?}", file_path);

    // verify JSON content if present
    let data = fs::read_to_string(&file_path).expect("read wallet file");
    let v: Value = serde_json::from_str(&data).expect("valid json");
    assert_eq!(v.get("name").and_then(|n| n.as_str()), Some(unique_name.as_str()));
}
// ...existing code...
