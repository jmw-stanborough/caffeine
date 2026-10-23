import {
  NUMBER_DIRECTION_LABELS,
  STATUS_DIRECTION_LABELS,
  TEXT_DIRECTION_LABELS,
  compareStatus,
} from "$lib/sort/sort";
import type { SortFieldDefinition } from "$lib/sort/types";
import type { HostState, Server } from "./types";

export const SERVER_SORT_STORAGE_KEY = "caffeine_sort_servers";

export const SERVER_SORT_FIELDS: SortFieldDefinition<
  Server,
  Record<string, HostState>
>[] = [
  {
    id: "status",
    label: "Status",
    directionLabels: STATUS_DIRECTION_LABELS,
    defaultDirection: "desc",
    compare: (a, b, direction, context) =>
      compareStatus(
        context?.[a.id]?.status,
        context?.[b.id]?.status,
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
    id: "host",
    label: "Host / IP",
    directionLabels: TEXT_DIRECTION_LABELS,
    defaultDirection: "asc",
    getValue: (item) => item.host,
  },
  {
    id: "latency",
    label: "Latency",
    directionLabels: NUMBER_DIRECTION_LABELS,
    defaultDirection: "asc",
    getValue: (item, context) => context?.[item.id]?.latencyMs,
  },
  {
    id: "description",
    label: "Description",
    directionLabels: TEXT_DIRECTION_LABELS,
    defaultDirection: "asc",
    getValue: (item) => item.description,
  },
];
