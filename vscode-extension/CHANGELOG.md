# Change Log

All notable changes to the "agent-md-formatter" extension will be documented in this file.

## [0.1.0] - 2024-04-04

### Added
- Initial release
- Document formatting support for Markdown files using agent-md CLI
- Configurable formatting options:
  - `agentMd.path` - Path to agent-md executable
  - `agentMd.format.removeBold` - Remove bold markers
  - `agentMd.format.compactBlankLines` - Compact blank lines
  - `agentMd.format.collapseSpaces` - Collapse multiple spaces
  - `agentMd.format.removeHorizontalRules` - Remove horizontal rules
  - `agentMd.format.removeEmphasis` - Remove emphasis markers
- Error handling for missing agent-md executable
- Format on save support via VS Code settings
