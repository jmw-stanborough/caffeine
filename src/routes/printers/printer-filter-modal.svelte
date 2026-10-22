<script lang="ts">
  import type { PrinterStore } from "./printers.svelte";
  import { PREMADE_TAGS, isFailureTag, isWarningTag } from "./tags";

  type DragSource = "suggested" | "all" | "any";

  interface PointerDragState {
    tag: string;
    source: DragSource;
    startX: number;
    startY: number;
    x: number;
    y: number;
    active: boolean;
  }

  let {
    open = $bindable(false),
    store,
  }: {
    open: boolean;
    store: PrinterStore;
  } = $props();

  let allInputText = $state("");
  let anyInputText = $state("");
  let allInputEl = $state<HTMLInputElement | undefined>(undefined);
  let anyInputEl = $state<HTMLInputElement | undefined>(undefined);
  let allBoxEl = $state<HTMLElement | undefined>(undefined);
  let anyBoxEl = $state<HTMLElement | undefined>(undefined);
  let isAllFocused = $state(false);
  let isAnyFocused = $state(false);

  let pointerDrag = $state<PointerDragState | null>(null);
  let htmlDrag = $state<{ tag: string; source: DragSource } | null>(null);
  let dropTarget = $state<"all" | "any" | null>(null);

  let isDragging = $derived(
    Boolean(pointerDrag?.active) || htmlDrag !== null
  );

  $effect(() => {
    if (!open) {
      allInputText = "";
      anyInputText = "";
      isAllFocused = false;
      isAnyFocused = false;
      pointerDrag = null;
      htmlDrag = null;
      dropTarget = null;
    }
  });

  function hitTestDropZone(clientX: number, clientY: number): "all" | "any" | null {
    if (allBoxEl) {
      const rect = allBoxEl.getBoundingClientRect();
      if (
        clientX >= rect.left - 6 &&
        clientX <= rect.right + 6 &&
        clientY >= rect.top - 6 &&
        clientY <= rect.bottom + 6
      ) {
        return "all";
      }
    }
    if (anyBoxEl) {
      const rect = anyBoxEl.getBoundingClientRect();
      if (
        clientX >= rect.left - 6 &&
        clientX <= rect.right + 6 &&
        clientY >= rect.top - 6 &&
        clientY <= rect.bottom + 6
      ) {
        return "any";
      }
    }
    return null;
  }

  function handleTagPointerDown(
    e: PointerEvent,
    tag: string,
    source: DragSource,
  ) {
    if (e.button !== 0) return;
    const targetEl = e.target as HTMLElement | null;
    if (targetEl?.closest(".entry-tag-remove")) return;

    e.preventDefault();
    e.stopPropagation();

    pointerDrag = {
      tag,
      source,
      startX: e.clientX,
      startY: e.clientY,
      x: e.clientX,
      y: e.clientY,
      active: false,
    };
    dropTarget = null;
  }

  function handleWindowPointerMove(e: PointerEvent) {
    if (!pointerDrag) return;
    pointerDrag.x = e.clientX;
    pointerDrag.y = e.clientY;

    if (!pointerDrag.active) {
      const dist = Math.hypot(
        e.clientX - pointerDrag.startX,
        e.clientY - pointerDrag.startY,
      );
      if (dist >= 4) {
        pointerDrag.active = true;
      }
    }

    if (pointerDrag.active) {
      dropTarget = hitTestDropZone(e.clientX, e.clientY);
    }
  }

  async function handleWindowPointerUp(e: PointerEvent) {
    if (!pointerDrag) return;
    const current = pointerDrag;
    const resolvedTarget = current.active
      ? hitTestDropZone(e.clientX, e.clientY) ?? dropTarget
      : null;

    pointerDrag = null;
    dropTarget = null;

    if (current.active && resolvedTarget) {
      await store.moveFilterTag(current.tag, resolvedTarget, current.source);
    }
  }

  function handleWindowPointerCancel() {
    pointerDrag = null;
    dropTarget = null;
  }

  function handleDragStart(e: DragEvent, tag: string, source: DragSource) {
    htmlDrag = { tag, source };
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "copyMove";
      e.dataTransfer.setData("text/plain", tag);
      e.dataTransfer.setData(
        "application/x-caffeine-tag",
        JSON.stringify({ tag, source }),
      );
    }
  }

  function handleDragEnd() {
    htmlDrag = null;
    dropTarget = null;
  }

  function handleZoneDragOver(e: DragEvent, zone: "all" | "any") {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect =
        htmlDrag?.source === "suggested" ? "copy" : "move";
    }
    dropTarget = zone;
  }

  function handleZoneDragLeave(e: DragEvent, zone: "all" | "any") {
    const currentTarget = e.currentTarget as HTMLElement | null;
    const related = e.relatedTarget as Node | null;
    if (currentTarget && related && currentTarget.contains(related)) {
      return;
    }
    if (dropTarget === zone) {
      dropTarget = null;
    }
  }

  async function handleZoneDrop(e: DragEvent, zone: "all" | "any") {
    e.preventDefault();
    e.stopPropagation();

    let tag = htmlDrag?.tag ?? "";
    let source: DragSource = htmlDrag?.source ?? "suggested";

    if (!tag && e.dataTransfer) {
      const customPayload = e.dataTransfer.getData("application/x-caffeine-tag");
      if (customPayload) {
        try {
          const parsed = JSON.parse(customPayload);
          if (parsed?.tag) tag = String(parsed.tag);
          if (parsed?.source) source = parsed.source;
        } catch {
          // ignore malformed payload
        }
      }
      if (!tag) {
        tag = e.dataTransfer.getData("text/plain") ?? "";
      }
    }

    htmlDrag = null;
    dropTarget = null;

    if (tag.trim()) {
      await store.moveFilterTag(tag, zone, source);
    }
  }

  async function handleAllKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      const trimmed = allInputText.trim();
      if (trimmed) {
        await store.addFilterAllTag(trimmed);
        allInputText = "";
      }
    } else if (
      e.key === "Backspace" &&
      allInputText === "" &&
      store.filterAllTags.length > 0
    ) {
      e.preventDefault();
      await store.removeLastFilterAllTag();
    }
  }

  async function handleAnyKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      e.stopPropagation();
      const trimmed = anyInputText.trim();
      if (trimmed) {
        await store.addFilterAnyTag(trimmed);
        anyInputText = "";
      }
    } else if (
      e.key === "Backspace" &&
      anyInputText === "" &&
      store.filterAnyTags.length > 0
    ) {
      e.preventDefault();
      await store.removeLastFilterAnyTag();
    }
  }

  async function commitPendingInputs() {
    const trimmedAll = allInputText.trim();
    if (trimmedAll) {
      await store.addFilterAllTag(trimmedAll);
      allInputText = "";
    }
    const trimmedAny = anyInputText.trim();
    if (trimmedAny) {
      await store.addFilterAnyTag(trimmedAny);
      anyInputText = "";
    }
  }

  async function closeWithCommit() {
    await commitPendingInputs();
    open = false;
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      closeWithCommit();
    }
  }

  function handleBackdropKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      open = false;
    }
  }
</script>

<svelte:window
  onpointermove={handleWindowPointerMove}
  onpointerup={handleWindowPointerUp}
  onpointercancel={handleWindowPointerCancel}
/>

{#if open}
  <div
    class="modal-backdrop"
    onclick={handleBackdropClick}
    onkeydown={handleBackdropKeyDown}
    role="presentation"
  >
    <div
      class="modal-card"
      tabindex="-1"
      role="dialog"
      aria-modal="true"
      aria-label="Filter Printers"
    >
      <div class="modal-header">
        <h2 class="modal-title">Filter Printers</h2>
        <button
          type="button"
          class="btn-close"
          onclick={() => (open = false)}
          aria-label="Close filter settings"
        >
          ✕
        </button>
      </div>

      <div class="modal-body">
        <div class="filter-group">
          <span class="filter-label">Show printers that have all of these tags</span>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="tag-entry-box"
            bind:this={allBoxEl}
            class:focused={isAllFocused}
            class:drop-ready={isDragging && dropTarget !== "all"}
            class:drop-active={dropTarget === "all"}
            onclick={() => allInputEl?.focus()}
            ondragover={(e) => handleZoneDragOver(e, "all")}
            ondragenter={(e) => handleZoneDragOver(e, "all")}
            ondragleave={(e) => handleZoneDragLeave(e, "all")}
            ondrop={(e) => handleZoneDrop(e, "all")}
          >
            {#each store.filterAllTags as tag (tag)}
              <span
                class="printer-tag entry-tag draggable-tag"
                class:tag-failure={isFailureTag(tag)}
                class:tag-warning={!isFailureTag(tag) && isWarningTag(tag)}
                class:tag-standard={!isFailureTag(tag) && !isWarningTag(tag)}
                draggable="true"
                onpointerdown={(e) => handleTagPointerDown(e, tag, "all")}
                ondragstart={(e) => handleDragStart(e, tag, "all")}
                ondragend={handleDragEnd}
              >
                <span class="entry-tag-label">{tag}</span>
                <button
                  type="button"
                  class="entry-tag-remove"
                  onclick={(e) => {
                    e.stopPropagation();
                    store.removeFilterAllTag(tag);
                    allInputEl?.focus();
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
              bind:this={allInputEl}
              bind:value={allInputText}
              onfocus={() => (isAllFocused = true)}
              onblur={() => (isAllFocused = false)}
              onkeydown={handleAllKeyDown}
              placeholder={store.filterAllTags.length === 0
                ? "Type a tag and press Enter, or drop a tag here..."
                : "Add tag..."}
              aria-label="Show printers that have all of these tags"
            />

            {#if store.filterAllTags.length > 0}
              <button
                type="button"
                class="btn-clear-tags"
                onclick={(e) => {
                  e.stopPropagation();
                  store.clearFilterAllTags();
                  allInputText = "";
                  allInputEl?.focus();
                }}
                title="Clear all AND filter tags"
              >
                Clear
              </button>
            {/if}
          </div>
        </div>

        <div class="filter-group">
          <span class="filter-label">Show printers that have at least one of these tags</span>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="tag-entry-box"
            bind:this={anyBoxEl}
            class:focused={isAnyFocused}
            class:drop-ready={isDragging && dropTarget !== "any"}
            class:drop-active={dropTarget === "any"}
            onclick={() => anyInputEl?.focus()}
            ondragover={(e) => handleZoneDragOver(e, "any")}
            ondragenter={(e) => handleZoneDragOver(e, "any")}
            ondragleave={(e) => handleZoneDragLeave(e, "any")}
            ondrop={(e) => handleZoneDrop(e, "any")}
          >
            {#each store.filterAnyTags as tag (tag)}
              <span
                class="printer-tag entry-tag draggable-tag"
                class:tag-failure={isFailureTag(tag)}
                class:tag-warning={!isFailureTag(tag) && isWarningTag(tag)}
                class:tag-standard={!isFailureTag(tag) && !isWarningTag(tag)}
                draggable="true"
                onpointerdown={(e) => handleTagPointerDown(e, tag, "any")}
                ondragstart={(e) => handleDragStart(e, tag, "any")}
                ondragend={handleDragEnd}
              >
                <span class="entry-tag-label">{tag}</span>
                <button
                  type="button"
                  class="entry-tag-remove"
                  onclick={(e) => {
                    e.stopPropagation();
                    store.removeFilterAnyTag(tag);
                    anyInputEl?.focus();
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
              bind:this={anyInputEl}
              bind:value={anyInputText}
              onfocus={() => (isAnyFocused = true)}
              onblur={() => (isAnyFocused = false)}
              onkeydown={handleAnyKeyDown}
              placeholder={store.filterAnyTags.length === 0
                ? "Type a tag and press Enter, or drop a tag here..."
                : "Add tag..."}
              aria-label="Show printers that have at least one of these tags"
            />

            {#if store.filterAnyTags.length > 0}
              <button
                type="button"
                class="btn-clear-tags"
                onclick={(e) => {
                  e.stopPropagation();
                  store.clearFilterAnyTags();
                  anyInputText = "";
                  anyInputEl?.focus();
                }}
                title="Clear all OR filter tags"
              >
                Clear
              </button>
            {/if}
          </div>
        </div>

        <div class="suggested-group">
          <div class="suggested-header">
            <span class="filter-label">Suggested tags</span>
            <span class="suggested-hint">Drag into either filter list above</span>
          </div>
          <div class="suggested-tags-list" role="list" aria-label="Suggested filter tags">
            {#each PREMADE_TAGS as tag (tag)}
              {@const inAll = store.hasFilterAllTag(tag)}
              {@const inAny = store.hasFilterAnyTag(tag)}
              <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
              <span
                role="listitem"
                class="printer-tag suggested-tag"
                class:tag-failure={isFailureTag(tag)}
                class:tag-warning={!isFailureTag(tag) && isWarningTag(tag)}
                class:tag-standard={!isFailureTag(tag) && !isWarningTag(tag)}
                class:in-use={inAll || inAny}
                class:being-dragged={pointerDrag?.active && pointerDrag.tag === tag && pointerDrag.source === "suggested"}
                draggable="true"
                onpointerdown={(e) => handleTagPointerDown(e, tag, "suggested")}
                ondragstart={(e) => handleDragStart(e, tag, "suggested")}
                ondragend={handleDragEnd}
                title="Drag '{tag}' into either filter list above"
              >
                <span>{tag}</span>
              </span>
            {/each}
          </div>
        </div>
      </div>

      <div class="modal-footer">
        {#if store.filterAllTags.length > 0 || store.filterAnyTags.length > 0}
          <button
            type="button"
            class="btn-clear-all"
            onclick={() => {
              store.clearFilterTags();
              allInputText = "";
              anyInputText = "";
            }}
          >
            Clear All Filters
          </button>
        {/if}

        <div class="footer-actions">
          <button type="button" class="btn-done" onclick={closeWithCommit}>
            Done
          </button>
        </div>
      </div>
    </div>
  </div>

  {#if pointerDrag?.active}
    <div
      class="printer-tag drag-ghost-tag"
      class:tag-failure={isFailureTag(pointerDrag.tag)}
      class:tag-warning={!isFailureTag(pointerDrag.tag) && isWarningTag(pointerDrag.tag)}
      class:tag-standard={!isFailureTag(pointerDrag.tag) && !isWarningTag(pointerDrag.tag)}
      style:left="{pointerDrag.x}px"
      style:top="{pointerDrag.y}px"
      aria-hidden="true"
    >
      <span>{pointerDrag.tag}</span>
    </div>
  {/if}
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    box-sizing: border-box;
    z-index: 100;
  }

  .modal-card {
    background-color: var(--color-surface);
    border: solid var(--width-border) var(--color-border);
    border-radius: 0.5rem;
    width: 100%;
    max-width: 36rem;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    outline: none;
    max-height: calc(100vh - 2rem);
    overflow-y: auto;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .modal-title {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--color-text);
  }

  .btn-close {
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    font-size: 1rem;
    cursor: pointer;
    line-height: 1;
    padding: 0.25rem;
    transition: color 0.15s ease;
  }

  .btn-close:hover {
    color: var(--color-text);
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 1.1rem;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .filter-label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text-muted);
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

  .printer-tag {
    display: inline-flex;
    align-items: center;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.22rem 0.5rem;
    border-radius: 0.25rem;
    line-height: 1;
    border: solid var(--width-border) var(--color-border);
    flex-shrink: 0;
    white-space: nowrap;
    box-sizing: border-box;
    user-select: none;
  }

  .tag-standard {
    background-color: var(--color-surface);
    color: var(--color-text);
    border-color: var(--color-border);
  }

  .tag-failure {
    background-color: var(--color-failure);
    color: #1c1917;
    border-color: color-mix(in srgb, var(--color-failure) 75%, #000000);
  }

  .tag-warning {
    background-color: var(--color-warning);
    color: #1c1917;
    border-color: color-mix(in srgb, var(--color-warning) 75%, #000000);
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

  .suggested-group {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    padding-top: 0.15rem;
  }

  .suggested-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .suggested-hint {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    opacity: 0.8;
  }

  .suggested-tags-list {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .suggested-tag {
    cursor: grab;
    transition: transform 0.12s ease, box-shadow 0.12s ease, opacity 0.12s ease;
  }

  .suggested-tag.tag-standard {
    background-color: var(--color-base);
  }

  .suggested-tag:hover {
    transform: translateY(-1px);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
  }

  .suggested-tag:active,
  .suggested-tag.being-dragged {
    cursor: grabbing;
  }

  .suggested-tag.being-dragged {
    opacity: 0.55;
  }

  .drag-ghost-tag {
    position: fixed;
    z-index: 250;
    pointer-events: none;
    transform: translate(-50%, -50%) scale(1.05);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.35);
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 0.25rem;
  }

  .footer-actions {
    display: flex;
    gap: 0.5rem;
    margin-left: auto;
  }

  .btn-clear-all {
    background-color: transparent;
    border: solid var(--width-border) var(--color-border);
    color: var(--color-text-muted);
    padding: 0.45rem 0.85rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .btn-clear-all:hover {
    color: var(--color-text);
    border-color: var(--color-text-muted);
  }

  .btn-done {
    background-color: var(--color-primary);
    border: solid var(--width-border) var(--color-primary);
    color: var(--color-base);
    padding: 0.45rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }
</style>
