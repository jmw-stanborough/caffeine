const MS_PER_MINUTE = 60_000;
const MINUTES_PER_HOUR = 60;
const MINUTES_PER_DAY = 1440;

function padTwo(value: number): string {
  return String(value).padStart(2, "0");
}

export function toDateTimeLocalValue(ms: number | null): string {
  if (ms === null) return "";
  const date = new Date(ms);
  if (Number.isNaN(date.getTime())) return "";
  return `${date.getFullYear()}-${padTwo(date.getMonth() + 1)}-${padTwo(date.getDate())}T${padTwo(date.getHours())}:${padTwo(date.getMinutes())}`;
}

export function fromDateTimeLocalValue(value: string): number | null {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const parsed = new Date(trimmed).getTime();
  if (Number.isNaN(parsed)) {
    throw new Error(`Invalid date/time value: "${value}"`);
  }
  return parsed;
}

export function formatTodoTimestamp(ms: number): string {
  return new Date(ms).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatMinuteDuration(totalMinutes: number): string {
  if (totalMinutes < 1) return "<1m";
  if (totalMinutes < MINUTES_PER_HOUR) return `${totalMinutes}m`;
  if (totalMinutes < MINUTES_PER_DAY) {
    const hours = Math.floor(totalMinutes / MINUTES_PER_HOUR);
    const mins = totalMinutes % MINUTES_PER_HOUR;
    return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
  }
  const days = Math.floor(totalMinutes / MINUTES_PER_DAY);
  const hours = Math.floor((totalMinutes % MINUTES_PER_DAY) / MINUTES_PER_HOUR);
  return hours > 0 ? `${days}d ${hours}h` : `${days}d`;
}

export function formatAttentionDetail(
  dueAt: number,
  nowMs: number,
  isOverdue: boolean,
): string {
  const diffMinutes = Math.round(Math.abs(dueAt - nowMs) / MS_PER_MINUTE);
  const span = formatMinuteDuration(diffMinutes);
  return isOverdue
    ? `Overdue by ${span} (${formatTodoTimestamp(dueAt)})`
    : `Due in ${span} (${formatTodoTimestamp(dueAt)})`;
}
