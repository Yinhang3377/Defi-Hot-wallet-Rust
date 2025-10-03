@echo off
echo 在 WSL 中运行测试覆盖率...
wsl bash -c "cd /mnt/c/Users/%USERNAME%/Desktop/Rust区块链/Defi-Hot-wallet-Rust && bash scripts/run_coverage.sh %*"
echo 完成！