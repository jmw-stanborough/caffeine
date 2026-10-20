import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DndServer, HostState, Server, ServerStatusTick } from "./types";

export class ServerStore {
  items = $state<DndServer[]>([]);
  statusMap = $state<Record<string, HostState>>({});
  loading = $state(true);
  errorMessage = $state<string | null>(null);

  private unlistenStatus: UnlistenFn | null = null;

  async init() {
    await this.fetchServers();

    this.unlistenStatus = await listen<ServerStatusTick>(
      "server-status-tick",
      (event) => {
        const { host, status, latency_ms } = event.payload;
        this.statusMap[host] = { status, latencyMs: latency_ms };
      },
    );
  }

  destroy() {
    this.unlistenStatus?.();
  }

  fetchServers = async () => {
    this.loading = true;
    this.errorMessage = null;
    try {
      const raw = await invoke<Server[]>("load_servers");
      this.items = raw.map((s) => ({ ...s, id: s.host }));
    } catch (err) {
      this.errorMessage = String(err);
    } finally {
      this.loading = false;
    }
  };

  async saveServer(savedServer: Server, originalHost?: string) {
    try {
      const cmd = originalHost ? "update_server" : "save_server";
      const payload = originalHost
        ? { originalHost, updatedServer: savedServer }
        : { newServer: savedServer };

      const updated = await invoke<Server[]>(cmd, payload);
      this.items = updated.map((s) => ({ ...s, id: s.host }));
    } catch (err) {
      this.errorMessage = String(err);
    }
  }

  async deleteServer(hostToDelete: string) {
    const backup = this.items;
    this.items = this.items.filter((s) => s.host !== hostToDelete);

    try {
      const updated = await invoke<Server[]>("delete_server", {
        host: hostToDelete,
      });
      this.items = updated.map((s) => ({ ...s, id: s.host }));

      const nextStatus = { ...this.statusMap };
      delete nextStatus[hostToDelete];
      this.statusMap = nextStatus;
    } catch (err) {
      this.errorMessage = String(err);
      this.items = backup;
    }
  }

  async reorderServers(items: DndServer[]) {
    this.items = items;
    const payload = items.map(({ id, ...rest }) => rest);
    try {
      await invoke("reorder_servers", { servers: payload });
    } catch (err) {
      this.errorMessage = String(err);
    }
  }
}
