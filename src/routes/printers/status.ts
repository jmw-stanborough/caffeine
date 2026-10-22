import type { PrinterState, StatusKind, TonerSupply } from "./types";

export const LOW_TONER_THRESHOLD_PERCENT = 15;

const JAM_PHRASES = [
  "paper jam",
  "media jam",
  "document jam",
  "jammed",
  "misfeed",
  "clear jam",
];

const JAM_WORDS = new Set(["jam", "jams", "jammed", "misfeed"]);

const NON_PAPER_SUPPLY_TERMS = ["toner", "ink", "waste", "output", "bin"];

const OUT_OF_PAPER_PHRASES = [
  "out of paper",
  "no paper",
  "paper out",
  "paper empty",
  "media empty",
  "media needed",
  "media out",
  "load paper",
  "add paper",
  "insert paper",
  "refill paper",
];

const PAPER_CONTAINER_TERMS = [
  "tray",
  "cassette",
  "drawer",
  "feeder",
  "paper",
  "media",
];

function isWasteSupply(toner: TonerSupply): boolean {
  return (
    toner.supply_type === "waste_toner" || toner.supply_type === "waste_ink"
  );
}

export function isEmptyToner(toner: TonerSupply): boolean {
  if (isWasteSupply(toner)) {
    return false;
  }
  if (toner.percent !== undefined && toner.percent !== null) {
    return toner.percent <= 0;
  }
  return toner.level === 0;
}

export function hasEmptyTonerSupply(toners?: TonerSupply[]): boolean {
  return (toners ?? []).some(isEmptyToner);
}

export function isLowToner(toner: TonerSupply): boolean {
  if (isWasteSupply(toner) || isEmptyToner(toner)) {
    return false;
  }
  return (
    toner.percent !== undefined &&
    toner.percent !== null &&
    toner.percent <= LOW_TONER_THRESHOLD_PERCENT
  );
}

export function hasLowTonerSupply(toners?: TonerSupply[]): boolean {
  return (toners ?? []).some(isLowToner);
}

export function isPaperJamText(text?: string): boolean {
  if (!text) return false;
  const lower = text.toLowerCase();
  if (JAM_PHRASES.some((phrase) => lower.includes(phrase))) {
    return true;
  }
  return lower.split(/[^a-z0-9]+/).some((w) => JAM_WORDS.has(w));
}

export function isOutOfPaperText(text?: string): boolean {
  if (!text) return false;
  const lower = text.toLowerCase();
  if (NON_PAPER_SUPPLY_TERMS.some((term) => lower.includes(term))) {
    return false;
  }
  if (OUT_OF_PAPER_PHRASES.some((phrase) => lower.includes(phrase))) {
    return true;
  }
  return (
    lower.includes("empty") &&
    PAPER_CONTAINER_TERMS.some((term) => lower.includes(term))
  );
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
