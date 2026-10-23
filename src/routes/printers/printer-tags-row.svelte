<script lang="ts">
  import {
    getPrinterColorTag,
    getPrinterDuplexTag,
    getPrinterTags,
    hasOutOfPaper,
    hasPaperJam,
    inferPrinterMake,
    isEmptyToner,
    isLowToner,
  } from "./tags";
  import type { Printer, TonerSupply } from "./types";

  let {
    printer,
    toners = [],
    detectedMake = undefined,
    detectedModel = undefined,
    detectedLocation = undefined,
    printerState = undefined,
    statusMessage = undefined,
    paperJam = false,
    outOfPaper = false,
    isColor = undefined,
    isDuplex = undefined,
    showMake = false,
    showLocation = false,
    showHiddenCount = true,
  }: {
    printer?: Printer;
    toners?: TonerSupply[];
    detectedMake?: string;
    detectedModel?: string;
    detectedLocation?: string;
    printerState?: string;
    statusMessage?: string;
    paperJam?: boolean;
    outOfPaper?: boolean;
    isColor?: boolean;
    isDuplex?: boolean;
    showMake?: boolean;
    showLocation?: boolean;
    showHiddenCount?: boolean;
  } = $props();

  let makeTag = $derived(
    showMake ? inferPrinterMake(printer, detectedModel, detectedMake) : undefined,
  );
  let displayLocation = $derived(
    showLocation ? printer?.location || detectedLocation : undefined,
  );
  let stateTag = $derived.by(() => {
    if (!printerState) return undefined;
    const normalized = printerState.trim().toLowerCase();
    if (normalized === "idle") return "Idle";
    if (normalized === "printing") return "Printing";
    return undefined;
  });
  let isJammed = $derived(hasPaperJam({ paperJam, printerState, statusMessage }));
  let isNoPaper = $derived(hasOutOfPaper({ outOfPaper, printerState, statusMessage }));
  let hasEmptyToner = $derived(toners.some(isEmptyToner));
  let hasLowToner = $derived(toners.some(isLowToner));
  let colorTag = $derived(
    getPrinterColorTag(printer, {
      status: "online",
      toners,
      detectedMake,
      detectedModel,
      isColor,
      isDuplex,
    }),
  );
  let duplexTag = $derived(
    getPrinterDuplexTag(printer, {
      status: "online",
      toners,
      detectedMake,
      detectedModel,
      isColor,
      isDuplex,
    }),
  );

  interface VisibleTag {
    id: string;
    label: string;
    className: string;
    title: string;
  }

  let visibleTags = $derived.by(() => {
    const list: VisibleTag[] = [];
    if (makeTag) {
      list.push({ id: "make", label: makeTag, className: "tag-make", title: `Make: ${makeTag}` });
    }
    if (displayLocation) {
      list.push({ id: "location", label: displayLocation, className: "tag-location", title: displayLocation });
    }
    if (stateTag) {
      list.push({ id: "state", label: stateTag, className: "tag-state", title: stateTag });
    }
    if (isJammed) {
      list.push({ id: "jam", label: "Paper Jam", className: "tag-failure", title: "Printer has a paper jam" });
    }
    if (isNoPaper) {
      list.push({ id: "no-paper", label: "Out of Paper", className: "tag-failure", title: "Printer is out of paper" });
    }
    if (hasEmptyToner) {
      list.push({ id: "toner-empty", label: "Toner Empty", className: "tag-failure", title: "One or more cartridges have reached 0% toner" });
    }
    if (hasLowToner) {
      list.push({ id: "toner-low", label: "Toner Low", className: "tag-warning", title: "One or more cartridges are low on toner" });
    }
    if (colorTag) {
      list.push({ id: "color", label: colorTag, className: "tag-capability", title: `Color capability: ${colorTag}` });
    }
    if (duplexTag) {
      list.push({ id: "duplex", label: duplexTag, className: "tag-capability", title: `Duplex capability: ${duplexTag}` });
    }
    return list;
  });

  let allTags = $derived(
    printer
      ? getPrinterTags(printer, {
          status: "online",
          toners,
          detectedMake,
          detectedModel,
          detectedLocation,
          printerState,
          statusMessage,
          paperJam,
          outOfPaper,
          isColor,
          isDuplex,
        })
      : [],
  );

  let hiddenTags = $derived.by(() => {
    if (!showHiddenCount) return [];
    const visibleSet = new Set(visibleTags.map((t) => t.label.toLowerCase()));
    return allTags.filter((tag) => !visibleSet.has(tag.toLowerCase()));
  });

  let hiddenCount = $derived(hiddenTags.length);
  let hiddenTitle = $derived(
    hiddenCount === 1
      ? `1 more tag: ${hiddenTags[0]}`
      : `${hiddenCount} more tags: ${hiddenTags.join(", ")}`,
  );
</script>

{#if visibleTags.length > 0 || (showHiddenCount && hiddenCount > 0)}
  <div class="tags-row">
    {#each visibleTags as tag, index (tag.id)}
      <span class="printer-tag {tag.className}" title={tag.title}>{tag.label}</span>
      {#if index === 0 && showHiddenCount && hiddenCount > 0}
        <span class="tag-hidden-count" title={hiddenTitle}>+{hiddenCount}</span>
      {/if}
    {/each}
    {#if visibleTags.length === 0 && showHiddenCount && hiddenCount > 0}
      <span class="tag-hidden-count" title={hiddenTitle}>+{hiddenCount}</span>
    {/if}
  </div>
{/if}

<style>
  .tags-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.35rem;
    min-width: 0;
  }

  .tags-row .printer-tag {
    font-size: calc(0.7rem * var(--card-font-scale, 1));
    padding: 0.2rem 0.45rem;
  }

  .tag-hidden-count {
    font-size: calc(0.72rem * var(--card-font-scale, 1));
    font-weight: 600;
    color: var(--color-text-muted);
    line-height: 1;
    user-select: none;
    cursor: default;
    padding: 0 0.1rem;
  }

  .tag-make,
  .tag-state,
  .tag-location,
  .tag-capability {
    background-color: color-mix(in srgb, var(--color-base) 65%, transparent);
    color: var(--color-text);
    border-color: var(--color-border);
  }
</style>
