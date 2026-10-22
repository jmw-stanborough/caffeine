import {
   MAX_CARD_SCALE,
  MIN_CARD_SCALE,
  MS_PER_SECOND,
  SECONDS_PER_DAY,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
} from "./constants";

export function formatRelativeTime(
  dateString?: string,
  currentTime: number = Date.now(),
): string {
  if (!dateString) return "";
  const parsed = new Date(dateString);
  if (isNaN(parsed.getTime())) return "";

  const diffSec = Math.floor((currentTime - parsed.getTime()) / MS_PER_SECOND);
  if (diffSec < SECONDS_PER_MINUTE) return "just now";
  if (diffSec < SECONDS_PER_HOUR) {
    return `${Math.floor(diffSec / SECONDS_PER_MINUTE)}m ago`;
  }
  if (diffSec < SECONDS_PER_DAY) {
    return `${Math.floor(diffSec / SECONDS_PER_HOUR)}h ago`;
  }
  return `${Math.floor(diffSec / SECONDS_PER_DAY)}d ago`;
}

export function formatTimestamp(isoString: string): string {
  const date = new Date(isoString);
  if (isNaN(date.getTime())) return isoString;
  return date.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function clampCardScale(currentScale: number, delta: number): number {
  return Math.min(
    MAX_CARD_SCALE,
    Math.max(MIN_CARD_SCALE, +(currentScale + delta).toFixed(2)),
  );
}

export function activateOnEnterOrSpace(
  e: KeyboardEvent,
  callback?: () => void,
) {
  if ((e.key === "Enter" || e.key === " ") && callback) {
    e.preventDefault();
    callback();
  }
}
