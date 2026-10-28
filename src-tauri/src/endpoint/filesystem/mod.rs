pub mod analyzer;
pub mod drives;
pub mod filters;
pub mod pruner;
pub mod search;
pub mod types;

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{AppHandle, Emitter};
use types::{
    BatchDeleteSummary, DirectoryListing, FileEntry, FilterCriterion, OperationProgress,
    SavedFilter, StorageVolume,
};

static ACTIVE_TASKS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn register_task(task_id: &str) -> Arc<AtomicBool> {
    let cancel_token = Arc::new(AtomicBool::new(false));
    if let Ok(mut tasks) = ACTIVE_TASKS.lock() {
        tasks.insert(task_id.to_string(), cancel_token.clone());
    }
    cancel_token
}

fn unregister_task(task_id: &str) {
    if let Ok(mut tasks) = ACTIVE_TASKS.lock() {
        tasks.remove(task_id);
    }
}

#[tauri::command]
pub async fn filesystem_list_volumes() -> Result<Vec<StorageVolume>, String> {
    tokio::task::spawn_blocking(drives::list_storage_volumes)
        .await
        .map_err(|e| format!("Failed to spawn volume scan task: {e}"))?
}

#[tauri::command]
pub async fn filesystem_read_directory(
    path: String,
    drive_total_bytes: u64,
    calculate_folder_sizes: bool,
) -> Result<DirectoryListing, String> {
    tokio::task::spawn_blocking(move || {
        analyzer::read_directory(
            Path::new(&path),
            drive_total_bytes,
            calculate_folder_sizes,
            None,
        )
    })
    .await
    .map_err(|e| format!("Directory listing task panicked: {e}"))?
}

#[tauri::command]
pub async fn filesystem_calculate_folder_size(path: String) -> Result<u64, String> {
    tokio::task::spawn_blocking(move || analyzer::calculate_dir_size(Path::new(&path), None))
        .await
        .map_err(|e| format!("Directory sizing task panicked: {e}"))?
}

#[tauri::command]
pub async fn filesystem_search(
    app: AppHandle,
    task_id: String,
    root_path: String,
    criteria: Vec<FilterCriterion>,
    match_all: bool,
    drive_total_bytes: u64,
) -> Result<Vec<FileEntry>, String> {
    let cancel_token = register_task(&task_id);
    let app_handle = app.clone();
    let task_id_clone = task_id.clone();

    let result = tokio::task::spawn_blocking(move || {
        let res = search::search_tree(
            Path::new(&root_path),
            &criteria,
            match_all,
            drive_total_bytes,
            &task_id_clone,
            &cancel_token,
            |progress| {
                let _ = app_handle.emit("fs:progress", &progress);
            },
        );

        let final_progress = OperationProgress {
            task_id: task_id_clone,
            operation: "search".to_string(),
            items_processed: 0,
            total_estimated: None,
            bytes_processed: 0,
            current_path: String::new(),
            is_complete: true,
            error: res.as_ref().err().cloned(),
        };
        let _ = app_handle.emit("fs:progress", &final_progress);

        res
    })
    .await
    .map_err(|e| format!("Filesystem search task panicked: {e}"))?;

    unregister_task(&task_id);
    result
}

#[tauri::command]
pub fn filesystem_cancel_task(task_id: String) -> Result<(), String> {
    if let Ok(tasks) = ACTIVE_TASKS.lock() {
        if let Some(token) = tasks.get(&task_id) {
            token.store(true, Ordering::Relaxed);
            return Ok(());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn filesystem_load_filters(app: AppHandle) -> Result<Vec<SavedFilter>, String> {
    filters::load_saved_filters(&app)
}

#[tauri::command]
pub fn filesystem_save_filter(app: AppHandle, filter: SavedFilter) -> Result<SavedFilter, String> {
    filters::save_saved_filter(&app, filter)
}

#[tauri::command]
pub fn filesystem_delete_filter(app: AppHandle, filter_id: String) -> Result<(), String> {
    filters::delete_saved_filter(&app, &filter_id)
}

#[tauri::command]
pub async fn filesystem_batch_delete(paths: Vec<String>) -> Result<BatchDeleteSummary, String> {
    tokio::task::spawn_blocking(move || pruner::batch_delete_items(&paths))
        .await
        .map_err(|e| format!("Batch delete task panicked: {e}"))
}
