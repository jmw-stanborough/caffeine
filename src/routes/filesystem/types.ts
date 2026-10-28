export interface StorageVolume {
  id: string;
  name: string;
  mount_point: string;
  file_system: string;
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  usage_percent: number;
  is_removable: boolean;
}

export interface FileEntry {
  path: string;
  name: string;
  is_dir: boolean;
  size_bytes: number;
  drive_usage_percent: number;
  modified_timestamp?: number | null;
  extension?: string | null;
  child_count?: number | null;
  is_calculating?: boolean;
}

export interface DirectoryListing {
  path: string;
  entries: FileEntry[];
  total_files: number;
  total_dirs: number;
  total_bytes: number;
}

export interface OperationProgress {
  task_id: string;
  operation: string;
  items_processed: number;
  total_estimated?: number | null;
  bytes_processed: number;
  current_path: string;
  is_complete: boolean;
  error?: string | null;
}

export type FilterTarget = "files" | "directories" | "both";
export type FilterProperty = "name" | "extension" | "path" | "size" | "content";
export type FilterCondition =
  | "is"
  | "starts_with"
  | "ends_with"
  | "contains"
  | "does_not_contain"
  | "matches"
  | "gt"
  | "lt";

export interface FilterCriterion {
  target: FilterTarget;
  property: FilterProperty;
  condition: FilterCondition;
  value: string;
}

export interface SavedFilter {
  id: string;
  name: string;
  criteria: FilterCriterion[];
  match_all: boolean;
  is_preset: boolean;
}

export interface BatchDeleteSummary {
  succeeded_paths: string[];
  failed_paths: [string, string][];
  total_bytes_freed: number;
}
