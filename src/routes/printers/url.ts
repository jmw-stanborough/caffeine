import type { Printer } from "./types";

export function resolvePrinterUrl(target: Printer): string {
  const raw = target.web_url?.trim() || target.host.trim();
  if (raw.startsWith("http://") || raw.startsWith("https://")) {
    return raw;
  }
  return `http://${raw}`;
}
