mod server_monitor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            server_monitor::config::load_servers,
            server_monitor::config::save_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
