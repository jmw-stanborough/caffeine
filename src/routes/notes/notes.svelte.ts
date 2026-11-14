import { invoke } from "@tauri-apps/api/core";
import { toErrorMessage } from "$lib/errors";
import type { NoteDocument, NoteSummary } from "./types";

export class NotesStore {
  notes = $state<NoteSummary[]>([]);
  activeNote = $state<NoteDocument | null>(null);
  loading = $state(false);
  saving = $state(false);
  errorMessage = $state<string | null>(null);

  private initPromise: Promise<void> | null = null;

  async init(): Promise<void> {
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = this.fetchNotes().finally(() => {
      this.initPromise = null;
    });
    return this.initPromise;
  }

  async fetchNotes(): Promise<void> {
    this.loading = true;
    this.errorMessage = null;
    try {
      this.notes = await invoke<NoteSummary[]>("workspace_list_notes");
    } catch (err: unknown) {
      this.errorMessage = `Failed to load notes: ${toErrorMessage(err)}`;
      throw err;
    } finally {
      this.loading = false;
    }
  }

  async selectNote(slug: string): Promise<void> {
    this.loading = true;
    this.errorMessage = null;
    try {
      this.activeNote = await invoke<NoteDocument>("workspace_read_note", { slug });
    } catch (err: unknown) {
      this.errorMessage = `Failed to read note: ${toErrorMessage(err)}`;
      throw err;
    } finally {
      this.loading = false;
    }
  }

  async saveNote(slug: string | null, title: string, content: string): Promise<NoteDocument> {
    this.saving = true;
    this.errorMessage = null;
    try {
      const saved = await invoke<NoteDocument>("workspace_save_note", {
        slug,
        title,
        content,
      });
      this.activeNote = saved;
      await this.fetchNotes();
      return saved;
    } catch (err: unknown) {
      this.errorMessage = `Failed to save note: ${toErrorMessage(err)}`;
      throw err;
    } finally {
      this.saving = false;
    }
  }

  async renameNote(slug: string, newTitle: string): Promise<NoteDocument> {
    this.saving = true;
    this.errorMessage = null;
    try {
      const renamed = await invoke<NoteDocument>("workspace_rename_note", {
        slug,
        newTitle,
      });
      if (this.activeNote?.slug === slug || this.activeNote?.slug === renamed.slug) {
        this.activeNote = renamed;
      }
      await this.fetchNotes();
      return renamed;
    } catch (err: unknown) {
      this.errorMessage = `Failed to rename note: ${toErrorMessage(err)}`;
      throw err;
    } finally {
      this.saving = false;
    }
  }

  async deleteNote(slug: string): Promise<void> {
    this.saving = true;
    this.errorMessage = null;
    try {
      await invoke("workspace_delete_note", { slug });
      if (this.activeNote?.slug === slug) {
        this.activeNote = null;
      }
      await this.fetchNotes();
    } catch (err: unknown) {
      this.errorMessage = `Failed to delete note: ${toErrorMessage(err)}`;
      throw err;
    } finally {
      this.saving = false;
    }
  }
}

export const notesStore = new NotesStore();
