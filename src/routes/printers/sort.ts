import {
  NUMBER_DIRECTION_LABELS,
  STATUS_DIRECTION_LABELS,
  TEXT_DIRECTION_LABELS,
  compareStatus,
} from "$lib/sort/sort";
import type { SortFieldDefinition } from "$lib/sort/types";
import { getEffectivePrinterStatus } from "./status";
import type { Printer, PrinterState } from "./types";

export const PRINTER_SORT_STORAGE_KEY = "caffeine_sort_printers";

export const PRINTER_SORT_FIELDS: SortFieldDefinition<
  Printer,
  Record<string, PrinterState>
>[] = [
  {
    id: "status",
    label: "Status",
    directionLabels: STATUS_DIRECTION_LABELS,
    defaultDirection: "desc",
    compare: (a, b, direction, context) =>
      compareStatus(
        getEffectivePrinterStatus(context?.[a.id]),
        getEffectivePrinterStatus(context?.[b.id]),
        direction,
      ),
  },
  {
    id: "label",
    label: "Label",
    directionLabels: TEXT_DIRECTION_LABELS,
    defaultDirection: "asc",
    getValue: (item) => item.label,
  },
  {
    id: "make",
    label: "Make",
    directionLabels: TEXT_DIRECTION_LABELS,
    defaultDirection: "asc",
    getValue: (item, context) => context?.[item.id]?.detectedMake,
  },
  {
    id: "model",
    label: "Model",
    directionLabels: TEXT_DIRECTION_LABELS,
    defaultDirection: "asc",
    getValue: (item, context) =>
      item.model || context?.[item.id]?.detectedModel,
  },
  {
    id: "host",
    label: "Host / IP",
    directionLabels: TEXT_DIRECTION_LABELS,
    defaultDirection: "asc",
    getValue: (item) => item.host,
  },
  {
    id: "location",
    label: "Location",
    directionLabels: TEXT_DIRECTION_LABELS,
    defaultDirection: "asc",
    getValue: (item, context) =>
      item.location || context?.[item.id]?.detectedLocation,
  },
  {
    id: "latency",
    label: "Latency",
    directionLabels: NUMBER_DIRECTION_LABELS,
    defaultDirection: "asc",
    getValue: (item, context) => context?.[item.id]?.latencyMs,
  },
];
