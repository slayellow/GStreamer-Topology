pub mod commands;
pub mod models;
pub mod parser;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::pipeline::load_local_pipeline_file,
            commands::pipeline::inspect_local_element,
            commands::pipeline::inspect_remote_element,
            commands::pipeline::normalize_rtf_text,
            commands::pipeline::parse_pipeline_text,
            commands::pipeline::probe_local_gstreamer,
            commands::pipeline::probe_remote_target,
            commands::pipeline::load_remote_pipeline
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
