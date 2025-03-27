# Kokoros Development Guidelines

## Build/Run/Test Commands
- Build: `cargo build --release`
- Run: `./target/release/koko text "Hello world!"` 
- Run single test: `cargo test --package kokoros test_name`
- Check format: `cargo fmt --check`
- Fix format: `cargo fmt`
- Lint check: `cargo clippy`
- Lint fix: `cargo clippy --fix`

## Code Style Guidelines
- **Imports**: Group imports by stdlib, third-party, and local modules
- **Formatting**: Follow rustfmt conventions
- **Error Handling**: Use Result<T, E> with thiserror for custom errors
- **Naming**:
  - snake_case for variables, functions, and modules
  - CamelCase for types and traits
  - ALL_CAPS for constants
- **Types**: Use strong typing and explicit type declarations
- **Documentation**: Comment public APIs with doc comments (///)
- **Testing**: Write unit tests for functionality with proper error messages
- **Safety**: Avoid unsafe code whenever possible