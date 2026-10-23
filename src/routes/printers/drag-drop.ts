import type { FilterMode } from "./types";

export type DragSource = "suggested" | FilterMode;

export const DROP_ZONE_HIT_MARGIN_PX = 6;
export const DRAG_ACTIVATION_DISTANCE_PX = 4;
export const CUSTOM_TAG_MIME = "application/x-caffeine-tag";

export interface PointerDragState {
  tag: string;
  source: DragSource;
  startX: number;
  startY: number;
  x: number;
  y: number;
  active: boolean;
}

export function isPointInsideBox(
  el: HTMLElement | undefined,
  clientX: number,
  clientY: number,
): boolean {
  if (!el) return false;
  const rect = el.getBoundingClientRect();
  return (
    clientX >= rect.left - DROP_ZONE_HIT_MARGIN_PX &&
    clientX <= rect.right + DROP_ZONE_HIT_MARGIN_PX &&
    clientY >= rect.top - DROP_ZONE_HIT_MARGIN_PX &&
    clientY <= rect.bottom + DROP_ZONE_HIT_MARGIN_PX
  );
}

export function hitTestDropZone(
  allBoxEl: HTMLElement | undefined,
  anyBoxEl: HTMLElement | undefined,
  clientX: number,
  clientY: number,
): FilterMode | null {
  if (isPointInsideBox(allBoxEl, clientX, clientY)) return "all";
  if (isPointInsideBox(anyBoxEl, clientX, clientY)) return "any";
  return null;
}

export function parseDragPayload(dataTransfer: DataTransfer | null): {
  tag: string;
  source: DragSource;
} {
  if (!dataTransfer) return { tag: "", source: "suggested" };

  const customPayload = dataTransfer.getData(CUSTOM_TAG_MIME);
  if (customPayload) {
    try {
      const parsed = JSON.parse(customPayload);
      const tag = parsed?.tag ? String(parsed.tag) : "";
      const source = parsed?.source ?? "suggested";
      if (tag) return { tag, source };
    } catch (err) {
      console.warn("Failed to parse drag JSON payload:", err);
    }
  }

  const plainText = dataTransfer.getData("text/plain") ?? "";
  return { tag: plainText, source: "suggested" };
}
