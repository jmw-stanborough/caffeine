use std::time::Duration;
use tauri::webview::WebviewBuilder;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Rect, WebviewUrl};

const CHILD_WEBVIEW_LABEL: &str = "printer-webui";
const MAIN_WINDOW_LABEL: &str = "main";
const WEBVIEW_CLOSE_SETTLE_DELAY: Duration = Duration::from_millis(60);
const MIN_INITIAL_WEBVIEW_DIMENSION: f64 = 100.0;
const MIN_RESIZED_WEBVIEW_DIMENSION: f64 = 50.0;

fn normalize_printer_url(raw: &str) -> Result<url::Url, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };

    url::Url::parse(&with_scheme).map_err(|e| format!("Invalid URL '{with_scheme}': {e}"))
}

#[tauri::command]
pub async fn open_printer_webview(
    app: AppHandle,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<String, String> {
    let parsed_url = normalize_printer_url(&url)?;
    let resolved_url_str = parsed_url.to_string();

    if let Some(existing) = app.get_webview(CHILD_WEBVIEW_LABEL) {
        let _ = existing.close();
        // Allow Tauri's window manager to finish unregistering the existing webview handle
        // before creating a replacement with the same label.
        tokio::time::sleep(WEBVIEW_CLOSE_SETTLE_DELAY).await;
    }

    let window = app
        .get_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| "Main window not found".to_string())?;

    let w = width.max(MIN_INITIAL_WEBVIEW_DIMENSION);
    let h = height.max(MIN_INITIAL_WEBVIEW_DIMENSION);

    let builder = WebviewBuilder::new(CHILD_WEBVIEW_LABEL, WebviewUrl::External(parsed_url));

    let webview = window
        .add_child(
            builder,
            LogicalPosition::new(x, y),
            LogicalSize::new(w, h),
        )
        .map_err(|e| format!("Failed to create child webview: {e}"))?;

    let _ = webview.set_focus();

    Ok(resolved_url_str)
}

#[tauri::command]
pub async fn resize_printer_webview(
    app: AppHandle,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if let Some(webview) = app.get_webview(CHILD_WEBVIEW_LABEL) {
        let w = width.max(MIN_RESIZED_WEBVIEW_DIMENSION);
        let h = height.max(MIN_RESIZED_WEBVIEW_DIMENSION);
        webview
            .set_bounds(Rect {
                position: LogicalPosition::new(x, y).into(),
                size: LogicalSize::new(w, h).into(),
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn navigate_printer_webview(app: AppHandle, url: String) -> Result<String, String> {
    let parsed_url = normalize_printer_url(&url)?;
    let resolved_url_str = parsed_url.to_string();

    if let Some(webview) = app.get_webview(CHILD_WEBVIEW_LABEL) {
        webview
            .navigate(parsed_url)
            .map_err(|e| format!("Failed to navigate child webview: {e}"))?;
    }
    Ok(resolved_url_str)
}

#[tauri::command]
pub async fn reload_printer_webview(app: AppHandle) -> Result<(), String> {
    if let Some(webview) = app.get_webview(CHILD_WEBVIEW_LABEL) {
        let _ = webview.eval("window.location.reload()");
    }
    Ok(())
}

#[tauri::command]
pub async fn close_printer_webview(app: AppHandle) -> Result<(), String> {
    if let Some(webview) = app.get_webview(CHILD_WEBVIEW_LABEL) {
        webview.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
