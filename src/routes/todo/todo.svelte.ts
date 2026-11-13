import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { formatAttentionDetail } from "./format";
import {
  DEFAULT_WARNING_MINUTES,
  categorizeTodos,
  resolveTodoStatus,
} from "./status";
import type {
  CreateTodoInput,
  TodoAttentionItem,
  TodoCategories,
  TodoItem,
  TodoStatusCounts,
  UpdateTodoInput,
} from "./types";

const TODO_CLOCK_TICK_MS = 30_000;
const TODOS_UPDATED_EVENT = "workspace:todos-updated";

export class TodoStore {
  items = $state<TodoItem[]>([]);
  loading = $state(false);
  errorMessage = $state<string | null>(null);
  warningMinutes = $state(DEFAULT_WARNING_MINUTES);
  nowMs = $state(Date.now());

  private initPromise: Promise<void> | null = null;
  private timer: ReturnType<typeof setInterval> | null = null;
  private unlistenUpdated: UnlistenFn | null = null;
  private destroyed = false;

  private categoriesDerived = $derived.by<TodoCategories>(() =>
    categorizeTodos(this.items, this.nowMs),
  );

  private warningActiveDerived = $derived.by<TodoItem[]>(() =>
    this.categoriesDerived.active.filter(
      (item) =>
        resolveTodoStatus(item, this.nowMs, this.warningMinutes) === "warning",
    ),
  );

  get categories(): TodoCategories {
    return this.categoriesDerived;
  }

  get attentionCount(): number {
    return (
      this.categoriesDerived.overdue.length + this.warningActiveDerived.length
    );
  }

  get statusCounts(): TodoStatusCounts {
    const overdueCount = this.categoriesDerived.overdue.length;
    const warningCount = this.warningActiveDerived.length;
    const neutralActiveCount =
      this.categoriesDerived.active.length - warningCount;

    return {
      online: neutralActiveCount,
      degraded: warningCount,
      offline: overdueCount,
      checking: 0,
    };
  }

  get completedCount(): number {
    return this.categoriesDerived.completed.length;
  }

  get attentionItems(): TodoAttentionItem[] {
    const overdueItems: TodoAttentionItem[] =
      this.categoriesDerived.overdue.map((item) => ({
        id: item.id,
        label: item.title,
        detail: formatAttentionDetail(item.due_at as number, this.nowMs, true),
        severity: "offline",
      }));

    const warningItems: TodoAttentionItem[] = this.warningActiveDerived.map(
      (item) => ({
        id: item.id,
        label: item.title,
        detail: formatAttentionDetail(
          item.due_at as number,
          this.nowMs,
          false,
        ),
        severity: "degraded",
      }),
    );

    return [...overdueItems, ...warningItems];
  }

  async init(): Promise<void> {
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = (async () => {
      this.destroyed = false;
      this.startClock();
      await this.subscribeUpdates();
      await Promise.all([this.refreshSettings(), this.fetchTodos()]);
    })().catch((err) => {
      this.stopListeners();
      this.initPromise = null;
      throw err;
    });

    return this.initPromise;
  }

  destroy(): void {
    this.destroyed = true;
    this.stopListeners();
    this.initPromise = null;
  }

  private startClock(): void {
    if (this.timer !== null) {
      clearInterval(this.timer);
    }
    this.nowMs = Date.now();
    this.timer = setInterval(() => {
      this.nowMs = Date.now();
    }, TODO_CLOCK_TICK_MS);
  }

  private async subscribeUpdates(): Promise<void> {
    if (this.unlistenUpdated !== null) {
      this.unlistenUpdated();
      this.unlistenUpdated = null;
    }

    const unlisten = await listen(TODOS_UPDATED_EVENT, () => {
      this.nowMs = Date.now();
      this.fetchTodos().catch((err) => {
        console.error("Failed to refresh todos after reminder update:", err);
      });
    });

    if (this.destroyed) {
      unlisten();
    } else {
      this.unlistenUpdated = unlisten;
    }
  }

  private stopListeners(): void {
    if (this.timer !== null) {
      clearInterval(this.timer);
      this.timer = null;
    }
    this.unlistenUpdated?.();
    this.unlistenUpdated = null;
  }

  refreshSettings = async (): Promise<void> => {
    try {
      const settings = await invoke<{ todo_warning_minutes?: number }>(
        "get_app_settings_cmd",
      );
      this.warningMinutes =
        settings.todo_warning_minutes ?? DEFAULT_WARNING_MINUTES;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  };

  fetchTodos = async (): Promise<void> => {
    this.loading = true;
    this.errorMessage = null;
    try {
      this.items = await invoke<TodoItem[]>("workspace_load_todos");
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    } finally {
      this.loading = false;
    }
  };

  async createTodo(input: CreateTodoInput): Promise<TodoItem> {
    this.errorMessage = null;
    try {
      const created = await invoke<TodoItem>("workspace_create_todo", {
        input,
      });
      this.items = [...this.items, created];
      return created;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  async updateTodo(input: UpdateTodoInput): Promise<TodoItem> {
    this.errorMessage = null;
    try {
      const updated = await invoke<TodoItem>("workspace_update_todo", {
        input,
      });
      this.items = this.items.map((item) =>
        item.id === updated.id ? updated : item,
      );
      return updated;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  async toggleTodo(id: string): Promise<TodoItem[]> {
    this.errorMessage = null;
    try {
      const updated = await invoke<TodoItem[]>("workspace_toggle_todo", {
        id,
      });
      this.items = updated;
      return updated;
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }

  async deleteTodo(id: string): Promise<void> {
    this.errorMessage = null;
    try {
      await invoke("workspace_delete_todo", { id });
      this.items = this.items.filter((item) => item.id !== id);
    } catch (err) {
      this.errorMessage = String(err);
      throw err;
    }
  }
}

export const todoStore = new TodoStore();
