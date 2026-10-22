import type { Printer, PrinterState, TonerSupply } from "./types";

export const PREMADE_TAGS = ["Printing", "Idle", "Toner Low"] as const;

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

export function isLowToner(toner: TonerSupply): boolean {
  if (
    toner.supply_type === "waste_toner" ||
    toner.supply_type === "waste_ink"
  ) {
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

const KNOWN_STATES = new Set([
  "printing",
  "idle",
  "warmup",
  "ready",
  "stopped",
  "other",
]);

function getTagFacet(
  tagLower: string,
  allMakes: Set<string>,
  allLocations: Set<string>,
  allStates: Set<string>,
): string {
  if (tagLower === "toner low") return "warning";
  if (KNOWN_STATES.has(tagLower) || allStates.has(tagLower)) return "state";
  if (
    allMakes.has(tagLower) ||
    KNOWN_BRANDS.some((b) => b.canonical.toLowerCase() === tagLower)
  ) {
    return "make";
  }
  if (allLocations.has(tagLower)) return "location";
  return `custom:${tagLower}`;
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
  filterTags: string[],
): Printer[] {
  const activeFilters = filterTags
    .map((t) => t.trim().toLowerCase())
    .filter(Boolean);

  if (activeFilters.length === 0) {
    return printers;
  }

  const allMakes = new Set<string>();
  const allLocations = new Set<string>();
  const allStates = new Set<string>();

  for (const p of printers) {
    const st = statusMap[p.id];
    const make = inferPrinterMake(p, st?.detectedModel);
    if (make) allMakes.add(make.toLowerCase());
    if (p.location?.trim()) allLocations.add(p.location.trim().toLowerCase());
    if (st?.printerState?.trim()) {
      allStates.add(st.printerState.trim().toLowerCase());
    }
  }

  // Group active filters by facet so mutually exclusive tags within the same facet
  // (e.g. "Printing" + "Idle" or "SHARP" + "Brother") combine with OR, while
  // tags across different facets (e.g. "SHARP" + "Toner Low") combine with AND.
  const facetGroups = new Map<string, string[]>();
  for (const f of activeFilters) {
    const facet = getTagFacet(f, allMakes, allLocations, allStates);
    const group = facetGroups.get(facet) ?? [];
    group.push(f);
    facetGroups.set(facet, group);
  }

  return printers.filter((printer) => {
    const state = statusMap[printer.id];
    const pTags = getPrinterTags(printer, state);

    for (const group of facetGroups.values()) {
      const matchesFacet = group.some((f) =>
        tagMatchesPrinter(f, pTags, state?.status),
      );
      if (!matchesFacet) {
        return false;
      }
    }
    return true;
  });
}
