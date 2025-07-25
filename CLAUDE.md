# Solana Wallet Project - Development Guide

## Project Overview
This is a high-performance desktop Solana wallet application built with Rust, GPUI framework, and gpui-component UI library. The wallet provides secure key management, transaction capabilities, and DeFi integration.

## Technology Stack
- **Language**: Rust
- **UI Framework**: GPUI + gpui-component
- **Blockchain**: Solana (solana-sdk, solana-client)
- **Cryptography**: ed25519-dalek, aes-gcm, argon2
- **Storage**: SQLite (sqlx) + encrypted local storage
- **Async Runtime**: tokio
- **Serialization**: serde, bincode

## Project Structure
```
solana-wallet/
├── Cargo.toml              # Project dependencies
├── src/
│   ├── main.rs            # Application entry point
│   ├── app/               # Application state and events
│   │   ├── mod.rs
│   │   ├── state.rs       # Global app state
│   │   └── events.rs      # Event definitions
│   ├── ui/                # UI components and views
│   │   ├── mod.rs
│   │   ├── components/    # Reusable UI components
│   │   ├── views/         # Main application views
│   │   └── theme.rs       # Theme configuration
│   ├── wallet/            # Core wallet functionality
│   │   ├── mod.rs
│   │   ├── account.rs     # Account management
│   │   ├── keypair.rs     # Key pair operations
│   │   └── transaction.rs # Transaction building
│   ├── security/          # Security and encryption
│   │   ├── mod.rs
│   │   ├── encryption.rs  # Encryption utilities
│   │   └── vault.rs       # Key vault management
│   ├── rpc/               # Solana RPC communication
│   │   ├── mod.rs
│   │   └── client.rs      # RPC client implementation
│   └── storage/           # Data persistence
│       ├── mod.rs
│       └── database.rs    # Database operations
├── assets/                # Static assets
│   ├── icons/            # Application icons
│   └── fonts/            # Custom fonts
└── tests/                # Test files
```

## Key Development Commands

### Build and Run
```bash
# Build the project
cargo build

# Build in release mode (optimized)
cargo build --release

# Run the application
cargo run

# Run with debug logging
RUST_LOG=debug cargo run
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test module
cargo test wallet::tests

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'
```

### Code Quality
```bash
# Format code
cargo fmt

# Check formatting without applying
cargo fmt -- --check

# Run linter
cargo clippy

# Run linter with all targets
cargo clippy --all-targets --all-features

# Check for security vulnerabilities
cargo audit
```

### Documentation
```bash
# Generate and open documentation
cargo doc --open

# Generate docs with private items
cargo doc --document-private-items
```

## Development Guidelines

### Code Style
1. Follow Rust naming conventions (snake_case for functions, CamelCase for types)
2. Use `rustfmt` for consistent formatting
3. Address all `clippy` warnings
4. Document public APIs with doc comments
5. Use descriptive variable names

### Security Practices
1. Never log private keys or sensitive data
2. Use `zeroize` for clearing sensitive memory
3. Validate all user inputs
4. Use constant-time comparisons for cryptographic operations
5. Store encrypted data only

### Error Handling
1. Use `anyhow::Result` for application errors
2. Create custom error types for domain-specific errors
3. Provide meaningful error messages
4. Log errors appropriately

### Performance Guidelines
1. Use `Arc` and `Rc` judiciously
2. Prefer iterators over collecting into vectors
3. Use async/await for I/O operations
4. Implement lazy loading for large datasets
5. Profile before optimizing

## Architecture Decisions

### State Management
- Use GPUI's Model system for global state
- Implement event-driven updates
- Keep UI state separate from domain logic

### Data Flow
1. User interaction → UI Event
2. UI Event → Controller/Handler
3. Controller → Domain Logic
4. Domain Logic → Infrastructure (RPC/Storage)
5. Result → State Update → UI Refresh

### Security Architecture
- Master password derives encryption key
- Private keys encrypted with AES-256-GCM
- Hardware wallet support through abstraction
- Automatic session locking

## Common Tasks

### Adding a New View
1. Create view file in `src/ui/views/`
2. Implement `View` trait
3. Add to main window navigation
4. Update route handling

### Adding RPC Method
1. Add method to `RpcClient` in `src/rpc/client.rs`
2. Implement error handling
3. Add retry logic if needed
4. Update relevant domain services

### Creating New Component
1. Check gpui-component library first
2. Create in `src/ui/components/`
3. Follow GPUI component patterns
4. Add examples in component file

## Dependencies Update
```bash
# Update dependencies
cargo update

# Check outdated dependencies
cargo outdated

# Update specific dependency
cargo update -p solana-sdk
```

## Debugging

### Enable Debug Logging
```bash
RUST_LOG=solana_wallet=debug cargo run
```

### Common Issues
1. **GPUI build errors**: Ensure you have latest Rust nightly
2. **Solana RPC timeout**: Check network and RPC endpoint
3. **Database locked**: Ensure only one instance running

## Release Process

### Pre-release Checklist
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Security audit passed
- [ ] Documentation updated
- [ ] Version bumped in Cargo.toml

### Build Release
```bash
# Build optimized binary
cargo build --release

# Run release tests
cargo test --release

# Package for distribution
# macOS: Create .app bundle
# Windows: Create installer
# Linux: Create AppImage
```

## Environment Variables
```bash
# RPC endpoint (default: mainnet-beta)
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com

# Enable debug mode
DEBUG=true

# Custom database path
DATABASE_PATH=/path/to/wallet.db

# Auto-lock timeout (seconds)
AUTO_LOCK_TIMEOUT=300
```

## Important Notes

### GPUI Specifics
- GPUI requires macOS 11+ or equivalent
- Uses GPU acceleration for rendering
- Single-threaded UI updates
- Event propagation follows tree structure

### Solana Integration
- Always simulate transactions before sending
- Handle rate limiting from RPC nodes
- Cache account data appropriately
- Subscribe to account changes via WebSocket

### Performance Tips
- Use virtualized lists for large datasets
- Implement pagination for transaction history
- Cache token metadata
- Batch RPC requests when possible

## Troubleshooting

### Build Issues
```bash
# Clean build artifacts
cargo clean

# Update Rust toolchain
rustup update

# Check Rust version
rustc --version
```

### Runtime Issues
- Check logs in `~/.solana-wallet/logs/`
- Verify RPC endpoint connectivity
- Ensure sufficient system resources
- Check file permissions for storage

## Contributing Guidelines
1. Fork the repository
2. Create feature branch
3. Write tests for new features
4. Ensure all tests pass
5. Submit pull request

## Resources
- [GPUI Documentation](https://github.com/zed-industries/zed)
- [gpui-component Examples](https://github.com/longbridge/gpui-component)
- [Solana Documentation](https://docs.solana.com)
- [Rust Book](https://doc.rust-lang.org/book/)

## Contact
For questions or issues, please open an issue on GitHub or contact the development team.