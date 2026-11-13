import type { Printer } from "./types";

export function resolvePrinterUrl(
  target: Printer,
  scheme?: "http" | "https",
): string {
  const raw = target.web_url?.trim() || target.host.trim();
  if (!raw) {
    throw new Error(`Printer '${target.label || target.id}' has no valid host or web URL`);
  }

  if (scheme) {
    const stripped = raw.replace(/^https?:\/\//i, "");
    return `${scheme}://${stripped}`;
  }

  if (raw.startsWith("http://") || raw.startsWith("https://")) {
    return raw;
  }
  return `http://${raw}`;
}
