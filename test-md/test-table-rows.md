# Perfect Document

| Name | Description |
|---|---|
| Item      | This is an item  |

Should be:

| Name | Description |
|---|---|
| Item | This is an item |

Expected error:

- Table row should not have trailing spaces, trailing space number should be 1 or 0.

```bash
agent-md lint test-md/test-table-rows.md
```
