# 本地 Git 钩子说明（.husky）

本项目已集成常用开发者本地钩子，位于 `.husky/` 目录：

- `pre-commit`：提交前自动执行
  - 自动格式化（cargo fmt）
  - 静态分析（cargo clippy）
  - 禁止含 TODO/调试代码（如 dbg!/println!）提交
- `pre-push`：推送前自动执行
  - 运行全部测试（cargo test）
  - 依赖安全审计（cargo audit）

## 使用方法
1. 确保已安装 Rust 工具链、cargo-audit。
2. 赋予钩子可执行权限（首次 clone 后）：
   ```sh
   chmod +x .husky/*
   ```
3. 提交/推送时自动触发，无需手动执行。

如需自定义钩子内容，可直接编辑 `.husky/pre-commit`、`.husky/pre-push`。
