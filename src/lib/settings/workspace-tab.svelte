<script lang="ts">
  let {
    todoWarningMinutes = $bindable(120),
  }: {
    todoWarningMinutes: number;
  } = $props();
</script>

<p class="settings-description">
  Configure task reminders and workspace attention thresholds.
</p>

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
