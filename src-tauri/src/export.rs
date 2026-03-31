//! Excel, HTML, and PDF export functionality.

use crate::analysis::{DetailResult, SimilarityPair};
use rust_xlsxwriter::*;
use std::path::Path;

/// Export results to Excel file with summary + detail sheets with highlighted text
pub fn export_excel(
    results: &[SimilarityPair],
    details: &[DetailResult],
    filepath: &str,
) -> Result<(), String> {
    let mut workbook = Workbook::new();

    // ─── Reusable formats ────────────────────────────────────
    let header_format = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2F5496))
        .set_align(FormatAlign::Center)
        .set_font_size(11);

    let high_format = Format::new()
        .set_background_color(Color::RGB(0xFFC7CE));
    let med_format = Format::new()
        .set_background_color(Color::RGB(0xFFEB9C));
    let score_center = Format::new()
        .set_align(FormatAlign::Center);
    let high_score = Format::new()
        .set_background_color(Color::RGB(0xFFC7CE))
        .set_align(FormatAlign::Center);
    let med_score = Format::new()
        .set_background_color(Color::RGB(0xFFEB9C))
        .set_align(FormatAlign::Center);

    // ═══ Sheet 1: Summary ════════════════════════════════════
    let summary = workbook.add_worksheet();
    summary
        .set_name("Summary")
        .map_err(|e| e.to_string())?;

    let headers = ["File A", "File B", "Cosine Score"];
    for (col, &h) in headers.iter().enumerate() {
        summary
            .write_string_with_format(0, col as u16, h, &header_format)
            .map_err(|e| e.to_string())?;
    }

    for (i, pair) in results.iter().enumerate() {
        let row = (i + 1) as u32;
        if pair.score >= 80.0 {
            summary.write_string_with_format(row, 0, &pair.file_a, &high_format).map_err(|e| e.to_string())?;
            summary.write_string_with_format(row, 1, &pair.file_b, &high_format).map_err(|e| e.to_string())?;
            summary.write_number_with_format(row, 2, pair.score, &high_score).map_err(|e| e.to_string())?;
        } else if pair.score >= 60.0 {
            summary.write_string_with_format(row, 0, &pair.file_a, &med_format).map_err(|e| e.to_string())?;
            summary.write_string_with_format(row, 1, &pair.file_b, &med_format).map_err(|e| e.to_string())?;
            summary.write_number_with_format(row, 2, pair.score, &med_score).map_err(|e| e.to_string())?;
        } else {
            summary.write_string(row, 0, &pair.file_a).map_err(|e| e.to_string())?;
            summary.write_string(row, 1, &pair.file_b).map_err(|e| e.to_string())?;
            summary.write_number_with_format(row, 2, pair.score, &score_center).map_err(|e| e.to_string())?;
        }
    }

    summary.set_column_width(0, 40).map_err(|e| e.to_string())?;
    summary.set_column_width(1, 40).map_err(|e| e.to_string())?;
    summary.set_column_width(2, 18).map_err(|e| e.to_string())?;

    // ═══ Sheet 2: Details (all pairs in one sheet) ════════════
    if !details.is_empty() {
        let label_format = Format::new()
            .set_bold()
            .set_font_size(11)
            .set_background_color(Color::RGB(0xD6E4F0));
        let pair_header_format = Format::new()
            .set_bold()
            .set_font_size(12)
            .set_font_color(Color::White)
            .set_background_color(Color::RGB(0x2F5496));
        let info_format = Format::new()
            .set_italic()
            .set_font_size(10);
        let wrap_format = Format::new()
            .set_text_wrap()
            .set_align(FormatAlign::Top)
            .set_font_size(10);
        let exact_fmt = Format::new()
            .set_bold()
            .set_font_color(Color::RGB(0x7B5B00))
            .set_background_color(Color::RGB(0xFECA57))
            .set_font_size(10);
        let para_fmt = Format::new()
            .set_bold()
            .set_font_color(Color::RGB(0x6B2D00))
            .set_background_color(Color::RGB(0xFF9F43))
            .set_font_size(10);
        let normal_fmt = Format::new()
            .set_font_size(10);
        let legend_exact = Format::new()
            .set_bold()
            .set_font_color(Color::RGB(0x7B5B00))
            .set_background_color(Color::RGB(0xFECA57));
        let legend_para = Format::new()
            .set_bold()
            .set_font_color(Color::RGB(0x6B2D00))
            .set_background_color(Color::RGB(0xFF9F43));

        let ws = workbook.add_worksheet();
        ws.set_name("Details").map_err(|e| e.to_string())?;
        ws.set_column_width(0, 120).map_err(|e| e.to_string())?;

        // Legend at top
        let legend_bold = Format::new().set_bold();
        let legend_normal = Format::new();
        let legend_normal2 = Format::new();
        ws.write_rich_string(0, 0, &[
            (&legend_bold, "Legend: "),
            (&legend_exact, " EXACT MATCH "),
            (&legend_normal, " = N-gram   "),
            (&legend_para, " PARAPHRASED "),
            (&legend_normal2, " = Jaccard/Levenshtein"),
        ]).map_err(|e| e.to_string())?;

        let mut row: u32 = 2;

        for (i, detail) in details.iter().enumerate() {
            let score = results.get(i).map(|p| p.score).unwrap_or(0.0);

            // ── Pair header ──
            ws.write_string_with_format(row, 0,
                &format!("━━━ Pair {} ━━━  {} ↔ {}  ━━━  Cosine: {:.1}%", i + 1, detail.file_a, detail.file_b, score),
                &pair_header_format,
            ).map_err(|e| e.to_string())?;
            ws.set_row_height(row, 24.0).map_err(|e| e.to_string())?;
            row += 1;

            // ── File A ──
            ws.write_string_with_format(row, 0,
                &format!("📄 {}", detail.file_a), &label_format,
            ).map_err(|e| e.to_string())?;
            row += 1;

            write_highlighted_cell(
                ws, row, 0,
                &detail.content_a,
                &detail.highlights_a,
                &detail.suspicious_sentences,
                true,
                &normal_fmt, &exact_fmt, &para_fmt, &wrap_format,
            )?;
            row += 2;

            // ── File B ──
            ws.write_string_with_format(row, 0,
                &format!("📄 {}", detail.file_b), &label_format,
            ).map_err(|e| e.to_string())?;
            row += 1;

            write_highlighted_cell(
                ws, row, 0,
                &detail.content_b,
                &detail.highlights_b,
                &detail.suspicious_sentences,
                false,
                &normal_fmt, &exact_fmt, &para_fmt, &wrap_format,
            )?;
            row += 1;

            // Info row
            ws.write_string_with_format(row, 0,
                &format!("N-gram matches: {} | Suspicious sentences: {}",
                    detail.common_phrase_count,
                    detail.suspicious_sentences.len()),
                &info_format,
            ).map_err(|e| e.to_string())?;
            row += 3; // spacing between pairs
        }
    }

    workbook
        .save(Path::new(filepath))
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Write document content with highlighted spans into a single cell using RichString
fn write_highlighted_cell(
    ws: &mut Worksheet,
    row: u32,
    col: u16,
    content: &str,
    ngram_ranges: &[[usize; 2]],
    suspicious: &[crate::sentence::SuspiciousPair],
    is_a: bool,
    normal_fmt: &Format,
    exact_fmt: &Format,
    para_fmt: &Format,
    wrap_format: &Format,
) -> Result<(), String> {
    // Collect all ranges
    let mut all_ranges: Vec<(usize, usize, &str)> = Vec::new();

    for [s, e] in ngram_ranges {
        all_ranges.push((*s, *e, "exact"));
    }
    for pair in suspicious {
        let [s, e] = if is_a { pair.pos_a } else { pair.pos_b };
        let already_covered = ngram_ranges.iter().any(|[ns, ne]| *ns <= s && *ne >= e);
        if !already_covered {
            all_ranges.push((s, e, "para"));
        }
    }

    // Truncate for Excel cell limits (~32k chars)
    let max_len = content.len().min(30000);
    let content_slice = &content[..max_len];

    if all_ranges.is_empty() {
        ws.write_string_with_format(row, col, content_slice, wrap_format)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    all_ranges.sort_by_key(|r| r.0);

    // Build rich string segments: Vec<(&Format, &str)>
    let mut segments: Vec<(&Format, String)> = Vec::new();
    let mut last_end = 0;

    for (start, end, kind) in &all_ranges {
        let start = *start;
        let end = (*end).min(max_len);
        if start >= max_len { break; }
        if start < last_end { continue; }

        if start > last_end {
            let normal_text = &content_slice[last_end..start];
            if !normal_text.is_empty() {
                segments.push((normal_fmt, normal_text.to_string()));
            }
        }

        let fmt = if *kind == "exact" { exact_fmt } else { para_fmt };
        segments.push((fmt, content_slice[start..end].to_string()));
        last_end = end;
    }

    if last_end < max_len {
        let tail = &content_slice[last_end..];
        if !tail.is_empty() {
            segments.push((normal_fmt, tail.to_string()));
        }
    }

    let truncate_fmt = Format::new().set_italic().set_font_size(9);

    if content.len() > max_len {
        segments.push((
            &truncate_fmt,
            format!("\n[... truncated, {} chars total]", content.len()),
        ));
    }

    // Convert to borrowed refs for write_rich_string
    let rich_refs: Vec<(&Format, &str)> = segments.iter().map(|(f, s)| (*f, s.as_str())).collect();

    ws.write_rich_string_with_format(row, col, &rich_refs, wrap_format)
        .map_err(|e| e.to_string())?;

    // Auto-size row height roughly (Excel doesn't do this automatically for rich strings)
    let line_count = content_slice.matches('\n').count() + 1;
    let estimated_height = (line_count as f64 * 14.0).min(400.0).max(60.0);
    ws.set_row_height(row, estimated_height).map_err(|e| e.to_string())?;

    Ok(())
}

/// Export an HTML comparison report with highlighted matching text
pub fn export_html_report(
    results: &[SimilarityPair],
    details: &[DetailResult],
    filepath: &str,
) -> Result<(), String> {
    let mut html = String::from(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>SoText — Similarity Report</title>
<style>
  body { font-family: 'Segoe UI', Arial, sans-serif; background: #f5f5f5; color: #333; margin: 0; padding: 20px; }
  h1 { color: #2F5496; border-bottom: 2px solid #2F5496; padding-bottom: 8px; }
  h2 { color: #444; margin-top: 30px; border-bottom: 1px solid #ccc; padding-bottom: 4px; }
  .summary-table { width: 100%; border-collapse: collapse; margin-bottom: 30px; }
  .summary-table th { background: #2F5496; color: white; padding: 10px; text-align: left; }
  .summary-table td { padding: 8px 10px; border-bottom: 1px solid #ddd; }
  .summary-table tr:nth-child(even) { background: #f0f0f0; }
  .score-high { background: #FFC7CE !important; }
  .score-med { background: #FFEB9C !important; }
  .pair-section { background: white; border-radius: 8px; padding: 20px; margin-bottom: 24px; box-shadow: 0 1px 4px rgba(0,0,0,0.1); }
  .compare-container { display: flex; gap: 16px; }
  .compare-panel { flex: 1; background: #fafafa; border: 1px solid #e0e0e0; border-radius: 4px; padding: 12px; overflow-x: auto; }
  .compare-panel h3 { margin: 0 0 8px 0; font-size: 14px; color: #555; }
  .compare-panel pre { white-space: pre-wrap; word-break: break-word; font-size: 12px; line-height: 1.6; font-family: 'Consolas', monospace; }
  .hl-exact { background: #feca57; padding: 0 2px; border-radius: 2px; }
  .hl-para { background: #ff9f43; padding: 0 2px; border-radius: 2px; }
  .legend { margin: 8px 0 12px; font-size: 12px; color: #666; }
  .legend span { display: inline-block; width: 14px; height: 14px; vertical-align: middle; margin-right: 4px; border-radius: 2px; }
  .legend .l-exact { background: #feca57; }
  .legend .l-para { background: #ff9f43; }
  @media print { body { background: white; } .pair-section { box-shadow: none; page-break-inside: avoid; } }
</style>
</head>
<body>
<h1>📝 SoText — Similarity Report</h1>
<p>Generated: "#);

    html.push_str(&chrono_now());
    html.push_str("</p>\n");

    // Summary table
    html.push_str("<h2>Summary</h2>\n<table class=\"summary-table\">\n<tr><th>#</th><th>File A</th><th>File B</th><th>Cosine Score</th></tr>\n");
    for (i, pair) in results.iter().enumerate() {
        let class = if pair.score >= 80.0 {
            " class=\"score-high\""
        } else if pair.score >= 60.0 {
            " class=\"score-med\""
        } else {
            ""
        };
        html.push_str(&format!(
            "<tr{}><td>{}</td><td>{}</td><td>{}</td><td>{:.1}%</td></tr>\n",
            class,
            i + 1,
            escape_html(&pair.file_a),
            escape_html(&pair.file_b),
            pair.score
        ));
    }
    html.push_str("</table>\n");

    // Legend
    html.push_str("<div class=\"legend\"><span class=\"l-exact\"></span> Exact N-gram match &nbsp; <span class=\"l-para\"></span> Paraphrased (Jaccard/Levenshtein)</div>\n");

    // Detail comparisons
    for (i, detail) in details.iter().enumerate() {
        html.push_str(&format!(
            "<div class=\"pair-section\">\n<h2>Pair {} — {} ↔ {} (Cosine: {:.1}%)</h2>\n",
            i + 1,
            escape_html(&detail.file_a),
            escape_html(&detail.file_b),
            results.get(i).map(|p| p.score).unwrap_or(0.0)
        ));

        html.push_str("<div class=\"compare-container\">\n");

        // Left panel
        html.push_str(&format!(
            "<div class=\"compare-panel\"><h3>{}</h3><pre>{}</pre></div>\n",
            escape_html(&detail.file_a),
            apply_highlights(&detail.content_a, &detail.highlights_a, &detail.suspicious_sentences, true)
        ));

        // Right panel
        html.push_str(&format!(
            "<div class=\"compare-panel\"><h3>{}</h3><pre>{}</pre></div>\n",
            escape_html(&detail.file_b),
            apply_highlights(&detail.content_b, &detail.highlights_b, &detail.suspicious_sentences, false)
        ));

        html.push_str("</div>\n</div>\n");
    }

    html.push_str("</body></html>");

    std::fs::write(filepath, html).map_err(|e| e.to_string())
}

/// Apply highlight spans to content
fn apply_highlights(
    content: &str,
    ngram_ranges: &[[usize; 2]],
    suspicious: &[crate::sentence::SuspiciousPair],
    is_a: bool,
) -> String {
    // Collect all ranges with types: (start, end, type)
    let mut all_ranges: Vec<(usize, usize, &str)> = Vec::new();

    // N-gram exact matches
    for [s, e] in ngram_ranges {
        all_ranges.push((*s, *e, "exact"));
    }

    // Suspicious sentence ranges (paraphrased)
    for pair in suspicious {
        let [s, e] = if is_a { pair.pos_a } else { pair.pos_b };
        // Only add if not already covered by exact matches
        let already_covered = ngram_ranges.iter().any(|[ns, ne]| *ns <= s && *ne >= e);
        if !already_covered {
            all_ranges.push((s, e, "para"));
        }
    }

    if all_ranges.is_empty() {
        return escape_html(content);
    }

    // Sort and merge
    all_ranges.sort_by_key(|r| r.0);

    let mut html = String::new();
    let mut last_end = 0;

    for (start, end, kind) in &all_ranges {
        let start = *start;
        let end = (*end).min(content.len());
        if start < last_end {
            continue; // skip overlapping
        }
        if start > last_end {
            html.push_str(&escape_html(&content[last_end..start]));
        }
        let class = if *kind == "exact" { "hl-exact" } else { "hl-para" };
        html.push_str(&format!(
            "<span class=\"{}\">{}</span>",
            class,
            escape_html(&content[start..end])
        ));
        last_end = end;
    }

    if last_end < content.len() {
        html.push_str(&escape_html(&content[last_end..]));
    }

    html
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Export a PDF comparison report with highlighted matching text
pub fn export_pdf_report(
    results: &[SimilarityPair],
    details: &[DetailResult],
    filepath: &str,
) -> Result<(), String> {
    use genpdf::elements::{Break, Paragraph, TableLayout};
    use genpdf::style::{Color, Style};
    use genpdf::Element;

    // Load font from system fonts directory
    let font_family = load_system_font()
        .map_err(|e| format!("Failed to load font for PDF: {}", e))?;

    let mut doc = genpdf::Document::new(font_family);
    doc.set_title("SoText - Similarity Report");

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(15);
    doc.set_page_decorator(decorator);

    // Define styles
    let title_style = Style::new().bold().with_font_size(18);
    let subtitle_style = Style::new().with_font_size(10);
    let header_style = Style::new().bold().with_font_size(10);
    let body_style = Style::new().with_font_size(9);
    let exact_style = Style::new()
        .bold()
        .with_font_size(9)
        .with_color(Color::Rgb(180, 120, 0)); // Dark yellow/orange for exact matches
    let para_highlight_style = Style::new()
        .bold()
        .with_font_size(9)
        .with_color(Color::Rgb(200, 60, 20)); // Dark red-orange for paraphrased
    let score_high_style = Style::new()
        .bold()
        .with_font_size(9)
        .with_color(Color::Rgb(200, 0, 0)); // Red for high scores
    let score_med_style = Style::new()
        .bold()
        .with_font_size(9)
        .with_color(Color::Rgb(180, 120, 0)); // Orange for medium scores

    // ─── Title ───────────────────────────────────────────────
    doc.push(
        Paragraph::default()
            .styled_string("SoText - Similarity Report", title_style),
    );
    doc.push(
        Paragraph::default()
            .styled_string(format!("Generated: {}", chrono_now()), subtitle_style),
    );
    doc.push(Break::new(1.5));

    // ─── Legend ──────────────────────────────────────────────
    let legend = Paragraph::default()
        .styled_string("Legend: ", header_style)
        .styled_string("[EXACT MATCH] ", exact_style)
        .styled_string("= N-gram exact match   ", body_style)
        .styled_string("[PARAPHRASED] ", para_highlight_style)
        .styled_string("= Jaccard/Levenshtein match", body_style);
    doc.push(legend);
    doc.push(Break::new(1.0));

    // ─── Summary Table ───────────────────────────────────────
    doc.push(
        Paragraph::default()
            .styled_string("Summary", Style::new().bold().with_font_size(14)),
    );
    doc.push(Break::new(0.5));

    let mut table = TableLayout::new(vec![1, 1, 1]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));

    // Table header row
    let header_row = table.row();
    header_row
        .element(
            Paragraph::default()
                .styled_string("File A", header_style)
                .padded(1),
        )
        .element(
            Paragraph::default()
                .styled_string("File B", header_style)
                .padded(1),
        )
        .element(
            Paragraph::default()
                .styled_string("Cosine Score", header_style)
                .padded(1),
        )
        .push()
        .map_err(|e| e.to_string())?;

    // Table data rows
    for pair in results {
        let score_style = if pair.score >= 80.0 {
            score_high_style
        } else if pair.score >= 60.0 {
            score_med_style
        } else {
            body_style
        };

        let row = table.row();
        row.element(
            Paragraph::default()
                .styled_string(truncate_filename(&pair.file_a, 40), body_style)
                .padded(1),
        )
        .element(
            Paragraph::default()
                .styled_string(truncate_filename(&pair.file_b, 40), body_style)
                .padded(1),
        )
        .element(
            Paragraph::default()
                .styled_string(format!("{:.1}%", pair.score), score_style)
                .padded(1),
        )
        .push()
        .map_err(|e| e.to_string())?;
    }

    doc.push(table);
    doc.push(Break::new(2.0));

    // ─── Detail Comparisons ──────────────────────────────────
    for (i, detail) in details.iter().enumerate() {
        let score = results.get(i).map(|p| p.score).unwrap_or(0.0);

        // Section header
        doc.push(
            Paragraph::default().styled_string(
                format!(
                    "Pair {} - {} <-> {} (Cosine: {:.1}%)",
                    i + 1,
                    detail.file_a,
                    detail.file_b,
                    score
                ),
                Style::new().bold().with_font_size(12),
            ),
        );
        doc.push(Break::new(0.5));

        // File A content with highlights
        doc.push(
            Paragraph::default()
                .styled_string(format!("--- {} ---", detail.file_a), header_style),
        );

        let para_a = build_highlighted_paragraph(
            &detail.content_a,
            &detail.highlights_a,
            &detail.suspicious_sentences,
            true,
            body_style,
            exact_style,
            para_highlight_style,
        );
        doc.push(para_a);
        doc.push(Break::new(0.8));

        // File B content with highlights
        doc.push(
            Paragraph::default()
                .styled_string(format!("--- {} ---", detail.file_b), header_style),
        );

        let para_b = build_highlighted_paragraph(
            &detail.content_b,
            &detail.highlights_b,
            &detail.suspicious_sentences,
            false,
            body_style,
            exact_style,
            para_highlight_style,
        );
        doc.push(para_b);
        doc.push(Break::new(2.0));
    }

    // Render to file
    doc.render_to_file(filepath).map_err(|e| e.to_string())?;

    Ok(())
}

/// Build a paragraph with highlighted text spans for PDF
fn build_highlighted_paragraph(
    content: &str,
    ngram_ranges: &[[usize; 2]],
    suspicious: &[crate::sentence::SuspiciousPair],
    is_a: bool,
    normal_style: genpdf::style::Style,
    exact_style: genpdf::style::Style,
    para_style: genpdf::style::Style,
) -> genpdf::elements::Paragraph {
    use genpdf::elements::Paragraph;

    let mut all_ranges: Vec<(usize, usize, &str)> = Vec::new();

    // N-gram exact matches
    for [s, e] in ngram_ranges {
        all_ranges.push((*s, *e, "exact"));
    }

    // Suspicious sentences
    for pair in suspicious {
        let [s, e] = if is_a { pair.pos_a } else { pair.pos_b };
        let already_covered = ngram_ranges.iter().any(|[ns, ne]| *ns <= s && *ne >= e);
        if !already_covered {
            all_ranges.push((s, e, "para"));
        }
    }

    if all_ranges.is_empty() {
        // Truncate long content for PDF readability
        let display = if content.len() > 3000 {
            format!(
                "{}...\n[Content truncated for PDF - {} chars total]",
                &content[..3000],
                content.len()
            )
        } else {
            content.to_string()
        };
        let mut p = Paragraph::default();
        p.push_styled(display, normal_style);
        return p;
    }

    all_ranges.sort_by_key(|r| r.0);

    let mut paragraph = Paragraph::default();
    let mut last_end = 0;
    let max_len = content.len().min(5000); // Limit for PDF

    for (start, end, kind) in &all_ranges {
        let start = *start;
        let end = (*end).min(max_len);
        if start >= max_len {
            break;
        }
        if start < last_end {
            continue;
        }
        if start > last_end {
            // Normal text before highlight
            let normal_text = &content[last_end..start];
            if !normal_text.is_empty() {
                paragraph.push_styled(normal_text, normal_style);
            }
        }
        let style = if *kind == "exact" {
            exact_style
        } else {
            para_style
        };
        let highlighted_text = &content[start..end];
        paragraph.push_styled(highlighted_text, style);
        last_end = end;
    }

    if last_end < max_len && last_end < content.len() {
        paragraph.push_styled(&content[last_end..max_len.min(content.len())], normal_style);
    }

    if content.len() > max_len {
        paragraph.push_styled(
            format!("\n[... truncated, {} chars total]", content.len()),
            genpdf::style::Style::new().italic().with_font_size(8),
        );
    }

    paragraph
}

/// Truncate a filename for display in the PDF table
fn truncate_filename(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len - 3])
    }
}

/// Try to find a font file by name in a directory (case-insensitive)
fn find_font_file(dir: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    let name_lower = name.to_lowercase();
    // Try exact match first
    let exact = dir.join(name);
    if exact.exists() {
        return Some(exact);
    }
    // Case-insensitive fallback
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().to_lowercase() == name_lower {
                return Some(entry.path());
            }
        }
    }
    None
}

/// Load a system font for PDF generation
fn load_system_font() -> Result<genpdf::fonts::FontFamily<genpdf::fonts::FontData>, String> {
    // Try common system font directories
    let font_dirs: Vec<std::path::PathBuf> = if cfg!(target_os = "windows") {
        vec![
            std::path::PathBuf::from(r"C:\Windows\Fonts"),
            std::env::var("WINDIR")
                .map(|w| std::path::PathBuf::from(w).join("Fonts"))
                .unwrap_or_default(),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            std::path::PathBuf::from("/Library/Fonts"),
            std::path::PathBuf::from("/System/Library/Fonts"),
            std::path::PathBuf::from("/System/Library/Fonts/Supplemental"),
            dirs_next()
                .map(|h| h.join("Library/Fonts"))
                .unwrap_or_default(),
        ]
    } else {
        vec![
            std::path::PathBuf::from("/usr/share/fonts/truetype"),
            std::path::PathBuf::from("/usr/share/fonts"),
        ]
    };

    for dir in &font_dirs {
        if !dir.exists() {
            continue;
        }

        // Try Arial with case-insensitive lookup (handles Windows "arial.ttf" and macOS "Arial.ttf")
        let regular = find_font_file(dir, "arial.ttf");
        let bold = find_font_file(dir, "arialbd.ttf")
            .or_else(|| find_font_file(dir, "Arial Bold.ttf"));
        let italic = find_font_file(dir, "ariali.ttf")
            .or_else(|| find_font_file(dir, "Arial Italic.ttf"));
        let bold_italic = find_font_file(dir, "arialbi.ttf")
            .or_else(|| find_font_file(dir, "Arial Bold Italic.ttf"));

        if let (Some(r_path), Some(b_path), Some(i_path), Some(bi_path)) =
            (&regular, &bold, &italic, &bold_italic)
        {
            let r =
                genpdf::fonts::FontData::new(std::fs::read(r_path).map_err(|e| e.to_string())?, None)
                    .map_err(|e| e.to_string())?;
            let b =
                genpdf::fonts::FontData::new(std::fs::read(b_path).map_err(|e| e.to_string())?, None)
                    .map_err(|e| e.to_string())?;
            let i =
                genpdf::fonts::FontData::new(std::fs::read(i_path).map_err(|e| e.to_string())?, None)
                    .map_err(|e| e.to_string())?;
            let bi = genpdf::fonts::FontData::new(
                std::fs::read(bi_path).map_err(|e| e.to_string())?,
                None,
            )
            .map_err(|e| e.to_string())?;

            return Ok(genpdf::fonts::FontFamily {
                regular: r,
                bold: b,
                italic: i,
                bold_italic: bi,
            });
        }

        // If only regular font found, use it for all styles (fallback for macOS single-file fonts)
        if let Some(r_path) = &regular {
            if let Ok(data) = std::fs::read(r_path) {
                if let Ok(r) = genpdf::fonts::FontData::new(data.clone(), None) {
                    let b = genpdf::fonts::FontData::new(data.clone(), None).unwrap();
                    let i = genpdf::fonts::FontData::new(data.clone(), None).unwrap();
                    let bi = genpdf::fonts::FontData::new(data, None).unwrap();
                    return Ok(genpdf::fonts::FontFamily {
                        regular: r,
                        bold: b,
                        italic: i,
                        bold_italic: bi,
                    });
                }
            }
        }

        // Try loading via genpdf's from_files (looks for {name}-Regular.ttf etc.)
        if let Ok(family) = genpdf::fonts::from_files(dir, "Arial", None) {
            return Ok(family);
        }
        // Try Helvetica (common on macOS)
        if let Ok(family) = genpdf::fonts::from_files(dir, "Helvetica", None) {
            return Ok(family);
        }
        if let Ok(family) = genpdf::fonts::from_files(dir, "LiberationSans", None) {
            return Ok(family);
        }
    }

    Err("No suitable font found. Please ensure Arial or Liberation Sans is installed.".to_string())
}

/// Helper for home directory (macOS)
#[cfg(target_os = "macos")]
fn dirs_next() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(std::path::PathBuf::from)
}

#[cfg(not(target_os = "macos"))]
fn dirs_next() -> Option<std::path::PathBuf> {
    None
}

fn chrono_now() -> String {
    // Simple timestamp without chrono crate
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Just show as Unix timestamp if no chrono; the user will see date in the report
    format!("{}", now)
}
/// Export a DOCX comparison report with highlighted matching text
pub fn export_docx_report(
    results: &[SimilarityPair],
    details: &[DetailResult],
    filepath: &str,
) -> Result<(), String> {
    use docx_rs::*;

    let mut docx = Docx::new();

    // ─── Title ───────────────────────────────────────────────
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text("SoText — Similarity Report")
                    .size(36) // 18pt
                    .bold()
                    .color("2F5496"),
            ),
    );
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(format!("Generated: {}", chrono_now()))
                    .size(20)
                    .color("666666"),
            ),
    );

    // ─── Legend ──────────────────────────────────────────────
    docx = docx.add_paragraph(Paragraph::new()); // spacer
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text("Legend: ").size(20).bold())
            .add_run(
                Run::new()
                    .add_text(" EXACT MATCH ")
                    .size(20)
                    .bold()
                    .color("7B5B00")
                    .highlight("yellow"),
            )
            .add_run(Run::new().add_text(" = N-gram   ").size(20))
            .add_run(
                Run::new()
                    .add_text(" PARAPHRASED ")
                    .size(20)
                    .bold()
                    .color("6B2D00")
                    .highlight("darkYellow"),
            )
            .add_run(Run::new().add_text(" = Jaccard/Levenshtein").size(20)),
    );
    docx = docx.add_paragraph(Paragraph::new()); // spacer

    // ─── Summary Table ──────────────────────────────────────
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text("Summary").size(28).bold().color("2F5496")),
    );

    // Build table rows
    let header_row = TableRow::new(vec![
        TableCell::new()
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text("File A").size(20).bold().color("FFFFFF")))
            .shading(Shading::new().fill("2F5496")),
        TableCell::new()
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text("File B").size(20).bold().color("FFFFFF")))
            .shading(Shading::new().fill("2F5496")),
        TableCell::new()
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Cosine Score").size(20).bold().color("FFFFFF")))
            .shading(Shading::new().fill("2F5496")),
    ]);

    let mut rows = vec![header_row];

    for pair in results {
        let (bg_color, score_color) = if pair.score >= 80.0 {
            ("FFC7CE", "CC0000")
        } else if pair.score >= 60.0 {
            ("FFEB9C", "996600")
        } else {
            ("FFFFFF", "333333")
        };

        rows.push(TableRow::new(vec![
            TableCell::new()
                .add_paragraph(Paragraph::new().add_run(Run::new().add_text(&pair.file_a).size(18)))
                .shading(Shading::new().fill(bg_color)),
            TableCell::new()
                .add_paragraph(Paragraph::new().add_run(Run::new().add_text(&pair.file_b).size(18)))
                .shading(Shading::new().fill(bg_color)),
            TableCell::new()
                .add_paragraph(Paragraph::new().add_run(
                    Run::new()
                        .add_text(format!("{:.1}%", pair.score))
                        .size(18)
                        .bold()
                        .color(score_color),
                ))
                .shading(Shading::new().fill(bg_color)),
        ]));
    }

    docx = docx.add_table(Table::new(rows));
    docx = docx.add_paragraph(Paragraph::new()); // spacer

    // ─── Detail Comparisons ─────────────────────────────────
    for (i, detail) in details.iter().enumerate() {
        let score = results.get(i).map(|p| p.score).unwrap_or(0.0);

        // Pair header
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(format!(
                            "━━━ Pair {} ━━━  {} ↔ {}  ━━━  Cosine: {:.1}%",
                            i + 1, detail.file_a, detail.file_b, score
                        ))
                        .size(24)
                        .bold()
                        .color("FFFFFF"),
                )
                .page_break_before(i > 0), // page break between pairs
        );
        // Add shading via a table cell trick — or just use colored text on white
        // Actually let's just use bold colored text for the header.

        // File A label
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(format!("📄 {}", detail.file_a))
                        .size(22)
                        .bold()
                        .color("2F5496"),
                ),
        );

        // File A content with highlights
        let para_a = build_docx_highlighted_paragraph(
            &detail.content_a,
            &detail.highlights_a,
            &detail.suspicious_sentences,
            true,
        );
        docx = docx.add_paragraph(para_a);
        docx = docx.add_paragraph(Paragraph::new()); // spacer

        // File B label
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(format!("📄 {}", detail.file_b))
                        .size(22)
                        .bold()
                        .color("2F5496"),
                ),
        );

        // File B content with highlights
        let para_b = build_docx_highlighted_paragraph(
            &detail.content_b,
            &detail.highlights_b,
            &detail.suspicious_sentences,
            false,
        );
        docx = docx.add_paragraph(para_b);

        // Info line
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(format!(
                            "N-gram matches: {} | Suspicious sentences: {}",
                            detail.common_phrase_count,
                            detail.suspicious_sentences.len()
                        ))
                        .size(18)
                        .italic()
                        .color("888888"),
                ),
        );
        docx = docx.add_paragraph(Paragraph::new()); // spacer
    }

    // Write file
    let file = std::fs::File::create(filepath).map_err(|e| e.to_string())?;
    docx.build().pack(file).map_err(|e| e.to_string())?;

    Ok(())
}

/// Build a DOCX paragraph with highlighted text runs
fn build_docx_highlighted_paragraph(
    content: &str,
    ngram_ranges: &[[usize; 2]],
    suspicious: &[crate::sentence::SuspiciousPair],
    is_a: bool,
) -> docx_rs::Paragraph {
    use docx_rs::{Paragraph, Run};

    let mut all_ranges: Vec<(usize, usize, &str)> = Vec::new();

    for [s, e] in ngram_ranges {
        all_ranges.push((*s, *e, "exact"));
    }
    for pair in suspicious {
        let [s, e] = if is_a { pair.pos_a } else { pair.pos_b };
        let already_covered = ngram_ranges.iter().any(|[ns, ne]| *ns <= s && *ne >= e);
        if !already_covered {
            all_ranges.push((s, e, "para"));
        }
    }

    let max_len = content.len().min(50000);
    let content_slice = &content[..max_len];

    if all_ranges.is_empty() {
        let mut p = Paragraph::new();
        p = p.add_run(Run::new().add_text(content_slice).size(18));
        return p;
    }

    all_ranges.sort_by_key(|r| r.0);

    let mut paragraph = Paragraph::new();
    let mut last_end = 0;

    for (start, end, kind) in &all_ranges {
        let start = *start;
        let end = (*end).min(max_len);
        if start >= max_len { break; }
        if start < last_end { continue; }

        if start > last_end {
            let normal_text = &content_slice[last_end..start];
            if !normal_text.is_empty() {
                paragraph = paragraph.add_run(Run::new().add_text(normal_text).size(18));
            }
        }

        let highlighted_text = &content_slice[start..end];
        let run = if *kind == "exact" {
            Run::new()
                .add_text(highlighted_text)
                .size(18)
                .bold()
                .color("7B5B00")
                .highlight("yellow")
        } else {
            Run::new()
                .add_text(highlighted_text)
                .size(18)
                .bold()
                .color("6B2D00")
                .highlight("darkYellow")
        };
        paragraph = paragraph.add_run(run);
        last_end = end;
    }

    if last_end < max_len {
        let tail = &content_slice[last_end..];
        if !tail.is_empty() {
            paragraph = paragraph.add_run(Run::new().add_text(tail).size(18));
        }
    }

    if content.len() > max_len {
        paragraph = paragraph.add_run(
            Run::new()
                .add_text(format!("\n[... truncated, {} chars total]", content.len()))
                .size(16)
                .italic()
                .color("999999"),
        );
    }

    paragraph
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn sample_data() -> (Vec<SimilarityPair>, Vec<crate::analysis::DetailResult>) {
        let results = vec![
            SimilarityPair {
                file_a: "high.txt".to_string(),
                file_b: "score.txt".to_string(),
                score: 92.3,
            },
            SimilarityPair {
                file_a: "med.txt".to_string(),
                file_b: "score.txt".to_string(),
                score: 65.0,
            },
        ];
        let details = vec![
            crate::analysis::DetailResult {
                file_a: "high.txt".to_string(),
                file_b: "score.txt".to_string(),
                content_a: "This is the first document content for testing.".to_string(),
                content_b: "This is the second document content for testing.".to_string(),
                highlights_a: vec![[0, 7]],
                highlights_b: vec![[0, 7]],
                common_phrase_count: 1,
                suspicious_sentences: vec![],
            },
            crate::analysis::DetailResult {
                file_a: "med.txt".to_string(),
                file_b: "score.txt".to_string(),
                content_a: "Medium similarity content here.".to_string(),
                content_b: "Medium similarity content there.".to_string(),
                highlights_a: vec![],
                highlights_b: vec![],
                common_phrase_count: 0,
                suspicious_sentences: vec![],
            },
        ];
        (results, details)
    }

    #[test]
    fn test_export_excel() {
        let (results, details) = sample_data();
        let path = std::env::temp_dir().join("test_sotext.xlsx");
        let path_str = path.to_string_lossy().to_string();

        export_excel(&results, &details, &path_str).unwrap();
        assert!(path.exists());

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_export_docx() {
        let (results, details) = sample_data();
        let path = std::env::temp_dir().join("test_sotext_report.docx");
        let path_str = path.to_string_lossy().to_string();

        export_docx_report(&results, &details, &path_str).unwrap();
        assert!(path.exists());
        assert!(fs::metadata(&path).unwrap().len() > 0);

        fs::remove_file(&path).ok();
    }
}
