# Configuration

Reference for every option usable in `agent-md.json` configuration files.

## Configuration files

`agent-md` resolves configuration in the following priority order:

1. `.agent-md.json`
2. `agent-md.json`
3. `markdownlintrc.*` fallback (`.markdownlint.json`, `.markdownlint.jsonc`, `.markdownlint.yaml`, `.markdownlint.yml`, `.markdownlintrc`, `.markdownlintrc.json`)

For a target Markdown file, discovery starts in the target's parent directory and walks up through ancestors, then falls back to the current working directory. The global `--config <PATH>` flag overrides discovery with an explicit file or directory. An explicit file path is always respected, even when it is a `markdownlintrc.*` file.

Supported formats depend on the file extension: `.json` is strict JSON, `.jsonc` allows `//` and `/* */` comments, `.yaml` and `.yml` are parsed as YAML mappings, and extensionless `.markdownlintrc` is tried as JSON, then JSONC, then YAML.

Unknown keys are ignored. Keys with an invalid type fall back to their default value.

```bash
agent-md init
agent-md config
agent-md config path/to/document.md
agent-md --config samples/agent-md.json lint document.md
```

## Linter options

| Option | Type | Default | Description |
|---|---|---|---|
| `blanks-around-headings` | bool | `true` | Require blank lines around headings |
| `blanks-around-lists` | bool | `true` | Require blank lines around lists |
| `blanks-around-fences` | bool | `true` | Require blank lines around fenced code blocks |
| `blanks-around-tables` | bool | `false` | Reserved for a future tables rule |
| `first-line-heading` | bool | `true` | Require the first line to be a top-level heading |
| `no-duplicate-heading` | bool | `true` | Alias of `no-duplicate-headings` below |
| `no-duplicate-headings` | bool | `true` | Forbid duplicate heading text |
| `line-length` | bool | `false` | Enable the line length rule |
| `max-line-length` | u64 | `0` | Maximum line length, `0` disables the check |
| `ol-prefix` | bool | `false` | Reserved for a future ordered list rule |
| `table-column-style` | bool | `false` | Reserved for a future table style rule |
| `no-hard-tabs` | bool | `true` | Forbid hard tab characters outside code blocks |
| `no-inline-html` | bool | `false` | Reserved for a future inline HTML rule |

Setting `line-length` to `true` only takes effect when `max-line-length` is greater than `0`.

```json
{
	"blanks-around-headings": true,
	"first-line-heading": false,
	"no-duplicate-headings": true,
	"line-length": true,
	"max-line-length": 80,
	"no-hard-tabs": true
}
```

## Discovery option

| Option | Type | Default | Description |
|---|---|---|---|
| `ignore-markdownlintrc` | bool | `false` | Skip `markdownlintrc.*` files during discovery |

The snake_case alias `ignore_markdownlintrc` is also accepted. The global `--ignore-markdownlintrc` CLI flag always wins over the file key. Only native `agent-md.json` files are consulted for this key; a `markdownlintrc.*` file can never set it.

```json
{
	"ignore-markdownlintrc": true
}
```

## Formatter options

Formatter options accept kebab-case or snake_case at the top level, or nested under a `format` object. CLI flags such as `--remove-bold=false` override the configured value per option.

| Option | Type | Default | Description |
|---|---|---|---|
| `remove-bold` | bool | `true` | Strip `**bold**` and `__bold__` markers |
| `compact-blank-lines` | bool | `true` | Collapse multiple consecutive blank lines |
| `collapse-spaces` | bool | `true` | Collapse multiple spaces between words |
| `remove-horizontal-rules` | bool | `true` | Remove `---`, `***`, and `___` lines |
| `remove-emphasis` | bool | `true` | Strip `*italic*` and `_italic_` markers |
| `minify-html` | bool | `true` | Collapse each HTML block onto a single line and remove useless whitespace; `pre`, `code`, `textarea`, `script`, and `style` keep line breaks |

```json
{
	"remove-bold": false,
	"compact-blank-lines": true,
	"format": {
		"remove_bold": false,
		"minify-html": false
	}
}
```

When both spellings are present, the nested `format` object wins, then kebab-case, then snake_case. The sample template created by `agent-md init` lists every option with its default value.

## Overrides

Use `overrides` to apply different rule values to matching files. Each entry lists `includes` glob patterns and a `rules` object with the same keys as the top-level config. Entries apply in order and later entries win. An optional `excludes` list skips files from that entry.

```json
{
  "blanks-around-lists": true,
  "overrides": [
    {
      "includes": [
        "scripts/*",
        "docs/*"
      ],
      "rules": {
        "blanks-around-lists": false
      }
    }
  ]
}
```

Matching details:

- Patterns match the target file path, its path relative to the config file directory, and its path relative to the current directory.
- `*` spans directories, so `scripts/*` matches `scripts/nested/tool.md`.
- A bare directory name matches everything underneath it.
- An empty `includes` list matches all files.
- `lint` resolves overrides per target file; `fmt` re-resolves per file when formatting a directory, keeping CLI flags such as `--remove-bold=false` intact.
