# Hướng dẫn sử dụng CLI

`agent-md` cung cấp một CLI mạnh mẽ cho cả quy trình làm việc của con người và AI. Nó hỗ trợ ba chế độ chính: kiểm tra quy tắc (linting), định dạng (formatting) và hỗ trợ viết cho AI.

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
# {"line":3,"message":"No bold text allowed","rule":"no-bold"},
# {"line":6,"message":"Complex table detected","rule":"simple-table"},
# {"line":15,"message":"No bold text allowed","rule":"no-bold"}
# ]}

# Kiểm tra file agent-md
agent-md lint agent-md-markdown.md
# {"valid":true,"errors":[],"warnings":[]}
```

Lợi ích:

- Trong một số trường hợp, giảm ~20% số token không cần thiết
- LLM đọc và xử lý nhanh hơn
- Vẫn giữ được thông tin đầy đủ
- Dễ dàng trích xuất phần cụ thể

## Thêm quy tắc hoặc lệnh cho LLM/Agents tuân theo

Ví dụ, khi thêm quy tắc mới, thêm vào tệp `AGENTS.md`.
>Sau khi tạo hoặc cập nhật bất kỳ tệp markdown nào, luôn chạy `agent-md lint path/to/file.md` để xác thực nội dung trước khi coi tác vụ hoàn thành.
>
>Khi hoàn thành tác vụ, bạn phải cập nhật tài liệu (`docs/`), cập nhật `README.md`, và viết unit tests cho các thay đổi của bạn.

Hoặc hướng dẫn LLM/Agents:
>Sử dụng `agent-md` CLI để chạy lint

## Các lệnh (đầu ra JSON thân thiện với LLM)

Tất cả các lệnh đều trả về JSON để dễ phân tích. Sử dụng cờ `--human` trước lệnh con để xuất dữ liệu định dạng đẹp:

```bash
agent-md --human lint README.md
agent-md --human read README.md --field headings
```

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

Liệt kê các tệp Markdown trong thư mục được chỉ định (mặc định là `.`). Tự động bỏ qua các tệp và thư mục khớp với `.markdownlintignore` và `.gitignore`.

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

### Thư mục làm việc cho cấu hình

Sử dụng tùy chọn toàn cục `--cwd` để chỉ định thư mục làm việc cho việc khám phá cấu hình. Điều này cho phép bạn đọc các tệp cấu hình từ một thư mục khác với thư mục làm việc hiện tại.

```bash
agent-md --cwd <path> <command> <args>
# Đọc tệp cấu hình từ thư mục được chỉ định

agent-md --cwd /path/to/project lint document.md
# Lint document.md bằng cấu hình từ /path/to/project

agent-md --cwd /path/to/project fmt document.md
# Định dạng document.md bằng cấu hình từ /path/to/project
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

### Khởi tạo cấu hình

Khởi tạo tệp cấu hình mặc định `.agent-md.json` tại thư mục hiện tại hoặc đường dẫn được chỉ định.

```bash
agent-md init
# Khởi tạo .agent-md.json trong thư mục hiện tại

agent-md init custom.json
# Khởi tạo cấu hình tại đường dẫn tệp cụ thể

agent-md init path/to/dir
# Khởi tạo .agent-md.json bên trong thư mục được chỉ định

agent-md init --force
# Ghi đè tệp cấu hình đã tồn tại (hỗ trợ cờ -f)

agent-md config --init
# Tùy chọn thay thế qua lệnh con config với cờ --force tùy chọn
```

### Kiểm tra cấu hình

Kiểm tra độ ưu tiên phân giải và trạng thái tệp cấu hình. Các tệp cấu hình được ưu tiên theo thứ tự sau:

1. `.agent-md.json`
2. `agent-md.json`
3. Dự phòng `markdownlintrc.*` (`.markdownlint.json`, `.markdownlint.jsonc`, `.markdownlint.yaml`, `.markdownlint.yml`, `.markdownlintrc`, `.markdownlintrc.json`)

Khi xử lý một tệp Markdown đích, `agent-md` tự động tìm kiếm thư mục cha của tệp và duyệt ngược lên các thư mục cha để tìm cấu hình, dự phòng về thư mục làm việc hiện tại trừ khi được ghi đè bằng `--config` hoặc `--cwd`.

Sử dụng cờ toàn cục `--cwd <path>` để chỉ định thư mục làm việc cho việc tìm kiếm cấu hình. Điều này hữu ích khi bạn muốn đọc cấu hình từ một thư mục khác với thư mục hiện tại.

Sử dụng cờ toàn cục `--ignore-markdownlintrc` để bỏ qua các tệp `markdownlintrc.*` trong quá trình tìm kiếm (mặc định là `false`, tức là được dùng làm dự phòng). Tùy chọn này cũng có thể đặt trong `agent-md.json` qua khóa `ignore-markdownlintrc` (hoặc `ignore_markdownlintrc`); cờ CLI luôn có độ ưu tiên cao hơn, và chỉ tệp gốc `agent-md.json` mới được tham khảo cho khóa này.

Tệp cấu hình mẫu được cung cấp tại `samples/.agent-md.json`.

```bash
agent-md config
# Trả về: {exists, path, config}

agent-md config --check
# Trả về: {exists, path}

agent-md config samples
# Kiểm tra cấu hình trong thư mục

agent-md config path/to/document.md
# Kiểm tra cấu hình đã giải quyết cho tệp cụ thể

agent-md config custom.json
# Kiểm tra đường dẫn cấu hình cụ thể

agent-md --config samples/.agent-md.json fmt document.md
# Sử dụng tệp cấu hình cụ thể qua cờ toàn cục --config

agent-md --ignore-markdownlintrc lint document.md
# Bỏ qua tệp markdownlintrc.* và chỉ sử dụng cấu hình của agent-md

# Hoặc đặt trong agent-md.json:
# { "ignore-markdownlintrc": true }

agent-md --cwd /path/to/project lint document.md
# Đọc tệp cấu hình từ thư mục /path/to/project

agent-md --cwd /path/to/project fmt document.md
# Định dạng document.md sử dụng cấu hình từ /path/to/project
```

### Kiểm tra quy tắc bỏ qua (ignore rules)

Đọc tệp `.markdownlintignore` nếu tồn tại trong thư mục đích, hợp nhất với danh sách Git ignore (`.gitignore`) và loại bỏ các mục trùng lặp:

```bash
agent-md ignore
# Trả về: ["dist/", "logs/", "target/", "/target", "temp/", "*.log", "*.tgz", ".antigravitycli"]

agent-md ignore path/to/dir
# Trả về các mẫu bị bỏ qua cho thư mục được chỉ định

agent-md --human ignore
# Mảng JSON định dạng đẹp
```

Các lệnh như `list` và `fmt` ở cấp thư mục sẽ tự động tuân theo các quy tắc này để bỏ qua các thư mục và tệp không cần thiết.

### Định dạng markdown

- Định dạng tệp markdown tại chỗ, loại bỏ khoảng trắng ở đầu và cuối ô bảng.
- Bảo toàn dấu gạch đứng thoát (`\|`) và dấu gạch đứng bên trong đoạn mã nội dòng trong ô bảng mà không tách ô hay chèn khoảng trắng thừa.
- Định dạng các ô bảng trống thành một khoảng trắng đơn (`| |`) để duy trì cấu trúc bảng sạch sẽ.
- Chuẩn hóa các hàng phân tách bảng (ví dụ: `|:---|:---|` thành `|---|---|`), loại bỏ các dấu hai chấm căn lề để tiết kiệm token.
- Loại bỏ dấu hai chấm ở cuối tiêu đề (ví dụ: `## header:` thành `## header`).
- Tự động thêm thẻ ngôn ngữ `text` cho các khối mã chưa khai báo ngôn ngữ (ví dụ: ` ``` ` thành ` ```text `), bao gồm cả các khối mã lồng trong mục danh sách.
- Loại bỏ các mục danh sách rỗng (ví dụ: dòng `- ` ở cuối không có nội dung).
- Bảo toàn nội dung khối mã, bao gồm cả thụt lề tương đối và cú pháp bên trong các khối mã lồng nhau trong mục danh sách.
- Thu gọn nhiều khoảng trắng trước comment `#` trong các khối mã shell (`bash`, `sh`, `shell`, `zsh`).
- Định dạng cấu trúc thư mục trong các khối mã `text`, `txt`, hoặc không gắn nhãn bằng cách loại bỏ các gạch ngang thừa `─` (ví dụ: `├──` thành `├─`, `└──` thành `└─`), loại bỏ các dòng đệm dọc (`│`), xóa khoảng trắng trước tên tệp, chuẩn hóa thụt lề 4 khoảng trắng thành 2 khoảng trắng cho các nhánh lồng nhau và thu gọn khoảng trắng trước comment.
- Tự động chuyển đổi 4 khoảng trắng thụt lề đầu dòng danh sách thành 2 khoảng trắng, và 2 tab đầu dòng thành 1 tab cho các mục con, giảm số token sử dụng trong các danh sách lồng nhau.
- Tuân theo `.markdownlintignore` và `.gitignore` khi định dạng thư mục, bỏ qua các thư mục như `target/`, `dist/`, `logs/` và các tệp tạm thời.

```bash
agent-md fmt <path>
# Trả về dữ liệu JSON: {success, message, document}
```

#### Định dạng comment trong khối mã

Đối với các khối mã, trình định dạng tự động thu gọn các khoảng trắng thừa trước dấu comment `#` trong khi vẫn bảo toàn thụt lề:

~~~text
Sau khi định dạng:

```bash
echo hello # this is a comment
    # indented comment
```

~~~

#### Định dạng cấu trúc thư mục

Đối với các khối mã có ngôn ngữ `text`, `txt`, hoặc không có nhãn ngôn ngữ chứa cây cấu trúc thư mục, trình định dạng sẽ thu gọn các ký hiệu nhánh thành tiền tố một gạch ngang (`├─` và `└─`), loại bỏ các dòng phân cách dọc (`│`), và căn chỉnh các comment:

~~~text

```text
data/
├─input/ # input
├─output/ # output
└─logs/ # logs
```

~~~

#### Bảo toàn khối mã nội dòng

Các khối mã nội dòng đặt trong dấu backtick được giữ nguyên y như cũ mà không bị thay đổi định dạng:

~~~text
khối mã: `let a = 1;`
~~~

Đầu ra giữ nguyên không đổi.

#### Định dạng tùy chọn

Trình định dạng áp dụng các quy tắc thu gọn theo mặc định để giảm số lượng token. Các tùy chọn này có thể cấu hình trong `.agent-md.json` hoặc `agent-md.json` (dùng kebab-case hoặc snake_case, ở cấp cao nhất hoặc trong đối tượng `format`) và có thể ghi đè qua các cờ CLI:

| Tùy chọn | Mô tả | Mặc định |
|---|---|---|
| `remove_bold` | Xóa các dấu `**bold**` và `__bold__` | `true` |
| `compact_blank_lines` | Thu gọn nhiều dòng trống liên tiếp (giữ lại các dòng trống đơn quanh tiêu đề) | `true` |
| `collapse_spaces` | Thu gọn nhiều khoảng trắng giữa các từ | `true` |
| `remove_horizontal_rules` | Xóa các dòng `---`, `***`, `___` | `true` |
| `remove_emphasis` | Xóa các dấu `*italic*` và `_italic_` | `true` |
| `blanks_around_lists` | Đảm bảo danh sách được bao quanh bởi các dòng trống (cấu hình trong `.agent-md.json` hoặc `.markdownlint.json`) | `true` |
| `blanks_around_fences` | Đảm bảo các khối mã được bao quanh bởi các dòng trống (cấu hình trong `.agent-md.json` hoặc `.markdownlint.json`) | `true` |
| `blanks_around_headings` | Đảm bảo tiêu đề được bao quanh bởi các dòng trống (cấu hình trong `.agent-md.json` hoặc `.markdownlint.json`) | `true` |
| `minify_html` | Thu nhỏ các thẻ và khối HTML bằng cách loại bỏ khoảng trắng và dòng mới không cần thiết | `true` |

Ví dụ:

```bash
agent-md fmt document.md
agent-md fmt --remove-bold=false document.md
```

##### Thu nhỏ HTML

Các khối và thẻ HTML được thu nhỏ để loại bỏ lãng phí token:

- Loại bỏ khoảng trắng, tab và dòng mới không cần thiết bên trong thẻ HTML
- Loại bỏ thụt đầu dòng thừa bên trong các khối HTML
- Thu gọn từng khối HTML về một dòng duy nhất, nối các ranh giới thẻ trực tiếp và nối các dòng văn bản bằng một khoảng trắng đơn
- Giữ nguyên ngắt dòng bên trong các khối `pre`, `code`, `textarea`, `script` và `style` nơi khoảng trắng có ý nghĩa cú pháp
- Giữ nguyên tên thẻ HTML gốc, các thuộc tính và giá trị thuộc tính
- Giữ nguyên các autolink Markdown và URL thuần (ví dụ: `<https://...>`, `<user@example.com>`, `<example.dev/path/>`)
- Đặt `minify-html` (hoặc `minify_html`) thành `false` trong `agent-md.json`, hoặc truyền `--minify-html=false`, để giữ nguyên các khối HTML không thay đổi

Đầu vào:

```html
<p align="center">
  <img src="badge.png" alt="Markdown" />
</p>
```

Đầu ra:

```html
<p align="center"><img src="badge.png" alt="Markdown" /></p>
```

## Cách thức hoạt động: Phân tích cấu trúc (Structured Parsing)

Khác với các trình định dạng theo dòng đơn giản, `agent-md` sử dụng bộ phân tích cấu trúc để:

1. Trích xuất YAML Frontmatter: Giữ nguyên metadata ở đầu tài liệu như ban đầu.
2. Nhận diện các khối tài liệu: Nhận diện tiêu đề, khối mã, bảng, danh sách, khối HTML và đoạn văn.
3. Áp dụng định dạng theo ngữ cảnh: Định dạng từng khối dựa trên loại của nó và các tùy chọn cấu hình của bạn.
4. Tối ưu hóa cho LLM: Đảm bảo đầu ra sạch sẽ, nhất quán và tiết kiệm token trong khi vẫn dễ đọc cho con người.
5. Tăng tốc phần cứng bằng SIMD: Tận dụng véc-tơ hóa đa nền tảng (ARM NEON và x86_64 AVX2/SSE2) để tăng tốc độ đếm dòng mới, quét tìm kiếm và kiểm tra quy tắc thoát sớm.

## Quy tắc xác thực

Bộ linter thực thi các tiêu chuẩn markdown thân thiện với AI.

### Quy tắc lỗi (nội dung khối)

- "no-bold": Không văn bản in đậm: `**bold**` và `__bold__` bị từ chối, ngoại trừ trong khối mã
- "heading-structure": Cấu trúc heading: Nhiều heading H1 và các cấp heading bị bỏ qua bị từ chối
- "table-syntax": Cú pháp bảng: Thuộc tính bảng phức tạp và định dạng hàng phân cách không chính xác bị từ chối
- "simple-table-syntax": Cú pháp bảng đơn giản: Bảng rất rộng và định dạng inline trong ô bảng bị từ chối
- "table-trailing-spaces": Khoảng trắng cuối ô bảng: Ô bảng có nhiều hơn 1 khoảng trắng ở cuối bị từ chối
- "no-ascii-graphs": Không đồ họa ASCII: Ký tự vẽ hộp và các mẫu trực quan bị từ chối, kể cả trong khối mã
- "code-blocks": Xác thực khối mã: Khối mã không có khai báo ngôn ngữ hoặc thiếu rào chắn đóng (khối mã chưa đóng) bị từ chối, với các khối chưa đóng sẽ dừng ngay lập tức quá trình định dạng và kiểm tra
- "list-formatting": Định dạng danh sách: Ký tự đầu dòng danh sách và đánh số không nhất quán bị từ chối
- "space-indentation": Thụt lề khoảng trắng: Thụt lề quá mức (nhiều hơn 2 khoảng trắng) trong văn bản thông thường bị từ chối (khối mã được miễn trừ)
- "no-useless-links": Không liên kết vô dụng: Liên kết có văn bản hiển thị trùng với URL bị từ chối

### Quy tắc cảnh báo (hướng dẫn phong cách)

- "no-duplicate-headings": Không tiêu đề trùng lặp: Các tiêu đề có cùng nội dung sẽ bị cảnh báo
- "no-multiple-blanks": Không nhiều dòng trống liên tiếp: Nhiều dòng trống liên tiếp sẽ bị cảnh báo

Chi tiết tại [docs/markdown-writing-rules.md](./markdown-writing-rules.md)

### Xác thực tự động

Lệnh `write` xác thực nội dung trước khi ghi để đảm bảo markdown thân thiện với AI.
