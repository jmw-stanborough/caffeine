<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { refreshClientMode } from "$lib/client-mode.svelte";
  import Modal from "$lib/components/modal.svelte";
  import { toErrorMessage } from "$lib/errors";
  import AppearanceTab from "$lib/settings/appearance-tab.svelte";
  import LogRetentionTab from "$lib/settings/log-retention-tab.svelte";
  import NetworkTab from "$lib/settings/network-tab.svelte";
  import SoundEffectsTab from "$lib/settings/sound-effects-tab.svelte";
  import { SETTINGS_TABS } from "$lib/settings/tabs";
  import type {
    AppSettings,
    GeneratedTokenResponse,
    ServerTokenPublic,
    SettingsTabId,
  } from "$lib/settings/types";
  import WorkspaceTab from "$lib/settings/workspace-tab.svelte";
  import { soundManager } from "$lib/sound.svelte";
  import { applyTheme } from "$lib/theme";
  import { notesStore } from "../../routes/notes/notes.svelte";
  import { printerStore } from "../../routes/printers/printers.svelte";
  import { scriptsStore } from "../../routes/scripts/scripts.svelte";
  import { serverStore } from "../../routes/servers/servers.svelte";
  import { todoStore } from "../../routes/todo/todo.svelte";

  let {
    open = $bindable(false),
  }: {
    open: boolean;
  } = $props();

  let activeTab = $state<SettingsTabId>(SETTINGS_TABS[0].id);

  let targetHost = $state("127.0.0.1");
  let targetPort = $state(6310);
  let serverMode = $state(false);
  let serverPort = $state(6310);
  let serverBind = $state("0.0.0.0");
  let serverTokens = $state<ServerTokenPublic[]>([]);
  let adminToken = $state("");
  let theme = $state("system");
  let persistedTheme = $state("system");
  let retentionDays = $state(14);
  let maxLogEntries = $state(1000);
  let todoWarningMinutes = $state(120);
  let workspaceDir = $state("");
  let soundServerUp = $state("");
  let soundServerDown = $state("");
  let soundNotification = $state("");

  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let successMsg = $state<string | null>(null);

  $effect(() => {
    if (open) {
      activeTab = SETTINGS_TABS[0].id;
      loadSettings();
    }
  });

  function handleClose() {
    theme = persistedTheme;
    applyTheme(persistedTheme);
    open = false;
  }

  async function loadSettings() {
    loading = true;
    error = null;
    successMsg = null;
    try {
      const settings = await invoke<AppSettings>("get_app_settings_cmd");
      targetHost = settings.target_host;
      targetPort = settings.target_port;
      serverMode = settings.server_mode;
      serverPort = settings.server_port;
      serverBind = settings.server_bind;
      serverTokens = settings.server_tokens ?? [];
      adminToken = settings.admin_token ?? "";
      persistedTheme = settings.theme;
      theme = settings.theme;
      applyTheme(persistedTheme);
      retentionDays = settings.retention_days;
      maxLogEntries = settings.max_log_entries;
      todoWarningMinutes = settings.todo_warning_minutes ?? 120;
      workspaceDir = settings.workspace_dir ?? "";
      soundServerUp = settings.sound_server_up ?? "";
      soundServerDown = settings.sound_server_down ?? "";
      soundNotification = settings.sound_notification ?? "";
    } catch (err) {
      error = toErrorMessage(err);
    } finally {
      loading = false;
    }
  }

  function validateIntegerRange(
    value: unknown,
    min: number,
    max: number,
    label: string,
    tab: SettingsTabId,
  ): number | null {
    const parsed = Number(value);
    if (!Number.isInteger(parsed) || parsed < min || parsed > max) {
      activeTab = tab;
      error = `${label} must be a whole number between ${min} and ${max}.`;
      return null;
    }
    return parsed;
  }

  function validatePayload(): AppSettings | null {
    const trimmedTargetHost = targetHost.trim();
    if (!trimmedTargetHost) {
      activeTab = "network";
      error = "Target Host / IP is required.";
      return null;
    }

    const validTargetPort = validateIntegerRange(
      targetPort,
      1,
      65535,
      "Target Port",
      "network",
    );
    if (validTargetPort === null) return null;

    const trimmedServerBind = serverBind.trim();
    let validServerPort = Number(serverPort);
    if (serverMode) {
      if (!trimmedServerBind) {
        activeTab = "network";
        error = "Bind Address is required when Server Mode is enabled.";
        return null;
      }
      const checkedServerPort = validateIntegerRange(
        serverPort,
        1,
        65535,
        "Server Port",
        "network",
      );
      if (checkedServerPort === null) return null;
      validServerPort = checkedServerPort;
    }

    const validRetentionDays = validateIntegerRange(
      retentionDays,
      1,
      3650,
      "Retention (Days)",
      "log-retention",
    );
    if (validRetentionDays === null) return null;

    const validMaxLogEntries = validateIntegerRange(
      maxLogEntries,
      50,
      100000,
      "Max Entries Per Entity",
      "log-retention",
    );
    if (validMaxLogEntries === null) return null;

    const validTodoWarningMinutes = validateIntegerRange(
      todoWarningMinutes,
      5,
      10080,
      "Approaching Due Warning (Minutes)",
      "workspace",
    );
    if (validTodoWarningMinutes === null) return null;

    return {
      target_host: trimmedTargetHost,
      target_port: validTargetPort,
      server_mode: serverMode,
      server_port: validServerPort,
      server_bind: trimmedServerBind,
      admin_token: adminToken.trim() || null,
      theme,
      retention_days: validRetentionDays,
      max_log_entries: validMaxLogEntries,
      todo_warning_minutes: validTodoWarningMinutes,
      workspace_dir: workspaceDir.trim() || null,
      sound_server_up: soundServerUp.trim() || null,
      sound_server_down: soundServerDown.trim() || null,
      sound_notification: soundNotification.trim() || null,
    };
  }

  async function handleSave() {
    error = null;
    successMsg = null;

    const payload = validatePayload();
    if (!payload) return;

    saving = true;
    try {
      await invoke<AppSettings>("update_app_settings_cmd", {
        settings: payload,
      });

      const refreshResults = await Promise.allSettled([
        serverStore.fetchServers(),
        printerStore.fetchPrinters(),
        todoStore.refreshSettings(),
        notesStore.fetchNotes(),
        scriptsStore.fetchScripts(),
        soundManager.refreshFromSettings(payload),
        refreshClientMode(),
      ]);

      const rejected = refreshResults.filter(
        (result): result is PromiseRejectedResult => result.status === "rejected",
      );
      if (rejected.length > 0) {
        throw new Error(
          rejected.map((result) => toErrorMessage(result.reason)).join("; "),
        );
      }

      persistedTheme = theme;
      applyTheme(theme);
      successMsg = "Settings saved successfully";
      setTimeout(() => {
        open = false;
      }, 600);
    } catch (err) {
      error = toErrorMessage(err);
    } finally {
      saving = false;
    }
  }
</script>

<Modal bind:open maxWidth="40rem" ariaLabel="Application Settings" onBackdropClose={handleClose}>
  <div class="modal-header">
    <div class="header-main">
      <h2 class="modal-title">Application Settings</h2>
      <div class="tab-nav" role="tablist" aria-label="Settings categories">
        {#each SETTINGS_TABS as tab (tab.id)}
          <button
            type="button"
            id={`settings-tab-${tab.id}`}
            role="tab"
            class="tab-btn"
            class:active={activeTab === tab.id}
            aria-selected={activeTab === tab.id}
            aria-controls={`settings-panel-${tab.id}`}
            onclick={() => (activeTab = tab.id)}
          >
            {tab.label}
          </button>
        {/each}
      </div>
    </div>
    <button
      type="button"
      class="btn-close"
      onclick={handleClose}
      aria-label="Close settings"
    >
      ✕
    </button>
  </div>

  {#if loading}
    <div class="settings-loading">Loading settings...</div>
  {:else}
    <form
      class="settings-form"
      novalidate
      onsubmit={(e) => {
        e.preventDefault();
        handleSave();
      }}
    >
      {#if error}
        <div class="error-banner">{error}</div>
      {/if}
      {#if successMsg}
        <div class="alert-success">{successMsg}</div>
      {/if}

      <div
        id={`settings-panel-${activeTab}`}
        role="tabpanel"
        aria-labelledby={`settings-tab-${activeTab}`}
        class="modal-body settings-body"
      >
        {#if activeTab === "appearance"}
          <AppearanceTab bind:theme />
        {:else if activeTab === "log-retention"}
          <LogRetentionTab bind:retentionDays bind:maxLogEntries />
        {:else if activeTab === "network"}
          <NetworkTab
            bind:targetHost
            bind:targetPort
            bind:adminToken
            bind:serverMode
            bind:serverBind
            bind:serverPort
            bind:serverTokens
          />
        {:else if activeTab === "sound-effects"}
          <SoundEffectsTab
            bind:soundServerUp
            bind:soundServerDown
            bind:soundNotification
          />
        {:else if activeTab === "workspace"}
          <WorkspaceTab bind:todoWarningMinutes bind:workspaceDir />
        {/if}
      </div>

      <div class="modal-footer">
        <div class="footer-actions">
          <button
            type="button"
            class="btn-cancel"
            onclick={handleClose}
            disabled={saving}
          >
            Cancel
          </button>
          <button type="submit" class="btn-save" disabled={saving}>
            {saving ? "Saving..." : "Save Settings"}
          </button>
        </div>
      </div>
    </form>
  {/if}
</Modal>

<style>
  .settings-loading {
    padding: 2rem;
    text-align: center;
    color: var(--color-text-muted);
  }

  .settings-form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .settings-body {
    min-height: 14rem;
    padding-top: 0.15rem;
  }

  .settings-body :global(.settings-description) {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-text-muted);
    line-height: 1.4;
  }

  .settings-body :global(.settings-description code) {
    background-color: var(--color-surface-active);
    padding: 2px 4px;
    border-radius: 4px;
  }

  .alert-success {
    padding: 0.6rem 0.85rem;
    border-radius: 0.375rem;
    font-size: 0.85rem;
    background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
    color: var(--color-success);
    border: solid var(--width-border) var(--color-success);
  }

  .btn-save:disabled,
  .btn-cancel:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
