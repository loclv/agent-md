
# Test Document

```bash
agent-md lint test-md/test-graph.md
```

## ASCII Graph Example

```text
┌───┐
│ A │
└───┘

```

Expected output:

```text
Error: ASCII graph detected in code block (line 10)
```

```text
├── public/
│   ├── pagefind/ # auto-generated when build
│   ├── favicon.svg
│   └── astropaper-og.jpg
```

Expected output:

```text
Error: ASCII graph detected in code block (line 23)
```
