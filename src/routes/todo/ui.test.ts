import { describe, expect, it, vi } from "vitest";
import { render } from "svelte/server";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { soundManager } from "$lib/sound.svelte";
import { fromDateTimeLocalValue, toDateTimeLocalValue } from "./format";
import TodoCategorySection from "./todo-category-section.svelte";
import TodoItemRow from "./todo-item-row.svelte";
import { TodoStore } from "./todo.svelte";
import type { TodoItem } from "./types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

vi.mock("$lib/sound.svelte", () => ({
  soundManager: {
    play: vi.fn().mockResolvedValue(undefined),
  },
}));

const NOW = Date.now();

function makeTodo(overrides: Partial<TodoItem> = {}): TodoItem {
  return {
    id: "todo-1",
    title: "Renew TLS certificate",
    completed: false,
    completed_at: null,
    due_at: null,
    reminder_at: null,
    reminder_sent: false,
    recurrence: null,
    created_at: NOW - 10_000,
    sort_order: 0,
    ...overrides,
  };
}

describe("todo format helpers", () => {
  it("round-trips local datetime values", () => {
    const sample = new Date(2026, 4, 15, 9, 30, 0, 0).getTime();
    const formatted = toDateTimeLocalValue(sample);
    expect(formatted).toBe("2026-05-15T09:30");
    expect(fromDateTimeLocalValue(formatted)).toBe(sample);
    expect(toDateTimeLocalValue(null)).toBe("");
    expect(fromDateTimeLocalValue("")).toBeNull();
  });

  it("throws on invalid datetime strings", () => {
    expect(() => fromDateTimeLocalValue("not-a-date")).toThrow();
  });
});

describe("TodoCategorySection", () => {
  it("renders nothing when items is empty", () => {
    const { html } = render(TodoCategorySection, {
      props: {
        title: "Overdue",
        items: [],
        variant: "error",
        onedit: () => {},
      },
    });
    expect(html.replace(/<!--[\s\S]*?-->/g, "").trim()).toBe("");
  });

  it("renders category header and items when non-empty", () => {
    const { html } = render(TodoCategorySection, {
      props: {
        title: "Active",
        items: [makeTodo({ title: "Patch gateway" })],
        onedit: () => {},
      },
    });
    expect(html).toContain("Active");
    expect(html).toContain("Patch gateway");
    expect(html).toContain('aria-expanded="true"');
  });
});

describe("TodoItemRow", () => {
  it("renders checkbox, status dot, due badge, reminder badge, and recurrence pill", () => {
    const item = makeTodo({
      due_at: NOW - 60_000,
      reminder_at: NOW + 60_000,
      recurrence: "weekly",
    });
    const { html } = render(TodoItemRow, {
      props: {
        item,
        onedit: () => {},
      },
    });
    expect(html).toContain('type="checkbox"');
    expect(html).toContain('class="status-dot error"');
    expect(html).toContain("Renew TLS certificate");
    expect(html).toContain("Weekly");
    expect(html).toContain("Due ");
    expect(html).toMatch(/Reminder\s+/);
  });

  it("renders completed state with line-through class and success status dot", () => {
    const item = makeTodo({
      completed: true,
      completed_at: NOW,
    });
    const { html } = render(TodoItemRow, {
      props: {
        item,
        onedit: () => {},
      },
    });
    expect(html).toMatch(/class="[^"]*\btodo-row\b[^"]*\bcompleted\b/);
    expect(html).toContain('class="status-dot success"');
    expect(html).toContain("checked");
  });
});

describe("TodoStore reminder sound", () => {
  it("plays notification sound when workspace:reminder-triggered fires and cleans up listener on destroy", async () => {
    const handlers = new Map<string, () => void>();
    const unlistenReminder = vi.fn<() => void>();

    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "get_app_settings_cmd") return {};
      if (cmd === "workspace_load_todos") return [];
      return null;
    });

    vi.mocked(listen).mockImplementation(async (event, handler) => {
      handlers.set(event, handler as () => void);
      return event === "workspace:reminder-triggered"
        ? unlistenReminder
        : () => {};
    });

    const store = new TodoStore();
    await store.init();

    const reminderHandler = handlers.get("workspace:reminder-triggered");
    expect(reminderHandler).toBeDefined();
    reminderHandler?.();

    expect(soundManager.play).toHaveBeenCalledWith("notification");

    store.destroy();
    expect(unlistenReminder).toHaveBeenCalledTimes(1);
  });
});
