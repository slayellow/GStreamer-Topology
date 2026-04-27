pub mod commands;
pub mod models;
pub mod parser;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::pipeline::load_local_pipeline_file,
            commands::pipeline::inspect_local_element,
            commands::pipeline::inspect_remote_element,
            commands::pipeline::parse_pipeline_text,
            commands::pipeline::probe_local_gstreamer,
            commands::pipeline::probe_remote_target,
            commands::pipeline::load_remote_pipeline,
            commands::pipeline::save_export_file,
            commands::pipeline::save_export_file_to_downloads,
            commands::pipeline::suggest_export_file_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
