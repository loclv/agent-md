# sample overrides

## agent-md.json: overrides /docs and scripts dir with no blank lines around list

```json
{
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
