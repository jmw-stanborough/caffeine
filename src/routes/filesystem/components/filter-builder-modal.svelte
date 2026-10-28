<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import type {
    FilterCondition,
    FilterCriterion,
    FilterProperty,
    FilterTarget,
    SavedFilter,
  } from "../types";

  interface Props {
    open: boolean;
    onApply: (criteria: FilterCriterion[], matchAll: boolean) => void;
    onSave: (filter: SavedFilter) => Promise<void>;
  }

  let { open = $bindable(false), onApply, onSave }: Props = $props();

  let includeFiles = $state(true);
  let includeDirs = $state(true);
  let matchAll = $state(true);
  let filterName = $state("");
  let saveFeedback = $state<string | null>(null);

  interface LocalCriterion {
    id: string;
    property: FilterProperty;
    condition: FilterCondition;
    value: string;
    sizeUnit?: "B" | "KB" | "MB" | "GB";
  }

  let criteriaRows = $state<LocalCriterion[]>([
    {
      id: "rule-1",
      property: "name",
      condition: "contains",
      value: "",
    },
  ]);

  let availableProperties = $derived.by(() => {
    const list: { id: FilterProperty; label: string }[] = [
      { id: "name", label: "Item name" },
      { id: "path", label: "Path" },
      { id: "size", label: "Item size" },
    ];
    if (includeFiles) {
      list.splice(1, 0, { id: "extension", label: "File extension" });
      list.push({ id: "content", label: "Contents include" });
    }
    return list;
  });

  $effect(() => {
    if (!includeFiles) {
      for (const row of criteriaRows) {
        if (row.property === "extension" || row.property === "content") {
          row.property = "name";
          row.condition = "contains";
        }
      }
    }
  });

  function addRule() {
    criteriaRows = [
      ...criteriaRows,
      {
        id: `rule-${Date.now()}`,
        property: "name",
        condition: "contains",
        value: "",
      },
    ];
  }

  function removeRule(id: string) {
    if (criteriaRows.length <= 1) return;
    criteriaRows = criteriaRows.filter((r) => r.id !== id);
  }

  function getConditionsForProperty(prop: FilterProperty): { id: FilterCondition; label: string }[] {
    if (prop === "size") {
      return [
        { id: "gt", label: "greater than" },
        { id: "lt", label: "less than" },
        { id: "is", label: "equals" },
      ];
    }
    return [
      { id: "contains", label: "contains" },
      { id: "is", label: "is" },
      { id: "starts_with", label: "starts with" },
      { id: "ends_with", label: "ends with" },
      { id: "matches", label: "matches (regex)" },
      { id: "does_not_contain", label: "does not contain" },
    ];
  }

  function getCalculatedTarget(): FilterTarget {
    if (includeFiles && !includeDirs) return "files";
    if (!includeFiles && includeDirs) return "directories";
    return "both";
  }

  function buildPayloadCriteria(): FilterCriterion[] {
    const target = getCalculatedTarget();
    return criteriaRows
      .filter((row) => row.value.trim().length > 0)
      .map((row) => {
        let val = row.value.trim();
        if (row.property === "size") {
          const num = parseFloat(val) || 0;
          const mult =
            row.sizeUnit === "GB"
              ? 1024 * 1024 * 1024
              : row.sizeUnit === "MB"
                ? 1024 * 1024
                : row.sizeUnit === "KB"
                  ? 1024
                  : 1;
          val = Math.round(num * mult).toString();
        }
        return {
          target,
          property: row.property,
          condition: row.condition,
          value: val,
        };
      });
  }

  function handleApply() {
    const payload = buildPayloadCriteria();
    if (payload.length === 0) return;
    onApply(payload, matchAll);
    open = false;
  }

  async function handleSave() {
    if (!filterName.trim()) return;
    const payload = buildPayloadCriteria();
    if (payload.length === 0) return;

    await onSave({
      id: "",
      name: filterName.trim(),
      criteria: payload,
      match_all: matchAll,
      is_preset: false,
    });

    saveFeedback = `Saved "${filterName.trim()}" to roaming profile!`;
    setTimeout(() => {
      saveFeedback = null;
    }, 3000);
  }
</script>

<Modal bind:open maxWidth="42rem" ariaLabel="Filter and Search File System">
  <div class="filter-modal-content">
    <div class="modal-header">
      <h3 class="modal-title">Custom File Filter</h3>
      <button
        type="button"
        class="btn-close"
        onclick={() => (open = false)}
        aria-label="Close"
      >
        &times;
      </button>
    </div>

    <div class="scope-row">
      <span class="scope-label">Target Scope:</span>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={includeFiles} />
        <span>Files</span>
      </label>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={includeDirs} />
        <span>Directories</span>
      </label>

      <div class="match-mode-wrap">
        <label class="radio-label">
          <input type="radio" name="matchAll" value={true} bind:group={matchAll} />
          <span>Match All (AND)</span>
        </label>
        <label class="radio-label">
          <input type="radio" name="matchAll" value={false} bind:group={matchAll} />
          <span>Match Any (OR)</span>
        </label>
      </div>
    </div>

    <div class="rules-container">
      {#each criteriaRows as row (row.id)}
        <div class="rule-row">
          <select
            class="select-property"
            bind:value={row.property}
            onchange={() => {
              const conds = getConditionsForProperty(row.property);
              row.condition = conds[0].id;
              if (row.property === "size") row.sizeUnit = "MB";
            }}
          >
            {#each availableProperties as prop}
              <option value={prop.id}>{prop.label}</option>
            {/each}
          </select>

          <select class="select-condition" bind:value={row.condition}>
            {#each getConditionsForProperty(row.property) as cond}
              <option value={cond.id}>{cond.label}</option>
            {/each}
          </select>

          {#if row.property === "size"}
            <div class="size-input-group">
              <input
                type="number"
                class="input-val number-input"
                bind:value={row.value}
                placeholder="100"
                min="0"
              />
              <select class="select-unit" bind:value={row.sizeUnit}>
                <option value="B">B</option>
                <option value="KB">KB</option>
                <option value="MB">MB</option>
                <option value="GB">GB</option>
              </select>
            </div>
          {:else}
            <input
              type="text"
              class="input-val"
              bind:value={row.value}
              placeholder={row.property === "content" ? 'text to find...' : 'value...'}
            />
          {/if}

          <button
            type="button"
            class="btn-remove-rule"
            onclick={() => removeRule(row.id)}
            disabled={criteriaRows.length <= 1}
            title="Remove rule"
            aria-label="Remove rule"
          >
            &times;
          </button>
        </div>
      {/each}
    </div>

    <button type="button" class="btn-add-rule" onclick={addRule}>
      + Add Another Rule
    </button>

    <div class="save-section">
      <span class="save-title">Save to Roaming Profile:</span>
      <div class="save-row">
        <input
          type="text"
          class="input-filter-name"
          bind:value={filterName}
          placeholder="Filter name (e.g. Temp files, Custom logs)"
        />
        <button
          type="button"
          class="btn-secondary"
          onclick={handleSave}
          disabled={!filterName.trim()}
        >
          Save Filter
        </button>
      </div>
      {#if saveFeedback}
        <span class="save-feedback">{saveFeedback}</span>
      {/if}
    </div>

    <div class="modal-footer">
      <button type="button" class="btn-secondary" onclick={() => (open = false)}>
        Cancel
      </button>
      <button type="button" class="btn-primary" onclick={handleApply}>
        Apply & Search
      </button>
    </div>
  </div>
</Modal>

<style>
  .filter-modal-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.25rem;
    color: var(--color-text);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .modal-title {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
  }

  .btn-close {
    border: none;
    background: transparent;
    font-size: 1.25rem;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .scope-row {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 0.75rem;
    background-color: var(--color-base);
    border-radius: 0.375rem;
    font-size: 0.8125rem;
  }

  .scope-label {
    font-weight: 600;
  }

  .checkbox-label,
  .radio-label {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    cursor: pointer;
  }

  .match-mode-wrap {
    margin-left: auto;
    display: flex;
    gap: 0.75rem;
  }

  .rules-container {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-height: 240px;
    overflow-y: auto;
  }

  .rule-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  select,
  input {
    font: inherit;
    font-size: 0.8125rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    background-color: var(--color-base);
    color: var(--color-text);
    color-scheme: inherit;
    padding: 0.375rem 0.5rem;
  }

  select:focus,
  input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  option {
    background-color: var(--color-surface);
    color: var(--color-text);
  }

  .select-property {
    width: 140px;
  }

  .select-condition {
    width: 150px;
  }

  .input-val {
    flex: 1;
    min-width: 0;
  }

  .size-input-group {
    display: flex;
    flex: 1;
    gap: 0.25rem;
  }

  .number-input {
    flex: 1;
  }

  .select-unit {
    width: 70px;
  }

  .btn-remove-rule {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 1.25rem;
    cursor: pointer;
    padding: 0 0.25rem;
  }

  .btn-remove-rule:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .btn-add-rule {
    align-self: flex-start;
    border: none;
    background: transparent;
    color: var(--color-primary);
    font-size: 0.8125rem;
    cursor: pointer;
    padding: 0;
  }

  .btn-add-rule:hover {
    text-decoration: underline;
  }

  .save-section {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    padding-top: 0.5rem;
    border-top: solid var(--width-border) var(--color-border);
  }

  .save-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-muted);
  }

  .save-row {
    display: flex;
    gap: 0.5rem;
  }

  .input-filter-name {
    flex: 1;
  }

  .save-feedback {
    font-size: 0.75rem;
    color: var(--color-success, #22c55e);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding-top: 0.5rem;
    border-top: solid var(--width-border) var(--color-border);
  }

  .btn-secondary {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.25rem;
    background-color: var(--color-base);
    color: var(--color-text);
    cursor: pointer;
  }

  .btn-primary {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
    border: none;
    border-radius: 0.25rem;
    background-color: var(--color-primary);
    color: #ffffff;
    cursor: pointer;
  }
</style>
