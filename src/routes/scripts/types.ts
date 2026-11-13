export interface DetectedScriptLanguage {
  id: string;
  label: string;
  extension: string;
  executable: string;
}

export interface ScriptSummary {
  id: string;
  name: string;
  filename: string;
  extension: string;
  language_id: string;
  updated_at_ms: number;
}

export interface ScriptDocument {
  id: string;
  name: string;
  filename: string;
  extension: string;
  language_id: string;
  content: string;
  path: string;
  updated_at_ms: number;
}
