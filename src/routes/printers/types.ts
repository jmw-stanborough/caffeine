export interface Printer {
  id: string;
  host: string;
  label: string;
  description?: string;
  model?: string;
  location?: string;
  snmp_community: string;
  web_url?: string;
}

export type StatusKind = "online" | "offline" | "degraded" | "checking";
export type FilterMode = "all" | "any";

export interface TonerSupply {
  name: string;
  color: string;
  level: number;
  max_capacity: number;
  percent?: number;
  supply_type: string;
}

export interface PrinterStatusTick {
  host: string;
  printer_id: string;
  status: "online" | "offline" | "degraded";
  latency_ms?: number;
  last_event?: string;
  last_event_at?: string;
  toners: TonerSupply[];
  detected_make?: string;
  detected_model?: string;
  detected_location?: string;
  serial_number?: string;
  page_count?: number;
  printer_state?: string;
  status_message?: string;
  paper_jam?: boolean;
  out_of_paper?: boolean;
  is_color?: boolean;
  is_duplex?: boolean;
}

export interface PrinterState {
  status: StatusKind;
  latencyMs?: number;
  lastEvent?: string;
  lastEventAt?: string;
  toners: TonerSupply[];
  detectedMake?: string;
  detectedModel?: string;
  detectedLocation?: string;
  serialNumber?: string;
  pageCount?: number;
  printerState?: string;
  statusMessage?: string;
  paperJam?: boolean;
  outOfPaper?: boolean;
  isColor?: boolean;
  isDuplex?: boolean;
}

export interface PrinterLog {
  id: number;
  printer_id: string;
  event_type: string;
  latency_ms?: number;
  timestamp: string;
}

export interface PrinterFilterConfig {
  all_tags: string[];
  any_tags: string[];
}
