## Copilot instructions for DeFi Hot Wallet (Rust)

This file tells AI coding agents how to be productive in this repository. Be concrete, follow existing code patterns, and reference the files and commands below when making changes.

### Big picture (quick)
- Layered Rust application: API/CLI -> Core services -> Security/crypto modules -> Blockchain clients -> Storage. See `src/lib.rs` for module exports and `src/main.rs` / `src/cli.rs` for entry points.
- The main orchestration component is the WalletManager in `src/core` (use `WalletManager::new(&WalletConfig).await?` patterns).
- Ethereum integration uses the `ethers` crate (abigen + rustls); blockchain-specific code lives under `src/blockchain`.

### Where to look for patterns and examples
- Entry points: `src/main.rs` (binary `hot_wallet`) and `src/cli.rs` (binary `wallet-cli`).
- Core logic: `src/core/` (wallet lifecycle, WalletManager, config types).
- Crypto primitives: `src/crypto/` (Shamir, zeroize, quantum-safe stubs).
- Storage: `src/storage/` and SQLx usage in code/config. Default DB env var is read in `src/main.rs` (look for `DATABASE_URL` fallback `sqlite://./wallets.db`).
- Monitoring & metrics: `src/monitoring/` and HTTP metrics endpoint exposed in server (`/api/metrics`).

### Coding conventions to follow
- Async-first: use `tokio` runtime, async functions, and `.await` for IO and crypto where present.
- Error handling: return `anyhow::Result` in top-level functions and use crate-local error types (see `thiserror` usage in `src/core/errors.rs`).
- Secrets: follow existing zeroization pattern — use the `zeroize` crate and prefer dropping/zeroing secrets after use.
- Logging: use `tracing` and `tracing_subscriber`; call `init_logging()` as in `src/main.rs` for consistency.
- Feature gates: check `Cargo.toml` features (`strict_security`, `sop_patch_tests`, etc.) and respect conditional compilation.

### Tests and CI patterns
- Unit/integration tests live under `tests/` and use `tokio::test(flavor = "current_thread")` frequently.
- Use `serial_test` for tests that must not run concurrently with DB/file state.
- Common commands:
  - Build: `cargo build` (or `cargo build --release`)
  - Test: `cargo test` (run specific modules with `cargo test crypto::tests`)
  - Format: `cargo fmt`
  - Lint: `cargo clippy -- -D warnings`
  - Coverage (dev tooling): `cargo tarpaulin --out Html` (see README)
  - Security audit: `cargo audit`

### How to run the server / CLI (examples)
- Run the server (reads env var `DATABASE_URL` or falls back to sqlite):
  - `cargo run --bin hot_wallet -- server` or build and run release binary `./target/release/hot_wallet server --port 8080`
- CLI example (create wallet):
  - `cargo run --bin wallet-cli -- create --name my-wallet`
- API examples (the server exposes endpoints; see `src/api`):
  - `curl http://localhost:8080/api/health`
  - `curl http://localhost:8080/api/metrics`

### Integration points / external dependencies to be aware of
- Ethereum RPC: environment variables in docs (`WALLET_ETHEREUM_RPC_URL`) — but code reads `DATABASE_URL` and WalletConfig for DB; search `WALLET_` env usage when changing RPC wiring.
- Local patches: `patches/elliptic-curve-tools` is used in `Cargo.toml` via `[patch.crates-io]` — prefer using the patched local crate shape.
- Prometheus: the server exposes metrics; follow `src/monitoring` conventions when adding metrics (use `prometheus` crate and named counters like `wallets_created_total`).

### Small gotchas / repo-specific notes
- Config vs env: `src/main.rs` reads `DATABASE_URL` (sqlite url string like `sqlite://./wallets.db`). README sometimes refers to `WALLET_DATABASE_URL` — prefer following code (`DATABASE_URL`) or update code and docs together.
- Tests often mock or simulate heavy crypto/chain interactions. When adding tests, prefer small, fast unit tests and use `httpmock` / `axum-test` or the provided mock helpers rather than hitting live RPC endpoints.
- Windows build: `Cargo.toml` includes windows-specific dependencies (`windows`, `winapi`) under target-specific sections; respect cross-platform guards.

### Minimal PR checklist for AI-generated changes
1. Run `cargo fmt` and `cargo clippy -- -D warnings` locally.
2. Run targeted tests: `cargo test <module_or_testname>`; prefer not to change global test ordering.
3. If adding features that touch secrets/crypto, ensure zeroization and no plaintext secrets in logs.
4. Update README or `docs/` if you change public CLI flags or API endpoints.

If anything in this file is unclear or you need examples for a specific area (DB layer, WalletManager flows, or signing logic), tell me which part and I'll expand with concrete, line-referenced examples.
