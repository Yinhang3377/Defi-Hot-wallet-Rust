use assert_cmd::Command;

#[test]
fn test_main_runs() {
    let mut cmd = Command::cargo_bin("hot_wallet").unwrap();
    cmd.arg("--help").assert().success(); // 测试帮助输出
}
