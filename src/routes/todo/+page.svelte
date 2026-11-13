<script lang="ts">
  import { onMount } from "svelte";
  import TodoCategorySection from "./todo-category-section.svelte";
  import TodoModal from "./todo-modal.svelte";
  import { todoStore } from "./todo.svelte";
  import type { CreateTodoInput, TodoItem, UpdateTodoInput } from "./types";

  let quickTitle = $state("");
  let quickSubmitting = $state(false);
  let isModalOpen = $state(false);
  let editingItem = $state<TodoItem | undefined>(undefined);

  let categories = $derived(todoStore.categories);
  let activeTotal = $derived(
    categories.overdue.length + categories.active.length,
  );
  let hasAnyItems = $derived(
    categories.overdue.length > 0 ||
      categories.active.length > 0 ||
      categories.completed.length > 0,
  );

  onMount(() => {
    todoStore.init().catch((err) => {
      console.error("Failed to initialize todo store:", err);
    });
  });

  function openCreateModal() {
    editingItem = undefined;
    isModalOpen = true;
  }

  function openEditModal(item: TodoItem) {
    editingItem = item;
    isModalOpen = true;
  }

  async function handleQuickAdd(e: SubmitEvent) {
    e.preventDefault();
    const trimmed = quickTitle.trim();
    if (!trimmed || quickSubmitting) return;

    quickSubmitting = true;
    try {
      await todoStore.createTodo({
        title: trimmed,
        due_at: null,
        reminder_at: null,
        recurrence: null,
      });
      quickTitle = "";
    } catch (err) {
      console.error("Failed to quick-add todo:", err);
    } finally {
      quickSubmitting = false;
    }
  }

  async function handleSaveModal(
    payload: CreateTodoInput | UpdateTodoInput,
    existingId?: string,
  ) {
    if (existingId && "id" in payload) {
      await todoStore.updateTodo(payload);
    } else {
      await todoStore.createTodo(payload);
    }
  }
</script>

<div class="page-container">
  <header class="page-header">
    <div class="header-titles">
      <h1 class="page-title">Todo</h1>
      <span class="item-count">{activeTotal} open</span>
    </div>
  </header>

  <form class="quick-add-bar" onsubmit={handleQuickAdd}>
    <input
      type="text"
      class="quick-add-input"
      bind:value={quickTitle}
      placeholder="Add a task and press Enter..."
      aria-label="Quick add task"
      disabled={quickSubmitting}
    />
    <button
      type="button"
      class="btn-toolbar"
      onclick={openCreateModal}
    >
      Add with Options
    </button>
  </form>

  {#if todoStore.errorMessage}
    <div class="error-banner">
      <span>{todoStore.errorMessage}</span>
      <button
        type="button"
        class="btn-retry"
        onclick={() => {
          todoStore.fetchTodos().catch((err) => {
            console.error("Failed to retry loading todos:", err);
          });
        }}
      >
        Retry
      </button>
    </div>
  {/if}

  {#if todoStore.loading && todoStore.items.length === 0}
    <p class="status-message">Loading tasks…</p>
  {:else if !hasAnyItems}
    <div class="empty-state">
      <p class="status-message">
        No tasks yet. Type a task above or click "Add with Options" to schedule one.
      </p>
    </div>
  {:else}
    <div class="categories-stack">
      <TodoCategorySection
        title="Overdue"
        items={categories.overdue}
        variant="error"
        onedit={openEditModal}
      />
      <TodoCategorySection
        title="Active"
        items={categories.active}
        variant="neutral"
        onedit={openEditModal}
      />
      <TodoCategorySection
        title="Completed"
        items={categories.completed}
        variant="success"
        onedit={openEditModal}
      />
    </div>
  {/if}
</div>

<TodoModal
  bind:open={isModalOpen}
  item={editingItem}
  onsave={handleSaveModal}
/>

<style>
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .header-titles {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
  }

  .item-count {
    font-size: 0.9rem;
    color: var(--color-text-muted);
  }

  .quick-add-bar {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .quick-add-input {
    flex: 1;
    min-width: 0;
    height: 2.25rem;
    padding: 0 0.75rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    color: var(--color-text);
    font-size: 0.92rem;
    outline: none;
    transition: border-color 0.15s ease;
  }

  .quick-add-input:focus {
    border-color: var(--color-primary);
  }

  .quick-add-bar .btn-toolbar {
    height: 2.25rem;
  }

  .categories-stack {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .empty-state {
    padding: 2rem 1rem;
    border: dashed var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    text-align: center;
  }
</style>
