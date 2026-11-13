import { invoke } from "@tauri-apps/api/core";
import type { DetectedScriptLanguage, ScriptDocument, ScriptSummary } from "./types";

function toErrorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

export class ScriptsStore {
  scripts = $state<ScriptSummary[]>([]);
  languages = $state<DetectedScriptLanguage[]>([]);
  activeScript = $state<ScriptDocument | null>(null);
  loading = $state(false);
  saving = $state(false);
  errorMessage = $state<string | null>(null);

  private initPromise: Promise<void> | null = null;

  async init(): Promise<void> {
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = Promise.all([
      this.fetchLanguages(),
      this.fetchScripts(),
    ])
      .then(() => undefined)
      .finally(() => {
        this.initPromise = null;
      });
    return this.initPromise;
  }

  async fetchLanguages(): Promise<void> {
    this.errorMessage = null;
    try {
      this.languages = await invoke<DetectedScriptLanguage[]>(
        "workspace_detect_script_languages",
      );
    } catch (err: unknown) {
      this.errorMessage = `Failed to detect script languages: ${toErrorMessage(err)}`;
      throw err;
    }
  }

  async fetchScripts(): Promise<void> {
    this.loading = true;
    this.errorMessage = null;
    try {
      this.scripts = await invoke<ScriptSummary[]>("workspace_list_scripts");
    } catch (err: unknown) {
      this.errorMessage = `Failed to load scripts: ${toErrorMessage(err)}`;
      throw err;
    } finally {
      this.loading = false;
    }
  }

  async selectScript(id: string): Promise<void> {
    this.loading = true;
    this.errorMessage = null;
    try {
      this.activeScript = await invoke<ScriptDocument>("workspace_read_script", { id });
    } catch (err: unknown) {
      this.errorMessage = `Failed to read script: ${toErrorMessage(err)}`;
      throw err;
    } finally {
      this.loading = false;
    }
  }

  async saveScript(
    id: string | null,
    name: string,
    languageId: string,
    content: string,
  ): Promise<ScriptDocument> {
    this.saving = true;
    this.errorMessage = null;
    try {
      const saved = await invoke<ScriptDocument>("workspace_save_script", {
        id,
        name,
        languageId,
        content,
      });
      this.activeScript = saved;
      await this.fetchScripts();
      return saved;
    } catch (err: unknown) {
      this.errorMessage = `Failed to save script: ${toErrorMessage(err)}`;
      throw err;
    } finally {
      this.saving = false;
    }
  }

  async deleteScript(id: string): Promise<void> {
    this.saving = true;
    this.errorMessage = null;
    try {
      await invoke("workspace_delete_script", { id });
      if (this.activeScript?.id === id) {
        this.activeScript = null;
      }
      await this.fetchScripts();
    } catch (err: unknown) {
      this.errorMessage = `Failed to delete script: ${toErrorMessage(err)}`;
      throw err;
    } finally {
      this.saving = false;
    }
  }

  async formatContent(languageId: string, content: string): Promise<string> {
    this.errorMessage = null;
    try {
      return await invoke<string>("workspace_format_script", {
        languageId,
        content,
      });
    } catch (err: unknown) {
      this.errorMessage = `Failed to format script: ${toErrorMessage(err)}`;
      throw err;
    }
  }

  async buildRunCommand(id: string): Promise<string> {
    this.errorMessage = null;
    try {
      return await invoke<string>("workspace_build_script_run_command", { id });
    } catch (err: unknown) {
      this.errorMessage = `Failed to build run command: ${toErrorMessage(err)}`;
      throw err;
    }
  }
}

export const scriptsStore = new ScriptsStore();
