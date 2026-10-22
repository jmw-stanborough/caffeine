import type { Printer, PrinterState, StatusKind, TonerSupply } from "./types";

export const PREMADE_TAGS = [
  "Printing",
  "Idle",
  "Paper Jam",
  "Out of Paper",
  "Toner Empty",
  "Toner Low",
] as const;

interface BrandRule {
  canonical: string;
  pattern: RegExp;
}

const KNOWN_BRANDS: BrandRule[] = [
  { canonical: "SHARP", pattern: /\b(sharp|(?:bp|mx)-[0-9]{2}[a-z0-9]+)\b/i },
  { canonical: "Brother", pattern: /\b(brother|(?:hl|mfc|dcp)-[a-z0-9]+)\b/i },
  {
    canonical: "HP",
    pattern: /\b(hp|hewlett[\s-]*packard|laserjet|officejet|deskjet|pagewide|designjet)\b/i,
  },
  {
    canonical: "Canon",
    pattern: /\b(canon|imagerunner|ir-adv|pixma|maxify|i-sensys)\b/i,
  },
  {
    canonical: "Epson",
    pattern: /\b(epson|workforce|ecotank|expression)\b/i,
  },
  { canonical: "Ricoh", pattern: /\b(ricoh|aficio)\b/i },
  {
    canonical: "Xerox",
    pattern: /\b(xerox|versalink|altalink|workcentre|phaser)\b/i,
  },
  { canonical: "Kyocera", pattern: /\b(kyocera|ecosys|taskalfa)\b/i },
  {
    canonical: "Konica Minolta",
    pattern: /\b(konica\s*minolta|konica|minolta|bizhub)\b/i,
  },
  { canonical: "Lexmark", pattern: /\blexmark\b/i },
  { canonical: "Toshiba", pattern: /\b(toshiba|e-studio)\b/i },
  { canonical: "OKI", pattern: /\boki\b/i },
  { canonical: "Zebra", pattern: /\bzebra\b/i },
  { canonical: "Samsung", pattern: /\bsamsung\b/i },
  { canonical: "Dell", pattern: /\bdell\b/i },
  { canonical: "Pantum", pattern: /\bpantum\b/i },
  { canonical: "Fujifilm", pattern: /\b(fujifilm|fuji\s*xerox|apeos)\b/i },
  { canonical: "DYMO", pattern: /\b(dymo|labelwriter)\b/i },
  { canonical: "Star", pattern: /\b(star\s*micronics)\b/i },
  { canonical: "Bixolon", pattern: /\bbixolon\b/i },
  { canonical: "Citizen", pattern: /\bcitizen\b/i },
];

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
]);

export function inferPrinterMake(
  printer?: Printer,
  detectedModel?: string,
): string | undefined {
  if (!printer && !detectedModel) return undefined;

  // 1. Check model and detectedModel against known brands first
  const primarySources = [printer?.model, detectedModel].filter(
    (s): s is string => Boolean(s && s.trim()),
  );

  for (const source of primarySources) {
    for (const brand of KNOWN_BRANDS) {
      if (brand.pattern.test(source)) {
        return brand.canonical;
      }
    }
  }

  // 2. Check label and description against known brands as a fallback
  const secondarySources = [printer?.label, printer?.description].filter(
    (s): s is string => Boolean(s && s.trim()),
  );

  for (const source of secondarySources) {
    for (const brand of KNOWN_BRANDS) {
      if (brand.pattern.test(source)) {
        return brand.canonical;
      }
    }
  }

  // 3. Fallback: if model/detectedModel starts with a clean brand-like word (letters only, >= 2 chars)
  const modelText = (printer?.model || detectedModel || "").trim();
  if (modelText) {
    const firstToken = modelText.split(/[\s,/()-]+/)[0];
    if (
      firstToken &&
      firstToken.length >= 2 &&
      /^[A-Za-z]+$/.test(firstToken) &&
      !GENERIC_MODEL_WORDS.has(firstToken.toLowerCase())
    ) {
      return firstToken;
    }
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

  const make = inferPrinterMake(printer, state?.detectedModel);
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

  for (const brand of KNOWN_BRANDS) {
    if (brand.canonical.toLowerCase() === lower) {
      return brand.canonical;
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
