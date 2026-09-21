# List available recipes
default:
    @just --list

# Show available commands
help:
    @just --list

# Build release binary
build:
    cargo build --release

# Build debug binary
build-dev:
    cargo build

# Build and run release binary
run *args:
    cargo run --release -- {{ args }}

# Run tests
test *args:
    cargo test --verbose {{ args }}

# Package crate for publishing / distribution
package:
    cargo package --allow-dirty

# Alias for package
bundle: package

# Clean build artifacts
clean:
    cargo clean

# Run all linting checks (clippy + format check)
lint: clippy check-format

# Format code with rustfmt
format:
    cargo fmt

# Alias for format
fmt:
    cargo fmt

# Check if code is formatted
check-format:
    cargo fmt --check

# Run clippy lints denying warnings
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run security audit
audit:
    cargo audit

# Watch for changes and rebuild
watch:
    cargo watch -x run

# Full CI pipeline (test + lint + audit)
ci: test lint audit

# Development environment setup
setup:
    rustup component add rustfmt clippy
    cargo install cargo-watch cargo-audit
    @echo "Development environment setup complete!"

# Quick check for development (format + clippy)
quick-check:
    cargo fmt
    cargo clippy --all-targets --all-features

# Build and open documentation
docs:
    cargo doc --open
