//! Tauri command handlers exposed to the frontend via `invoke`.

use crate::analysis;
use crate::export;
use tauri_plugin_dialog::DialogExt;

/// Scan a folder for text similarity
#[tauri::command]
pub fn scan_folder(
    path: String,
    threshold: f64,
    template_path: Option<String>,
) -> Result<analysis::ScanResult, String> {
    if path.is_empty() {
        return Err("No folder path provided.".to_string());
    }
    if !std::path::Path::new(&path).is_dir() {
        return Err(format!("'{}' is not a valid directory.", path));
    }

    let template_text = template_path
        .as_deref()
        .and_then(|p| analysis::extract_template_text(p));

    let threshold_decimal = threshold / 100.0;
    Ok(analysis::scan_folder(
        &path,
        threshold_decimal,
        template_text.as_deref(),
    ))
}

/// Get detailed comparison between two files with highlighted phrases
#[tauri::command]
pub fn get_detail(
    folder: String,
    file_a: String,
    file_b: String,
    ngram_size: usize,
) -> Result<analysis::DetailResult, String> {
    Ok(analysis::get_detail(
        &folder, &file_a, &file_b, ngram_size, 0.7,
    ))
}

/// Pick a folder using native dialog
#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .pick_folder(move |folder_path| {
            let result = folder_path.map(|p| p.to_string());
            let _ = tx.send(result);
        });
    rx.recv().map_err(|e| e.to_string())
}

/// Count supported files in a folder
#[tauri::command]
pub fn count_files(path: String) -> Result<usize, String> {
    Ok(analysis::count_supported_files(&path))
}

/// Pick a single file (for template upload)
#[tauri::command]
pub async fn pick_template_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Text files", &["txt", "docx", "pdf", "html"])
        .pick_file(move |file_path| {
            let result = file_path.map(|p| p.to_string());
            let _ = tx.send(result);
        });
    rx.recv().map_err(|e| e.to_string())
}

/// Pick a save file path using native dialog
#[tauri::command]
pub async fn pick_save_file(
    app: tauri::AppHandle,
    default_name: String,
    filter_name: String,
    filter_ext: String,
) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter(&filter_name, &[&filter_ext])
        .save_file(move |file_path| {
            let result = file_path.map(|p| p.to_string());
            let _ = tx.send(result);
        });
    rx.recv().map_err(|e| e.to_string())
}

/// Export similarity results to CSV
#[tauri::command]
pub fn export_csv(results: Vec<analysis::SimilarityPair>, filepath: String) -> Result<(), String> {
    export::export_csv(&results, &filepath)
}

/// Export similarity results to Excel
#[tauri::command]
pub fn export_excel(
    results: Vec<analysis::SimilarityPair>,
    filepath: String,
) -> Result<(), String> {
    export::export_excel(&results, &filepath)
}

/// Export HTML comparison report
#[tauri::command]
pub fn export_html(
    results: Vec<analysis::SimilarityPair>,
    folder: String,
    ngram_size: usize,
    filepath: String,
) -> Result<(), String> {
    // Gather detail for each pair
    let details: Vec<analysis::DetailResult> = results
        .iter()
        .map(|pair| analysis::get_detail(&folder, &pair.file_a, &pair.file_b, ngram_size, 0.7))
        .collect();

    export::export_html_report(&results, &details, &filepath)
}

/// Export PDF comparison report
#[tauri::command]
pub fn export_pdf(
    results: Vec<analysis::SimilarityPair>,
    folder: String,
    ngram_size: usize,
    filepath: String,
) -> Result<(), String> {
    let details: Vec<analysis::DetailResult> = results
        .iter()
        .map(|pair| analysis::get_detail(&folder, &pair.file_a, &pair.file_b, ngram_size, 0.7))
        .collect();

    export::export_pdf_report(&results, &details, &filepath)
}
