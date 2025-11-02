# Contributing to NEXUS

Thank you for your interest in contributing to NEXUS! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork locally
3. Create a feature branch from `main`
4. Make your changes
5. Test your changes
6. Open a pull request

## Development Setup

### Prerequisites

- Rust 1.75+ ([Install Rust](https://rustup.rs/))
- Git

### Local Development

```bash
# Clone the repository
git clone https://github.com/your-username/nexus.git
cd nexus

# Run tests
cargo test

# Run CLI
cargo run -p nexus-cli -- version --verbose

# Check formatting
cargo fmt --check

# Run lints
cargo clippy -- -D warnings
```

## Code Style

- Follow standard Rust formatting (enforced by `cargo fmt`)
- Use `cargo clippy` to catch common issues
- Add tests for new functionality
- Document public APIs with doc comments

## Pull Request Process

1. **Create Feature Branch**: Use descriptive names like `feat/agent-system` or `fix/cli-parsing`
2. **Small PRs**: Keep changes focused and reviewable
3. **Tests**: Add tests for new functionality
4. **Documentation**: Update docs if needed
5. **CI**: Ensure all checks pass
6. **Review**: Request review from maintainers

## Commit Message Format

```
type(scope): brief description

- Detailed explanation of changes
- Why the change was needed
- Any breaking changes

Fixes: #issue-number
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and inclusive.

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).