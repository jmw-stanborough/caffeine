<script lang="ts">
  import TodoItemRow from "./todo-item-row.svelte";
  import type { TodoItem } from "./types";

  interface Props {
    title: string;
    items: readonly TodoItem[];
    variant?: "error" | "neutral" | "success";
    onedit: (item: TodoItem) => void;
  }

  let {
    title,
    items,
    variant = "neutral",
    onedit,
  }: Props = $props();

  let collapsed = $state(false);
</script>

{#if items.length > 0}
  <section class="category-section" class:variant-error={variant === "error"}>
    <button
      type="button"
      class="category-header"
      aria-expanded={!collapsed}
      onclick={() => (collapsed = !collapsed)}
    >
      <svg
        class="chevron"
        class:collapsed
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <polyline points="6 9 12 15 18 9" />
      </svg>
      <span class="category-title">{title}</span>
      <span class="category-count">{items.length}</span>
    </button>

    {#if !collapsed}
      <ul class="category-list">
        {#each items as item (item.id)}
          <TodoItemRow {item} {onedit} />
        {/each}
      </ul>
    {/if}
  </section>
{/if}

<style>
  .category-section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .category-header {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    width: fit-content;
    padding: 0.25rem 0.5rem 0.25rem 0.25rem;
    background: transparent;
    border: none;
    border-radius: 0.375rem;
    color: var(--color-text);
    font-size: 0.95rem;
    font-weight: 700;
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease;
  }

  .category-header:hover {
    background-color: var(--color-surface);
  }

  .category-section.variant-error .category-title {
    color: var(--color-failure);
  }

  .chevron {
    color: var(--color-text-muted);
    transition: transform 0.15s ease;
  }

  .chevron.collapsed {
    transform: rotate(-90deg);
  }

  .category-count {
    font-size: 0.78rem;
    font-weight: 600;
    padding: 0.1rem 0.45rem;
    border-radius: 9999px;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text-muted);
  }

  .category-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }
</style>
