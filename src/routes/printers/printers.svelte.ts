import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Printer, PrinterState, PrinterStatusTick } from "./types";

export class PrinterStore {
  items = $state<Printer[]>([]);
  statusMap = $state<Record<string, PrinterState>>({});
  loading = $state(true);
  errorMessage = $state<string | null>(null);
  now = $state(Date.now());

  private unlistenStatus: UnlistenFn | null = null;
  private timer: number | null = null;
  private destroyed = false;
  private soundDown =
    typeof Audio !== "undefined"
      ? new Audio("/sound-files/server-monitor/down.mp3")
      : null;
  private soundUp =
    typeof Audio !== "undefined"
      ? new Audio("/sound-files/server-monitor/up.mp3")
      : null;

  async init() {
    this.destroyed = false;
    await this.fetchPrinters();
    if (this.destroyed) return;

    this.timer = window.setInterval(() => {
      this.now = Date.now();
    }, 10_000);

    const unlisten = await listen<PrinterStatusTick>(
      "printer-status-tick",
      (event) => {
        const {
          printer_id,
          status,
          latency_ms,
          last_event,
          last_event_at,
          toners,
          detected_model,
          serial_number,
          page_count,
          printer_state,
          status_message,
        } = event.payload;
        const previousState = this.statusMap[printer_id];

        if (previousState && previousState.status !== "checking") {
          if (previousState.status === "online" && status === "offline") {
            if (this.soundDown) {
              this.soundDown.currentTime = 0;
              this.soundDown.play().catch(() => {});
            }
          } else if (
            previousState.status === "offline" &&
            status === "online"
          ) {
            if (this.soundUp) {
              this.soundUp.currentTime = 0;
              this.soundUp.play().catch(() => {});
            }
          }
        }

        this.statusMap[printer_id] = {
          status,
          latencyMs: latency_ms,
          lastEvent:
            (last_event as "connected" | "disconnected") ??
            previousState?.lastEvent,
          lastEventAt: last_event_at ?? previousState?.lastEventAt,
          toners: toners ?? previousState?.toners ?? [],
          detectedModel: detected_model ?? previousState?.detectedModel,
          serialNumber: serial_number ?? previousState?.serialNumber,
          pageCount: page_count ?? previousState?.pageCount,
          printerState: printer_state ?? previousState?.printerState,
          statusMessage: status_message ?? previousState?.statusMessage,
        };
      },
    );

    if (this.destroyed) {
      unlisten();
    } else {
      this.unlistenStatus = unlisten;
    }
  }

  destroy() {
    this.destroyed = true;
    this.unlistenStatus?.();
    this.unlistenStatus = null;
    if (this.timer !== null) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  fetchPrinters = async () => {
    this.loading = true;
    this.errorMessage = null;
    try {
      this.items = await invoke<Printer[]>("load_printers");
    } catch (err) {
      this.errorMessage = String(err);
    } finally {
      this.loading = false;
    }
  };

  async savePrinter(savedPrinter: Printer, existingId?: string) {
    try {
      if (existingId) {
        this.items = await invoke<Printer[]>("update_printer", {
          id: existingId,
          updatedPrinter: savedPrinter,
        });
      } else {
        this.items = await invoke<Printer[]>("save_printer", {
          newPrinter: savedPrinter,
        });
      }
    } catch (err) {
      this.errorMessage = String(err);
    }
  }

  async deletePrinter(idToDelete: string) {
    const backup = this.items;
    this.items = this.items.filter((p) => p.id !== idToDelete);

    try {
      this.items = await invoke<Printer[]>("delete_printer", {
        id: idToDelete,
      });
      delete this.statusMap[idToDelete];
    } catch (err) {
      this.errorMessage = String(err);
      this.items = backup;
    }
  }

  async reorderPrinters(newItems: Printer[]) {
    const previous = this.items;

    const sanitized: Printer[] = newItems.map((item) => ({
      id: item.id,
      host: item.host,
      label: item.label,
      description: item.description,
      model: item.model,
      location: item.location,
      snmp_community: item.snmp_community || "public",
      web_url: item.web_url,
    }));

    this.items = sanitized;

    try {
      this.items = await invoke<Printer[]>("reorder_printers", {
        printers: sanitized,
      });
    } catch (err) {
      this.errorMessage = String(err);
      this.items = previous;
    }
  }
}
