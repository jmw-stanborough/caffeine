import {
  getEffectivePrinterStatus,
  hasEmptyTonerSupply,
  hasLowTonerSupply,
  hasOutOfPaper,
  hasPaperJam,
  isEmptyToner,
  isLowToner,
  isOutOfPaperText,
  isPaperJamText,
} from "./status";
import type { Printer, PrinterState } from "./types";
import { extractPrinterMakeFromText, inferPrinterMake } from "./vendor";

export {
  extractPrinterMakeFromText,
  getEffectivePrinterStatus,
  hasEmptyTonerSupply,
  hasLowTonerSupply,
  hasOutOfPaper,
  hasPaperJam,
  inferPrinterMake,
  isEmptyToner,
  isLowToner,
  isOutOfPaperText,
  isPaperJamText,
};

export const PREMADE_TAGS = [
  "Printing",
  "Idle",
  "Paper Jam",
  "Out of Paper",
  "Toner Empty",
  "Toner Low",
] as const;

export function isFailureTag(tag: string): boolean {
  const lower = tag.trim().toLowerCase();
  return (
    lower === "paper jam" ||
    lower === "out of paper" ||
    lower === "toner empty"
  );
}

export function isWarningTag(tag: string): boolean {
  return tag.trim().toLowerCase() === "toner low";
}

export function getPrinterTags(
  printer: Printer,
  state?: PrinterState,
): string[] {
  const tags: string[] = [];

  const make = inferPrinterMake(
    printer,
    state?.detectedModel,
    state?.detectedMake,
  );
  if (make) {
    tags.push(make);
  }

  if (printer.location?.trim()) {
    tags.push(printer.location.trim());
  }

  if (state?.printerState?.trim()) {
    tags.push(state.printerState.trim());
  }

  if (hasPaperJam(state)) {
    tags.push("Paper Jam");
  }

  if (hasOutOfPaper(state)) {
    tags.push("Out of Paper");
  }

  if (hasEmptyTonerSupply(state?.toners)) {
    tags.push("Toner Empty");
  }

  if (hasLowTonerSupply(state?.toners)) {
    tags.push("Toner Low");
  }

  return tags;
}

export function normalizeTagLabel(
  rawTag: string,
  knownTags: Iterable<string> = [],
): string {
  const trimmed = rawTag.trim();
  if (!trimmed) return "";
  const lower = trimmed.toLowerCase();

  for (const premade of PREMADE_TAGS) {
    if (premade.toLowerCase() === lower) {
      return premade;
    }
  }

  for (const known of knownTags) {
    if (known.toLowerCase() === lower) {
      return known;
    }
  }

  return trimmed;
}

function tagMatchesPrinter(
  filterTagLower: string,
  printerTags: string[],
  status?: string,
): boolean {
  for (const pTag of printerTags) {
    const pLower = pTag.toLowerCase();
    if (pLower === filterTagLower || pLower.includes(filterTagLower)) {
      return true;
    }
  }
  return Boolean(status && status.toLowerCase() === filterTagLower);
}

export function filterPrintersByTags(
  printers: Printer[],
  statusMap: Record<string, PrinterState>,
  allTags: string[],
  anyTags: string[] = [],
): Printer[] {
  const activeAll = allTags
    .map((t) => t.trim().toLowerCase())
    .filter(Boolean);
  const activeAny = anyTags
    .map((t) => t.trim().toLowerCase())
    .filter(Boolean);

  if (activeAll.length === 0 && activeAny.length === 0) {
    return printers;
  }

  return printers.filter((printer) => {
    const state = statusMap[printer.id];
    const pTags = getPrinterTags(printer, state);

    if (
      activeAll.length > 0 &&
      !activeAll.every((f) => tagMatchesPrinter(f, pTags, state?.status))
    ) {
      return false;
    }

    if (
      activeAny.length > 0 &&
      !activeAny.some((f) => tagMatchesPrinter(f, pTags, state?.status))
    ) {
      return false;
    }

    return true;
  });
}
