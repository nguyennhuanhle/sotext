mod analysis;
mod commands;
mod export;
mod sentence;
mod tfidf;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_folder,
            commands::get_detail,
            commands::pick_folder,
            commands::count_files,
            commands::pick_template_file,
            commands::pick_save_file,
            commands::export_csv,
            commands::export_excel,
            commands::export_html,
            commands::export_pdf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
