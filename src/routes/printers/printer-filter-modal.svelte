<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import type { PrinterStore } from "./printers.svelte";
  import TagFilterGroup from "./tag-filter-group.svelte";
  import { PREMADE_TAGS, isFailureTag, isWarningTag } from "./tags";
  import type { FilterMode } from "./types";

  type DragSource = "suggested" | FilterMode;

  const DROP_ZONE_HIT_MARGIN_PX = 6;
  const DRAG_ACTIVATION_DISTANCE_PX = 4;
  const CUSTOM_TAG_MIME = "application/x-caffeine-tag";

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
  let allBoxEl = $state<HTMLElement | undefined>(undefined);
  let anyBoxEl = $state<HTMLElement | undefined>(undefined);

  let pointerDrag = $state<PointerDragState | null>(null);
  let htmlDrag = $state<{ tag: string; source: DragSource } | null>(null);
  let dropTarget = $state<FilterMode | null>(null);

  let isDragging = $derived(
    Boolean(pointerDrag?.active) || htmlDrag !== null,
  );

  $effect(() => {
    if (!open) {
      allInputText = "";
      anyInputText = "";
      pointerDrag = null;
      htmlDrag = null;
      dropTarget = null;
    }
  });

  function isPointInsideBox(
    el: HTMLElement | undefined,
    clientX: number,
    clientY: number,
  ): boolean {
    if (!el) return false;
    const rect = el.getBoundingClientRect();
    return (
      clientX >= rect.left - DROP_ZONE_HIT_MARGIN_PX &&
      clientX <= rect.right + DROP_ZONE_HIT_MARGIN_PX &&
      clientY >= rect.top - DROP_ZONE_HIT_MARGIN_PX &&
      clientY <= rect.bottom + DROP_ZONE_HIT_MARGIN_PX
    );
  }

  function hitTestDropZone(clientX: number, clientY: number): FilterMode | null {
    if (isPointInsideBox(allBoxEl, clientX, clientY)) return "all";
    if (isPointInsideBox(anyBoxEl, clientX, clientY)) return "any";
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
      if (dist >= DRAG_ACTIVATION_DISTANCE_PX) {
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
      ? (hitTestDropZone(e.clientX, e.clientY) ?? dropTarget)
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
        CUSTOM_TAG_MIME,
        JSON.stringify({ tag, source }),
      );
    }
  }

  function handleDragEnd() {
    htmlDrag = null;
    dropTarget = null;
  }

  function handleZoneDragOver(e: DragEvent, zone: FilterMode) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect =
        htmlDrag?.source === "suggested" ? "copy" : "move";
    }
    dropTarget = zone;
  }

  function handleZoneDragLeave(e: DragEvent, zone: FilterMode) {
    const currentTarget = e.currentTarget as HTMLElement | null;
    const related = e.relatedTarget as Node | null;
    if (currentTarget && related && currentTarget.contains(related)) {
      return;
    }
    if (dropTarget === zone) {
      dropTarget = null;
    }
  }

  async function handleZoneDrop(e: DragEvent, zone: FilterMode) {
    e.preventDefault();
    e.stopPropagation();

    let tag = htmlDrag?.tag ?? "";
    let source: DragSource = htmlDrag?.source ?? "suggested";

    if (!tag && e.dataTransfer) {
      const customPayload = e.dataTransfer.getData(CUSTOM_TAG_MIME);
      if (customPayload) {
        try {
          const parsed = JSON.parse(customPayload);
          if (parsed?.tag) tag = String(parsed.tag);
          if (parsed?.source) source = parsed.source;
        } catch {
          // Fall back to plain-text dataTransfer when custom MIME JSON cannot be parsed.
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

  async function commitPendingInputs() {
    const trimmedAll = allInputText.trim();
    if (trimmedAll) {
      await store.addFilterTag("all", trimmedAll);
      allInputText = "";
    }
    const trimmedAny = anyInputText.trim();
    if (trimmedAny) {
      await store.addFilterTag("any", trimmedAny);
      anyInputText = "";
    }
  }

  async function closeWithCommit() {
    await commitPendingInputs();
    open = false;
  }
</script>

<svelte:window
  onpointermove={handleWindowPointerMove}
  onpointerup={handleWindowPointerUp}
  onpointercancel={handleWindowPointerCancel}
/>

<Modal
  bind:open
  maxWidth="36rem"
  ariaLabel="Filter Printers"
  onBackdropClose={closeWithCommit}
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

  <div class="modal-body filter-modal-body">
    <TagFilterGroup
      mode="all"
      label="Show printers that have all of these tags"
      clearTitle="Clear all AND filter tags"
      tags={store.filterAllTags}
      bind:inputValue={allInputText}
      bind:boxEl={allBoxEl}
      {isDragging}
      isDropActive={dropTarget === "all"}
      onAddTag={(raw) => store.addFilterTag("all", raw)}
      onRemoveTag={(t) => store.removeFilterTag("all", t)}
      onRemoveLastTag={() => store.removeLastFilterTag("all")}
      onClearTags={() => store.clearFilterModeTags("all")}
      onTagPointerDown={handleTagPointerDown}
      onTagDragStart={handleDragStart}
      onTagDragEnd={handleDragEnd}
      onZoneDragOver={handleZoneDragOver}
      onZoneDragLeave={handleZoneDragLeave}
      onZoneDrop={handleZoneDrop}
    />

    <TagFilterGroup
      mode="any"
      label="Show printers that have at least one of these tags"
      clearTitle="Clear all OR filter tags"
      tags={store.filterAnyTags}
      bind:inputValue={anyInputText}
      bind:boxEl={anyBoxEl}
      {isDragging}
      isDropActive={dropTarget === "any"}
      onAddTag={(raw) => store.addFilterTag("any", raw)}
      onRemoveTag={(t) => store.removeFilterTag("any", t)}
      onRemoveLastTag={() => store.removeLastFilterTag("any")}
      onClearTags={() => store.clearFilterModeTags("any")}
      onTagPointerDown={handleTagPointerDown}
      onTagDragStart={handleDragStart}
      onTagDragEnd={handleDragEnd}
      onZoneDragOver={handleZoneDragOver}
      onZoneDragLeave={handleZoneDragLeave}
      onZoneDrop={handleZoneDrop}
    />

    <div class="suggested-group">
      <div class="suggested-header">
        <span class="filter-label">Suggested tags</span>
        <span class="suggested-hint">Drag into either filter list above</span>
      </div>
      <div class="suggested-tags-list" role="list" aria-label="Suggested filter tags">
        {#each PREMADE_TAGS as tag (tag)}
          {@const inAll = store.hasFilterTag("all", tag)}
          {@const inAny = store.hasFilterTag("any", tag)}
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <span
            role="listitem"
            class="printer-tag suggested-tag"
            class:tag-failure={isFailureTag(tag)}
            class:tag-warning={!isFailureTag(tag) && isWarningTag(tag)}
            class:tag-standard={!isFailureTag(tag) && !isWarningTag(tag)}
            class:in-use={inAll || inAny}
            class:being-dragged={pointerDrag?.active &&
              pointerDrag.tag === tag &&
              pointerDrag.source === "suggested"}
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
        class="btn-cancel"
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
      <button type="button" class="btn-save" onclick={closeWithCommit}>
        Done
      </button>
    </div>
  </div>
</Modal>

{#if open && pointerDrag?.active}
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

<style>
  .filter-modal-body {
    gap: 1.1rem;
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
</style>
