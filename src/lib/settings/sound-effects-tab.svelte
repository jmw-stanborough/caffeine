<script lang="ts">
  import SoundSettingInput from "$lib/components/sound-setting-input.svelte";
  import { DEFAULT_SOUND_PATHS } from "$lib/sound.svelte";

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
  placeholder={`Default (${DEFAULT_SOUND_PATHS.server_up})`}
  bind:value={soundServerUp}
/>

<SoundSettingInput
  id="soundServerDown"
  label="Server Offline (Down) Sound"
  placeholder={`Default (${DEFAULT_SOUND_PATHS.server_down})`}
  bind:value={soundServerDown}
/>

<SoundSettingInput
  id="soundNotification"
  label="Task Reminder / Notification Sound"
  placeholder={`Default (${DEFAULT_SOUND_PATHS.notification})`}
  bind:value={soundNotification}
/>
