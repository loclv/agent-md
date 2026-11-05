# Release Process

This document outlines the standard workflow for cutting a new release of `agent-md`.

## Pre-Release Checklist

1. Working directory is clean or contains only intended release changes.
2. All feature changes are covered by unit and integration tests.
3. Documentation in `README.md` and `docs/` is up to date.

## Step-by-Step Release Workflow

### 1. Update Version in `Cargo.toml`

Update the `version` field under `[package]` in `Cargo.toml`:

```toml
[package]
name = "agent-md"
version = "X.Y.Z"
```

### 2. Update `CHANGELOG.md`

1. Move unreleased entries under a new version heading with the release date: `## [X.Y.Z] - YYYY-MM-DD`.
2. Add a fresh, empty `## [Unreleased]` section directly above the new version heading.
3. Group entries under standard categories (`### Added`, `### Fixed`, `### Changed`, `### Removed`).
4. Ensure no bold markdown formatting is used; use italics or backticks instead.

### 3. Synchronize `Cargo.lock`

Update dependencies and lockfile:

```bash
cargo check
```

### 4. Build Release Binary

Build the optimized release binary:

```bash
cargo build --release
```

Verify that the compiled binary outputs the expected version:

```bash
./target/release/agent-md -v
```

### 5. Validate Code Quality and Tests

Run the full verification suite:

```bash
# Check code formatting
cargo fmt --check

# Run linter with warnings treated as errors
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests under release profile
cargo test --release
```

### 6. Lint Markdown Files

Validate modified markdown files using `agent-md`:

```bash
./target/release/agent-md lint CHANGELOG.md
```

### 7. Commit and Tag

Commit release changes and create an annotated git tag:

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md docs/
git commit -m "chore(release): vX.Y.Z"
git tag -a vX.Y.Z -m "Release version X.Y.Z"
```

### 8. Log Release Activity

Record the release action using the `l-log` CLI tool:

```bash
l-log add ./logs/chat.csv "Release version X.Y.Z" \
  --tags="release,version,changelog" \
  --problem="Release version X.Y.Z" \
  --solution="Bump version in Cargo.toml, update CHANGELOG.md, synchronize Cargo.lock, and verify test suite" \
  --action="Bumped version, updated changelog, built release binary, ran test suite" \
  --files="Cargo.toml,Cargo.lock,CHANGELOG.md" \
  --tech-stack="rust" \
  --created-by-agent="Antigravity"
```
