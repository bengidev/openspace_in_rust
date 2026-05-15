# Contributing to OpenSpace

Thank you for your interest in contributing to OpenSpace! This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/openspace_in_rust.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Submit a pull request

## Development Setup

### Requirements

- Rust 1.75+ (latest stable recommended)
- Cargo

### Building

```bash
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check without building
cargo check
```

## Coding Standards

- Follow Rust naming conventions and idioms
- Write clear, concise commit messages
- Add tests for new functionality
- Document public APIs with rustdoc comments
- Keep the core engine modular and testable

## Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Reference issues and pull requests where appropriate

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Ensure your branch is up to date with `main`
4. Fill out the pull request template
5. Request review from maintainers

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Questions?

Open an issue or start a discussion in the repository.
