use super::types::{FilterCondition, FilterCriterion, FilterProperty, FilterTarget, SavedFilter};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const APP_DIR_NAME: &str = "caffeine";
const FILTERS_FILE_NAME: &str = "filesystem_filters.json";

fn get_filters_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .or_else(|_| {
            app.path()
                .config_dir()
                .map(|p| p.join(APP_DIR_NAME))
        })
        .map_err(|e| format!("Failed to resolve user config directory: {e}"))?;

    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory at {config_dir:?}: {e}"))?;

    Ok(config_dir.join(FILTERS_FILE_NAME))
}

pub fn get_preset_filters() -> Vec<SavedFilter> {
    vec![
        SavedFilter {
            id: "preset-temp-files".to_string(),
            name: "Temporary Files".to_string(),
            criteria: vec![
                FilterCriterion {
                    target: FilterTarget::Files,
                    property: FilterProperty::Extension,
                    condition: FilterCondition::Is,
                    value: "tmp".to_string(),
                },
                FilterCriterion {
                    target: FilterTarget::Files,
                    property: FilterProperty::Extension,
                    condition: FilterCondition::Is,
                    value: "temp".to_string(),
                },
            ],
            match_all: false,
            is_preset: true,
        },
        SavedFilter {
            id: "preset-large-files".to_string(),
            name: "Large Files (> 1 GB)".to_string(),
            criteria: vec![FilterCriterion {
                target: FilterTarget::Files,
                property: FilterProperty::Size,
                condition: FilterCondition::Gt,
                value: "1073741824".to_string(),
            }],
            match_all: true,
            is_preset: true,
        },
        SavedFilter {
            id: "preset-log-files".to_string(),
            name: "Log Files (*.log)".to_string(),
            criteria: vec![FilterCriterion {
                target: FilterTarget::Files,
                property: FilterProperty::Extension,
                condition: FilterCondition::Is,
                value: "log".to_string(),
            }],
            match_all: true,
            is_preset: true,
        },
        SavedFilter {
            id: "preset-empty-folders".to_string(),
            name: "Empty Folders".to_string(),
            criteria: vec![FilterCriterion {
                target: FilterTarget::Directories,
                property: FilterProperty::Size,
                condition: FilterCondition::Is,
                value: "0".to_string(),
            }],
            match_all: true,
            is_preset: true,
        },
    ]
}

pub fn load_saved_filters(app: &AppHandle) -> Result<Vec<SavedFilter>, String> {
    let path = get_filters_path(app)?;
    if !path.exists() {
        let defaults = get_preset_filters();
        let payload = serde_json::to_string_pretty(&defaults)
            .map_err(|e| format!("Failed to serialize preset filters: {e}"))?;
        fs::write(&path, payload)
            .map_err(|e| format!("Failed to write preset filters to {path:?}: {e}"))?;
        return Ok(defaults);
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read filter file at {path:?}: {e}"))?;

    let mut saved = serde_json::from_str::<Vec<SavedFilter>>(&content)
        .map_err(|e| format!("Failed to deserialize saved filters: {e}"))?;

    let presets = get_preset_filters();
    for preset in presets {
        if !saved.iter().any(|f| f.id == preset.id) {
            saved.push(preset);
        }
    }

    Ok(saved)
}

pub fn save_saved_filter(app: &AppHandle, mut filter: SavedFilter) -> Result<SavedFilter, String> {
    if filter.name.trim().is_empty() {
        return Err("Filter name cannot be empty".to_string());
    }

    if filter.id.trim().is_empty() {
        filter.id = uuid::Uuid::new_v4().to_string();
    }
    filter.is_preset = false;

    let path = get_filters_path(app)?;
    let mut filters = load_saved_filters(app).unwrap_or_else(|_| get_preset_filters());

    if let Some(idx) = filters.iter().position(|f| f.id == filter.id) {
        filters[idx] = filter.clone();
    } else {
        filters.push(filter.clone());
    }

    let payload = serde_json::to_string_pretty(&filters)
        .map_err(|e| format!("Failed to serialize saved filters: {e}"))?;
    fs::write(&path, payload)
        .map_err(|e| format!("Failed to persist saved filters to {path:?}: {e}"))?;

    Ok(filter)
}

pub fn delete_saved_filter(app: &AppHandle, filter_id: &str) -> Result<(), String> {
    let path = get_filters_path(app)?;
    let mut filters = load_saved_filters(app)?;

    if let Some(f) = filters.iter().find(|f| f.id == filter_id) {
        if f.is_preset {
            return Err("Preset filters cannot be removed".to_string());
        }
    }

    let initial_len = filters.len();
    filters.retain(|f| f.id != filter_id);

    if filters.len() == initial_len {
        return Err(format!("Filter with id '{filter_id}' not found"));
    }

    let payload = serde_json::to_string_pretty(&filters)
        .map_err(|e| format!("Failed to serialize saved filters: {e}"))?;
    fs::write(&path, payload)
        .map_err(|e| format!("Failed to persist updated filters: {e}"))?;

    Ok(())
}
