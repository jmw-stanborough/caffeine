use super::types::StorageVolume;
use std::path::Path;
use sysinfo::Disks;

pub fn list_storage_volumes() -> Result<Vec<StorageVolume>, String> {
    let disks = Disks::new_with_refreshed_list();
    let mut volumes = Vec::new();

    for disk in &disks {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);
        let usage_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let mount = disk.mount_point().to_string_lossy().to_string();
        let raw_name = disk.name().to_string_lossy().to_string();

        let display_name = if raw_name.trim().is_empty() {
            mount.clone()
        } else {
            raw_name
        };

        let id = format!("{:x}", md5_digest(mount.as_bytes()));

        volumes.push(StorageVolume {
            id,
            name: display_name,
            mount_point: mount,
            file_system: disk.file_system().to_string_lossy().to_string(),
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            usage_percent,
            is_removable: disk.is_removable(),
        });
    }

    volumes.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    Ok(volumes)
}

pub fn get_volume_for_path(path: &Path) -> Option<StorageVolume> {
    let volumes = list_storage_volumes().ok()?;
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    volumes
        .into_iter()
        .filter(|v| canonical.starts_with(Path::new(&v.mount_point)))
        .max_by_key(|v| v.mount_point.len())
}

fn md5_digest(bytes: &[u8]) -> u128 {
    let mut hash = 0x811c9dc5u32;
    for &byte in bytes {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash as u128
}
