---
name: release
description: Release process
tags: ["release"]
---

# Release

```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md
# 3. Run full test suite
cargo test

# 4. Build release
cargo build --release

# Or install to ~/bin
cargo build --release --target-dir ~/bin

# 5. Tag release
git tag -a v0.1.0 -m "Release version 0.1.0"
git push origin v0.1.0
```
