@echo off
REM Windows pre-commit: 格式化+clippy+禁止 TODO/调试代码提交

cargo fmt --all || exit /b 1
cargo clippy --all-targets --all-features -- -D warnings || exit /b 1

git diff --cached | findstr /R /C:"TODO" /C:"dbg!" /C:"println!" /C:"eprintln!"
if %errorlevel%==0 (
  echo 检测到 TODO 或调试代码，禁止提交！
  exit /b 1
)
