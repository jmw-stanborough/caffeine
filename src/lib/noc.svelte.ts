import { DEFAULT_CARD_SCALE } from "$lib/constants";
import { clampCardScale } from "$lib/format";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function createNocController(onNocActivated?: () => void) {
  const appWindow = getCurrentWindow();
  let isNocMode = $state(false);
  let cardScale = $state(DEFAULT_CARD_SCALE);

  function adjustScale(delta: number) {
    cardScale = clampCardScale(cardScale, delta);
  }

  async function setNocMode(active: boolean) {
    if (active && onNocActivated) {
      onNocActivated();
    }
    isNocMode = active;
    try {
      await appWindow.setFullscreen(active);
    } catch (err) {
      console.error("Failed to toggle fullscreen:", err);
      throw err;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape" && isNocMode) {
      setNocMode(false).catch((err) => {
        console.error("Failed to exit NOC mode on Escape:", err);
      });
    }
  }

  async function cleanup() {
    try {
      await appWindow.setFullscreen(false);
    } catch (err) {
      console.error("Failed to reset fullscreen on cleanup:", err);
    }
  }

  return {
    get isNocMode() {
      return isNocMode;
    },
    get cardScale() {
      return cardScale;
    },
    adjustScale,
    setNocMode,
    handleKeyDown,
    cleanup,
  };
}
