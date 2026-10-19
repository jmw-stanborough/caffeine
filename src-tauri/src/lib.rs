mod server_monitor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            server_monitor::worker::start_ping_worker(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            server_monitor::config::load_servers,
            server_monitor::config::save_server,
            server_monitor::config::update_server,
            server_monitor::config::reorder_servers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
