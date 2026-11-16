<script lang="ts">
  import { AlertTriangle } from "$lib/components/icons";
  import { toErrorMessage } from "$lib/errors";
  import { formatTodoTimestamp } from "../../routes/todo/format";
  import { todoStore } from "../../routes/todo/todo.svelte";

  const SNOOZE_OPTIONS = [
    { label: "5m", minutes: 5 },
    { label: "15m", minutes: 15 },
    { label: "1h", minutes: 60 },
  ] as const;

  let busyIds = $state<Record<string, boolean>>({});
  let actionError = $state<string | null>(null);

  let reminders = $derived(todoStore.unacknowledgedReminders);

  async function runAction(id: string, fn: () => Promise<unknown>) {
    if (busyIds[id]) return;
    actionError = null;
    busyIds = { ...busyIds, [id]: true };
    try {
      await fn();
    } catch (err) {
      actionError = toErrorMessage(err);
    } finally {
      const next = { ...busyIds };
      delete next[id];
      busyIds = next;
    }
  }
</script>

{#if reminders.length > 0}
  <section class="reminder-banner" aria-label="Active task reminders">
    <div class="banner-header">
      <AlertTriangle size={16} />
      <span>
        {reminders.length} {reminders.length === 1 ? "Reminder" : "Reminders"} awaiting acknowledgment
      </span>
    </div>

    {#if actionError}
      <div class="banner-error">{actionError}</div>
    {/if}

    <ul class="reminder-list">
      {#each reminders as item (item.id)}
        {@const busy = Boolean(busyIds[item.id])}
        <li class="reminder-item">
          <div class="reminder-info">
            <a class="reminder-title" href="/todo">{item.title}</a>
            <span class="reminder-meta">
              {#if item.reminder_at !== null}
                Reminded {formatTodoTimestamp(item.reminder_at)}
              {/if}
              {#if item.due_at !== null}
                · Due {formatTodoTimestamp(item.due_at)}
              {/if}
            </span>
          </div>

          <div class="reminder-actions">
            <button
              type="button"
              class="btn-action btn-complete"
              disabled={busy}
              onclick={() => runAction(item.id, () => todoStore.toggleTodo(item.id))}
            >
              Complete
            </button>

            <div class="snooze-group" role="group" aria-label="Snooze {item.title}">
              <span class="snooze-label">Snooze:</span>
              {#each SNOOZE_OPTIONS as option}
                <button
                  type="button"
                  class="btn-action btn-snooze"
                  disabled={busy}
                  onclick={() =>
                    runAction(item.id, () =>
                      todoStore.snoozeReminder(item.id, option.minutes),
                    )}
                >
                  {option.label}
                </button>
              {/each}
            </div>

            <button
              type="button"
              class="btn-action btn-dismiss"
              disabled={busy}
              onclick={() =>
                runAction(item.id, () => todoStore.acknowledgeReminder(item.id))}
            >
              Dismiss
            </button>
          </div>
        </li>
      {/each}
    </ul>
  </section>
{/if}

<style>
  .reminder-banner {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.65rem 1.25rem;
    background-color: color-mix(in srgb, var(--color-warning) 14%, var(--color-surface));
    border-bottom: solid var(--width-border) var(--color-warning);
    flex-shrink: 0;
  }

  .banner-header {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--color-warning);
  }

  .banner-error {
    font-size: 0.8rem;
    color: var(--color-failure);
  }

  .reminder-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .reminder-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    flex-wrap: wrap;
    padding: 0.4rem 0.65rem;
    border-radius: 0.375rem;
    background-color: color-mix(in srgb, var(--color-base) 75%, transparent);
    border: solid var(--width-border) var(--color-border);
  }

  .reminder-info {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    flex-wrap: wrap;
    min-width: 0;
  }

  .reminder-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .reminder-title:hover {
    color: var(--color-primary);
    text-decoration: underline;
  }

  .reminder-meta {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .reminder-actions {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
  }

  .snooze-group {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

  .snooze-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .btn-action {
    height: 1.6rem;
    padding: 0 0.5rem;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .btn-complete {
    color: var(--color-success);
    border-color: color-mix(in srgb, var(--color-success) 50%, var(--color-border));
  }

  .btn-dismiss {
    color: var(--color-text-muted);
  }
</style>
