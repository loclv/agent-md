---
name: code-block-syntax-errors
description: Tests how the formatter handles malformed code blocks.
---

## Code Block Syntax Errors

This document tests how the formatter handles various syntax errors within code blocks.

### Missing Closing Fence

```javascript
function test() {
    console.log('This block is missing a closing fence');
// Missing ``` here

### Unclosed Code Block

```python
def hello():
    print("Hello, world!")

# This block is never closed

print("After the block")

### Code Block in List

- Here is a list:
  ```
  * item 1
  * item 2
  ```

### Incorrect Nesting

```
Outer block:
```js
Inner block should be here
```
```

### Mixed Content in Code Block

```
This is a code block.
* This should be code, not a list item.
```