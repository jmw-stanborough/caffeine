<script lang="ts">
  import {
    getPrinterColorTag,
    getPrinterDuplexTag,
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
</script>

{#if makeTag || displayLocation || stateTag || isJammed || isNoPaper || hasEmptyToner || hasLowToner || colorTag || duplexTag}
  <div class="tags-row">
    {#if makeTag}
      <span class="printer-tag tag-make" title="Make: {makeTag}">{makeTag}</span>
    {/if}
    {#if displayLocation}
      <span class="printer-tag tag-location" title={displayLocation}>{displayLocation}</span>
    {/if}
    {#if stateTag}
      <span class="printer-tag tag-state">{stateTag}</span>
    {/if}
    {#if isJammed}
      <span class="printer-tag tag-failure" title="Printer has a paper jam">Paper Jam</span>
    {/if}
    {#if isNoPaper}
      <span class="printer-tag tag-failure" title="Printer is out of paper">Out of Paper</span>
    {/if}
    {#if hasEmptyToner}
      <span class="printer-tag tag-failure" title="One or more cartridges have reached 0% toner">Toner Empty</span>
    {/if}
    {#if hasLowToner}
      <span class="printer-tag tag-warning" title="One or more cartridges are low on toner">Toner Low</span>
    {/if}
    {#if colorTag}
      <span class="printer-tag tag-capability" title="Color capability: {colorTag}">{colorTag}</span>
    {/if}
    {#if duplexTag}
      <span class="printer-tag tag-capability" title="Duplex capability: {duplexTag}">{duplexTag}</span>
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

  .tag-make,
  .tag-state,
  .tag-location,
  .tag-capability {
    background-color: color-mix(in srgb, var(--color-base) 65%, transparent);
    color: var(--color-text);
    border-color: var(--color-border);
  }
</style>
