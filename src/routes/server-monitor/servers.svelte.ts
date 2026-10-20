import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { HostState, Server, ServerStatusTick } from "./types";

export class ServerStore {
  items = $state<Server[]>([]);
  statusMap = $state<Record<string, HostState>>({});
  loading = $state(true);
  errorMessage = $state<string | null>(null);

  private unlistenStatus: UnlistenFn | null = null;
  private soundDown = typeof Audio !== "undefined" ? new Audio("/sound-files/server-monitor/down.mp3") : null;
  private soundUp = typeof Audio !== "undefined" ? new Audio("/sound-files/server-monitor/up.mp3") : null;

  async init() {
    await this.fetchServers();

    this.unlistenStatus = await listen<ServerStatusTick>("server-status-tick", (event) => {
      const { host, status, latency_ms, last_event, last_event_at } = event.payload;
      const previousState = this.statusMap[host];

      if (previousState && previousState.status !== "checking") {
        if (previousState.status === "online" && status === "offline") {
          if (this.soundDown) {
            this.soundDown.currentTime = 0;
            this.soundDown.play().catch(() => {});
          }
        } else if (previousState.status === "offline" && status === "online") {
          if (this.soundUp) {
            this.soundUp.currentTime = 0;
            this.soundUp.play().catch(() => {});
          }
        }
      }

      this.statusMap[host] = {
        status,
        latencyMs: latency_ms,
        lastEvent: (last_event as "connected" | "disconnected") ?? previousState?.lastEvent,
        lastEventAt: last_event_at ?? previousState?.lastEventAt,
      };
    });
  }

  destroy() {
    this.unlistenStatus?.();
  }

  fetchServers = async () => {
    this.loading = true;
    this.errorMessage = null;
    try {
      this.items = await invoke<Server[]>("load_servers");
    } catch (err) {
      this.errorMessage = String(err);
    } finally {
      this.loading = false;
    }
  };

  async saveServer(savedServer: Server, existingId?: string) {
    try {
      if (existingId) {
        this.items = await invoke<Server[]>("update_server", {
          id: existingId,
          updatedServer: savedServer,
        });
      } else {
        this.items = await invoke<Server[]>("save_server", {
          newServer: savedServer,
        });
      }
    } catch (err) {
      this.errorMessage = String(err);
    }
  }

  async deleteServer(idToDelete: string) {
    const target = this.items.find((s) => s.id === idToDelete);
    const backup = this.items;
    this.items = this.items.filter((s) => s.id !== idToDelete);

    try {
      this.items = await invoke<Server[]>("delete_server", { id: idToDelete });
      if (target && !this.items.some((s) => s.host === target.host)) {
        const nextStatus = { ...this.statusMap };
        delete nextStatus[target.host];
        this.statusMap = nextStatus;
      }
    } catch (err) {
      this.errorMessage = String(err);
      this.items = backup;
    }
  }

  async reorderServers(newItems: Server[]) {
    const previous = this.items;

    const sanitized: Server[] = newItems.map((item) => ({
      id: item.id,
      host: item.host,
      label: item.label,
      description: item.description,
      agent: item.agent ? { port: item.agent.port, api_key: item.agent.api_key } : undefined,
    }));

    this.items = sanitized;

    try {
      await invoke("reorder_servers", { servers: sanitized });
    } catch (err) {
      this.errorMessage = String(err);
      this.items = previous;
    }
  }
}
