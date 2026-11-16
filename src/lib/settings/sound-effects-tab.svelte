<script lang="ts">
  import SoundSettingInput from "$lib/components/sound-setting-input.svelte";

  let {
    soundServerUp = $bindable(""),
    soundServerDown = $bindable(""),
    soundNotification = $bindable(""),
  }: {
    soundServerUp: string;
    soundServerDown: string;
    soundNotification: string;
  } = $props();
</script>

<p class="settings-description">
  Leave a field blank to use the built-in default sound, or enter a path to a custom
  <code>.mp3</code> file (falls back to the default sound if the file cannot be located).
</p>

<SoundSettingInput
  id="soundServerUp"
  label="Server Online (Up) Sound"
  bind:value={soundServerUp}
/>

<SoundSettingInput
  id="soundServerDown"
  label="Server Offline (Down) Sound"
  bind:value={soundServerDown}
/>

<SoundSettingInput
  id="soundNotification"
  label="Task Reminder / Notification Sound"
  bind:value={soundNotification}
/>
