# Example

Input:

```text
data/
│
├── input/     # input
│
├── output/    # output
│
└── logs/      # logs
```

Expect:

```text
data/
├─input/ # input
├─output/ # output
└─logs/ # logs
```

Check folder structure syntax:

- If it is there is a line use '└─' instead of '├─' at first position.
- code block language must be 'text', 'txt' or empty..

Formatted changes:

- removed redundant '─'.
- removed spaces before '# comment' for each file/folder.
