<script lang="ts">
  let {
    todoWarningMinutes = $bindable(120),
    workspaceDir = $bindable(""),
  }: {
    todoWarningMinutes: number;
    workspaceDir: string;
  } = $props();
</script>

<p class="settings-description">
  Configure where your personal Notes and Scripts are stored, as well as task reminder thresholds.
</p>

<label class="form-field" for="workspaceDir">
  <span>Workspace Directory</span>
  <input
    id="workspaceDir"
    type="text"
    bind:value={workspaceDir}
    placeholder="Default (~/Caffeine)"
  />
  <span class="field-hint">
    Leave blank to use your local home directory (~/Caffeine). Changing this path automatically moves existing Notes and Scripts from the previous directory.
  </span>
</label>

<label class="form-field" for="todoWarningMinutes">
  <span>Approaching Due Warning (Minutes)</span>
  <input
    id="todoWarningMinutes"
    type="number"
    bind:value={todoWarningMinutes}
    min="5"
    max="10080"
  />
  <span class="field-hint">
    Active tasks due within this window display a warning status (default: 120 minutes / 2 hours).
  </span>
</label>
