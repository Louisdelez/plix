# Contributing to Plix

Thank you for your interest in contributing to Plix! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

1. **Fork the repository** and clone your fork
2. **Set up the development environment**:
   ```bash
   # Install Rust 1.83+ (stable)
   rustup update stable

   # Build the project
   cargo build

   # Run tests
   cargo test --all
   ```
3. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Workflow

### Before Making Changes

1. Check existing [issues](https://github.com/your-org/plix/issues) and [pull requests](https://github.com/your-org/plix/pulls)
2. For significant changes, open an issue first to discuss the approach
3. Ensure you understand the relevant parts of the codebase

### Making Changes

1. Write clear, focused commits
2. Follow the coding style (see below)
3. Add tests for new functionality
4. Update documentation as needed

### Commit Message Format

Use conventional commits format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, no code change
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(server): add migration support for v1.0

Implements automatic migration from v0.x to v1.0 configuration format.
Creates backups before migration and logs progress.

Closes #123
```

```
fix(client): resolve crash on disconnect

The client would panic when the server closed the connection
during the handshake phase. Now handles this gracefully.
```

## Code Style

### Rust

- Follow standard Rust conventions (`rustfmt` defaults)
- Run `cargo fmt --all` before committing
- Run `cargo clippy --all-targets` and address warnings
- Use meaningful variable and function names
- Document public APIs with doc comments

### Formatting Check

```bash
# Format all code
cargo fmt --all

# Check formatting (CI uses this)
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets -- -D warnings
```

### Testing

```bash
# Run all tests
cargo test --all

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

## Pull Request Process

### Before Submitting

1. Ensure all tests pass: `cargo test --all`
2. Ensure code is formatted: `cargo fmt --all -- --check`
3. Ensure clippy passes: `cargo clippy --all-targets -- -D warnings`
4. Update CHANGELOG.md if applicable
5. Update documentation if needed

### PR Requirements

- Clear title describing the change
- Description of what and why
- Link to related issue(s)
- Tests for new functionality
- Documentation updates if applicable

### PR Template

When creating a PR, include:

```markdown
## Summary
Brief description of changes.

## Changes
- List of specific changes

## Testing
How was this tested?

## Related Issues
Fixes #123
```

### Review Process

1. Maintainers will review PRs within a few days
2. Address feedback promptly
3. Keep PRs focused and reasonably sized
4. Large changes may be split into smaller PRs

## Reporting Issues

### Bug Reports

Include:
- Plix version (`plix-client --version`)
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs or error messages

### Feature Requests

Include:
- Clear description of the feature
- Use case / motivation
- Proposed implementation (if any)
- Willingness to help implement

## Development Setup

### Required Tools

- Rust 1.83+ (stable toolchain)
- Git
- (Optional) Docker for container testing

### Project Structure

```
plix/
  crates/           # Rust crates
    plix-common/    # Shared types and utilities
    plix-server/    # Game server
    plix-client/    # Game client
    plix-arena/     # Arena loading
    plix-tools/     # Development tools
    plix-mod-*/     # Mod API crates
  assets/           # Game assets
  docs/             # Documentation
  scripts/          # Build and utility scripts
  specs/            # Feature specifications
```

### Running Locally

```bash
# Start server
cargo run --release -p plix-server -- --port 7777

# Start client (separate terminal)
cargo run --release -p plix-client -- --server 127.0.0.1:7777
```

## Getting Help

- [GitHub Discussions](https://github.com/your-org/plix/discussions) for questions
- [GitHub Issues](https://github.com/your-org/plix/issues) for bugs and features
- See [SUPPORT.md](SUPPORT.md) for more options

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
