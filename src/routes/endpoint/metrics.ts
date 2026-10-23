export function getUsageColor(percent: number): string {
  if (percent < 70) {
    return "var(--color-success)";
  }
  if (percent < 85) {
    return "var(--color-warning)";
  }
  return "var(--color-failure)";
}

export function getTemperatureColor(celsius: number): string {
  if (celsius < 60) {
    return "var(--color-success)";
  }
  if (celsius < 80) {
    return "var(--color-warning)";
  }
  return "var(--color-failure)";
}

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const normalized = bytes / Math.pow(1024, i);
  return `${normalized.toFixed(1)} ${units[i]}`;
}

export function formatUptime(uptimeSeconds: number): string {
  const days = Math.floor(uptimeSeconds / 86400);
  const hours = Math.floor((uptimeSeconds % 86400) / 3600);
  const minutes = Math.floor((uptimeSeconds % 3600) / 60);

  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0 || days > 0) parts.push(`${hours}h`);
  parts.push(`${minutes}m`);
  return parts.join(" ");
}
