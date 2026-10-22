import { DEFAULT_SNMP_COMMUNITY } from "$lib/constants";
import { MonitorStore } from "$lib/monitor-store.svelte";
import { invoke } from "@tauri-apps/api/core";
import {
  PREMADE_TAGS,
  getPrinterTags,
  normalizeTagLabel,
} from "./tags";
import type {
  FilterMode,
  Printer,
  PrinterFilterConfig,
  PrinterState,
  PrinterStatusTick,
} from "./types";

const PRINTER_STATUS_EVENT = "printer-status-tick";

export class PrinterStore extends MonitorStore<
  Printer,
  PrinterState,
  PrinterStatusTick
> {
  filterAllTags = $state<string[]>([]);
  filterAnyTags = $state<string[]>([]);

  async init() {
    this.destroyed = false;
    await Promise.all([this.fetchPrinters(), this.fetchFilterTags()]);
    await this.startTickSubscription(PRINTER_STATUS_EVENT, (payload) => {
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
      } = payload;
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
    });
  }

  fetchPrinters = async () => {
    await this.loadItems("load_printers");
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
      await invoke<PrinterFilterConfig>("save_printer_filter_tags", {
        allTags: [...this.filterAllTags],
        anyTags: [...this.filterAnyTags],
      });
    } catch (err) {
      console.error("Failed to save printer filter tags:", err);
    }
  }

  getTagsByMode(mode: FilterMode): string[] {
    return mode === "all" ? this.filterAllTags : this.filterAnyTags;
  }

  private setTagsByMode(mode: FilterMode, next: string[]) {
    if (mode === "all") {
      this.filterAllTags = next;
    } else {
      this.filterAnyTags = next;
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

  hasFilterTag(mode: FilterMode, tag: string): boolean {
    const lower = tag.trim().toLowerCase();
    return this.getTagsByMode(mode).some((t) => t.toLowerCase() === lower);
  }

  async addFilterTag(mode: FilterMode, rawTag: string): Promise<boolean> {
    const normalized = normalizeTagLabel(rawTag, this.getAllKnownTags());
    if (!normalized || this.hasFilterTag(mode, normalized)) {
      return false;
    }

    this.setTagsByMode(mode, [...this.getTagsByMode(mode), normalized]);
    await this.persistFilterTags();
    return true;
  }

  async moveFilterTag(
    rawTag: string,
    target: FilterMode,
    source?: "suggested" | FilterMode,
  ): Promise<boolean> {
    const normalized = normalizeTagLabel(rawTag, this.getAllKnownTags());
    if (!normalized) return false;
    const lower = normalized.toLowerCase();

    let changed = false;

    if ((source === "all" || source === "any") && source !== target) {
      const currentSourceTags = this.getTagsByMode(source);
      const nextSourceTags = currentSourceTags.filter(
        (t) => t.toLowerCase() !== lower,
      );
      if (nextSourceTags.length !== currentSourceTags.length) {
        this.setTagsByMode(source, nextSourceTags);
        changed = true;
      }
    }

    if (!this.hasFilterTag(target, normalized)) {
      this.setTagsByMode(target, [...this.getTagsByMode(target), normalized]);
      changed = true;
    }

    if (changed) {
      await this.persistFilterTags();
    }
    return changed;
  }

  async removeFilterTag(mode: FilterMode, tagToRemove: string) {
    const lower = tagToRemove.trim().toLowerCase();
    const current = this.getTagsByMode(mode);
    const next = current.filter((t) => t.toLowerCase() !== lower);
    if (next.length !== current.length) {
      this.setTagsByMode(mode, next);
      await this.persistFilterTags();
    }
  }

  async removeLastFilterTag(mode: FilterMode) {
    const current = this.getTagsByMode(mode);
    if (current.length === 0) return;
    this.setTagsByMode(mode, current.slice(0, -1));
    await this.persistFilterTags();
  }

  async clearFilterModeTags(mode: FilterMode) {
    if (this.getTagsByMode(mode).length === 0) return;
    this.setTagsByMode(mode, []);
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
    await this.saveEntity(
      {
        createCmd: "save_printer",
        createArgKey: "newPrinter",
        updateCmd: "update_printer",
        updateArgKey: "updatedPrinter",
      },
      savedPrinter,
      existingId,
    );
  }

  async deletePrinter(idToDelete: string) {
    await this.deleteEntity("delete_printer", idToDelete);
  }

  async reorderPrinters(newItems: Printer[]) {
    // Strip transient properties injected by svelte-dnd-action before invoking Tauri.
    const sanitized: Printer[] = newItems.map((item) => ({
      id: item.id,
      host: item.host,
      label: item.label,
      description: item.description,
      model: item.model,
      location: item.location,
      snmp_community: item.snmp_community || DEFAULT_SNMP_COMMUNITY,
      web_url: item.web_url,
    }));

    await this.reorderEntities("reorder_printers", "printers", sanitized);
  }
}
