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
  parseStatusChips,
  type StatusChip,
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
  parseStatusChips,
  type StatusChip,
};

export const PREMADE_TAGS = [
  "Printing",
  "Idle",
  "Colour",
  "Mono",
  "Duplex",
  "Simplex",
  "Toner Low",
  "Paper Jam",
  "Out of Paper",
  "Toner Empty",
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

export function getPrinterColorTag(
  printer?: Printer,
  state?: PrinterState,
): "Colour" | "Mono" | undefined {
  if (state?.isColor !== undefined) {
    return state.isColor ? "Colour" : "Mono";
  }
  if (state?.toners && state.toners.length > 0) {
    const hasColor = state.toners.some((t) => {
      const c = t.color.toLowerCase();
      return c === "cyan" || c === "magenta" || c === "yellow";
    });
    if (hasColor) return "Colour";
    const hasBlack = state.toners.some((t) => t.color.toLowerCase() === "black");
    if (hasBlack) return "Mono";
  }
  const text = `${printer?.model ?? ""} ${state?.detectedModel ?? ""} ${printer?.description ?? ""}`.toLowerCase();
  if (text.includes("color") || text.includes("colour") || /\b(cdw|cdn)\b/i.test(text)) {
    return "Colour";
  }
  if (text.includes("mono") || text.includes("monochrome")) {
    return "Mono";
  }
  return undefined;
}

export function getPrinterDuplexTag(
  printer?: Printer,
  state?: PrinterState,
): "Duplex" | "Simplex" | undefined {
  if (state?.isDuplex !== undefined) {
    return state.isDuplex ? "Duplex" : "Simplex";
  }
  const text = `${printer?.model ?? ""} ${state?.detectedModel ?? ""} ${printer?.description ?? ""}`.toLowerCase();
  if (text.includes("duplex") || /\b[a-z0-9-]*\d[a-z]*(dn|dw|cdw|fdw|dne|dni|dwt|d)\b/i.test(text)) {
    return "Duplex";
  }
  if (text.includes("simplex")) {
    return "Simplex";
  }
  return undefined;
}

export function getPrinterTags(
  printer: Printer,
  state?: PrinterState,
): string[] {
  const tags: string[] = [];

  const stateLower = state?.printerState?.trim().toLowerCase();
  if (stateLower === "printing") {
    tags.push("Printing");
  } else if (stateLower === "idle") {
    tags.push("Idle");
  } else if (
    state?.printerState?.trim() &&
    !isPaperJamText(state.printerState) &&
    !isOutOfPaperText(state.printerState)
  ) {
    tags.push(state.printerState.trim());
  }

  const colorTag = getPrinterColorTag(printer, state);
  if (colorTag) {
    tags.push(colorTag);
  }

  const duplexTag = getPrinterDuplexTag(printer, state);
  if (duplexTag) {
    tags.push(duplexTag);
  }

  if (hasLowTonerSupply(state?.toners)) {
    tags.push("Toner Low");
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

  const make = inferPrinterMake(
    printer,
    state?.detectedModel,
    state?.detectedMake,
  );
  if (make) {
    tags.push(make);
  }

  const locationTag = printer.location?.trim() || state?.detectedLocation?.trim();
  if (locationTag) {
    tags.push(locationTag);
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

  if (lower === "color" || lower === "colour") {
    return "Colour";
  }
  if (lower === "mono" || lower === "monochrome") {
    return "Mono";
  }
  if (lower === "duplex") {
    return "Duplex";
  }
  if (lower === "simplex") {
    return "Simplex";
  }

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
    if (
      (filterTagLower === "color" && pLower === "colour") ||
      (filterTagLower === "colour" && pLower === "color")
    ) {
      return true;
    }
    if (
      (filterTagLower === "monochrome" && pLower === "mono") ||
      (filterTagLower === "mono" && pLower === "monochrome")
    ) {
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
