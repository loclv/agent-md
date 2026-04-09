# 🚀 Introducing agent-md

## 🇬🇧 English
Stop token waste! 🤖
LLMs pay a "tax" for bold text, tables, and extra spaces. 💸 `agent-md` is a Rust CLI that optimizes markdown for AI agents!

✨ Features:
- 📉 -20% tokens
- 🔍 JSON-first output
- 🛠️ Tools: read, write, lint, fmt
- 📏 AI-friendly standards
- 🔌 VS Code extension

### Lint: 11 Rules for AI-Friendly Markdown
Catches issues that hurt AI parsing:

- `no-bold` - No `**bold**` or `__bold__` (use italics instead)
- `simple-tables` - Tables with 5 columns max, 3-dash separators
- `useless-links` - Link text must differ from URL
- `no-ascii-graph` - Use JSON/CSV/Mermaid instead of ASCII art - <https://en.wikipedia.org/wiki/ASCII_art>
- `heading-structure` - No level skipping, single H1 per doc
- `code-blocks` - Always specify language
- `list-formatting` - Consistent list style
- `no-duplicate-headings` - Unique heading content
- `single-title` - One H1 per document
- `space-indentation` - Max 2 spaces (code blocks exempt)
- `table-trailing-spaces` - No extra spaces in cells

### Format: Auto-Fix for Clean Output
`agent-md fmt` automatically:
- Removes trailing spaces in table cells
- Normalizes formatting for AI consumption
- Preserves code block indentation

### VS Code Extension
Format Markdown directly in VS Code.

Write once, optimize for both! 🌍

## 🇻🇳 Tiếng Việt
Ngừng lãng phí token! 🤖
LLM phải trả "thuế" cho chữ đậm, bảng và khoảng trắng thừa. `agent-md` là công cụ CLI (Rust) giúp tối ưu markdown cho AI!

Điểm nổi bật:
- Giảm ~20% token
- JSON-first dễ xử lý
- Công cụ: read, write, lint, fmt
- Chuẩn thân thiện AI
- VS Code extension

### Lint: 11 Luật cho Markdown AI-Friendly
Phát hiện các vấn đề gây khó cho AI:

- `no-bold` - Không `**bold**` hay `__bold__` (dùng nghiêng)
- `simple-tables` - Bảng tối đa 5 cột, ngăn cách 3 gạch
- `useless-links` - Text link phải khác URL
- `no-ascii-graph` - Dùng JSON/CSV/Mermaid thay vì ASCII art - <https://en.wikipedia.org/wiki/ASCII_art>
- `heading-structure` - Không bỏ qua cấp, 1 H1/tài liệu
- `code-blocks` - Luôn chỉ định ngôn ngữ
- `list-formatting` - Đồng bộ kiểu danh sách
- `no-duplicate-headings` - Nội dung heading duy nhất
- `single-title` - Một H1 mỗi tài liệu
- `space-indentation` - Tối đa 2 khoảng trắng (code block ngoại lệ)
- `table-trailing-spaces` - Không khoảng trắng thừa trong cell

### Format: Tự Động Sửa Lỗi
`agent-md fmt` tự động:
- Xóa khoảng trắng thừa trong table cells
- Chuẩn hóa định dạng cho AI
- Giữ nguyên thụt lề code block

### VS Code Extension
Format Markdown trực tiếp trong VS Code.

Viết một lần, tối ưu cho cả người và máy! 🌍
