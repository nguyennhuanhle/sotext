//! CSV and Excel export functionality.

use crate::analysis::{DetailResult, SimilarityPair};
use rust_xlsxwriter::*;
use std::path::Path;

/// Export results to CSV file
pub fn export_csv(results: &[SimilarityPair], filepath: &str) -> Result<(), String> {
    let mut wtr = csv::Writer::from_path(filepath).map_err(|e| e.to_string())?;

    wtr.write_record(["File A", "File B", "Cosine Score"])
        .map_err(|e| e.to_string())?;

    for pair in results {
        wtr.write_record([
            &pair.file_a,
            &pair.file_b,
            &format!("{:.1}", pair.score),
        ])
        .map_err(|e| e.to_string())?;
    }

    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Export results to Excel file with color-coded rows
pub fn export_excel(results: &[SimilarityPair], filepath: &str) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("Similarity Results")
        .map_err(|e| e.to_string())?;

    // Header format
    let header_format = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2F5496))
        .set_align(FormatAlign::Center)
        .set_font_size(11);

    // Write headers
    let headers = ["File A", "File B", "Cosine Score"];
    for (col, &header) in headers.iter().enumerate() {
        worksheet
            .write_string_with_format(0, col as u16, header, &header_format)
            .map_err(|e| e.to_string())?;
    }

    // Row formats
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

    // Write data rows
    for (i, pair) in results.iter().enumerate() {
        let row = (i + 1) as u32;

        if pair.score >= 80.0 {
            worksheet
                .write_string_with_format(row, 0, &pair.file_a, &high_format)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string_with_format(row, 1, &pair.file_b, &high_format)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number_with_format(row, 2, pair.score, &high_score)
                .map_err(|e| e.to_string())?;
        } else if pair.score >= 60.0 {
            worksheet
                .write_string_with_format(row, 0, &pair.file_a, &med_format)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string_with_format(row, 1, &pair.file_b, &med_format)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number_with_format(row, 2, pair.score, &med_score)
                .map_err(|e| e.to_string())?;
        } else {
            worksheet
                .write_string(row, 0, &pair.file_a)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 1, &pair.file_b)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number_with_format(row, 2, pair.score, &score_center)
                .map_err(|e| e.to_string())?;
        }
    }

    // Set column widths
    worksheet.set_column_width(0, 40).map_err(|e| e.to_string())?;
    worksheet.set_column_width(1, 40).map_err(|e| e.to_string())?;
    worksheet.set_column_width(2, 18).map_err(|e| e.to_string())?;

    workbook
        .save(Path::new(filepath))
        .map_err(|e| e.to_string())?;

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

fn chrono_now() -> String {
    // Simple timestamp without chrono crate
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Just show as Unix timestamp if no chrono; the user will see date in the report
    format!("{}", now)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_export_csv() {
        let results = vec![
            SimilarityPair {
                file_a: "test_a.txt".to_string(),
                file_b: "test_b.txt".to_string(),
                score: 85.5,
            },
        ];
        let path = std::env::temp_dir().join("test_sotext.csv");
        let path_str = path.to_string_lossy().to_string();

        export_csv(&results, &path_str).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("File A"));
        assert!(content.contains("test_a.txt"));
        assert!(content.contains("85.5"));

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_export_excel() {
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
        let path = std::env::temp_dir().join("test_sotext.xlsx");
        let path_str = path.to_string_lossy().to_string();

        export_excel(&results, &path_str).unwrap();
        assert!(path.exists());

        fs::remove_file(&path).ok();
    }
}
