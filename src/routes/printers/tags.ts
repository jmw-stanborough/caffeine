import type { Printer, PrinterState, StatusKind, TonerSupply } from "./types";

export const PREMADE_TAGS = [
  "Printing",
  "Idle",
  "Paper Jam",
  "Out of Paper",
  "Toner Empty",
  "Toner Low",
] as const;

const GENERIC_MODEL_WORDS = new Set([
  "printer",
  "printers",
  "series",
  "color",
  "colour",
  "laser",
  "inkjet",
  "thermal",
  "network",
  "mfp",
  "mono",
  "monochrome",
  "duplex",
  "generic",
  "unknown",
  "office",
  "copy",
  "copier",
  "scanner",
  "ethernet",
]);

const CORPORATE_SUFFIXES = new Set([
  "corporation",
  "corp",
  "inc",
  "ltd",
  "co",
  "industries",
  "company",
  "international",
  "electronics",
  "group",
  "systems",
]);

function cleanVendorName(raw: string): string | undefined {
  const primary = raw.split(/[,;\n/]/)[0]?.trim() ?? "";
  if (!primary) return undefined;

  const words = primary.split(/\s+/);
  while (words.length > 1) {
    const lastClean = words[words.length - 1]
      .replace(/^[^A-Za-z0-9]+|[^A-Za-z0-9]+$/g, "")
      .toLowerCase();
    if (CORPORATE_SUFFIXES.has(lastClean)) {
      words.pop();
    } else {
      break;
    }
  }

  const cleaned = words
    .join(" ")
    .replace(/^[^A-Za-z0-9]+|[^A-Za-z0-9]+$/g, "");
  if (
    cleaned.length >= 2 &&
    /^[A-Za-z]/.test(cleaned) &&
    !GENERIC_MODEL_WORDS.has(cleaned.toLowerCase())
  ) {
    return cleaned;
  }
  return undefined;
}

export function extractPrinterMakeFromText(
  text?: string,
): string | undefined {
  const trimmed = text?.trim();
  if (!trimmed) return undefined;

  // 1. Check IEEE 1284 / structured key-value segments (MFG:, MANUFACTURER:, PID:)
  for (const part of trimmed.split(/[;,\n]/)) {
    const seg = part.trim();
    const colonIdx = seg.indexOf(":");
    if (colonIdx > 0) {
      const key = seg.slice(0, colonIdx).trim().toLowerCase();
      const val = seg.slice(colonIdx + 1).trim();
      if (key === "mfg" || key === "manufacturer") {
        const vendor = cleanVendorName(val);
        if (vendor) return vendor;
      } else if (key === "pid" || key === "mdl" || key === "model") {
        const make = extractPrinterMakeFromText(val);
        if (make) return make;
      }
    }
  }

  // 2. Extract leading manufacturer word from "<Make> <Model...>" string
  const firstClause = trimmed.split(/[;,\n/]/)[0]?.trim() ?? "";
  const tokens = firstClause.split(/\s+/).filter(Boolean);
  if (tokens.length >= 2) {
    const candidate = tokens[0].replace(/^[^A-Za-z0-9-]+|[^A-Za-z0-9-]+$/g, "");
    if (
      candidate.length >= 2 &&
      /^[A-Za-z](?:[A-Za-z-]*[A-Za-z])?$/.test(candidate) &&
      !GENERIC_MODEL_WORDS.has(candidate.toLowerCase())
    ) {
      return candidate;
    }
  }

  return undefined;
}

export function inferPrinterMake(
  printer?: Printer,
  detectedModel?: string,
  detectedMake?: string,
): string | undefined {
  if (detectedMake?.trim()) {
    return detectedMake.trim();
  }

  const fromDetected = extractPrinterMakeFromText(detectedModel);
  if (fromDetected) {
    return fromDetected;
  }

  const fromConfiguredModel = extractPrinterMakeFromText(printer?.model);
  if (fromConfiguredModel) {
    return fromConfiguredModel;
  }

  return undefined;
}

export function isEmptyToner(toner: TonerSupply): boolean {
  if (
    toner.supply_type === "waste_toner" ||
    toner.supply_type === "waste_ink"
  ) {
    return false;
  }
  if (toner.percent !== undefined && toner.percent !== null) {
    return toner.percent <= 0;
  }
  return toner.level === 0;
}

export function hasEmptyTonerSupply(toners?: TonerSupply[]): boolean {
  return (toners ?? []).some((t) => isEmptyToner(t));
}

export function isLowToner(toner: TonerSupply): boolean {
  if (
    toner.supply_type === "waste_toner" ||
    toner.supply_type === "waste_ink"
  ) {
    return false;
  }
  if (isEmptyToner(toner)) {
    return false;
  }
  return (
    toner.percent !== undefined &&
    toner.percent !== null &&
    toner.percent <= 15
  );
}

export function hasLowTonerSupply(toners?: TonerSupply[]): boolean {
  return (toners ?? []).some((t) => isLowToner(t));
}

export function isPaperJamText(text?: string): boolean {
  if (!text) return false;
  const lower = text.toLowerCase();
  if (
    lower.includes("paper jam") ||
    lower.includes("media jam") ||
    lower.includes("document jam") ||
    lower.includes("jammed") ||
    lower.includes("misfeed") ||
    lower.includes("clear jam")
  ) {
    return true;
  }
  return lower
    .split(/[^a-z0-9]+/)
    .some((w) => w === "jam" || w === "jams" || w === "jammed" || w === "misfeed");
}

export function isOutOfPaperText(text?: string): boolean {
  if (!text) return false;
  const lower = text.toLowerCase();
  if (
    lower.includes("toner") ||
    lower.includes("ink") ||
    lower.includes("waste") ||
    lower.includes("output") ||
    lower.includes("bin")
  ) {
    return false;
  }
  if (
    lower.includes("out of paper") ||
    lower.includes("no paper") ||
    lower.includes("paper out") ||
    lower.includes("paper empty") ||
    lower.includes("media empty") ||
    lower.includes("media needed") ||
    lower.includes("media out") ||
    lower.includes("load paper") ||
    lower.includes("add paper") ||
    lower.includes("insert paper") ||
    lower.includes("refill paper")
  ) {
    return true;
  }
  if (
    lower.includes("empty") &&
    (lower.includes("tray") ||
      lower.includes("cassette") ||
      lower.includes("drawer") ||
      lower.includes("feeder") ||
      lower.includes("paper") ||
      lower.includes("media"))
  ) {
    return true;
  }
  return false;
}

export function hasPaperJam(
  state?: Pick<PrinterState, "paperJam" | "printerState" | "statusMessage">,
): boolean {
  if (!state) return false;
  return (
    Boolean(state.paperJam) ||
    isPaperJamText(state.printerState) ||
    isPaperJamText(state.statusMessage)
  );
}

export function hasOutOfPaper(
  state?: Pick<PrinterState, "outOfPaper" | "printerState" | "statusMessage">,
): boolean {
  if (!state) return false;
  return (
    Boolean(state.outOfPaper) ||
    isOutOfPaperText(state.printerState) ||
    isOutOfPaperText(state.statusMessage)
  );
}

export function getEffectivePrinterStatus(
  state?: Partial<PrinterState>,
): StatusKind {
  const baseStatus = state?.status ?? "checking";
  if (
    baseStatus === "offline" ||
    hasPaperJam(state) ||
    hasOutOfPaper(state) ||
    hasEmptyTonerSupply(state?.toners)
  ) {
    return "offline";
  }
  if (hasLowTonerSupply(state?.toners)) {
    return "degraded";
  }
  return baseStatus;
}

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
  if (status && status.toLowerCase() === filterTagLower) {
    return true;
  }
  return false;
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

    if (activeAll.length > 0) {
      const matchesAll = activeAll.every((f) =>
        tagMatchesPrinter(f, pTags, state?.status),
      );
      if (!matchesAll) {
        return false;
      }
    }

    if (activeAny.length > 0) {
      const matchesAny = activeAny.some((f) =>
        tagMatchesPrinter(f, pTags, state?.status),
      );
      if (!matchesAny) {
        return false;
      }
    }

    return true;
  });
}
