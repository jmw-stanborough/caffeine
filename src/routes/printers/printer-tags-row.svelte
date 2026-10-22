<script lang="ts">
  import {
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
    printerState = undefined,
    statusMessage = undefined,
    paperJam = false,
    outOfPaper = false,
  }: {
    printer?: Printer;
    toners?: TonerSupply[];
    detectedMake?: string;
    detectedModel?: string;
    printerState?: string;
    statusMessage?: string;
    paperJam?: boolean;
    outOfPaper?: boolean;
  } = $props();

  let makeTag = $derived(inferPrinterMake(printer, detectedModel, detectedMake));
  let isJammed = $derived(hasPaperJam({ paperJam, printerState, statusMessage }));
  let isNoPaper = $derived(hasOutOfPaper({ outOfPaper, printerState, statusMessage }));
  let hasEmptyToner = $derived(toners.some(isEmptyToner));
  let hasLowToner = $derived(toners.some(isLowToner));
</script>

{#if makeTag || printer?.location || printerState || isJammed || isNoPaper || hasEmptyToner || hasLowToner}
  <div class="tags-row">
    {#if makeTag}
      <span class="printer-tag tag-make" title="Make: {makeTag}">{makeTag}</span>
    {/if}
    {#if printer?.location}
      <span class="printer-tag tag-location" title={printer.location}>{printer.location}</span>
    {/if}
    {#if printerState}
      <span class="printer-tag tag-state">{printerState}</span>
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
  .tag-location {
    background-color: color-mix(in srgb, var(--color-base) 65%, transparent);
    color: var(--color-text);
    border-color: var(--color-border);
  }
</style>
