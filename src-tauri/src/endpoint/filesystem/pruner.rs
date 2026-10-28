use super::analyzer::calculate_dir_size;
use super::types::BatchDeleteSummary;
use std::fs;
use std::path::Path;

pub fn batch_delete_items(paths: &[String]) -> BatchDeleteSummary {
    let mut succeeded_paths = Vec::new();
    let mut failed_paths = Vec::new();
    let mut total_bytes_freed = 0u64;

    for raw_path in paths {
        let path = Path::new(raw_path);
        if !path.exists() {
            failed_paths.push((raw_path.clone(), "Item no longer exists".to_string()));
            continue;
        }

        let is_dir = path.is_dir();
        let bytes = if is_dir {
            calculate_dir_size(path, None).unwrap_or(0)
        } else {
            path.metadata().map(|m| m.len()).unwrap_or(0)
        };

        let result = if is_dir {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };

        match result {
            Ok(_) => {
                total_bytes_freed = total_bytes_freed.saturating_add(bytes);
                succeeded_paths.push(raw_path.clone());
            }
            Err(e) => {
                failed_paths.push((raw_path.clone(), e.to_string()));
            }
        }
    }

    BatchDeleteSummary {
        succeeded_paths,
        failed_paths,
        total_bytes_freed,
    }
}
