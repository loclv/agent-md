# Markdown Writing Rules for AI Agents

This document outlines all the validation rules enforced by the agent-md linter when writing markdown content. Each rule includes examples of invalid syntax and recommended alternatives.

## Overview

The agent-md linter enforces AI-friendly markdown standards to ensure content is easily readable and parseable by AI agents. The rules focus on simplicity, clarity, and machine-readability.

---

## Rule 1: No Bold Text

Rule ID: `no-bold`
Severity: Error
Description: Bold text formatting is not allowed for AI agents.

### Invalid Examples

❌ Invalid - Double asterisks

```text
This is **bold text** that will be rejected.
```

❌ Invalid - Double underscores

```text
This is __also bold__ and will cause an error.
```

❌ Invalid - Mixed usage

```text
Both **bold** and __bold__ are prohibited.
```

### Recommended Alternatives

✅ Valid - Use plain text

```text
This is important text without bold formatting.
```

✅ Valid - Use italics for emphasis

```text
This is *italic text* which is allowed.
```

✅ Valid - Use headings for emphasis

```text
## Important Section
This section contains important information.
```

✅ Valid - Use code formatting for technical terms

```text
Use `monospace` for technical terms or variable names.
```

### Rationale for No Bold Rule

Bold text creates visual noise for AI agents and doesn't add semantic meaning that can't be conveyed through other means like headings or code formatting.

---

## Rule 2: Simple Table Syntax

Rule ID: `simple-tables`
Severity: Warning/Error
Description: Tables should use simple syntax without complex attributes.

### Rule 2 Invalid Examples

❌ Invalid - Complex table attributes

```text
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| colspan="2" | Cell 3 |
```

❌ Invalid - Inline formatting in cells

```text
| Name | Description |
|------|-------------|
| Item | This has **bold** text |
| Test | This has *italic* text |
```

❌ Invalid - Too many dashes for table separator (should be exactly 3)

```text
| Name | Description |
|------|-------------|
| Test | text        |
```

### Rule 2 Recommended Alternatives

✅ Valid - Simple table syntax with 3 dash separators

```text
| Name | Description |
|---|---|
| Item | Simple description |
| Test | Another description |
```

✅ Valid - Keep tables narrow (≤5 columns)

```text
| Name | Type | Status | Priority | Owner |
|------|------|--------|----------|-------|
| Task | Bug | Open | High | Team |
```

✅ Valid - Use lists for complex data

```text
- Item 1: Simple description
- Item 2: Another description
- Item 3: Third description
```

✅ Valid - Use structured formats for complex data

```json
{
  "items": [
    {"name": "Item 1", "description": "Simple description"},
    {"name": "Item 2", "description": "Another description"}
  ]
}
```

### Rationale for Simple Tables Rule

Complex tables are difficult for AI agents to parse and can introduce formatting inconsistencies. Simple tables are more reliable for machine processing.

---

## Rule 3: No Useless Links

Rule ID: `useless-links`
Severity: Warning
Description: Link text should not be identical to the URL.

### Rule 3 Invalid Examples

```text
# ❌ Invalid - Link text equals URL
Visit [https://example.com](https://example.com) for more info.

# ❌ Invalid - Link text equals URL without protocol
Check out [example.com](https://www.example.com) today.

# ❌ Invalid - Link text equals URL with www
Go to [www.example.com](https://example.com/) now.
```

### Rule 3 Recommended Alternatives

✅ Valid - Descriptive link text

```text
Visit [Example Website](https://example.com) for more info.
```

✅ Valid - Action-oriented link text

```text
[Check out our documentation](https://docs.example.com)
```

✅ Valid - Contextual link text

```text
See the [API reference](https://api.example.com) for implementation details.
```

✅ Valid - Plain URLs (when appropriate)

```text
For more information: https://example.com
```

### Rationale for Useless Links Rule

Links where the text equals the URL provide no additional context and create redundant information. Descriptive link text helps AI agents understand the purpose and destination of links.

---

## Rule 4: No ASCII Graphs

Rule ID: `no-ascii-graph`
Severity: Warning
Description: Human-readable ASCII graphs should be replaced with LLM-readable formats.

### Rule 4 Invalid Examples

```text
# ❌ Invalid - Box drawing characters
┌─────────┬─────────┐
│ Name    | Value   │
├─────────┼─────────┤
│ Item 1  | 100     │
│ Item 2  | 200     │
└─────────┴─────────┘

# ❌ Invalid - Tree structures
root
├── branch1
│   ├── leaf1
│   └── leaf2
└── branch2
    └── leaf3

# ❌ Invalid - Flow chart patterns
[Start] -> [Process] -> [End]

# ❌ Invalid - Progress bars
Progress: [████████░░] 80%

# ❌ Invalid - Graph-like patterns
A --- B --- C
 \    |    /
  D --- E
```

### Rule 4 Recommended Alternatives

✅ Valid - Structured CSV

```text
name,value
Item 1,100
Item 2,200
```

✅ Valid - JSON format

```json
{
  "tree": {
    "root": {
      "branch1": {
        "leaf1": {},
        "leaf2": {}
      },
      "branch2": {
        "leaf3": {}
      }
    }
  }
}
```

✅ Valid - Mermaid Diagram

```mermaid
flowchart LR
    Start --> Process --> End
```

✅ Valid - Numbered List with Conditions

```text
1. Start: Initialize process
2. Process: Execute main logic
3. End: Finalize and return
```

✅ Valid - ZON format (Zero Overhead Notation)

See: <https://github.com/ZON-Format/ZON>, <https://zonformat.org/>.

```txt
users:@(3):id,tier
1,premium
3,free
5,pro
```

✅ Valid - Simple progress indicator

```text
Progress: 80% complete
```

### Rationale for ASCII Graphs Rule

ASCII graphs are visually appealing but difficult for AI agents to parse reliably. Structured formats like JSON, CSV, or Mermaid diagrams provide machine-readable alternatives.

---

## Rule 5: Proper Heading Structure

Rule ID: `heading-structure`
Severity: Warning
Description: Headings should follow logical hierarchy and not skip levels.

### Rule 5 Invalid Examples

❌ Invalid - Skipping heading levels

```text
# Title
### Subsection (skips H2)
```

❌ Invalid - Multiple H1 headings

```text
# First Title
# Second Title
```

❌ Invalid - Inconsistent heading style

```text
## Heading 1
### Heading 2
#### Heading 3
###### Heading 4 (skips H5)
```

✅ Valid - Sequential heading levels

```text
# Title
## Section 1
### Subsection 1.1
#### Sub-subsection 1.1.1
```

✅ Valid - Single H1 per document

```text
# Main Title
## Section 1
## Section 2
```

✅ Valid - Consistent hierarchy

```text
# Document Title
## Overview
## Implementation
### Details
### Examples
## Conclusion
```

### Rationale for Heading Structure Rule

Proper heading structure creates a logical document outline that AI agents can easily navigate and understand.

---

## Rule 6: Code Block Best Practices

Rule ID: `code-blocks`
Severity: Warning
Description: Code blocks should specify language when possible.

### Rule 6 Invalid Examples

❌ Invalid - Unspecified language

```text
"```"
function example() {
    return "Hello World";
}
"```"
```

❌ Invalid - Inline code for multiline content

```text
function example() {
    return "Hello World";
}
```

✅ Valid - Specify language

```javascript
function example() {
    return "Hello World";
}
```

✅ Valid - Use appropriate language

```python
def example():
    return "Hello World"
```

✅ Valid - Inline code for short snippets

Use `console.log()` for debugging.

✅ Valid - When no language is specified, use `text`

```text
This is a code block without specific language
```

### Rationale for Code Blocks Rule

Specifying language helps AI agents understand the context and apply appropriate parsing rules.

---

## Rule 7: List Formatting

Rule ID: `list-formatting`
Severity: Warning
Description: Lists should be consistent and properly formatted.

### Rule 7 Invalid Examples for List Formatting

❌ Invalid - Mixed list types

```text
1. First item
- Second item
2. Third item
```

❌ Invalid - Inconsistent spacing

```text
1. First item
   2. Second item
3. Third item
```

❌ Invalid - Empty list items

```text
1. First item
2.
3. Third item
```

### Rule 7 Recommended Alternatives for List Formatting

✅ Valid - Consistent ordered list

```text
1. First item
2. Second item
3. Third item
```

✅ Valid - Consistent unordered list

```text
- First item
- Second item
- Third item
```

✅ Valid - Nested lists

```text
1. Main item
   1.1. Sub item
   1.2. Another sub item
2. Second main item
```

### Rationale for List Formatting Rule

Consistent list formatting improves readability and parsing reliability for AI agents.

---

## Rule 8: No Duplicate Headings

Rule ID: `no-duplicate-headings`
Severity: Warning
Description: Headings with the same content should not appear multiple times in a document.

### Rule 8 Invalid Examples

❌ Invalid - Same heading content at different levels

```text
## Introduction
Some content here

### Introduction
Different content but same heading text

```

❌ Invalid - Same heading content at same level

```text
## Overview
First overview content

## Overview
Second overview content
```

❌ Invalid - Multiple identical headings

```text
## Usage
How to use the tool

## Usage
More usage instructions

## Usage
Additional usage notes
```

✅ Valid - Unique heading content

```text
## Introduction
Some content here

### Getting Started
Different content with unique heading
```

✅ Valid - More specific headings

```text
## Overview
First overview content

## Detailed Overview
Second overview content

## Quick Reference
Additional reference material
```

✅ Valid - Numbered or qualified headings

```text
## Usage: Basic Operations
How to use the tool

## Usage: Advanced Features
More usage instructions

## Usage: Troubleshooting
Additional usage notes
```

✅ Valid - Hierarchical structure

```text
## Usage
### Basic Operations
How to use the tool

### Advanced Features
More usage instructions

### Troubleshooting
Additional usage notes
```

### Rationale for No Duplicate Headings Rule

Duplicate headings create ambiguity for AI agents when navigating and referencing document sections. Unique heading content ensures clear navigation and unambiguous section identification, making it easier for AI agents to understand and process document structure.

---

## Rule 9: Single H1 Title

Rule ID: `single-title`
Severity: Error
Description: Documents should have only one top-level heading (H1).

### Rule 9 Invalid Examples

❌ Invalid - Multiple H1 headings

```text
# First Title
Some content here

# Second Title
More content here
```

❌ Invalid - H1 headings mixed with other levels

```text
# Main Title
## Section
# Another Main Title
```

✅ Valid - Single H1 heading

```text
# Document Title
## Section 1
Content for section 1

## Section 2
Content for section 2

```

✅ Valid - Proper hierarchy

```text
# Main Title
## Overview
## Details
### Specific Detail
## Conclusion
```

### Rationale for Single H1 Title Rule

Multiple H1 headings create confusion about the document's main title and structure. AI agents rely on a clear document hierarchy, and a single H1 provides an unambiguous entry point for understanding the document's purpose.

---

## Validation Output Format

When using the `agent-md lint` command, validation results are returned in JSON format:

```json
{
  "valid": false,
  "errors": [
    {
      "line": 3,
      "column": 12,
      "message": "Bold text is not allowed for AI agents",
      "rule": "no-bold"
    }
  ],
  "warnings": [
    {
      "line": 7,
      "column": 1,
      "message": "Link text should not be the same as the URL - provide meaningful link text",
      "rule": "useless-links"
    }
  ]
}
```

### Field Descriptions

- valid: Boolean indicating if the document passes all validation rules
- errors: Array of validation errors that prevent writing
- warnings: Array of warnings that don't prevent writing but should be addressed
- line: Line number where the issue occurs (1-based)
- column: Column number where the issue occurs (1-based)
- message: Human-readable description of the issue
- rule: Internal rule identifier

---

## Best Practices Summary

1. Use plain text instead of bold formatting
2. Keep tables simple with ≤5 columns and no complex attributes
3. Write descriptive link text rather than repeating URLs
4. Use structured formats (JSON, CSV, Mermaid) instead of ASCII art
5. Maintain proper heading hierarchy without skipping levels
6. Specify code block languages when possible
7. Format lists consistently with proper spacing and structure
8. Avoid duplicate headings with the same content
9. Use only one H1 heading per document

---

## Integration with agent-md

These rules are automatically enforced when using the `agent-md write` command. The tool will reject content that contains errors and return warnings for style issues.

```bash
# Validate content before writing
agent-md lint --content "# Title\nContent with **bold** text"

# Lint file with human-readable output
agent-md lint-file README.md

# Write with automatic validation
agent-md write document.md "# Title\nValid content without bold"
```

By following these rules, you ensure your markdown content is optimized for AI agent consumption and processing.
