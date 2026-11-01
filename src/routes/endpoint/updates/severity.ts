export function getSeverityClass(severity?: string): string {
  if (!severity) return "sev-none";
  const s = severity.toLowerCase();
  if (s.includes("critical")) return "sev-critical";
  if (s.includes("important")) return "sev-important";
  if (s.includes("moderate")) return "sev-moderate";
  if (s.includes("low")) return "sev-low";
  return "sev-other";
}

export function getSeverityWeight(severity?: string): number {
  if (!severity) return 0;
  const s = severity.toLowerCase();
  if (s.includes("critical")) return 4;
  if (s.includes("important")) return 3;
  if (s.includes("moderate")) return 2;
  if (s.includes("low")) return 1;
  return 0;
}
