import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  PREMADE_TAGS,
  getPrinterTags,
  normalizeTagLabel,
} from "./tags";
import type {
  Printer,
  PrinterFilterConfig,
  PrinterState,
  PrinterStatusTick,
} from "./types";

export class PrinterStore {
  items = $state<Printer[]>([]);
  statusMap = $state<Record<string, PrinterState>>({});
  filterAllTags = $state<string[]>([]);
  filterAnyTags = $state<string[]>([]);
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
          detected_make,
          detected_model,
          serial_number,
          page_count,
          printer_state,
          status_message,
          paper_jam,
          out_of_paper,
        } = event.payload;
        const previousState = this.statusMap[printer_id];

        this.statusMap[printer_id] = {
          status,
          latencyMs: latency_ms,
          lastEvent: last_event ?? previousState?.lastEvent,
          lastEventAt: last_event_at ?? previousState?.lastEventAt,
          toners: toners ?? previousState?.toners ?? [],
          detectedMake: detected_make ?? previousState?.detectedMake,
          detectedModel: detected_model ?? previousState?.detectedModel,
          serialNumber: serial_number ?? previousState?.serialNumber,
          pageCount: page_count ?? previousState?.pageCount,
          printerState: printer_state ?? previousState?.printerState,
          statusMessage: status_message,
          paperJam: paper_jam ?? false,
          outOfPaper: out_of_paper ?? false,
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
      const saved = await invoke<PrinterFilterConfig>("load_printer_filter_tags");
      this.filterAllTags = saved?.all_tags ?? [];
      this.filterAnyTags = saved?.any_tags ?? [];
    } catch (err) {
      console.error("Failed to load printer filter tags:", err);
    }
  };

  private async persistFilterTags() {
    try {
      const allTags = [...this.filterAllTags];
      const anyTags = [...this.filterAnyTags];
      await invoke<PrinterFilterConfig>("save_printer_filter_tags", {
        allTags,
        anyTags,
      });
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

  getSuggestedTags(): readonly string[] {
    return PREMADE_TAGS;
  }

  hasFilterAllTag(tag: string): boolean {
    const lower = tag.trim().toLowerCase();
    return this.filterAllTags.some((t) => t.toLowerCase() === lower);
  }

  hasFilterAnyTag(tag: string): boolean {
    const lower = tag.trim().toLowerCase();
    return this.filterAnyTags.some((t) => t.toLowerCase() === lower);
  }

  async addFilterAllTag(rawTag: string): Promise<boolean> {
    const normalized = normalizeTagLabel(rawTag, this.getAllKnownTags());
    if (!normalized) return false;

    if (this.hasFilterAllTag(normalized)) {
      return false;
    }

    this.filterAllTags = [...this.filterAllTags, normalized];
    await this.persistFilterTags();
    return true;
  }

  async addFilterAnyTag(rawTag: string): Promise<boolean> {
    const normalized = normalizeTagLabel(rawTag, this.getAllKnownTags());
    if (!normalized) return false;

    if (this.hasFilterAnyTag(normalized)) {
      return false;
    }

    this.filterAnyTags = [...this.filterAnyTags, normalized];
    await this.persistFilterTags();
    return true;
  }

  async moveFilterTag(
    rawTag: string,
    target: "all" | "any",
    source?: "suggested" | "all" | "any",
  ): Promise<boolean> {
    const normalized = normalizeTagLabel(rawTag, this.getAllKnownTags());
    if (!normalized) return false;
    const lower = normalized.toLowerCase();

    let changed = false;

    if (source === "any" && target === "all") {
      const nextAny = this.filterAnyTags.filter(
        (t) => t.toLowerCase() !== lower,
      );
      if (nextAny.length !== this.filterAnyTags.length) {
        this.filterAnyTags = nextAny;
        changed = true;
      }
    } else if (source === "all" && target === "any") {
      const nextAll = this.filterAllTags.filter(
        (t) => t.toLowerCase() !== lower,
      );
      if (nextAll.length !== this.filterAllTags.length) {
        this.filterAllTags = nextAll;
        changed = true;
      }
    }

    if (target === "all" && !this.hasFilterAllTag(normalized)) {
      this.filterAllTags = [...this.filterAllTags, normalized];
      changed = true;
    } else if (target === "any" && !this.hasFilterAnyTag(normalized)) {
      this.filterAnyTags = [...this.filterAnyTags, normalized];
      changed = true;
    }

    if (changed) {
      await this.persistFilterTags();
    }
    return changed;
  }

  async removeFilterAllTag(tagToRemove: string) {
    const lower = tagToRemove.trim().toLowerCase();
    const next = this.filterAllTags.filter((t) => t.toLowerCase() !== lower);
    if (next.length !== this.filterAllTags.length) {
      this.filterAllTags = next;
      await this.persistFilterTags();
    }
  }

  async removeFilterAnyTag(tagToRemove: string) {
    const lower = tagToRemove.trim().toLowerCase();
    const next = this.filterAnyTags.filter((t) => t.toLowerCase() !== lower);
    if (next.length !== this.filterAnyTags.length) {
      this.filterAnyTags = next;
      await this.persistFilterTags();
    }
  }

  async removeLastFilterAllTag() {
    if (this.filterAllTags.length === 0) return;
    this.filterAllTags = this.filterAllTags.slice(0, -1);
    await this.persistFilterTags();
  }

  async removeLastFilterAnyTag() {
    if (this.filterAnyTags.length === 0) return;
    this.filterAnyTags = this.filterAnyTags.slice(0, -1);
    await this.persistFilterTags();
  }

  async clearFilterAllTags() {
    if (this.filterAllTags.length === 0) return;
    this.filterAllTags = [];
    await this.persistFilterTags();
  }

  async clearFilterAnyTags() {
    if (this.filterAnyTags.length === 0) return;
    this.filterAnyTags = [];
    await this.persistFilterTags();
  }

  async clearFilterTags() {
    if (this.filterAllTags.length === 0 && this.filterAnyTags.length === 0) {
      return;
    }
    this.filterAllTags = [];
    this.filterAnyTags = [];
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
