<script lang="ts">
  import StatusDot from "$lib/components/status-dot.svelte";
  import { formatTodoTimestamp } from "./format";
  import { resolveTodoStatus } from "./status";
  import { todoStore } from "./todo.svelte";
  import type { RecurrenceRule, TodoItem } from "./types";

  interface Props {
    item: TodoItem;
    onedit: (item: TodoItem) => void;
  }

  let { item, onedit }: Props = $props();

  const RECURRENCE_LABELS: Record<Exclude<RecurrenceRule, "none">, string> = {
    daily: "Daily",
    weekly: "Weekly",
    monthly: "Monthly",
  };

  let status = $derived(
    resolveTodoStatus(item, todoStore.nowMs, todoStore.warningMinutes),
  );

  let recurrenceLabel = $derived(
    item.recurrence && item.recurrence !== "none"
      ? RECURRENCE_LABELS[item.recurrence]
      : null,
  );

  function handleToggle() {
    todoStore.toggleTodo(item.id).catch((err) => {
      console.error("Failed to toggle todo:", err);
    });
  }

  function handleDelete() {
    todoStore.deleteTodo(item.id).catch((err) => {
      console.error("Failed to delete todo:", err);
    });
  }
</script>

<li class="todo-row" class:completed={item.completed}>
  <div class="todo-main">
    <input
      type="checkbox"
      class="todo-checkbox"
      checked={item.completed}
      onchange={handleToggle}
      aria-label="Mark {item.title} as {item.completed ? 'incomplete' : 'complete'}"
    />

    <StatusDot {status} size={8} />

    <div class="todo-content">
      <span class="todo-title selectable">{item.title}</span>

      {#if item.due_at !== null || item.reminder_at !== null || recurrenceLabel}
        <div class="todo-meta">
          {#if item.due_at !== null}
            <span
              class="meta-badge"
              class:badge-error={status === "error"}
              class:badge-warning={status === "warning"}
            >
              Due {formatTodoTimestamp(item.due_at)}
            </span>
          {/if}

          {#if item.reminder_at !== null}
            <span class="meta-badge" class:badge-muted={item.reminder_sent}>
              {item.reminder_sent ? "Reminded" : "Reminder"}
              {formatTodoTimestamp(item.reminder_at)}
            </span>
          {/if}

          {#if recurrenceLabel}
            <span class="meta-badge badge-recurrence">{recurrenceLabel}</span>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <div class="todo-actions">
    <button
      type="button"
      class="btn-action btn-row-action"
      onclick={() => onedit(item)}
      aria-label="Edit {item.title}"
    >
      Edit
    </button>
    <button
      type="button"
      class="btn-action btn-row-action btn-row-delete"
      onclick={handleDelete}
      aria-label="Delete {item.title}"
    >
      Delete
    </button>
  </div>
</li>

<style>
  .todo-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.65rem 0.85rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .todo-row:hover {
    border-color: var(--color-text-muted);
  }

  .todo-row.completed {
    opacity: 0.75;
  }

  .todo-main {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    min-width: 0;
    flex: 1;
  }

  .todo-checkbox {
    width: 1rem;
    height: 1rem;
    margin: 0;
    accent-color: var(--color-primary);
    cursor: pointer;
    flex-shrink: 0;
  }

  .todo-content {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    min-width: 0;
    flex: 1;
  }

  .todo-title {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--color-text);
    word-break: break-word;
  }

  .todo-row.completed .todo-title {
    text-decoration: line-through;
    color: var(--color-text-muted);
  }

  .todo-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .meta-badge {
    display: inline-flex;
    align-items: center;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.45rem;
    border-radius: 0.25rem;
    border: solid var(--width-border) var(--color-border);
    background-color: var(--color-base);
    color: var(--color-text-muted);
    white-space: nowrap;
  }

  .meta-badge.badge-error {
    background-color: color-mix(in srgb, var(--color-failure) 16%, var(--color-base));
    border-color: var(--color-failure);
    color: var(--color-failure);
  }

  .meta-badge.badge-warning {
    background-color: color-mix(in srgb, var(--color-warning) 16%, var(--color-base));
    border-color: var(--color-warning);
    color: var(--color-warning);
  }

  .meta-badge.badge-recurrence {
    color: var(--color-link);
    border-color: color-mix(in srgb, var(--color-link) 45%, var(--color-border));
  }

  .meta-badge.badge-muted {
    opacity: 0.7;
  }

  .todo-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .btn-row-action {
    padding: 0 0.6rem;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .btn-row-action:hover {
    background-color: var(--color-base);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .btn-row-delete:hover {
    border-color: var(--color-failure);
    color: var(--color-failure);
  }
</style>
