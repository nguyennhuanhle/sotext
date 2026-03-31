# 📝 SoText — Text Similarity Detector / Công cụ Phát hiện Tương đồng Văn bản

> **SoText** is a desktop application built with **Tauri + Rust** for fast, offline text similarity detection. It helps educators, researchers, and content reviewers identify copied or paraphrased content across multiple documents.
>
> **SoText** là ứng dụng desktop được xây dựng bằng **Tauri + Rust**, phát hiện tương đồng văn bản nhanh chóng và hoạt động offline. Hỗ trợ giáo viên, nhà nghiên cứu và người kiểm duyệt nội dung phát hiện sao chép hoặc diễn giải lại.

## 📥 Download / Tải về

| Installer | Link |
|---|---|
| **Windows Setup (EXE)** | [SoText_1.3.0_x64-setup.exe](https://github.com/nguyennhuanhle/sotext/releases/download/v1.3.0/SoText_1.3.0_x64-setup.exe) |
| **Windows Installer (MSI)** | [SoText_1.3.0_x64_en-US.msi](https://github.com/nguyennhuanhle/sotext/releases/download/v1.3.0/SoText_1.3.0_x64_en-US.msi) |
| **macOS (Apple Silicon)** | [SoText.app.zip](https://github.com/nguyennhuanhle/sotext/releases/download/v1.3.0/SoText.app.zip) |

👉 [All releases / Tất cả phiên bản](https://github.com/nguyennhuanhle/sotext/releases)

---

## ✨ Key Features / Tính năng chính

### 📁 Multi-format Support / Hỗ trợ đa định dạng
- **`.txt`** — Plain text files (UTF-8) / File văn bản thuần
- **`.docx`** — Microsoft Word documents / Tài liệu Word
- **`.pdf`** — PDF documents (text extracted) / Tài liệu PDF (trích xuất văn bản)
- **`.html / .htm`** — Web pages (tags stripped) / Trang web (loại bỏ thẻ HTML)

### 🧠 4-Level Detection Algorithms / 4 thuật toán phát hiện

| Level | Algorithm / Thuật toán | Detects / Phát hiện |
|:---:|---|---|
| 1 | **Normalized MD5 Hash** | Exact copies with minor formatting changes / Bản sao y hệt chỉ khác định dạng |
| 2 | **TF-IDF + Cosine Similarity** with Snowball Stemmer | Similar vocabulary across tenses, plurals / Từ vựng tương tự dù thay đổi thì, số nhiều |
| 3 | **N-gram Matching** (configurable size) | Word-for-word copied passages / Đoạn sao chép nguyên văn |
| 4 | **Sentence-level Analysis** — Jaccard Similarity + Levenshtein Distance | Paraphrased & reordered text / Câu diễn giải lại hoặc đảo thứ tự |

### 📋 Template Exclusion / Loại trừ đề bài
- Upload a prompt/question file to exclude shared content from comparison, reducing false positives.
- Tải file đề bài/câu hỏi để loại trừ nội dung chung, giảm kết quả dương tính giả.

### 🔍 Side-by-Side Comparison / So sánh song song
- Click any result row to view a detailed side-by-side comparison with color-coded highlighting:
  - 🟡 **Yellow** — Exact match (N-gram) / Khớp chính xác
  - 🟠 **Orange** — Paraphrased (Jaccard/Levenshtein) / Diễn giải lại

### 📊 Export Results / Xuất kết quả
- **Excel (.xlsx)** — Multi-sheet workbook with summary + highlighted comparisons / Bảng tính nhiều sheet với tóm tắt + so sánh highlight
- **HTML Report** — Full report with highlighted comparisons / Báo cáo đầy đủ với highlight so sánh
- **PDF Report** — Professional PDF with color-coded highlights / Báo cáo PDF chuyên nghiệp với highlight màu
- **DOCX Report** — Word document with rich-text highlighted comparisons / Tài liệu Word với so sánh highlight rich-text

### ⚙️ Configurable Parameters / Thông số tùy chỉnh
- **Threshold (10–100%)** — Filter results by minimum Cosine Score / Lọc kết quả theo điểm Cosine tối thiểu
- **N-gram size (3–10)** — Adjust exact phrase detection granularity / Điều chỉnh độ chi tiết phát hiện cụm từ

### 🌐 Bilingual UI / Giao diện song ngữ
- Switch between English and Vietnamese with one click / Chuyển đổi Anh–Việt chỉ bằng một cú nhấp

---

## 🚀 Quick Start / Bắt đầu nhanh

1. **Browse** — Select a folder containing your files / Chọn thư mục chứa các file
2. **Template** _(optional)_ — Upload a prompt file to exclude / Tải đề bài để loại trừ
3. **Scan** — Click 🔍 to start analysis / Nhấn 🔍 để bắt đầu phân tích
4. **Review** — Click any row for side-by-side details / Nhấn vào hàng để xem chi tiết
5. **Export** — Save results as Excel, HTML, PDF, or DOCX / Lưu kết quả ra Excel, HTML, PDF, hoặc DOCX

---

## 🛠 Tech Stack / Công nghệ

| Component | Technology |
|---|---|
| Frontend | HTML, CSS, Vanilla JS |
| Backend | Rust (Tauri v2) |
| Stemming | Snowball Stemmer (`rust-stemmers`) |
| String Similarity | `strsim` (Levenshtein, Jaccard) |
| DOCX Parsing | `dotext` |
| PDF Parsing | `pdf-extract` |
| HTML Parsing | `scraper` |
| Hashing | `md-5` |
| Export | `rust_xlsxwriter`, `genpdf`, `docx-rs` |

---

## 📦 Development / Phát triển

### Prerequisites / Yêu cầu
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

### Run in Dev Mode / Chạy chế độ phát triển
```bash
npm run tauri dev
```

### Build for Production / Build bản phát hành
```bash
npm run tauri build
```

---

## 📸 Screenshots / Ảnh chụp màn hình

> _Coming soon / Sắp cập nhật_

---

## 👨‍💻 Author / Tác giả

Developed by [Mr Le Nguyen Nhu Anh](https://edtechcorner.com/) © 2026

---

## 📄 License / Giấy phép

MIT License
