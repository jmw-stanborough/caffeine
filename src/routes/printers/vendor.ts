import type { Printer } from "./types";

const MIN_VENDOR_NAME_LENGTH = 2;

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
    cleaned.length >= MIN_VENDOR_NAME_LENGTH &&
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

  // IEEE 1284 device ID strings embed manufacturer and model inside semicolon-delimited
  // key-value pairs (e.g. "MFG:Brother Industries, Ltd.;MDL:HL-L3240CDW;"), so check
  // structured segments before falling back to leading-word extraction.
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

  const firstClause = trimmed.split(/[;,\n/]/)[0]?.trim() ?? "";
  const tokens = firstClause.split(/\s+/).filter(Boolean);
  if (tokens.length >= MIN_VENDOR_NAME_LENGTH) {
    const candidate = tokens[0].replace(/^[^A-Za-z0-9-]+|[^A-Za-z0-9-]+$/g, "");
    if (
      candidate.length >= MIN_VENDOR_NAME_LENGTH &&
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

  return (
    extractPrinterMakeFromText(detectedModel) ??
    extractPrinterMakeFromText(printer?.model)
  );
}
