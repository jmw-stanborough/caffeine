// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "windows")]
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let webview_cache = std::path::Path::new(&local_app_data)
            .join("caffeine")
            .join("webview");
        std::fs::create_dir_all(&webview_cache)
            .expect("failed to create local webview cache directory");

        unsafe {
            std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", webview_cache);
        }
    }

    caffeine_lib::run()
}
