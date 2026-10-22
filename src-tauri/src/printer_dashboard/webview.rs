use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Rect, WebviewUrl};
use tauri::webview::WebviewBuilder;

const CHILD_WEBVIEW_LABEL: &str = "printer-webui";

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
        tokio::time::sleep(std::time::Duration::from_millis(60)).await;
    }

    let window = app
        .get_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    let w = width.max(100.0);
    let h = height.max(100.0);

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
        let w = width.max(50.0);
        let h = height.max(50.0);
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
