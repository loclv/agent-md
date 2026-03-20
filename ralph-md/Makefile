# Makefile for agent-md Rust project
# Provides convenient commands for building, testing, and linting

.PHONY: help build run test clean lint format check-format clippy audit watch

# Default target
help:
	@echo "Available commands:"
	@echo "  build        - Build the project"
	@echo "  run          - Build and run the project"
	@echo "  test         - Run tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  lint         - Run all linting checks (clippy + format check)"
	@echo "  format       - Format code with rustfmt"
	@echo "  check-format - Check if code is formatted"
	@echo "  clippy       - Run clippy lints"
	@echo "  audit        - Run security audit"
	@echo "  watch        - Watch for changes and rebuild"
	@echo "  ci           - Run CI pipeline (test + lint + audit)"

# Build the project
build:
	cargo build --release

# Build and run
run:
	cargo run --release

# Run tests
test:
	cargo test --verbose

# Clean build artifacts
clean:
	cargo clean

# Run all linting checks
lint: clippy check-format

# Format code with rustfmt
format:
	cargo fmt

# Check if code is formatted
check-format:
	cargo fmt --check

# Run clippy lints
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Run security audit
audit:
	cargo audit

# Watch for changes and rebuild
watch:
	cargo watch -x run

# Full CI pipeline
ci: test lint audit

# Development setup
setup:
	rustup component add rustfmt clippy
	cargo install cargo-watch cargo-audit
	@echo "Development environment setup complete!"

# Quick check for development (format + clippy warnings)
quick-check:
	cargo fmt
	cargo clippy --all-targets --all-features

# Build documentation
docs:
	cargo doc --open

# Check for outdated dependencies
outdated:
	cargo outdated
