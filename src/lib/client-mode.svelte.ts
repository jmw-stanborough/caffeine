import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "./components/settings-modal.svelte";

let isRemoteClient = $state(false);
let canEdit = $state(true);
let initialized = $state(false);

export async function refreshClientMode(): Promise<void> {
  try {
    const settings = await invoke<AppSettings>("get_app_settings_cmd");
    const host = settings.target_host.trim().toLowerCase();
    const remote =
      host !== "127.0.0.1" &&
      host !== "localhost" &&
      host !== "::1" &&
      host !== "0.0.0.0";

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
