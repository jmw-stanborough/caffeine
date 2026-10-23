<script lang="ts" generics="T, TContext = unknown">
  import Modal from "$lib/components/modal.svelte";
  import {
    NUMBER_DIRECTION_LABELS,
    STATUS_DIRECTION_LABELS,
    TEXT_DIRECTION_LABELS,
  } from "$lib/sort/sort";
  import type {
    SortDirection,
    SortDirectionLabels,
    SortFieldDefinition,
    SortRule,
  } from "$lib/sort/types";

  let {
    open = $bindable(false),
    title = "Sort by",
    rules = $bindable<SortRule[]>([]),
    fields,
    onApplyToOrder = undefined,
  }: {
    open: boolean;
    title?: string;
    rules: SortRule[];
    fields: readonly SortFieldDefinition<T, TContext>[];
    onApplyToOrder?: () => void;
  } = $props();

  function getField(fieldId: string): SortFieldDefinition<T, TContext> | undefined {
    return fields.find((f) => f.id === fieldId);
  }

  function getDirectionLabels(fieldId: string): SortDirectionLabels {
    const field = getField(fieldId);
    if (field?.directionLabels) {
      return field.directionLabels;
    }
    if (fieldId === "status") {
      return STATUS_DIRECTION_LABELS;
    }
    if (fieldId === "latency") {
      return NUMBER_DIRECTION_LABELS;
    }
    return TEXT_DIRECTION_LABELS;
  }

  function getFirstUnusedFieldId(): string {
    const used = new Set(rules.map((r) => r.fieldId));
    const unused = fields.find((f) => !used.has(f.id));
    return (unused ?? fields[0])?.id ?? "";
  }

  function addRule() {
    const fieldId = getFirstUnusedFieldId();
    if (!fieldId) return;

    const field = getField(fieldId);
    const direction: SortDirection = field?.defaultDirection ?? (fieldId === "status" ? "desc" : "asc");

    rules = [
      ...rules,
      {
        id: `rule-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        fieldId,
        direction,
      },
    ];
  }

  function removeRule(index: number) {
    rules = rules.filter((_, i) => i !== index);
  }

  function moveRule(index: number, delta: number) {
    const target = index + delta;
    if (target < 0 || target >= rules.length) return;

    const updated = [...rules];
    const temp = updated[index];
    updated[index] = updated[target];
    updated[target] = temp;
    rules = updated;
  }

  function handleFieldChange(index: number, newFieldId: string) {
    const field = getField(newFieldId);
    const updated = [...rules];
    updated[index] = {
      ...updated[index],
      fieldId: newFieldId,
      direction: field?.defaultDirection ?? (newFieldId === "status" ? "desc" : "asc"),
    };
    rules = updated;
  }

  function handleDirectionChange(index: number, newDirection: SortDirection) {
    const updated = [...rules];
    updated[index] = {
      ...updated[index],
      direction: newDirection,
    };
    rules = updated;
  }

  function resetRules() {
    rules = [];
  }
</script>

<Modal bind:open maxWidth="34rem" ariaLabel={title}>
  <div class="modal-header">
    <div class="header-main">
      <h2 class="modal-title">{title}</h2>
      <span class="sort-description">
        Order items by one or more attributes. Rules are evaluated from top to bottom.
      </span>
    </div>
    <button
      type="button"
      class="btn-close"
      onclick={() => (open = false)}
      aria-label="Close sort settings"
    >
      ✕
    </button>
  </div>

  <div class="modal-body sort-modal-body">
    {#if rules.length === 0}
      <div class="sort-empty-state">
        <p class="empty-text">No active sort rules. Items are in custom manual order.</p>
        <button type="button" class="btn-add-initial" onclick={addRule}>
          + Add sort rule
        </button>
      </div>
    {:else}
      <div class="rules-list" role="list" aria-label="Active sort rules">
        {#each rules as rule, index (rule.id)}
          {@const dirLabels = getDirectionLabels(rule.fieldId)}
          <div class="rule-row" role="listitem">
            <div class="rule-priority">
              <span class="priority-label">{index === 0 ? "Sort by" : "Then by"}</span>
            </div>

            <div class="rule-inputs">
              <select
                class="field-select"
                value={rule.fieldId}
                onchange={(e) => handleFieldChange(index, e.currentTarget.value)}
                aria-label="Sort attribute"
              >
                {#each fields as field (field.id)}
                  <option value={field.id}>{field.label}</option>
                {/each}
              </select>

              <select
                class="direction-select"
                value={rule.direction}
                onchange={(e) => handleDirectionChange(index, e.currentTarget.value as SortDirection)}
                aria-label="Sort direction"
              >
                <option value="asc">{dirLabels.asc}</option>
                <option value="desc">{dirLabels.desc}</option>
              </select>
            </div>

            <div class="rule-actions">
              <button
                type="button"
                class="btn-icon"
                disabled={index === 0}
                onclick={() => moveRule(index, -1)}
                title="Move up"
                aria-label="Move rule up"
              >
                ↑
              </button>
              <button
                type="button"
                class="btn-icon"
                disabled={index === rules.length - 1}
                onclick={() => moveRule(index, 1)}
                title="Move down"
                aria-label="Move rule down"
              >
                ↓
              </button>
              <button
                type="button"
                class="btn-icon btn-remove"
                onclick={() => removeRule(index)}
                title="Remove rule"
                aria-label="Remove sort rule"
              >
                ✕
              </button>
            </div>
          </div>
        {/each}
      </div>

      {#if rules.length < fields.length}
        <button type="button" class="btn-add-more" onclick={addRule}>
          + Add another attribute
        </button>
      {/if}
    {/if}
  </div>

  <div class="modal-footer">
    <div class="footer-left">
      {#if rules.length > 0}
        <button type="button" class="btn-cancel" onclick={resetRules}>
          Reset to manual order
        </button>
        {#if onApplyToOrder}
          <button
            type="button"
            class="btn-apply-order"
            onclick={() => {
              onApplyToOrder();
              open = false;
            }}
            title="Set current sorted order as default manual order"
          >
            Save as manual order
          </button>
        {/if}
      {/if}
    </div>

    <div class="footer-actions">
      <button type="button" class="btn-save" onclick={() => (open = false)}>
        Done
      </button>
    </div>
  </div>
</Modal>

<style>
  .sort-description {
    font-size: 0.82rem;
    color: var(--color-text-muted);
  }

  .sort-modal-body {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding-top: 0.25rem;
  }

  .sort-empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 1.75rem 1rem;
    border: 1px dashed var(--color-border);
    border-radius: 0.45rem;
    background-color: color-mix(in srgb, var(--color-surface) 40%, transparent);
  }

  .empty-text {
    margin: 0;
    font-size: 0.9rem;
    color: var(--color-text-muted);
    text-align: center;
  }

  .btn-add-initial,
  .btn-add-more {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    background-color: var(--color-base);
    border: 1px dashed var(--color-border);
    color: var(--color-text);
    padding: 0.45rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: border-color 0.15s ease, color 0.15s ease;
  }

  .btn-add-initial:hover,
  .btn-add-more:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .rules-list {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .rule-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.45rem;
    padding: 0.55rem 0.75rem;
  }

  .rule-priority {
    width: 4.5rem;
    flex-shrink: 0;
  }

  .priority-label {
    font-size: 0.82rem;
    font-weight: 700;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .rule-inputs {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
    min-width: 0;
  }

  .field-select {
    flex: 1;
    min-width: 7rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    color: var(--color-text);
    padding: 0.4rem 0.55rem;
    font-size: 0.85rem;
    outline: none;
    cursor: pointer;
  }

  .direction-select {
    flex: 1.4;
    min-width: 9rem;
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    color: var(--color-text);
    padding: 0.4rem 0.55rem;
    font-size: 0.85rem;
    outline: none;
    cursor: pointer;
  }

  .field-select:focus,
  .direction-select:focus {
    border-color: var(--color-primary);
  }

  .rule-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    flex-shrink: 0;
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    background: transparent;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.3rem;
    color: var(--color-text-muted);
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .btn-icon:hover:not(:disabled) {
    background-color: var(--color-surface);
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }

  .btn-icon:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .btn-icon.btn-remove:hover:not(:disabled) {
    background-color: color-mix(in srgb, var(--color-failure) 20%, transparent);
    color: var(--color-failure);
    border-color: var(--color-failure);
  }

  .footer-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .btn-apply-order {
    background-color: transparent;
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text);
    padding: 0.45rem 0.85rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    cursor: pointer;
    transition: border-color 0.15s ease, color 0.15s ease;
  }

  .btn-apply-order:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }
</style>
