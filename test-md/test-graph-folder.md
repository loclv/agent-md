
# Test Document

```bash
agent-md lint test-md/test-graph-folder.md
```

## ASCII Graph Example

```text
├── public/
│   ├── pagefind/ # auto-generated when build
│   ├── favicon.svg
│   └── astropaper-og.jpg
```

Expected output:

```text
Error: ASCII graph detected in code block (line 11)
```
