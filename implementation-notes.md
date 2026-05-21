# Implementation Notes - agent-md

## Rule 8: No Duplicate Headings (MD024)

### Decisions
- Added to `src/rules/heading_structure.rs` to reuse heading extraction logic.
- Implemented as a `LintWarning` by default (Severity: Warning in docs).
- Checks all headings in the document, regardless of level.
- Case-sensitive check for now, matching the most common `markdownlint` implementation.

### Tradeoffs
- Storing all heading texts in a `HashSet` might consume slightly more memory for very large documents, but for typical markdown it is negligible.

## Rule 17: No Multiple Blank Lines (MD012)

### Decisions
- Added to `src/rules/whitespace.rs`.
- Warns when there are more than 2 consecutive newlines (i.e., more than one blank line).
- This aligns with the `compact_blank_lines` option in the formatter.

### Tradeoffs
- Some people like extra spacing, but the "ideal" for LLMs is token efficiency, so multiple blank lines are discouraged.
