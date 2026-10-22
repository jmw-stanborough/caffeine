<script lang="ts">
  import { isFailureTag, isWarningTag } from "./tags";
  import type { FilterMode } from "./types";

  let {
    mode,
    label,
    clearTitle,
    tags,
    inputValue = $bindable(""),
    boxEl = $bindable(undefined),
    isDragging = false,
    isDropActive = false,
    onAddTag,
    onRemoveTag,
    onRemoveLastTag,
    onClearTags,
    onTagPointerDown,
    onTagDragStart,
    onTagDragEnd,
    onZoneDragOver,
    onZoneDragLeave,
    onZoneDrop,
  }: {
    mode: FilterMode;
    label: string;
    clearTitle: string;
    tags: string[];
    inputValue: string;
    boxEl?: HTMLElement;
    isDragging?: boolean;
    isDropActive?: boolean;
    onAddTag: (raw: string) => Promise<unknown> | unknown;
    onRemoveTag: (tag: string) => Promise<void> | void;
    onRemoveLastTag: () => Promise<void> | void;
    onClearTags: () => Promise<void> | void;
    onTagPointerDown: (e: PointerEvent, tag: string, source: FilterMode) => void;
    onTagDragStart: (e: DragEvent, tag: string, source: FilterMode) => void;
    onTagDragEnd: () => void;
    onZoneDragOver: (e: DragEvent, zone: FilterMode) => void;
    onZoneDragLeave: (e: DragEvent, zone: FilterMode) => void;
    onZoneDrop: (e: DragEvent, zone: FilterMode) => void;
  } = $props();

  let inputEl = $state<HTMLInputElement | undefined>(undefined);
  let isFocused = $state(false);

  async function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      const trimmed = inputValue.trim();
      if (trimmed) {
        await onAddTag(trimmed);
        inputValue = "";
      }
    } else if (e.key === "Backspace" && inputValue === "" && tags.length > 0) {
      e.preventDefault();
      await onRemoveLastTag();
    }
  }
</script>

<div class="filter-group">
  <span class="filter-label">{label}</span>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="tag-entry-box"
    bind:this={boxEl}
    class:focused={isFocused}
    class:drop-ready={isDragging && !isDropActive}
    class:drop-active={isDropActive}
    onclick={() => inputEl?.focus()}
    ondragover={(e) => onZoneDragOver(e, mode)}
    ondragenter={(e) => onZoneDragOver(e, mode)}
    ondragleave={(e) => onZoneDragLeave(e, mode)}
    ondrop={(e) => onZoneDrop(e, mode)}
  >
    {#each tags as tag (tag)}
      <span
        class="printer-tag entry-tag draggable-tag"
        class:tag-failure={isFailureTag(tag)}
        class:tag-warning={!isFailureTag(tag) && isWarningTag(tag)}
        class:tag-standard={!isFailureTag(tag) && !isWarningTag(tag)}
        draggable="true"
        onpointerdown={(e) => onTagPointerDown(e, tag, mode)}
        ondragstart={(e) => onTagDragStart(e, tag, mode)}
        ondragend={onTagDragEnd}
      >
        <span class="entry-tag-label">{tag}</span>
        <button
          type="button"
          class="entry-tag-remove"
          onclick={(e) => {
            e.stopPropagation();
            onRemoveTag(tag);
            inputEl?.focus();
          }}
          aria-label="Remove {tag} filter"
          title="Remove {tag}"
        >
          ×
        </button>
      </span>
    {/each}

    <input
      type="text"
      class="tag-entry-input"
      bind:this={inputEl}
      bind:value={inputValue}
      onfocus={() => (isFocused = true)}
      onblur={() => (isFocused = false)}
      onkeydown={handleKeyDown}
      placeholder={tags.length === 0
        ? "Type a tag and press Enter, or drop a tag here..."
        : "Add tag..."}
      aria-label={label}
    />

    {#if tags.length > 0}
      <button
        type="button"
        class="btn-clear-tags"
        onclick={(e) => {
          e.stopPropagation();
          onClearTags();
          inputValue = "";
          inputEl?.focus();
        }}
        title={clearTitle}
      >
        Clear
      </button>
    {/if}
  </div>
</div>

<style>
  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .tag-entry-box {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
    min-height: 2.5rem;
    padding: 0.35rem 0.65rem;
    background-color: var(--color-base);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.375rem;
    cursor: text;
    transition: border-color 0.15s ease, background-color 0.15s ease, box-shadow 0.15s ease;
  }

  .tag-entry-box.focused {
    border-color: var(--color-primary);
  }

  .tag-entry-box.drop-ready {
    border-style: dashed;
    border-color: var(--color-text-muted);
  }

  .tag-entry-box.drop-active {
    border-style: solid;
    border-color: var(--color-primary);
    background-color: color-mix(in srgb, var(--color-primary) 12%, var(--color-base));
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-primary) 35%, transparent);
  }

  .draggable-tag {
    cursor: grab;
  }

  .draggable-tag:active {
    cursor: grabbing;
  }

  .entry-tag {
    gap: 0.3rem;
    padding-right: 0.3rem;
  }

  .entry-tag-label {
    line-height: 1;
  }

  .entry-tag-remove {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: inherit;
    font-size: 0.85rem;
    font-weight: 700;
    line-height: 1;
    padding: 0.05rem 0.18rem;
    border-radius: 0.2rem;
    cursor: pointer;
    opacity: 0.7;
    transition: opacity 0.12s ease, background-color 0.12s ease;
  }

  .entry-tag-remove:hover {
    opacity: 1;
    background-color: rgba(0, 0, 0, 0.12);
  }

  .tag-entry-input {
    flex: 1;
    min-width: 10rem;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: 0.88rem;
    font-family: inherit;
    padding: 0.15rem 0.1rem;
    outline: none;
  }

  .tag-entry-input::placeholder {
    color: var(--color-text-muted);
    opacity: 0.8;
  }

  .btn-clear-tags {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    font-size: 0.78rem;
    font-weight: 600;
    padding: 0.2rem 0.45rem;
    border-radius: 0.25rem;
    cursor: pointer;
    margin-left: auto;
    transition: color 0.15s ease, background-color 0.15s ease;
  }

  .btn-clear-tags:hover {
    color: var(--color-text);
    background-color: var(--color-surface);
  }
</style>
