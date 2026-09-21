# Packages

`agent-md` has been structured and configured as a dual library and binary package that can be consumed both as a Rust library dependency and as a CLI binary tool.

## Changes Made

1. Library Crate Root ([`src/lib.rs`](./src/lib.rs)):
   - Created the root library crate exporting public modules: `commands`, `config`, `format`, `linter`, `parser`, `rules`, `types`, and `cli`.
   - Re-exported core programmatic APIs at crate root for consumers:
     ```rust
     use agent_md::{format_markdown, format_markdown_with_options, parse_markdown, validate_markdown};
     ```

2. CLI Module Extraction ([`src/cli.rs`](./src/cli.rs)) & Main Entry ([`src/main.rs`](./src/main.rs)):
   - Extracted [`Cli`](./src/cli.rs#L8-L27), [`Commands`](./src/cli.rs#L30-L173), and [`get_format_options`](./src/cli.rs#L176-L198) into `cli.rs` so they can be reused programmatically or by tests.
   - Refactored `src/main.rs` to import from `agent_md` as an external consumer of the library.

3. Crate & Packaging Metadata ([`Cargo.toml`](./Cargo.toml)):
   - Configured `[lib]` target (`name = "agent_md"`, `path = "src/lib.rs"`) alongside `[[bin]]`.
   - Added standard crates.io metadata: `description`, `license = "MIT"`, `repository`, `homepage`, `documentation`, `keywords`, and `categories`.
   - Added `exclude` rules to keep the published crate archive compact (reduced from ~4.8 MiB to ~105 KiB).

4. Packaging & Distribution:
   - Build and verify distribution archives with `cargo package --allow-dirty`.

## How Others Can Use It

### As a Rust Library Dependency

Add to `Cargo.toml`:

```toml
[dependencies]
agent-md = "0.2.14"
```

In Rust code:

```rust
use agent_md::{format_markdown, parse_markdown, validate_markdown};

fn main() {
    let raw = "# Hello\n\n**bold** and <p align=\"center\">\n  <img src=\"badge.png\" />\n</p>";

    // Format according to AI token-saving standards
    let formatted = format_markdown(raw);

    // Validate with AI lint rules
    let lint = validate_markdown(&formatted);
    println!("Valid: {}, errors: {:?}", lint.valid, lint.errors);
}
```

#### As a CLI Tool

```bash
cargo install agent-md
```

Or packaged via `cargo package --allow-dirty` for local distribution.
