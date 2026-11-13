export interface NoteSummary {
  slug: string;
  title: string;
  filename: string;
  updated_at_ms: number;
}

export interface NoteDocument {
  slug: string;
  title: string;
  filename: string;
  content: string;
  updated_at_ms: number;
}

export type NoteViewMode = "split" | "edit" | "read";
