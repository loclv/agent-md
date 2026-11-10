# Release Guide

This guide describes how to release a new `agent-md` version and update the `loclv/tools` Homebrew tap. The `0.5.3` release used this process.

## Prepare the release

1. Check that the working tree is clean and that the changes to release are committed:

   ```bash
   git status -sb
   git log -5 --oneline
   ```

2. Choose the next version from the changes since the previous release. For example, the configuration fixes after `0.5.2` became `0.5.3`.
3. Update the package version in `Cargo.toml`.
4. Move the entries under `Unreleased` in `CHANGELOG.md` into a dated section for the new version. Leave a fresh `Unreleased` section above it.
5. Synchronize `Cargo.lock`, build the release binary, and verify its version:

   ```bash
   cargo check
   cargo build --release
   ./target/release/agent-md -v
   ```

## Validate the release

Run the release checks and lint the modified Markdown:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --release
./target/release/agent-md lint CHANGELOG.md
```

Review any lint warnings. Record the work in `logs/chat.csv` with `l-log` as required by the project instructions.

## Publish the GitHub release

Replace `0.5.3` with the version being released. Review the staged files before committing.

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git diff --cached
git commit -m 'chore(release): v0.5.3'
git tag v0.5.3
git push o main v0.5.3
gh release create v0.5.3 --title v0.5.3 --notes 'Summarize the changes in this release.'
```

Use the appropriate remote and branch if they differ from `o` and `main`. Confirm the release page on GitHub after publication.

## Update the Homebrew tap

1. Download the tagged source archive and calculate its SHA-256 checksum:

   ```bash
   curl -fsSL -o /tmp/agent-md-v0.5.3.tar.gz https://github.com/loclv/agent-md/archive/refs/tags/v0.5.3.tar.gz
   shasum -a 256 /tmp/agent-md-v0.5.3.tar.gz
   ```

2. In the `loclv/homebrew-tools` repository, update `Formula/agent-md.rb`: set the archive URL to the new tag and replace `sha256` with the checksum just calculated. Keep the existing build and test instructions unless they need to change.
3. Commit and push the formula update to the tap. For `0.5.3`, the GitHub contents API was used to update the formula directly; its current file SHA was supplied to avoid overwriting a concurrent change.
4. Verify the published formula points to the correct tag and checksum. Where possible, install or upgrade from the tap and run `brew test agent-md`.

Users can update an existing installation with:

```bash
brew update
brew upgrade agent-md
```
