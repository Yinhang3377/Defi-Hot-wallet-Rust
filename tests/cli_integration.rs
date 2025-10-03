use assert_cmd::Command;

#[test]
fn test_cli_create_wallet() {
    let mut cmd = Command::cargo_bin("wallet-cli").unwrap();
    // `create` 瀛愬懡浠ら渶瑕佷竴涓?`name` 鍙傛暟
    cmd.arg("create").arg("--name").arg("cli-integration-test-wallet").assert().success();
}
