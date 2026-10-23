import { MonitorStore } from "$lib/monitor-store.svelte";
import type { HostState, Server, ServerStatusTick } from "./types";

const SOUND_DOWN_PATH = "/sound-files/servers/down.mp3";
const SOUND_UP_PATH = "/sound-files/servers/up.mp3";
const SERVER_STATUS_EVENT = "server-status-tick";

function playAlertSound(audio: HTMLAudioElement | null) {
  if (!audio) return;
  audio.currentTime = 0;
  audio.play().catch((err: unknown) => {
    console.warn("Audio playback failed:", err);
  });
}

export class ServerStore extends MonitorStore<
  Server,
  HostState,
  ServerStatusTick
> {
  private soundDown =
    typeof Audio !== "undefined" ? new Audio(SOUND_DOWN_PATH) : null;
  private soundUp =
    typeof Audio !== "undefined" ? new Audio(SOUND_UP_PATH) : null;

  async init() {
    this.destroyed = false;
    await this.fetchServers();
    await this.startTickSubscription(SERVER_STATUS_EVENT, (payload) => {
      const { server_id, status, latency_ms, last_event, last_event_at } =
        payload;
      const previousState = this.statusMap[server_id];

      if (previousState && previousState.status !== "checking") {
        if (previousState.status === "online" && status === "offline") {
          playAlertSound(this.soundDown);
        } else if (previousState.status === "offline" && status === "online") {
          playAlertSound(this.soundUp);
        }
      }

      this.statusMap[server_id] = {
        status,
        latencyMs: latency_ms,
        lastEvent:
          (last_event as "connected" | "disconnected") ??
          previousState?.lastEvent,
        lastEventAt: last_event_at ?? previousState?.lastEventAt,
      };
    });
  }

  fetchServers = async () => {
    await this.loadItems("load_servers");
  };

  async saveServer(savedServer: Server, existingId?: string) {
    await this.saveEntity(
      {
        createCmd: "save_server",
        createArgKey: "newServer",
        updateCmd: "update_server",
        updateArgKey: "updatedServer",
      },
      savedServer,
      existingId,
    );
  }

  async deleteServer(idToDelete: string) {
    await this.deleteEntity("delete_server", idToDelete);
  }

  async reorderServers(newItems: Server[]) {
    // Strip transient properties injected by svelte-dnd-action before invoking Tauri.
    const sanitized: Server[] = newItems.map((item) => ({
      id: item.id,
      host: item.host,
      label: item.label,
      description: item.description,
      agent: item.agent
        ? { port: item.agent.port, api_key: item.agent.api_key }
        : undefined,
    }));

    await this.reorderEntities("reorder_servers", "servers", sanitized);
  }
}
