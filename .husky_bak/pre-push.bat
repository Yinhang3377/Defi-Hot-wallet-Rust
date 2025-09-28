@echo off
REM Windows pre-push: 测试+安全审计

cargo test --all || exit /b 1
cargo audit || exit /b 1
