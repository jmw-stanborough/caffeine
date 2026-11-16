export interface ServerTokenPublic {
  id: string;
  label: string;
  created_at: string;
  token_preview: string;
}

export interface GeneratedTokenResponse {
  id: string;
  label: string;
  token: string;
  created_at: string;
}

export interface AppSettings {
  target_host: string;
  target_port: number;
  server_mode: boolean;
  server_port: number;
  server_bind: string;
  server_tokens?: ServerTokenPublic[];
  server_admin_token?: string | null;
  admin_token: string | null;
  theme: string;
  retention_days: number;
  max_log_entries: number;
  todo_warning_minutes?: number;
  sound_server_up?: string | null;
  sound_server_down?: string | null;
  sound_notification?: string | null;
}

export type SettingsTabId =
  | "appearance"
  | "log-retention"
  | "network"
  | "sound-effects"
  | "workspace";

export interface SettingsTab {
  id: SettingsTabId;
  label: string;
}
