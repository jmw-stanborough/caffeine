import { MonitorStore } from "$lib/monitor-store.svelte";
import { soundManager } from "$lib/sound.svelte";
import type { HostState, Server, ServerStatusTick, StatusKind } from "./types";

const SERVER_STATUS_EVENT = "server-status-tick";

export class ServerStore extends MonitorStore<
  Server,
  HostState,
  ServerStatusTick
> {
  private initPromise: Promise<void> | null = null;

  async init(): Promise<void> {
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = (async () => {
      this.destroyed = false;
      await this.fetchServers();
      await this.startTickSubscription(SERVER_STATUS_EVENT, (payload) => {
        const { server_id, status, latency_ms, last_event, last_event_at } =
          payload;
        const previousState = this.statusMap[server_id];

        if (previousState && previousState.status !== "checking") {
          if (previousState.status === "online" && status === "offline") {
            soundManager.play("server_down").catch((err: unknown) => {
              console.warn("Audio playback failed:", err);
            });
          } else if (previousState.status === "offline" && status === "online") {
            soundManager.play("server_up").catch((err: unknown) => {
              console.warn("Audio playback failed:", err);
            });
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
    })().catch((err) => {
      this.initPromise = null;
      throw err;
    });

    return this.initPromise;
  }

  override destroy() {
    super.destroy();
    this.initPromise = null;
  }

  protected effectiveStatus(server: Server): StatusKind {
    return this.statusMap[server.id]?.status ?? "checking";
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

export const serverStore = new ServerStore();
