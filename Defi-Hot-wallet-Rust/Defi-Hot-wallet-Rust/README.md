# Defi-Hot-wallet-Rust/README.md

# Defi Hot Wallet

This project is a Rust-based decentralized finance (DeFi) hot wallet application. It provides functionalities for managing cryptocurrency wallets, interacting with blockchain networks, and performing secure transactions.

## Project Structure

- **.github/workflows/ci.yml**: Defines the CI/CD pipeline for building, testing, and deploying the application.
- **patches/elliptic-curve-tools**: Contains patches for the `elliptic-curve-tools` crate to apply local modifications.
- **src/main.rs**: Entry point of the application, initializing and running the server or CLI.
- **src/lib.rs**: Library root that exports core modules and functionalities.
- **src/blockchain/ethereum.rs**: Implementation of the Ethereum client using the `ethers` crate for blockchain interactions.
- **tests/test_request_tests.rs**: Unit tests for request handling functionality using the `axum` framework.
- **Cargo.toml**: Configuration file specifying project metadata, dependencies, and features.

## Setup Instructions

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/Yinhang3377/Defi-Hot-wallet-Rust.git
   cd Defi-Hot-wallet-Rust
   ```

2. **Install Rust**:
   Ensure you have Rust installed. If not, you can install it using [rustup](https://rustup.rs/).

3. **Build the Project**:
   ```bash
   cargo build
   ```

4. **Run the Server**:
   ```bash
   cargo run --bin hot_wallet -- server
   ```

5. **Run the CLI**:
   ```bash
   cargo run --bin wallet-cli -- create --name my-wallet
   ```

## Usage Examples

- **Check API Health**:
   ```bash
   curl http://localhost:8080/api/health
   ```

- **Fetch Metrics**:
   ```bash
   curl http://localhost:8080/api/metrics
   ```

## Testing

To run the tests, use the following command:

```bash
cargo test
```

## Security

This project follows best practices for security, including zeroization of sensitive data and regular security audits. Ensure to keep dependencies up to date and monitor for vulnerabilities.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any enhancements or bug fixes.

## License

This project is licensed under the MIT License. See the LICENSE file for more details.