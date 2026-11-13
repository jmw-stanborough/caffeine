<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import { fromDateTimeLocalValue, toDateTimeLocalValue } from "./format";
  import type {
    CreateTodoInput,
    RecurrenceRule,
    TodoItem,
    UpdateTodoInput,
  } from "./types";

  interface Props {
    open: boolean;
    item?: TodoItem;
    onsave: (
      payload: CreateTodoInput | UpdateTodoInput,
      existingId?: string,
    ) => Promise<void>;
  }

  let { open = $bindable(false), item = undefined, onsave }: Props = $props();

  let title = $state("");
  let dueLocal = $state("");
  let reminderLocal = $state("");
  let recurrence = $state<RecurrenceRule>("none");
  let saving = $state(false);
  let formError = $state<string | null>(null);

  $effect(() => {
    if (open) {
      formError = null;
      saving = false;
      if (item) {
        title = item.title;
        dueLocal = toDateTimeLocalValue(item.due_at);
        reminderLocal = toDateTimeLocalValue(item.reminder_at);
        recurrence = item.recurrence ?? "none";
      } else {
        title = "";
        dueLocal = "";
        reminderLocal = "";
        recurrence = "none";
      }
    }
  });

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    const trimmed = title.trim();
    if (!trimmed) {
      formError = "Task title is required.";
      return;
    }

    saving = true;
    formError = null;
    try {
      const base: CreateTodoInput = {
        title: trimmed,
        due_at: fromDateTimeLocalValue(dueLocal),
        reminder_at: fromDateTimeLocalValue(reminderLocal),
        recurrence: recurrence === "none" ? null : recurrence,
      };

      if (item) {
        await onsave({ ...base, id: item.id }, item.id);
      } else {
        await onsave(base);
      }
      open = false;
    } catch (err) {
      formError = String(err);
    } finally {
      saving = false;
    }
  }
</script>

<Modal
  bind:open
  maxWidth="28rem"
  ariaLabel={item ? `Edit task ${item.title}` : "Add task"}
>
  <form class="todo-modal-form" onsubmit={handleSubmit}>
    <div class="modal-header">
      <h2 class="modal-title">{item ? "Edit Task" : "Add Task"}</h2>
      <button
        type="button"
        class="btn-close"
        onclick={() => (open = false)}
        aria-label="Close modal"
      >
        ✕
      </button>
    </div>

    <div class="modal-body">
      {#if formError}
        <div class="error-banner">{formError}</div>
      {/if}

      <label class="form-field">
        <span>Title</span>
        <input
          type="text"
          bind:value={title}
          placeholder="What needs to be done?"
          required
        />
      </label>

      <div class="form-field">
        <span>Due Date &amp; Time (Optional)</span>
        <div class="datetime-row">
          <input type="datetime-local" bind:value={dueLocal} />
          {#if dueLocal}
            <button
              type="button"
              class="btn-clear"
              onclick={() => (dueLocal = "")}
            >
              Clear
            </button>
          {/if}
        </div>
      </div>

      <div class="form-field">
        <span>Reminder Date &amp; Time (Optional)</span>
        <div class="datetime-row">
          <input type="datetime-local" bind:value={reminderLocal} />
          {#if reminderLocal}
            <button
              type="button"
              class="btn-clear"
              onclick={() => (reminderLocal = "")}
            >
              Clear
            </button>
          {/if}
        </div>
      </div>

      <label class="form-field">
        <span>Recurrence</span>
        <select bind:value={recurrence}>
          <option value="none">None</option>
          <option value="daily">Daily</option>
          <option value="weekly">Weekly</option>
          <option value="monthly">Monthly</option>
        </select>
      </label>
    </div>

    <div class="modal-footer">
      <div class="footer-actions">
        <button
          type="button"
          class="btn-cancel"
          onclick={() => (open = false)}
          disabled={saving}
        >
          Cancel
        </button>
        <button type="submit" class="btn-save" disabled={saving}>
          {saving ? "Saving..." : item ? "Save Changes" : "Add Task"}
        </button>
      </div>
    </div>
  </form>
</Modal>

<style>
  .todo-modal-form {
    display: flex;
    flex-direction: column;
    gap: 1.1rem;
  }

  .datetime-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .datetime-row input {
    flex: 1;
    min-width: 0;
  }

  .form-field select {
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    padding: 0.5rem 0.65rem;
    color: var(--color-text);
    font-size: 0.9rem;
    outline: none;
  }

  .form-field select:focus {
    border-color: var(--color-primary);
  }

  .btn-clear {
    background: transparent;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    color: var(--color-text-muted);
    padding: 0.45rem 0.65rem;
    font-size: 0.8rem;
    cursor: pointer;
    flex-shrink: 0;
  }

  .btn-clear:hover {
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }
</style>
