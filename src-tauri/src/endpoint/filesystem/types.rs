use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageVolume {
    pub id: String,
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
    pub is_removable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub drive_usage_percent: f64,
    pub modified_timestamp: Option<i64>,
    pub extension: Option<String>,
    pub child_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectoryListing {
    pub path: String,
    pub entries: Vec<FileEntry>,
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperationProgress {
    pub task_id: String,
    pub operation: String,
    pub items_processed: u64,
    pub total_estimated: Option<u64>,
    pub bytes_processed: u64,
    pub current_path: String,
    pub is_complete: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FilterTarget {
    Files,
    Directories,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterProperty {
    Name,
    Extension,
    Path,
    Size,
    Content,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterCondition {
    Is,
    StartsWith,
    EndsWith,
    Contains,
    DoesNotContain,
    Matches,
    Gt,
    Lt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterCriterion {
    pub target: FilterTarget,
    pub property: FilterProperty,
    pub condition: FilterCondition,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedFilter {
    pub id: String,
    pub name: String,
    pub criteria: Vec<FilterCriterion>,
    pub match_all: bool,
    pub is_preset: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatchDeleteSummary {
    pub succeeded_paths: Vec<String>,
    pub failed_paths: Vec<(String, String)>,
    pub total_bytes_freed: u64,
}
