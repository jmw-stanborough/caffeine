export type SeverityLevel = "critical" | "important" | "moderate" | "low" | "other" | "none";

export function parseSeverity(severity?: string): SeverityLevel {
  if (!severity) return "none";
  const s = severity.toLowerCase();
  if (s.includes("critical")) return "critical";
  if (s.includes("important")) return "important";
  if (s.includes("moderate")) return "moderate";
  if (s.includes("low")) return "low";
  return "other";
}

const SEVERITY_CLASSES: Record<SeverityLevel, string> = {
  critical: "sev-critical",
  important: "sev-important",
  moderate: "sev-moderate",
  low: "sev-low",
  other: "sev-other",
  none: "sev-none",
};

const SEVERITY_WEIGHTS: Record<SeverityLevel, number> = {
  critical: 4,
  important: 3,
  moderate: 2,
  low: 1,
  other: 0,
  none: 0,
};

export function getSeverityClass(severity?: string): string {
  return SEVERITY_CLASSES[parseSeverity(severity)];
}

export function getSeverityWeight(severity?: string): number {
  return SEVERITY_WEIGHTS[parseSeverity(severity)];
}
