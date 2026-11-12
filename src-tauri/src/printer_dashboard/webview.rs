use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::webview::WebviewBuilder;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Rect, WebviewUrl};
use tokio::sync::Mutex;

const CHILD_WEBVIEW_LABEL: &str = "printer-webui";
const MAIN_WINDOW_LABEL: &str = "main";
const WEBVIEW_CLOSE_SETTLE_DELAY: Duration = Duration::from_millis(60);
const MIN_INITIAL_WEBVIEW_DIMENSION: f64 = 100.0;
const MIN_RESIZED_WEBVIEW_DIMENSION: f64 = 50.0;

static WEBVIEW_LOCK: Mutex<()> = Mutex::const_new(());
static WEBVIEW_GENERATION: AtomicU64 = AtomicU64::new(0);

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

fn restore_main_window_focus(app: &AppHandle) {
    if let Some(window) = app.get_window(MAIN_WINDOW_LABEL) {
        if let Err(err) = window.set_focus() {
            eprintln!("Failed to focus main window: {err}");
        }
    }
    if let Some(webview) = app.get_webview(MAIN_WINDOW_LABEL) {
        if let Err(err) = webview.set_focus() {
            eprintln!("Failed to focus main webview: {err}");
        }
    }
}

async fn teardown_child_webview(app: &AppHandle, restore_main_focus: bool) -> Result<(), String> {
    if let Some(existing) = app.get_webview(CHILD_WEBVIEW_LABEL) {
        let _ = existing.set_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(0.0, 0.0).into(),
        });
        existing
            .close()
            .map_err(|e| format!("Failed to close existing webview: {e}"))?;
        tokio::time::sleep(WEBVIEW_CLOSE_SETTLE_DELAY).await;
    }

    if restore_main_focus {
        restore_main_window_focus(app);
    }

    Ok(())
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

    let _guard = WEBVIEW_LOCK.lock().await;
    let current_gen = WEBVIEW_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    teardown_child_webview(&app, false).await?;

    if WEBVIEW_GENERATION.load(Ordering::SeqCst) != current_gen {
        restore_main_window_focus(&app);
        return Ok(resolved_url_str);
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

    if WEBVIEW_GENERATION.load(Ordering::SeqCst) != current_gen {
        teardown_child_webview(&app, true).await?;
        return Ok(resolved_url_str);
    }

    if let Err(err) = webview.set_focus() {
        eprintln!("Failed to focus child webview: {err}");
    }

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
    let _guard = WEBVIEW_LOCK.lock().await;
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

    let _guard = WEBVIEW_LOCK.lock().await;
    if let Some(webview) = app.get_webview(CHILD_WEBVIEW_LABEL) {
        webview
            .navigate(parsed_url)
            .map_err(|e| format!("Failed to navigate child webview: {e}"))?;
    }
    Ok(resolved_url_str)
}

#[tauri::command]
pub async fn reload_printer_webview(app: AppHandle) -> Result<(), String> {
    let _guard = WEBVIEW_LOCK.lock().await;
    if let Some(webview) = app.get_webview(CHILD_WEBVIEW_LABEL) {
        webview
            .eval("window.location.reload()")
            .map_err(|e| format!("Failed to reload child webview: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn close_printer_webview(app: AppHandle) -> Result<(), String> {
    let _guard = WEBVIEW_LOCK.lock().await;
    WEBVIEW_GENERATION.fetch_add(1, Ordering::SeqCst);
    teardown_child_webview(&app, true).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_printer_url_adds_http_scheme() {
        let parsed = normalize_printer_url("192.168.1.50").unwrap();
        assert_eq!(parsed.as_str(), "http://192.168.1.50/");
    }

    #[test]
    fn test_normalize_printer_url_preserves_https_scheme() {
        let parsed = normalize_printer_url("https://10.0.0.12:8443").unwrap();
        assert_eq!(parsed.as_str(), "https://10.0.0.12:8443/");
    }

    #[test]
    fn test_normalize_printer_url_rejects_empty_and_whitespace() {
        assert!(normalize_printer_url("").is_err());
        assert!(normalize_printer_url("   ").is_err());
    }

    #[test]
    fn test_normalize_printer_url_trims_input() {
        let parsed = normalize_printer_url("  http://printer.local/  ").unwrap();
        assert_eq!(parsed.as_str(), "http://printer.local/");
    }
}
