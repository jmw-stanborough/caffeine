import type { TodoCategories, TodoItem, TodoStatus } from "./types";

export const DEFAULT_WARNING_MINUTES = 120;

const MS_PER_MINUTE = 60_000;

export function resolveTodoStatus(
  item: TodoItem,
  nowMs: number,
  warningMinutes: number = DEFAULT_WARNING_MINUTES,
): TodoStatus {
  if (item.completed) {
    return "success";
  }

  if (item.due_at !== null) {
    if (nowMs > item.due_at) {
      return "error";
    }
    if (item.due_at - nowMs <= warningMinutes * MS_PER_MINUTE) {
      return "warning";
    }
  }

  return "neutral";
}

export function categorizeTodos(
  items: readonly TodoItem[],
  nowMs: number,
): TodoCategories {
  const overdue: TodoItem[] = [];
  const active: TodoItem[] = [];
  const completed: TodoItem[] = [];

  for (const item of items) {
    if (item.completed) {
      completed.push(item);
    } else if (item.due_at !== null && nowMs > item.due_at) {
      overdue.push(item);
    } else {
      active.push(item);
    }
  }

  completed.sort(
    (a, b) =>
      (b.completed_at ?? 0) - (a.completed_at ?? 0) ||
      b.created_at - a.created_at,
  );

  overdue.sort(
    (a, b) =>
      (a.due_at as number) - (b.due_at as number) ||
      b.created_at - a.created_at,
  );

  active.sort((a, b) => {
    if (a.due_at !== null && b.due_at !== null) {
      return a.due_at - b.due_at || b.created_at - a.created_at;
    }
    if (a.due_at !== null) return -1;
    if (b.due_at !== null) return 1;
    return b.created_at - a.created_at;
  });

  return { overdue, active, completed };
}
