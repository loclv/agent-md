# `agent-md` - Một công cụ CLI giúp bạn viết markdown thân thiện với LLM

<div align="center"><img src="logo.svg" alt="agent-md logo" width="128" height="128"><br /><div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 12px; margin-top: 22px;"><div style="margin-top: auto; margin-bottom: auto;"><a href="https://dopana.com" target="_blank">Sponsored by dopana.com</a></div><a href="https://unikorn.vn/p/agent-md?ref=embed-agent-md" target="_blank"><img src="https://unikorn.vn/api/widgets/badge/agent-md?theme=dark" alt="agent-md trên Unikorn.vn" style="width: 256px; height: 64px;" width="256" height="64" /></a></div></div>

<div align="center"><a href="https://marketplace.visualstudio.com/items?itemName=loclv.agent-md-formatter&ssr=false#overview" target="_blank">Extention in VS Code, Cursor, Devin, Antigravity</a></div>

Format markdown file cho LLM và Agents:

```bash
agent-md README.md
```

Một ví dụ cho bảng trước và sau khi format:

```diff
 | Function | Arguments | Return Type | Description                   |
-|----------|-----------|-------------|-------------------------------|
-| `add()`  | a, b      | i32         | Adds two numbers              |
-| `sub()`  | a, b      | i32         | Subtracts numbers             |
+|---|---|---|---|
+| `add()` | a, b | i32 | Adds two numbers |
+| `sub()` | a, b | i32 | Subtracts numbers |
```

Một ví dụ cho comment trong code block trước và sau khi format:

```diff
-bun i                      # Install dependencies
-bun run dev                # Run the development server
-bun run something-else     # Run something else
+bun i # Install dependencies
+bun run dev # Run the development server
+bun run something-else # Run something else
```

Một ví dụ về cây thư mục trước và sau khi format:

```diff
 example-folder/
-├── file-1.txt
-├── file-2.txt
-└── sub-folder/
-    ├── file-3.txt
-    └── file-4.txt
+├─file-1.txt
+├─file-2.txt
+└─sub-folder/
+  ├─file-3.txt
+  └─file-4.txt
```

- LLM đọc ít token hơn và nhanh hơn.
- Tiết kiệm token.
- Tiết kiệm tiền.

Kiểm tra các quy tắc trong file markdown cho LLM và Agents:

```bash
cd project-folder-name
# định dạng đệ quy thư mục hiện tại
agent-md .

agent-md lint README.md
# {"valid":false,"errors":[{"line":7,"column":1,"message":...
```

## Tại sao công cụ này tồn tại

Nhiều file markdown hiện nay được tạo bởi LLM hoặc AI agents đang lãng phí rất nhiều token. Khi một LLM khác đọc lại những file này, nó tiếp tục tốn thêm token không cần thiết. Thậm chí file này được đọc đi đọc lại mỗi lần chat.
Nguyên nhân là markdown được thiết kế để con người dễ đọc, nên thường chứa các yếu tố như in đậm, ký tự trang trí, khoảng trắng thừa... Những thứ này hữu ích cho người, nhưng không cần thiết với LLM.

Thực tế, LLM/agents không cần đọc toàn bộ file. Chúng chỉ cần truy cập đúng phần nội dung cần thiết (ví dụ: ## Development) thay vì xử lý cả tài liệu.

### Vấn đề

- Lãng phí token: Các định dạng như in đậm, bảng phức tạp và ký tự trang trí làm tăng số token mà không giúp ích cho AI
- Đọc không hiệu quả: LLM vẫn phải xử lý các yếu tố trực quan như bold và ASCII art
- Cấu trúc dư thừa: Nhiều thành phần chỉ hữu ích cho con người (bảng phức tạp, bố cục đẹp mắt) nhưng không cần thiết cho AI
- Chi phí cao hơn: Mỗi lần LLM đọc tài liệu, nó phải trả một khoản token cho định dạng hướng tới con người

### Giải pháp

`agent-md` đưa ra một tiêu chuẩn markdown tối giản, thân thiện với AI, giúp:

- Giảm các token không cần thiết
- Giữ nội dung có cấu trúc rõ ràng, dễ truy cập theo từng phần
- Vẫn đảm bảo tính dễ đọc cho con người khi cần

Mục tiêu: viết một lần, tối ưu cho cả con người và AI—không lãng phí tài nguyên.

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

Hoặc cài đặt trực tiếp qua Cargo:

```bash
cargo install agent-md
```

Hoặc cài đặt qua Homebrew (macOS và Linux):

```bash
brew tap loclv/tools
brew install agent-md
```

### Sử dụng như một thư viện Rust

Thêm `agent-md` vào `Cargo.toml`:

```toml
[dependencies]
agent-md = "x.y.z" # replace with actual version
```

Sử dụng các API lập trình trong ứng dụng của bạn:

```rust
use agent_md::{format_markdown, parse_markdown, validate_markdown};

let input = "# Tiêu đề\n\nVăn bản với *in nghiêng*.";
let formatted = format_markdown(input);

let result = validate_markdown(&formatted);
assert!(result.valid);
```

## Tài liệu

Tài liệu hướng dẫn chi tiết được sắp xếp trong thư mục `docs/`:

- [Hướng dẫn sử dụng CLI](docs/cli-usage-guide-vi.md): Danh mục lệnh đầy đủ, thao tác theo phần, tùy chọn định dạng và phân tích cú pháp có cấu trúc
- [Quy tắc cho LLM / Agent](docs/llm-agent-rule.md): Các mẫu tích hợp, đọc/ghi theo phần và quy trình làm việc cho AI agent
- [Quy tắc viết Markdown](docs/markdown-writing-rules.md): Tiêu chuẩn định dạng và quy tắc thân thiện với AI
- [Hướng dẫn cấu hình](docs/config.md): Thứ tự ưu tiên cấu hình và bảng tham chiếu tùy chọn
- [Tiện ích mở rộng VS Code](docs/vscode-extension.md): Hướng dẫn cài đặt, thiết lập cấu hình và phím tắt
- [Hướng dẫn phát triển](docs/DEV.md): Kiến trúc, biên dịch, kiểm thử và hướng dẫn đóng góp
- [Tài liệu tiếng Anh](README.md): Phiên bản tiếng Anh của tài liệu

## Cách dùng nhanh

```bash
# Định dạng một tệp markdown trực tiếp
agent-md fmt README.md

# Kiểm tra quy tắc tệp với đầu ra JSON
agent-md lint README.md

# Đọc riêng một phần theo đường dẫn tiêu đề
agent-md read README.md --content "## Cài đặt"

# Trích xuất các trường thông tin (path, content, word_count, line_count, headings)
agent-md read README.md --field headings

# Tìm kiếm trong tệp
agent-md search README.md "TODO"
```

Để xem toàn bộ hướng dẫn lệnh chi tiết, xem [Hướng dẫn sử dụng CLI](docs/cli-usage-guide-vi.md).

## Quy tắc cho LLMs/Agents: Sử dụng agent-md CLI

Khi làm việc với các tệp markdown, luôn ưu tiên công cụ CLI `agent-md` hơn là chỉnh sửa tệp trực tiếp.
Xem `docs/llm-agent-rule.md` để có hướng dẫn tích hợp hoàn chỉnh và các thực hành tốt nhất.

QUY TẮC: Khi cần đọc, ghi, hoặc sửa đổi các tệp markdown, sử dụng các lệnh `agent-md` thay vì các thao tác tệp trực tiếp. Công cụ cung cấp dữ liệu có cấu trúc và đảm bảo chất lượng nội dung.

Khi hoàn thành tác vụ, bạn phải cập nhật tài liệu (`docs/`), cập nhật `README.md`, và viết unit tests cho các thay đổi của bạn.

```bash
# Quy trình cốt lõi
agent-md read README.md --field headings
agent-md search README.md "TODO"
agent-md lint --content "# Tiêu đề Mới\nNội dung ở đây"
agent-md write README.md "# Tiêu đề Mới\nNội dung hợp lệ"
```

## Tiện ích mở rộng VS Code

`agent-md` có sẵn dưới dạng tiện ích mở rộng cho VS Code để định dạng các tệp Markdown theo yêu cầu hoặc tự động khi lưu.

- Cài đặt từ [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=loclv.agent-md-formatter).
- Xem [Hướng dẫn tiện ích mở rộng VS Code](docs/vscode-extension.md) để biết thêm về cấu hình và cài đặt từ mã nguồn.

## Phát triển

Xem `docs/DEV.md` để có hướng dẫn phát triển hoàn chỉnh.

```bash
cargo build --release # Xây dựng phiên bản release
cargo test # Chạy tests
cargo fmt # Định dạng mã nguồn
cargo fmt --check # Kiểm tra định dạng mã nguồn
cargo clippy --all-targets --all-features -- -D warnings # Chạy clippy lints
cargo audit # Kiểm tra bảo mật
```

## Giấy phép

MIT
