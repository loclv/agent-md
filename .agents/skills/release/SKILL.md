---
name: release
description: Step-by-step procedure for preparing, validating, and cutting a new release of agent-md, including version bumping, changelog maintenance, testing, and logging.
---

# Release Workflow

This skill documents the step-by-step process for cutting a new release of `agent-md`.

## Pre-Release Checklist

Before starting a release, ensure:

1. Working directory is clean or only contains changes intended for the release.
2. All feature changes are already covered by tests.
3. Documentation in `README.md` and `docs/` is up to date.

## Release Steps

### 1. Update Version in `Cargo.toml`

Update the `version` field under `[package]` in `Cargo.toml`:

```toml
[package]
name = "agent-md"
version = "X.Y.Z"
```

### 2. Update `CHANGELOG.md`

1. Move items under the Unreleased heading into a new version section with the release date.
2. Add a fresh, empty Unreleased section directly above the new version heading.
3. Group entries under standard categories (`### Added`, `### Fixed`, `### Changed`, `### Removed`).
4. Ensure all Markdown rules are satisfied (no double asterisks or double underscores; use italics or backticks).

### 3. Synchronize `Cargo.lock`

Run `cargo check` to update `Cargo.lock` with the new version:

```bash
rtk cargo check
```

### 4. Build Release Binary

Build the optimized binary:

```bash
rtk cargo build --release
```

Verify that the compiled binary outputs the expected version:

```bash
rtk ./target/release/agent-md -v
```

### 5. Validate Code Quality and Tests

Run the full validation suite:

```bash
# Check code formatting
rtk cargo fmt --check

# Run linter with warnings treated as errors
rtk cargo clippy --all-targets --all-features -- -D warnings

# Run all unit and integration tests under release profile
rtk cargo test --release
```

### 6. Lint Markdown Files

Validate modified Markdown files using the `agent-md` CLI:

```bash
rtk ./target/release/agent-md lint CHANGELOG.md
```

### 7. Log Release Activity

Record the release action in `logs/chat.csv` using the `l-log` CLI tool:

```bash
rtk l-log add ./logs/chat.csv "Release version X.Y.Z" \
  --tags="release,version,changelog" \
  --problem="Release version X.Y.Z" \
  --solution="Bump version in Cargo.toml, update CHANGELOG.md, synchronize Cargo.lock, and verify test suite" \
  --action="Bumped version, updated changelog, built release binary, ran test suite" \
  --files="Cargo.toml,Cargo.lock,CHANGELOG.md" \
  --tech-stack="rust" \
  --created-by-agent="Antigravity"
```
