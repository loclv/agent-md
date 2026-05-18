# Markdown Writing Rules for AI Agents

This document outlines all the validation rules enforced by the agent-md linter when writing markdown content. Each rule includes examples of invalid syntax and recommended alternatives.

## Overview

The agent-md linter enforces AI-friendly markdown standards to ensure content is easily readable and parseable by AI agents. The rules focus on simplicity, clarity, and machine-readability.

## Rule 1: No Bold Text

Rule ID: `no-bold`
Severity: Error
Description: Bold text formatting is not allowed for AI agents, except in code blocks.

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

✅ Valid - Bold text is allowed inside code blocks

```javascript
function example() {
    // This **bold** text is allowed in code blocks
    console.log("**This bold text is also allowed**");
    return __bold__;
}
```

```text
This is a code block with **bold** text that is allowed.
```

✅ Valid - Bold text is also allowed in inline code

```text
Use `**bold**` and `__bold__` in inline code for emphasis.
```

✅ Valid - Bold text in table cells is auto-formatted

```text
| Name | Description |
|---|---|
| Item | This has bold text |
| Test | This has italic text |
```

When formatting markdown, `strip_bold_from_cell` automatically removes `**` and `__` markers from table cell content while preserving inline code spans.
`**bold**` and `__bold__` are allowed in both inline code (` `) and fenced code blocks (``` ```). Bold in table cells is automatically stripped during `agent-md fmt`.

### Rationale for No Bold Rule

Bold text creates visual noise for AI agents and doesn't add semantic meaning that can't be conveyed through other means like headings or code formatting.
Code blocks are exempt because they preserve original syntax and formatting for programming languages, documentation, and other contexts where bold characters may have specific meaning or be part of the code syntax itself.

Table cells are auto-formatted — bold markers are stripped during `agent-md fmt` so the resulting table contains plain text. Inline code within table cells is preserved unchanged.

## Rule 2: Simple Table Syntax

Rule ID: `simple-tables`
Severity: Error
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
|---|---|---|---|---|
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

## Rule 3: No Useless Links

Rule ID: `useless-links`
Severity: Error
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

## Rule 4: No ASCII Graphs

Rule ID: `no-ascii-graph`
Severity: Error
Description: Human-readable ASCII graphs should be replaced with LLM-readable formats. This rule applies to ALL content, including inside code blocks.

### Rule 4 Invalid Examples

Example - Box drawing characters in regular text (invalid)

```text
┌─────────┬─────────┐
│ Name    | Value   │
├─────────┼─────────┤
│ Item 1  | 100     │
│ Item 2  | 200     │
└─────────┴─────────┘
```

Example - Box drawing characters in code blocks (invalid)

```text
┌───┐
│ A │
└───┘
```

Example - Tree structures in regular text (invalid)

```text
root
├── branch1
│   ├── leaf1
│   └── leaf2
└── branch2
    └── leaf3
```

Example - Tree structures in code blocks (invalid)

```text
├── public/
│   ├── pagefind/
│   └── favicon.svg
```

Example - Flow chart patterns (invalid)

```text
[Start] -> [Process] -> [End]
```

Example - Progress bars (invalid)

```text
Progress: [████████░░] 80%
```

Example - Graph-like patterns (invalid)

```text
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
Unlike bold text (which is allowed in code blocks because it may be part of programming language syntax), ASCII graphs in code blocks are still visual patterns that AI agents struggle to interpret. The `agent-md` linter therefore rejects ASCII graphs even inside code blocks, as they provide no additional semantic value for AI processing.

When you need to represent tree structures, diagrams, or visual hierarchies, use these alternatives:
- JSON for nested data structures
- CSV for tabular data
- Mermaid for flowcharts and diagrams
- Numbered lists with conditions for processes
- Simple text descriptions for progress indicators

## Rule 5: Proper Heading Structure

Rule ID: `heading-structure`
Severity: Error
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

## Rule 6: Code Block Best Practices

Rule ID: `code-blocks`
Severity: Error
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

## Rule 7: List Formatting

Rule ID: `list-formatting`
Severity: Error
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

## Rule 10: Limited Space Indentation

Rule ID: `space-indentation`
Severity: Error
Description: Use at most 2 spaces for indentation in regular text. Code blocks are exempt from this rule.

### Rule 10 Invalid Examples

❌ Invalid - 4 spaces for text indentation

```text
    This paragraph is indented with 4 spaces.
    It should use 2 spaces or less.
```

❌ Invalid - Excessive indentation in lists

```text
1. First item
      2. Second item with deep indentation
   3. Third item
```

### Rule 10 Recommended Alternatives

✅ Valid - 2 spaces or fewer for text

```text
  This paragraph uses 2 spaces for emphasis.
  Second line of the paragraph.
```

✅ Valid - No indentation for regular text

```text
This paragraph starts at the left margin.
No extra spaces are needed.
```

✅ Valid - Code blocks are exempt

```javascript
function example() {
    // Code blocks can use proper indentation
    // regardless of space count
    return true;
}
```

✅ Valid - Indented code blocks preserve formatting

```python
def example():
    # Python code uses 4 spaces
    # This is intentional and allowed
    pass
```

✅ Valid - Tab character for text indentation

```text
	This paragraph uses a tab character.
```

### Rationale for Space Indentation Rule

Excessive indentation (more than 2 spaces) in regular text can cause parsing issues and formatting inconsistencies. Code blocks are exempt because they require proper indentation for readability and many programming languages mandate specific indentation styles.

## Rule 11: No Trailing Spaces in Table Cells

Rule ID: `table-trailing-spaces`
Severity: Error
Description: Table cells should not have trailing spaces, except for a single optional space.

### Rule 11 Invalid Examples

 Invalid - Multiple trailing spaces in table cells
| Name | Description |
|---|---|
| Item | This is an item |

 Invalid - Many trailing spaces in cells
| Column 1 | Column 2 | Column 3 |
|---|---|---|
| Value 1 | Value 2 | Value 3 |

### Rule 11 Recommended Alternatives

 Valid - No trailing spaces in cells
| Name | Description |
|---|---|
| Item | This is an item |

 Valid - Single trailing space in cells (allowed but not recommended)
| Name | Description |
|---|---|
| Item | This is an item |

 Valid - Table separator rows are ignored
| Name | Description |
|---|---|
| Item | This is an item |

### Auto-fix

Use the `fmt` command to automatically remove leading and trailing spaces from all table cells:

```bash
agent-md fmt path/to/file.md
```

### Rationale for Table Trailing Spaces Rule

Trailing spaces in table cells are unnecessary and can cause formatting inconsistencies. They add visual noise without any semantic value and may cause issues with markdown parsers and version control systems.

## Rule 12: No Trailing Spaces

Rule ID: `no-trailing-spaces`
Severity: Warning
Description: Trailing spaces at the end of lines are unnecessary and should be removed.

### Rule 12 Invalid Examples

❌ Invalid - Trailing spaces

```text
This line has a trailing space 
This line has multiple trailing spaces   
```

### Rule 12 Recommended Alternatives

✅ Valid - No trailing spaces

```text
This line has no trailing spaces
This line is also clean
```

### Rationale for No Trailing Spaces Rule

Trailing spaces are often invisible to the reader but can cause formatting inconsistencies, bloat the file size, and create unnecessary diffs in version control.

## Rule 13: No Hard Tabs

Rule ID: `no-hard-tabs`
Severity: Warning
Description: Use spaces for indentation instead of hard tabs.

### Rule 13 Invalid Examples

❌ Invalid - Hard tabs

```text
	This line starts with a tab.
-	List item with a tab.
```

### Rule 13 Recommended Alternatives

✅ Valid - Spaces for indentation

```text
  This line starts with spaces.
- List item with a space.
```

### Rationale for No Hard Tabs Rule

Tabs can be rendered inconsistently across different editors and platforms, leading to alignment issues. Spaces ensure consistent rendering everywhere.

## Rule 14: Blanks Around Headings

Rule ID: `blanks-around-headings`
Severity: Warning
Description: Headings should be surrounded by blank lines for better readability.

### Rule 14 Invalid Examples

❌ Invalid - Missing blank lines

```text
Some paragraph text.
# Heading 1
More paragraph text.
```

### Rule 14 Recommended Alternatives

✅ Valid - Blank lines around headings

```text
Some paragraph text.

# Heading 1

More paragraph text.
```

### Rationale for Blanks Around Headings Rule

Proper spacing around headings makes the document structure clearer and visually separates sections, enhancing readability for both humans and AI agents.

## Rule 15: First Line Heading

Rule ID: `first-line-h1`
Severity: Warning
Description: The first non-blank line in a document should be a top-level heading (H1).

### Rule 15 Invalid Examples

❌ Invalid - Missing H1 at start

```text
This is an introduction paragraph.

# Main Title
```

### Rule 15 Recommended Alternatives

✅ Valid - Starts with H1

```text
# Main Title

This is an introduction paragraph.
```

### Rationale for First Line Heading Rule

Starting a document with an H1 heading immediately establishes its main subject and purpose, making it easier to identify the content's context.

## Rule 16: Single Trailing Newline

Rule ID: `single-trailing-newline`
Severity: Warning
Description: Files should end with exactly one single newline character.

### Rule 16 Invalid Examples

❌ Invalid - Missing trailing newline

```text
# Title
Content ends here without a newline
```

❌ Invalid - Multiple trailing newlines

```text
# Title
Content ends here.


```

### Rule 16 Recommended Alternatives

✅ Valid - Single trailing newline

```text
# Title
Content ends here.
<newline>
```

### Rationale for Single Trailing Newline Rule

A single trailing newline is a POSIX standard for text files, ensuring that tools and parsers process the file correctly without missing the last line or complaining about EOF behavior.

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

## Best Practices Summary

- Use plain text instead of bold formatting
- Keep tables simple with ≤5 columns and no complex attributes
- Write descriptive link text rather than repeating URLs
- Use structured formats (JSON, CSV, Mermaid) instead of ASCII art
- Maintain proper heading hierarchy without skipping levels
- Specify code block languages when possible
- Format lists consistently with proper spacing and structure
- Avoid duplicate headings with the same content
- Use only one H1 heading per document
- Limit text indentation to 2 spaces or fewer (code blocks exempt)
- Avoid trailing spaces in table cells (0 or 1 space maximum)
- Remove trailing spaces at the end of lines
- Use spaces instead of hard tabs for indentation
- Surround headings with blank lines
- Start documents with an H1 heading
- End files with a single newline character

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
