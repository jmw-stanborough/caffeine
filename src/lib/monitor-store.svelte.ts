import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { RELATIVE_TIME_TICK_MS } from "./constants";

export type MonitorStatus = "online" | "offline" | "degraded" | "checking";

export abstract class MonitorStore<
  TItem extends { id: string },
  TState,
  TTick,
> {
  items = $state<TItem[]>([]);
  statusMap = $state<Record<string, TState>>({});
  loading = $state(true);
  errorMessage = $state<string | null>(null);
  now = $state(Date.now());

  private unlistenStatus: UnlistenFn | null = null;
  private timer: ReturnType<typeof setInterval> | null = null;
  protected destroyed = false;

  protected abstract effectiveStatus(item: TItem): MonitorStatus;

  get statusCounts(): Record<MonitorStatus, number> {
    const counts: Record<MonitorStatus, number> = {
      online: 0,
      offline: 0,
      degraded: 0,
      checking: 0,
    };
    for (const item of this.items) {
      counts[this.effectiveStatus(item)]++;
    }
    return counts;
  }

  get errorCount(): number {
    return this.statusCounts.offline;
  }

  get warningCount(): number {
    return this.statusCounts.degraded;
  }

  get alertCount(): number {
    return this.errorCount + this.warningCount;
  }

  get hasError(): boolean {
    return this.errorCount > 0;
  }

  get hasWarning(): boolean {
    return this.warningCount > 0;
  }

  protected async startTickSubscription(
    eventName: string,
    onTick: (payload: TTick) => void,
  ) {
    if (this.destroyed) return;

    if (this.timer !== null) {
      clearInterval(this.timer);
      this.timer = null;
    }
    this.timer = setInterval(() => {
      this.now = Date.now();
    }, RELATIVE_TIME_TICK_MS);

    if (this.unlistenStatus !== null) {
      this.unlistenStatus();
      this.unlistenStatus = null;
    }

    const unlisten = await listen<TTick>(eventName, (event) => {
      onTick(event.payload);
    });

    if (this.destroyed) {
      unlisten();
    } else {
      this.unlistenStatus = unlisten;
    }
  }

  destroy() {
    this.destroyed = true;
    this.unlistenStatus?.();
    this.unlistenStatus = null;
    if (this.timer !== null) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  protected async loadItems(command: string) {
    this.loading = true;
    this.errorMessage = null;
    try {
      this.items = await invoke<TItem[]>(command);
    } catch (err) {
      this.errorMessage = String(err);
    } finally {
      this.loading = false;
    }
  }

  protected async saveEntity(
    commands: {
      createCmd: string;
      createArgKey: string;
      updateCmd: string;
      updateArgKey: string;
    },
    savedItem: TItem,
    existingId?: string,
  ) {
    try {
      if (existingId) {
        this.items = await invoke<TItem[]>(commands.updateCmd, {
          id: existingId,
          [commands.updateArgKey]: savedItem,
        });
      } else {
        this.items = await invoke<TItem[]>(commands.createCmd, {
          [commands.createArgKey]: savedItem,
        });
      }
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  protected async deleteEntity(command: string, idToDelete: string) {
    const backup = this.items;
    this.items = this.items.filter((item) => item.id !== idToDelete);

    try {
      this.items = await invoke<TItem[]>(command, { id: idToDelete });
      delete this.statusMap[idToDelete];
    } catch (err) {
      this.errorMessage = String(err);
      this.items = backup;
      throw err;
    }
  }

  protected async reorderEntities(
    command: string,
    argKey: string,
    sanitizedItems: TItem[],
  ) {
    const previous = this.items;
    this.items = sanitizedItems;

    try {
      this.items = await invoke<TItem[]>(command, {
        [argKey]: sanitizedItems,
      });
    } catch (err) {
      this.errorMessage = String(err);
      this.items = previous;
      throw err;
    }
  }
}
