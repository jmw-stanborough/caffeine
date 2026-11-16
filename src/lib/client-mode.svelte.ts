import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "$lib/settings/types";

const LOCAL_HOSTS = new Set(["127.0.0.1", "localhost", "::1", "0.0.0.0"]);

export function isRemoteHost(host: string): boolean {
  const normalized = host.trim().toLowerCase();
  return normalized.length > 0 && !LOCAL_HOSTS.has(normalized);
}

let isRemoteClient = $state(false);
let canEdit = $state(true);
let initialized = $state(false);

export async function refreshClientMode(): Promise<void> {
  try {
    const settings = await invoke<AppSettings>("get_app_settings_cmd");
    const remote = isRemoteHost(settings.target_host);

    isRemoteClient = remote;
    canEdit = !remote || Boolean(settings.admin_token?.trim());
    initialized = true;
  } catch (err) {
    console.error("Failed to load client mode settings:", err);
  }
}

export const clientMode = {
  get isRemoteClient() {
    return isRemoteClient;
  },
  get canEdit() {
    return canEdit;
  },
  get initialized() {
    return initialized;
  },
  refresh: refreshClientMode,
};
