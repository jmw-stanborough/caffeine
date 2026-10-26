use tauri::Manager;

pub mod config;
pub mod db;
pub mod endpoint;
pub mod net;
pub mod printer_dashboard;
pub mod server_monitor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                if let Some(icon) = app.default_window_icon() {
                    window.set_icon(icon.clone())?;
                }
            }

            let db_state = db::init_db(app.handle()).map_err(std::io::Error::other)?;
            app.manage(db_state);

            server_monitor::worker::start_ping_worker(app.handle().clone());
            printer_dashboard::worker::start_printer_worker(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            server_monitor::config::load_servers,
            server_monitor::config::save_server,
            server_monitor::config::update_server,
            server_monitor::config::delete_server,
            server_monitor::config::reorder_servers,
            server_monitor::config::get_server_logs,
            printer_dashboard::config::load_printers,
            printer_dashboard::config::save_printer,
            printer_dashboard::config::update_printer,
            printer_dashboard::config::delete_printer,
            printer_dashboard::config::reorder_printers,
            printer_dashboard::config::get_printer_logs,
            printer_dashboard::config::load_printer_filter_tags,
            printer_dashboard::config::save_printer_filter_tags,
            printer_dashboard::webview::open_printer_webview,
            printer_dashboard::webview::resize_printer_webview,
            printer_dashboard::webview::navigate_printer_webview,
            printer_dashboard::webview::reload_printer_webview,
            printer_dashboard::webview::close_printer_webview,
            endpoint::endpoint_check_os_support,
            endpoint::endpoint_get_machine_info,
            endpoint::endpoint_check_updates,
            endpoint::endpoint_download_updates,
            endpoint::endpoint_install_updates,
            endpoint::endpoint_detect_shell,
            endpoint::endpoint_start_terminal,
            endpoint::endpoint_write_terminal,
            endpoint::endpoint_resize_terminal,
            endpoint::endpoint_close_terminal,
            config::get_app_settings_cmd,
            config::update_app_settings_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
