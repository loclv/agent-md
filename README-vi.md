# `agent-md` - Một công cụ CLI giúp bạn viết markdown thân thiện với LLM

```bash
agent-md lint README.md
# {"valid":false,"errors":[{"line":7,"column":1,"message":"Use at most 2 spaces for indentation in regular text. Code blocks are exempt from this rule.","rule":"space-indentation"},{"line":28,"column":1,"message":"Use at most 2 spaces for indentation in regular text. Code blocks are exempt from this rule.","rule":"space-indentation"},{"line":34,"column":1,"message":"Human-readable ASCII graph detected. Use LLM-readable formats instead: Structured CSV, JSON, Mermaid Diagram, Numbered List with Conditions, ZON format, or simple progress indicators","rule":"no-ascii-graph"},{"line":36,"column":1,"message":"Human-readable ASCII graph detected. Use LLM-readable formats instead: Structured CSV, JSON, Mermaid Diagram, Numbered List with Conditions, ZON format, or simple progress indicators","rule":"no-ascii-graph"}],"warnings":[]}
```

## Tại sao công cụ này tồn tại

Nhiều file markdown hiện nay được viết bởi LLM hoặc AI agents đang lãng phí rất nhiều token. Khi một LLM khác đọc lại những file này, nó tiếp tục tốn thêm token không cần thiết. Thậm chí file này được đọc đi đọc lại mỗi lần chat.
Nguyên nhân là markdown được thiết kế để con người dễ đọc, nên thường chứa các yếu tố như in đậm, ký tự trang trí, khoảng trắng... Những thứ này hữu ích cho người, nhưng không cần thiết với LLM.

Thực tế, LLM/agents không cần đọc toàn bộ file. Chúng chỉ cần truy cập đúng phần nội dung cần thiết (ví dụ: ## Development) thay vì xử lý cả tài liệu.

### Vấn đề

- Lãng phí token: Các định dạng như in đậm, bảng, ký tự trang trí làm tăng số token mà không giúp ích cho AI
- Đọc không hiệu quả: LLM vẫn phải xử lý cả các yếu tố trình bày như bold, ASCII art…
- Cấu trúc dư thừa: Nhiều thành phần chỉ hữu ích cho người (bảng phức tạp, layout đẹp) nhưng không cần cho AI
- Tăng chi phí: Mỗi lần LLM đọc lại tài liệu đều phải “trả phí” cho những phần định dạng này

### Giải pháp

agent-md đưa ra một cách viết markdown tối giản, thân thiện với AI, giúp:
- Giảm token không cần thiết
- Giữ nội dung rõ ràng, dễ truy cập theo từng phần
- Vẫn đảm bảo con người có thể đọc được khi cần

Mục tiêu là: viết một lần, tối ưu cho cả người và AI, nhưng không lãng phí tài nguyên.

## Cài đặt

Xây dựng từ mã nguồn:
- Cài đặt Rust trước nếu chưa được cài đặt
- Sau đó xây dựng phiên bản release:

```bash
cargo build --release
# Binary tại target/release/agent-md
```

- Thêm vào PATH (tùy chọn):
```bash
# agent-md command
export PATH="/Users/username/w/agent-md/target/release:$PATH"
```

Bây giờ bạn có thể sử dụng lệnh `agent-md` từ bất cứ đâu.

## Ví dụ sử dụng cụ thể

### Đầu vào: Markdown thông thường

Đây là một file markdown tiêu chuẩn mà nhiều LLM tạo ra:
```markdown
# Dự án Của Tôi

## Tổng quan

Đây là một dự án rất tuyệt vời với nhiều tính năng nổi bật:

| Tính năng | Mô tả | Trạng thái |
|---|---|---|
| API | RESTful API hoàn chỉnh | ✅ Hoàn thành |
| UI | Giao diện người dùng hiện đại | 🚧 Đang phát triển |
| Tests | Unit tests và integration tests | ✅ Hoàn thành |

### Các bước thực hiện

1. Clone repository
2. Cài đặt dependencies: `bun i`
3. Chạy server: `bun dev`

>Lưu ý: Đảm bảo bạn có Node.js phiên bản 22+ được cài đặt!

```

### Đầu ra: Markdown thân thiện với AI qua agent-md

Sau khi xử lý với agent-md, nội dung trở nên gọn gàng hơn:
```markdown
# Dự án Của Tôi

## Tổng quan

Đây là một dự án tuyệt vời với nhiều tính năng nổi bật:

- API: RESTful API hoàn chỉnh (Hoàn thành)
- UI: Giao diện người dùng hiện đại (Đang phát triển)
- Tests: Unit tests và integration tests (Hoàn thành)

## Các bước thực hiện

1. Clone repository
2. Cài đặt dependencies: `bun i`
3. Chạy server: `bun dev`

Lưu ý: Đảm bảo bạn có Node.js phiên bản 22+ được cài đặt.
```

### So sánh hiệu quả

```bash
# Kiểm tra file markdown thông thường
agent-md lint regular-markdown.md
# {"valid":false,"errors":[
#   {"line":3,"message":"No bold text allowed","rule":"no-bold"},
#   {"line":6,"message":"Complex table detected","rule":"simple-table"},
#   {"line":15,"message":"No bold text allowed","rule":"no-bold"}
# ]}

# Kiểm tra file agent-md
agent-md lint agent-md-markdown.md
# {"valid":true,"errors":[],"warnings":[]}
```

Lợi ích:
- Giảm ~20% số token không cần thiết
- LLM đọc và xử lý nhanh hơn
- Vẫn giữ được thông tin đầy đủ
- Dễ dàng trích xuất phần cụ thể

## Thêm quy tắc hoặc lệnh cho LLM/Agents tuân theo

Ví dụ, khi thêm quy tắc mới, thêm vào tệp `AGENTS.md`.
>Sau khi tạo hoặc cập nhật bất kỳ tệp markdown nào, luôn chạy `agent-md lint path/to/file.md` để xác thực nội dung trước khi coi tác vụ hoàn thành.

Hoặc chat với LLM/Agents:
>Use `agent-md` CLI to run lint

## Các lệnh (đầu ra JSON thân thiện với LLM)

Tất cả các lệnh đều trả về JSON để dễ phân tích.

### Đọc một tệp

```bash
agent-md read <path>
# Trả về: {path, content, word_count, line_count, headings}

# Trích xuất trường cụ thể
agent-md read <path> --field <field_name>
# Các trường có sẵn: path, content, word_count, line_count, headings

# Đọc phần cụ thể theo đường dẫn heading (không cần đọc toàn bộ tệp)
agent-md read <path> --content <section_path>
# Ví dụ: agent-md read README.md --content "## Development"
# Các phần lồng nhau: agent-md read README.md --content "## Development > Build"
```

### Ghi một tệp

```bash
agent-md write <path> <content>
# Trả về: {success, message, document}
```

### Ghi vào một phần cụ thể

```bash
agent-md write-section <path> --section <heading_path> --content <content>
# Thay thế nội dung phần hiện có hoặc tạo phần mới
# Ví dụ: agent-md write-section README.md --section "## Development" --content "Nội dung mới"
# Các phần lồng nhau: agent-md write-section README.md --section "## Development > Build" --content "Nội dung mới"
```

```bash
agent-md write <path> <content>
# Trả về: {success, message, document}
```

### Thêm vào tệp

```bash
agent-md append <path> <content>
# Trả về: {success, message, document}
```

### Chèn vào dòng

```bash
agent-md insert <path> <line> <content>
# Trả về: {success, message, document}
```

### Xóa dòng

```bash
agent-md delete <path> <line> [count]
# Trả về: {success, message, document}
```

### Liệt kê các tệp markdown

```bash
agent-md list <directory>
# Trả về: [file paths...]
```

### Tìm kiếm trong tệp

```bash
agent-md search <path> <query>
# Trả về: {query, matches: [{line, content}], total}
```

### Lấy các heading

```bash
agent-md headings <path>
# Trả về: [{level, text, line}...]
```

### Lấy thống kê

```bash
agent-md stats <path>
# Trả về: {path, word_count, line_count, heading_count}
```

### Chuyển đổi sang JSONL

```bash
agent-md to-jsonl <path>
# Trả về: các dòng JSONL với {type, content, level, language}
```

### Kiểm tra/Xác thực markdown

```bash
agent-md lint <path>
# Trả về: {valid, errors: [{line, column, message, rule}], warnings: [{line, column, message, rule}]}

agent-md lint --content "# Markdown content"
# Xác thực nội dung trực tiếp mà không cần tệp

agent-md lint-file <path>
# Trả về: đầu ra kiểm tra dễ đọc với lỗi, cảnh báo, và tóm tắt
```

## Quy tắc xác thực

Công cụ kiểm tra thực thi các tiêu chuẩn markdown thân thiện với AI.

### Quy tắc lỗi (nội dung khối)

- Không văn bản in đậm: `**bold**` và `__bold__` bị từ chối (lỗi), ngoại trừ trong khối mã
- Cấu trúc heading: Nhiều heading H1 và các cấp heading bị bỏ qua bị từ chối (lỗi)
- Cú pháp bảng: Thuộc tính bảng phức tạp và định dạng phân tách không chính xác bị từ chối (lỗi)
- Cú pháp bảng đơn giản: Bảng rất rộng và định dạng inline trong ô bị từ chối (lỗi)
- Không đồ họa ASCII: Ký tự vẽ hộp và các mẫu trực quan bị từ chối (lỗi)
- Thực hành tốt nhất khối mã: Các khối mã không có đặc tả ngôn ngữ bị từ chối (lỗi)
- Định dạng danh sách: Các dấu danh sách không nhất quán và đánh số bị từ chối (lỗi)
- Thụt lề khoảng trắng: Thụt lề quá nhiều (hơn 2 khoảng trắng) trong văn bản thường bị từ chối (lỗi) (khối mã miễn trừ)
- Không liên kết vô dụng: Các liên kết có văn bản bằng URL bị từ chối (lỗi)

### Quy tắc cảnh báo (hướng dẫn kiểu)

Chi tiết tại <docs/markdown-writing-rules.md>

### Xác thực tự động

Lệnh `write` xác thực nội dung trước khi ghi để đảm bảo markdown thân thiện với AI.

## Chạy Tests

```bash
cargo test
```

## Quy tắc cho LLMs/Agents: Sử dụng agent-md CLI

Khi làm việc với các tệp markdown, luôn ưu tiên công cụ CLI `agent-md` hơn là chỉnh sửa tệp trực tiếp.
Xem `docs/llm-agent-rule.md` để có hướng dẫn tích hợp hoàn chỉnh và các thực hành tốt nhất.

### Tại sao sử dụng agent-md?

- Đầu ra JSON: Tất cả các lệnh đều trả về JSON có cấu trúc để dễ phân tích
- Xác thực tích hợp: Xác thực markdown tự động đảm bảo nội dung thân thiện với AI
- Tối ưu hóa cho LLM: Được thiết kế đặc biệt cho các quy trình làm việc của AI agent
- Định dạng nhất quán: Thực thi các tiêu chuẩn markdown để dễ đọc hơn

### Mô hình sử dụng cốt lõi

```bash
# Đọc nội dung tệp
CONTENT=$(agent-md read <path> --field content)

# Đọc các phần cụ thể (không cần đọc toàn bộ tệp)
agent-md read <path> --content "## Tên Phần"
# Các phần lồng nhau: agent-md read <path> --content "## Development > Build"

# Luôn ghi tệp với agent-md (xác thực trước)
agent-md write <path> "<nội dung>"

# Ghi vào phần cụ thể
agent-md write-section <path> --section "## Phần" --content "<nội dung>"

# Tìm kiếm trong các tệp
agent-md search <path> "<truy vấn>"

# Lấy cấu trúc tài liệu
agent-md headings <path>
```

### Quy tắc tích hợp

QUY TẮC: Khi cần đọc, ghi, hoặc sửa đổi các tệp markdown, sử dụng các lệnh `agent-md` thay vì các thao tác tệp trực tiếp. Công cụ cung cấp dữ liệu có cấu trúc và đảm bảo chất lượng nội dung.

### Quy trình làm việc ví dụ

```bash
# 1. Lấy cấu trúc tài liệu
agent-md read README.md --field headings

# 2. Tìm kiếm nội dung cụ thể
agent-md search README.md "TODO"

# 3. Xác thực nội dung mới
agent-md lint --content "# Tiêu đề Mới\nNội dung ở đây"

# 4. Ghi nội dung đã xác thực
agent-md write README.md "# Tiêu đề Mới\nNội dung hợp lệ"
```

## Đọc tệp và trích xuất trường

Sử dụng tùy chọn --field (được khuyến nghị):
```bash
agent-md read README.md --field path # Lấy đường dẫn tệp
agent-md read README.md --field content # Lấy nội dung
agent-md read README.md --field headings # Lấy các heading
agent-md read README.md -f word_count # Form ngắn cho số từ
```

Đọc phần "Development" - không cần LLM đọc toàn bộ tệp:
```bash
agent-md read README.md -c="Development"
# Các phần lồng nhau: agent-md read README.md -c="Development > Build"
```

## Ví dụ sử dụng cho LLMs

```bash
# Tìm kiếm nội dung
agent-md search /path/to/file.md "TODO"
# ví dụ:
agent-md search README.md "TODO"

# Lấy tất cả các heading để điều hướng
agent-md headings /path/to/file.md
# ví dụ:
agent-md headings README.md

# Kiểm tra một tệp
agent-md lint README.md
# ví dụ:
agent-md lint README.md

# Kiểm tra với đầu ra dễ đọc
agent-md lint-file README.md

# Xác thực markdown trước khi ghi
agent-md lint --content "# Tiêu đề\nNội dung với văn bản **in đậm**"
agent-md write document.md "# Tiêu đề\nNội dung hợp lệ không có in đậm"
```

## Phát triển

Xem `docs/DEV.md` để có hướng dẫn phát triển hoàn chỉnh.

## Giấy phép

MIT
