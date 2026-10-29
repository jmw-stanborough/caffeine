use super::types::{DirectoryListing, FileEntry};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn read_directory(
    dir_path: &Path,
    drive_total_bytes: u64,
    calculate_folder_sizes: bool,
    cancel: Option<&AtomicBool>,
) -> Result<DirectoryListing, String> {
    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", dir_path.display()));
    }
    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", dir_path.display()));
    }

    let read_dir = fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory {}: {e}", dir_path.display()))?;

    let mut entries = Vec::new();
    let mut total_files = 0;
    let mut total_dirs = 0;
    let mut total_bytes = 0u64;

    for item in read_dir {
        if let Some(c) = cancel {
            if c.load(Ordering::Relaxed) {
                return Err("Operation cancelled by user".to_string());
            }
        }

        let entry = match item {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let symlink_meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = symlink_meta.is_dir();
        let name = entry.file_name().to_string_lossy().to_string();

        let (size_bytes, child_count) = if is_dir {
            total_dirs += 1;
            if calculate_folder_sizes && !symlink_meta.file_type().is_symlink() {
                let size = calculate_dir_size(&path, cancel)?;
                (size, count_direct_children(&path))
            } else {
                (0, count_direct_children(&path))
            }
        } else {
            total_files += 1;
            let size = symlink_meta.len();
            total_bytes = total_bytes.saturating_add(size);
            (size, None)
        };

        let drive_usage_percent = if drive_total_bytes > 0 {
            (size_bytes as f64 / drive_total_bytes as f64) * 100.0
        } else {
            0.0
        };

        let modified_timestamp = symlink_meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);

        let extension = if is_dir {
            None
        } else {
            path.extension().map(|e| e.to_string_lossy().to_string())
        };

        entries.push(FileEntry {
            path: path.to_string_lossy().to_string(),
            name,
            is_dir,
            size_bytes,
            drive_usage_percent,
            modified_timestamp,
            extension,
            child_count,
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => b.size_bytes.cmp(&a.size_bytes),
    });

    Ok(DirectoryListing {
        path: dir_path.to_string_lossy().to_string(),
        entries,
        total_files,
        total_dirs,
        total_bytes,
    })
}

pub fn calculate_dir_size(dir_path: &Path, cancel: Option<&AtomicBool>) -> Result<u64, String> {
    let mut total = 0u64;
    let mut stack = vec![dir_path.to_path_buf()];
    let mut items_count = 0usize;

    while let Some(current) = stack.pop() {
        if let Some(c) = cancel {
            if c.load(Ordering::Relaxed) {
                return Err("Operation cancelled by user".to_string());
            }
        }

        let read_dir = match fs::read_dir(&current) {
            Ok(rd) => rd,
            Err(_) => continue,
        };

        for item in read_dir {
            items_count += 1;
            if items_count % 1024 == 0 {
                std::thread::yield_now();
            }

            let entry = match item {
                Ok(e) => e,
                Err(_) => continue,
            };

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_dir() {
                stack.push(entry.path());
            } else if file_type.is_file() {
                if let Ok(meta) = entry.metadata() {
                    total = total.saturating_add(meta.len());
                }
            }
        }
    }

    Ok(total)
}

fn count_direct_children(dir_path: &Path) -> Option<usize> {
    fs::read_dir(dir_path).ok().map(|iter| iter.count())
}
