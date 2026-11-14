<script lang="ts">
  import { AlertTriangle } from "$lib/components/icons";
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    id: string;
    label: string;
    placeholder: string;
    value: string;
  }

  let { id, label, placeholder, value = $bindable("") }: Props = $props();

  let isMissing = $state(false);

  async function validateSoundPath(path: string): Promise<boolean> {
    const trimmed = path.trim();
    if (trimmed === "") return false;
    return !(await invoke<boolean>("check_sound_file_cmd", { path: trimmed }));
  }

  async function checkValidity() {
    const current = value;
    const missing = await validateSoundPath(current);
    // Ignore stale async checks if the user edited the input while validation was in flight.
    if (value === current) {
      isMissing = missing;
    }
  }

  $effect(() => {
    void checkValidity();
  });
</script>

<div class="form-group">
  <label for={id}>{label}</label>
  <div class="sound-input-row">
    <input
      {id}
      type="text"
      bind:value
      {placeholder}
      oninput={checkValidity}
      onchange={checkValidity}
    />
    {#if isMissing}
      <span
        class="sound-warning-icon"
        title="File cannot be found at this path. The default sound will be used in its stead."
        aria-label="Custom sound file not found"
      >
        <AlertTriangle size={18} />
      </span>
    {/if}
  </div>
  {#if isMissing}
    <span class="sound-warning-text">
      File could not be found at this path. The default sound will be used instead.
    </span>
  {/if}
</div>

<style>
  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 12px;
  }

  label {
    font-size: 0.875rem;
    font-weight: 500;
  }

  .sound-input-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sound-input-row input {
    flex: 1;
    min-width: 0;
  }

  .sound-warning-icon {
    display: inline-flex;
    align-items: center;
    color: var(--color-warning, #f59e0b);
    flex-shrink: 0;
  }

  .sound-warning-text {
    font-size: 0.75rem;
    color: var(--color-warning, #f59e0b);
  }
</style>
