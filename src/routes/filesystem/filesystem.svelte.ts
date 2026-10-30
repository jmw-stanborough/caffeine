import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  BatchDeleteSummary,
  DirectoryListing,
  FileEntry,
  FilterCriterion,
  OperationProgress,
  SavedFilter,
  StorageVolume,
} from "./types";

export class FilesystemStore {
  volumes = $state<StorageVolume[]>([]);
  selectedVolume = $state<StorageVolume | null>(null);
  currentPath = $state<string>("");
  listing = $state<DirectoryListing | null>(null);
  searchResults = $state<FileEntry[] | null>(null);

  isLoading = $state(false);
  isSearching = $state(false);
  isDeleting = $state(false);
  error = $state<string | null>(null);

  activeTaskId = $state<string | null>(null);
  progress = $state<OperationProgress | null>(null);

  savedFilters = $state<SavedFilter[]>([]);
  selectedPaths = $state<Set<string>>(new Set());
  deleteSummary = $state<BatchDeleteSummary | null>(null);

  displayedEntries = $derived<FileEntry[]>(
    this.searchResults ?? this.listing?.entries ?? [],
  );

  totalSelectedBytes = $derived.by<number>(() => {
    let sum = 0;
    for (const entry of this.displayedEntries) {
      if (this.selectedPaths.has(entry.path)) {
        sum += entry.size_bytes;
      }
    }
    return sum;
  });

  private unlistenProgress: UnlistenFn | null = null;
  private isInitialized = false;
  private initPromise: Promise<void> | null = null;
  private currentScanGeneration = 0;

  async init(): Promise<void> {
    if (this.isInitialized) return;
    if (this.initPromise) return this.initPromise;

    this.initPromise = (async () => {
      try {
        this.unlistenProgress = await listen<OperationProgress>(
          "fs:progress",
          (event) => {
            this.progress = event.payload;
            if (event.payload.is_complete) {
              this.activeTaskId = null;
              this.isSearching = false;
            }
          },
        );
      } catch (err: unknown) {
        this.error = String(err);
        throw err;
      }

      await Promise.all([this.loadVolumes(), this.loadSavedFilters()]);

      if (this.volumes.length > 0 && !this.selectedVolume) {
        await this.selectVolume(this.volumes[0]);
      }
      this.isInitialized = true;
    })();

    try {
      await this.initPromise;
    } finally {
      this.initPromise = null;
    }
  }

  destroy(): void {
    if (this.unlistenProgress) {
      this.unlistenProgress();
      this.unlistenProgress = null;
    }
    this.isInitialized = false;
  }

  async loadVolumes(): Promise<void> {
    try {
      this.volumes = await invoke<StorageVolume[]>("filesystem_list_volumes");
      if (this.selectedVolume) {
        const found = this.volumes.find((v) => v.id === this.selectedVolume?.id);
        if (found) this.selectedVolume = found;
      }
    } catch (err: unknown) {
      this.error = String(err);
      throw err;
    }
  }

  async selectVolume(vol: StorageVolume): Promise<void> {
    this.selectedVolume = vol;
    this.searchResults = null;
    this.selectedPaths = new Set();
    await this.navigateTo(vol.mount_point);
  }

  async navigateTo(path: string): Promise<void> {
    this.currentScanGeneration++;
    const generation = this.currentScanGeneration;

    this.isLoading = true;
    this.error = null;
    this.searchResults = null;
    this.selectedPaths = new Set();

    const driveTotal = this.selectedVolume?.total_bytes ?? 0;

    try {
      const result = await invoke<DirectoryListing>("filesystem_read_directory", {
        path,
        driveTotalBytes: driveTotal,
        calculateFolderSizes: false,
      });

      for (const entry of result.entries) {
        if (entry.is_dir) {
          entry.is_calculating = true;
        }
      }

      this.listing = result;
      this.currentPath = result.path;
      this.isLoading = false;

      this.processFolderSizingQueue(generation, driveTotal).catch((err: unknown) => {
        console.error("Folder sizing queue failed:", err);
      });
    } catch (err: unknown) {
      this.error = String(err);
      this.isLoading = false;
      throw err;
    }
  }

  private async processFolderSizingQueue(generation: number, driveTotal: number): Promise<void> {
    if (!this.listing) return;

    for (let i = 0; i < this.listing.entries.length; i++) {
      if (this.currentScanGeneration !== generation) return;

      const entry = this.listing.entries[i];
      if (!entry.is_dir || !entry.is_calculating) continue;

      try {
        const size = await invoke<number>("filesystem_calculate_folder_size", {
          path: entry.path,
        });

        if (this.currentScanGeneration !== generation) return;

        entry.size_bytes = size;
        entry.drive_usage_percent = driveTotal > 0 ? (size / driveTotal) * 100 : 0;
      } catch (err: unknown) {
        this.error = String(err);
        throw err;
      } finally {
        if (this.currentScanGeneration === generation) {
          entry.is_calculating = false;
        }
      }

      await new Promise((resolve) => setTimeout(resolve, 60));
    }
  }

  async navigateUp(): Promise<void> {
    if (!this.currentPath) return;
    const normalized = this.currentPath.replace(/[/\\]+$/, "");
    const lastSlash = Math.max(
      normalized.lastIndexOf("/"),
      normalized.lastIndexOf("\\"),
    );

    if (lastSlash >= 0) {
      const parent = normalized.slice(0, lastSlash) || "/";
      await this.navigateTo(parent);
    }
  }

  async runSearch(criteria: FilterCriterion[], matchAll: boolean): Promise<void> {
    if (!this.currentPath) return;

    this.isSearching = true;
    this.error = null;
    this.selectedPaths = new Set();
    const taskId = `task-${Date.now()}`;
    this.activeTaskId = taskId;

    const driveTotal = this.selectedVolume?.total_bytes ?? 0;

    try {
      const results = await invoke<FileEntry[]>("filesystem_search", {
        taskId,
        rootPath: this.currentPath,
        criteria,
        matchAll,
        driveTotalBytes: driveTotal,
      });
      this.searchResults = results;
    } catch (err: unknown) {
      this.error = String(err);
      throw err;
    } finally {
      this.isSearching = false;
      this.activeTaskId = null;
    }
  }

  async cancelActiveTask(): Promise<void> {
    if (!this.activeTaskId) return;
    try {
      await invoke("filesystem_cancel_task", { taskId: this.activeTaskId });
    } catch (err: unknown) {
      this.error = String(err);
      throw err;
    } finally {
      this.isSearching = false;
      this.activeTaskId = null;
    }
  }

  clearSearch(): void {
    this.searchResults = null;
    this.selectedPaths = new Set();
  }

  toggleSelect(path: string): void {
    const next = new Set(this.selectedPaths);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    this.selectedPaths = next;
  }

  selectAll(): void {
    const next = new Set<string>();
    for (const item of this.displayedEntries) {
      next.add(item.path);
    }
    this.selectedPaths = next;
  }

  deselectAll(): void {
    this.selectedPaths = new Set();
  }

  async deleteSelected(): Promise<void> {
    const paths = Array.from(this.selectedPaths);
    if (paths.length === 0) return;

    this.isDeleting = true;
    this.error = null;

    try {
      const summary = await invoke<BatchDeleteSummary>("filesystem_batch_delete", {
        paths,
      });
      this.deleteSummary = summary;
      this.selectedPaths = new Set();

      await this.loadVolumes();
      if (this.currentPath) {
        await this.navigateTo(this.currentPath);
      }
    } catch (err: unknown) {
      this.error = String(err);
      throw err;
    } finally {
      this.isDeleting = false;
    }
  }

  async loadSavedFilters(): Promise<void> {
    try {
      this.savedFilters = await invoke<SavedFilter[]>("filesystem_load_filters");
    } catch (err: unknown) {
      this.error = String(err);
      throw err;
    }
  }

  async saveFilter(filter: SavedFilter): Promise<SavedFilter> {
    try {
      const saved = await invoke<SavedFilter>("filesystem_save_filter", { filter });
      await this.loadSavedFilters();
      return saved;
    } catch (err: unknown) {
      this.error = String(err);
      throw err;
    }
  }

  async deleteFilter(filterId: string): Promise<void> {
    try {
      await invoke("filesystem_delete_filter", { filterId });
      await this.loadSavedFilters();
    } catch (err: unknown) {
      this.error = String(err);
      throw err;
    }
  }
}

export const filesystemStore = new FilesystemStore();
