import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  PREMADE_TAGS,
  getPrinterTags,
  inferPrinterMake,
  normalizeTagLabel,
} from "./tags";
import type { Printer, PrinterState, PrinterStatusTick } from "./types";

export class PrinterStore {
  items = $state<Printer[]>([]);
  statusMap = $state<Record<string, PrinterState>>({});
  filterTags = $state<string[]>([]);
  loading = $state(true);
  errorMessage = $state<string | null>(null);
  now = $state(Date.now());

  private unlistenStatus: UnlistenFn | null = null;
  private timer: number | null = null;
  private destroyed = false;

  async init() {
    this.destroyed = false;
    await Promise.all([this.fetchPrinters(), this.fetchFilterTags()]);
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

  fetchFilterTags = async () => {
    try {
      const saved = await invoke<string[]>("load_printer_filter_tags");
      this.filterTags = saved ?? [];
    } catch (err) {
      console.error("Failed to load printer filter tags:", err);
    }
  };

  private async persistFilterTags() {
    try {
      const snapshot = [...this.filterTags];
      await invoke<string[]>("save_printer_filter_tags", { tags: snapshot });
    } catch (err) {
      console.error("Failed to save printer filter tags:", err);
    }
  }

  getAllKnownTags(): string[] {
    const known = new Set<string>(PREMADE_TAGS);
    for (const printer of this.items) {
      const state = this.statusMap[printer.id];
      for (const tag of getPrinterTags(printer, state)) {
        known.add(tag);
      }
    }
    return Array.from(known);
  }

  getPremadeTagSuggestions(): string[] {
    const result: string[] = [...PREMADE_TAGS];
    const seenLower = new Set(result.map((t) => t.toLowerCase()));

    for (const printer of this.items) {
      const state = this.statusMap[printer.id];
      const make = inferPrinterMake(printer, state?.detectedModel);
      if (make && !seenLower.has(make.toLowerCase())) {
        seenLower.add(make.toLowerCase());
        result.push(make);
      }
    }

    for (const printer of this.items) {
      const loc = printer.location?.trim();
      if (loc && !seenLower.has(loc.toLowerCase())) {
        seenLower.add(loc.toLowerCase());
        result.push(loc);
      }
    }

    return result;
  }

  hasFilterTag(tag: string): boolean {
    const lower = tag.trim().toLowerCase();
    return this.filterTags.some((t) => t.toLowerCase() === lower);
  }

  async addFilterTag(rawTag: string): Promise<boolean> {
    const normalized = normalizeTagLabel(rawTag, this.getAllKnownTags());
    if (!normalized) return false;

    if (this.hasFilterTag(normalized)) {
      return false;
    }

    this.filterTags = [...this.filterTags, normalized];
    await this.persistFilterTags();
    return true;
  }

  async removeFilterTag(tagToRemove: string) {
    const lower = tagToRemove.trim().toLowerCase();
    const next = this.filterTags.filter((t) => t.toLowerCase() !== lower);
    if (next.length !== this.filterTags.length) {
      this.filterTags = next;
      await this.persistFilterTags();
    }
  }

  async removeLastFilterTag() {
    if (this.filterTags.length === 0) return;
    this.filterTags = this.filterTags.slice(0, -1);
    await this.persistFilterTags();
  }

  async toggleFilterTag(tag: string) {
    if (this.hasFilterTag(tag)) {
      await this.removeFilterTag(tag);
    } else {
      await this.addFilterTag(tag);
    }
  }

  async clearFilterTags() {
    if (this.filterTags.length === 0) return;
    this.filterTags = [];
    await this.persistFilterTags();
  }

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
