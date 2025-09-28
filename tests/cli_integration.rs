use assert_cmd::Command;

#[test]
fn test_cli_create_wallet() {
    let mut cmd = Command::cargo_bin("wallet-cli").unwrap();
    // `create` 子命令需要一个 `name` 参数
    cmd.arg("create").arg("--name").arg("cli-integration-test-wallet").assert().success();
}
