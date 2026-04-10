# Changelog

All notable changes to this project will be documented in this file.

## [0.1.6] - 2026-03-30

### Features
- Enable token-saving format options by default and remove --token-saver flag
- Add code block with nested markdown content to format-all.md test file

### Refactor
- Extract validation rules into separate module

## [0.1.4] - 2026-03-23

### Features
- Add table-trailing-spaces validation rule
- Add --human flag for pretty-printed JSON output

### Documentation
- Add example output for lint command with --human flag

### Tests
- Add comprehensive test coverage for content processing and validation

## [0.1.3] - 2026-03-23

### Features
- Add --human flag for pretty-printed JSON output

### Tests
- Add comprehensive test coverage for content processing and validation

## [0.1.2] - 2026-03-22

### Features
- Enforce ASCII graph detection in code blocks as errors
- Add version flag, logging rules, and improve CLI argument handling
- Add agent-md logo and update Vietnamese README introduction

### Documentation
- Add concrete before/after markdown examples to README files
- Add code formatting to command examples in README files
- Add Vietnamese translation of README
- Add MIT license section to README
- Update README title to emphasize LLM-friendly markdown focus
- Restructure README sections to improve clarity and flow

### Refactor
- Change validation rules from errors to warnings and improve code quality

## [0.1.0] - 2026-03-20

### Features
- Initial release of agent-md (formerly ralph-md)
- Add lint-file command with human-readable output
- Add lint command for markdown validation
- Add README, single H1 validation rule, and markdown writing guide
- Add write-section command for targeted section updates
- Add --field option to read command for direct field extraction
- Add --content/-c flag to read command for section extraction
- Add space-indentation validation rule
- Skip validation inside code blocks and detect multiple violations per line
- Reorganize validation rules documentation with heading structure, code block, and list formatting validators

### Documentation
- Add AGENTS.md with project rules
- Add LLM agent integration guidelines
- Improve installation instructions
- Add no-duplicate-headings rule
- Update simple-tables rule examples

### Refactor
- Rename ralph-md to agent-md
- Flatten project structure
- Remove bold formatting from markdown documentation

### Chore
- Add VS Code settings, project guide, and CI/CD configuration
- Add markdownlint config

## Recent Development (Unreleased)

### 2026-04-10
- Update Vietnamese README examples and add markdown code block tests
- Apply formatting rules to markdown code blocks
- Remove extra blank lines throughout Vietnamese README
- Add GitHub repository link to intro blog post
- Expand intro blog with lint rules and VS Code extension details

### 2026-04-09
- Add intro blog post and simplify logo design

### 2026-04-05
- Add VS Code extension for agent-md formatter
- Add stdin formatting support and preserve code block indentation
- Add YAML frontmatter preservation and improve code formatting
- Preserve blank lines before code fences
- Configure language-specific formatters in VS Code settings
- Add MIT license and repository URL to VS Code extension

### 2026-04-04
- Handle empty content before comments in code blocks
- Add comprehensive test coverage for HTML tags in Markdown
- Add test coverage for underscore handling in blockquotes
- Add Setext-style heading detection and validation
- Extract blockquote normalization logic into separate module
- Extract code block formatting logic into separate code_blocks module
- Extract table formatting logic into separate tables module
- Add implicit fmt command when markdown file path provided
- Compact table separator rows to exactly 3 dashes
- Preserve leading whitespace when collapsing spaces

### 2026-04-02
- Add test coverage for validation rules and improve code formatting

### 2026-04-01
- Enable token-saving format options by default

### 2026-03-26
- fmt command: Preserves separator rows and code block content

### 2026-03-24
- Improve table formatting to trim leading and trailing spaces from cells
- Add fmt command to auto-format markdown tables
- Add comprehensive test coverage for table formatting edge cases
