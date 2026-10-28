export interface Agent {
  port: number;
  api_key: string;
}

export interface Server {
  id: string;
  host: string;
  label: string;
  description?: string;
  agent?: Agent;
  ssh_port?: number;
  rdp_port?: number;
  remote_username?: string;
}

export type StatusKind = "online" | "offline" | "degraded" | "checking";

export interface ServerStatusTick {
  host: string;
  server_id: string;
  status: "online" | "offline" | "degraded";
  latency_ms?: number;
  last_event?: string;
  last_event_at?: string;
}

export interface HostState {
  status: StatusKind;
  latencyMs?: number;
  lastEvent?: "connected" | "disconnected";
  lastEventAt?: string;
}

export interface ServerLog {
  id: number;
  server_id: string;
  event_type: string;
  latency_ms?: number;
  timestamp: string;
}
