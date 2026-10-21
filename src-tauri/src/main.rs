// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Some(app_data) = std::env::var_os("APPDATA") {
        let webview_cache = std::path::Path::new(&app_data)
            .join("caffeine")
            .join("webview");
        let _ = std::fs::create_dir_all(&webview_cache);

        unsafe {
            std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", webview_cache);
        }
    }

    caffeine_lib::run()
}
