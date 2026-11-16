<script lang="ts">
  let {
    retentionDays = $bindable(14),
    maxLogEntries = $bindable(1000),
  }: {
    retentionDays: number;
    maxLogEntries: number;
  } = $props();
</script>

<p class="settings-description">
  Controls how much event history is kept in the local database replica.
</p>

<div class="settings-row">
  <label class="form-field" for="retentionDays">
    <span>Retention (Days)</span>
    <input
      id="retentionDays"
      type="number"
      bind:value={retentionDays}
      min="1"
      max="3650"
    />
  </label>

  <label class="form-field" for="maxLogEntries">
    <span>Max Entries Per Entity</span>
    <input
      id="maxLogEntries"
      type="number"
      bind:value={maxLogEntries}
      min="50"
      max="100000"
    />
  </label>
</div>
