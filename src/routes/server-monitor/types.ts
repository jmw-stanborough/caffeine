export interface Agent {
  port: number;
  api_key: string;
}

export interface Server {
  host: string;
  label: string;
  description?: string;
  agent?: Agent;
}

export type DndServer = Server & { id: string };

export type StatusKind = "online" | "offline" | "degraded" | "checking";

export interface ServerStatusTick {
  host: string;
  status: "online" | "offline" | "degraded";
  latency_ms?: number;
}

export interface HostState {
  status: StatusKind;
  latencyMs?: number;
}
