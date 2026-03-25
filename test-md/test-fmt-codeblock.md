# Test Code Block Preservation

This text has **bold** that should be removed.

```
This code block has **bold** that should be preserved.
And __underlined__ text too.
```

More text with **bold** that should be removed.

```javascript
function test() {
    // Comment with **bold**
    let x = "**not bold**";
    return x;
}
```

Final text with **bold** removed.
