import { compareFeaturesAlphabetically } from "$lib/features/order";
import type { SettingsTab } from "./types";

const DEFINITIONS: readonly SettingsTab[] = [
  { id: "appearance", label: "Appearance" },
  { id: "log-retention", label: "Log Retention" },
  { id: "network", label: "Network" },
  { id: "sound-effects", label: "Sound Effects" },
  { id: "workspace", label: "Workspace" },
];

export function orderSettingsTabs<T extends { label: string }>(
  tabs: readonly T[],
): T[] {
  return tabs.slice().sort(compareFeaturesAlphabetically);
}

export const SETTINGS_TABS: readonly SettingsTab[] =
  orderSettingsTabs(DEFINITIONS);
