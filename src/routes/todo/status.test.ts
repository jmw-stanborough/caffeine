import { describe, expect, it } from "vitest";
import {
  DEFAULT_WARNING_MINUTES,
  categorizeTodos,
  resolveTodoStatus,
} from "./status";
import type { TodoItem } from "./types";

const MINUTE_MS = 60_000;
const NOW = 1_700_000_000_000;

function makeTodo(overrides: Partial<TodoItem> = {}): TodoItem {
  return {
    id: "todo-1",
    title: "Test task",
    completed: false,
    completed_at: null,
    due_at: null,
    reminder_at: null,
    reminder_sent: false,
    reminder_acknowledged: false,
    recurrence: null,
    created_at: NOW - 10_000,
    sort_order: 0,
    ...overrides,
  };
}

describe("resolveTodoStatus", () => {
  it("returns success when item is completed regardless of due_at", () => {
    const completedPastDue = makeTodo({
      completed: true,
      completed_at: NOW - 1_000,
      due_at: NOW - 60 * MINUTE_MS,
    });
    expect(resolveTodoStatus(completedPastDue, NOW)).toBe("success");
  });

  it("returns neutral when item is uncompleted and has no due_at", () => {
    const simple = makeTodo({ due_at: null });
    expect(resolveTodoStatus(simple, NOW)).toBe("neutral");
  });

  it("returns error when nowMs is strictly greater than due_at", () => {
    const overdue = makeTodo({ due_at: NOW - 1 });
    expect(resolveTodoStatus(overdue, NOW)).toBe("error");
  });

  it("returns warning when due_at equals nowMs or is within warning window", () => {
    const dueNow = makeTodo({ due_at: NOW });
    expect(resolveTodoStatus(dueNow, NOW)).toBe("warning");

    const atBoundary = makeTodo({
      due_at: NOW + DEFAULT_WARNING_MINUTES * MINUTE_MS,
    });
    expect(resolveTodoStatus(atBoundary, NOW)).toBe("warning");
  });

  it("returns neutral when due_at is beyond the warning window", () => {
    const justOutside = makeTodo({
      due_at: NOW + DEFAULT_WARNING_MINUTES * MINUTE_MS + 1,
    });
    expect(resolveTodoStatus(justOutside, NOW)).toBe("neutral");
  });

  it("respects configurable warningMinutes", () => {
    const dueIn45m = makeTodo({ due_at: NOW + 45 * MINUTE_MS });
    expect(resolveTodoStatus(dueIn45m, NOW, 30)).toBe("neutral");
    expect(resolveTodoStatus(dueIn45m, NOW, 45)).toBe("warning");
    expect(resolveTodoStatus(dueIn45m, NOW, 60)).toBe("warning");

    const dueNow = makeTodo({ due_at: NOW });
    expect(resolveTodoStatus(dueNow, NOW, 0)).toBe("warning");
    const dueIn1m = makeTodo({ due_at: NOW + MINUTE_MS });
    expect(resolveTodoStatus(dueIn1m, NOW, 0)).toBe("neutral");
  });
});

describe("categorizeTodos", () => {
  it("buckets items into overdue, active, and completed", () => {
    const overdue = makeTodo({ id: "overdue", due_at: NOW - 5_000 });
    const dueExactlyNow = makeTodo({ id: "due-now", due_at: NOW });
    const activeFuture = makeTodo({ id: "future", due_at: NOW + 50_000 });
    const activeNoDue = makeTodo({ id: "no-due", due_at: null });
    const done = makeTodo({
      id: "done",
      completed: true,
      completed_at: NOW - 1_000,
      due_at: NOW - 20_000,
    });

    const result = categorizeTodos(
      [done, activeNoDue, overdue, activeFuture, dueExactlyNow],
      NOW,
    );

    expect(result.overdue.map((t) => t.id)).toEqual(["overdue"]);
    expect(result.active.map((t) => t.id)).toEqual([
      "due-now",
      "future",
      "no-due",
    ]);
    expect(result.completed.map((t) => t.id)).toEqual(["done"]);
  });

  it("sorts overdue by due_at ascending", () => {
    const olderOverdue = makeTodo({ id: "older", due_at: NOW - 100_000 });
    const newerOverdue = makeTodo({ id: "newer", due_at: NOW - 10_000 });

    const result = categorizeTodos([newerOverdue, olderOverdue], NOW);
    expect(result.overdue.map((t) => t.id)).toEqual(["older", "newer"]);
  });

  it("sorts active by due_at ascending with nulls last, then created_at descending", () => {
    const dueSoon = makeTodo({
      id: "soon",
      due_at: NOW + 10_000,
      created_at: NOW - 50_000,
    });
    const dueLater = makeTodo({
      id: "later",
      due_at: NOW + 90_000,
      created_at: NOW - 1_000,
    });
    const nullOld = makeTodo({
      id: "null-old",
      due_at: null,
      created_at: NOW - 40_000,
    });
    const nullNew = makeTodo({
      id: "null-new",
      due_at: null,
      created_at: NOW - 5_000,
    });

    const result = categorizeTodos(
      [nullOld, dueLater, nullNew, dueSoon],
      NOW,
    );
    expect(result.active.map((t) => t.id)).toEqual([
      "soon",
      "later",
      "null-new",
      "null-old",
    ]);
  });

  it("sorts completed by completed_at descending, then created_at descending", () => {
    const firstDone = makeTodo({
      id: "first",
      completed: true,
      completed_at: NOW - 50_000,
      created_at: NOW - 60_000,
    });
    const secondDone = makeTodo({
      id: "second",
      completed: true,
      completed_at: NOW - 10_000,
      created_at: NOW - 80_000,
    });
    const sameTimeNewerCreated = makeTodo({
      id: "same-new",
      completed: true,
      completed_at: NOW - 10_000,
      created_at: NOW - 5_000,
    });

    const result = categorizeTodos(
      [firstDone, secondDone, sameTimeNewerCreated],
      NOW,
    );
    expect(result.completed.map((t) => t.id)).toEqual([
      "same-new",
      "second",
      "first",
    ]);
  });

  it("returns empty buckets when input is empty", () => {
    expect(categorizeTodos([], NOW)).toEqual({
      overdue: [],
      active: [],
      completed: [],
    });
  });
});
