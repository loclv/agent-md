# Test Space Indentation Rule

This paragraph has no indentation.

  This paragraph has 2 spaces - should be valid.

    This paragraph has 4 spaces - should trigger warning.

      This paragraph has 6 spaces - should trigger warning.

1. Ordered list item with proper indentation.
    2. Nested ordered list item - should be valid.

- Unordered list item
  - Nested unordered list - should be valid.

```javascript
function example() {
    // Code blocks should be exempt
    return true;
}
```

# Heading should be exempt

    This is indented text in a regular paragraph - should trigger warning.
