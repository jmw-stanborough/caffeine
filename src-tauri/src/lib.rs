use tauri::Manager;

pub mod client;
pub mod config;
pub mod crypto;
pub mod datetime;
pub mod db;
pub mod endpoint;
pub mod net;
pub mod printer_dashboard;
pub mod process;
pub mod server;
pub mod server_monitor;
pub mod sound;
pub mod webview_recovery;
pub mod workspace;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            match sound::resolve_notification_icon_path(app.handle()) {
                Ok(icon_path) => {
                    if let Err(e) = sound::ensure_windows_aumid_registered(&icon_path) {
                        eprintln!("Failed to register Windows AUMID for notifications: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to materialize notification icon: {e}");
                }
            }

            if let Some(window) = app.get_webview_window("main") {
                if let Some(icon) = app.default_window_icon() {
                    window.set_icon(icon.clone())?;
                }
                webview_recovery::install(&window);
            }

            let db_state = db::init_db(app.handle()).map_err(std::io::Error::other)?;
            app.manage(db_state);

            let workspace_db_state =
                workspace::db::init_workspace_db(app.handle()).map_err(std::io::Error::other)?;
            app.manage(workspace_db_state);

            let active_server = server::ActiveServer::new();
            app.manage(active_server.clone());

            let settings =
                config::load_app_settings(app.handle()).map_err(std::io::Error::other)?;
            if settings.server_mode {
                if let Ok(machine_db) = db::get_machine_db_path() {
                    let bind = settings.server_bind.clone();
                    let port = settings.server_port;
                    let srv = active_server.clone();
                    let handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = srv.start(Some(handle), &bind, port, machine_db).await {
                            eprintln!("Failed to start embedded server on startup: {e}");
                        }
                    });
                }
            }

            server_monitor::worker::start_ping_worker(app.handle().clone());
            printer_dashboard::worker::start_printer_worker(app.handle().clone());
            client::sync::start_client_sync_worker(app.handle().clone());
            workspace::worker::start_reminder_worker(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            server_monitor::config::load_servers,
            server_monitor::config::save_server,
            server_monitor::config::update_server,
            server_monitor::config::delete_server,
            server_monitor::config::reorder_servers,
            server_monitor::config::get_server_logs,
            server_monitor::remote::launch_remote_desktop,
            server_monitor::remote::start_server_ssh_terminal,
            server_monitor::remote::write_server_ssh_terminal,
            server_monitor::remote::resize_server_ssh_terminal,
            server_monitor::remote::close_server_ssh_terminal,
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
            endpoint::endpoint_restart_system,
            endpoint::endpoint_detect_shell,
            endpoint::endpoint_start_terminal,
            endpoint::endpoint_write_terminal,
            endpoint::endpoint_resize_terminal,
            endpoint::endpoint_close_terminal,
            endpoint::filesystem::filesystem_list_volumes,
            endpoint::filesystem::filesystem_read_directory,
            endpoint::filesystem::filesystem_calculate_folder_size,
            endpoint::filesystem::filesystem_search,
            endpoint::filesystem::filesystem_cancel_task,
            endpoint::filesystem::filesystem_load_filters,
            endpoint::filesystem::filesystem_save_filter,
            endpoint::filesystem::filesystem_delete_filter,
            endpoint::filesystem::filesystem_batch_delete,
            config::get_app_settings_cmd,
            config::update_app_settings_cmd,
            config::get_server_tokens_cmd,
            config::create_server_token_cmd,
            config::regenerate_server_token_cmd,
            config::delete_server_token_cmd,
            sound::check_sound_file_cmd,
            sound::load_custom_sound_cmd,
            workspace::todo::workspace_load_todos,
            workspace::todo::workspace_create_todo,
            workspace::todo::workspace_update_todo,
            workspace::todo::workspace_toggle_todo,
            workspace::todo::workspace_delete_todo,
            workspace::todo::workspace_acknowledge_todo_reminder,
            workspace::todo::workspace_snooze_todo_reminder,
            workspace::notes::workspace_list_notes,
            workspace::notes::workspace_read_note,
            workspace::notes::workspace_save_note,
            workspace::notes::workspace_delete_note,
            workspace::scripts::workspace_detect_script_languages,
            workspace::scripts::workspace_list_scripts,
            workspace::scripts::workspace_read_script,
            workspace::scripts::workspace_save_script,
            workspace::scripts::workspace_delete_script,
            workspace::scripts::workspace_format_script,
            workspace::scripts::workspace_build_script_run_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
