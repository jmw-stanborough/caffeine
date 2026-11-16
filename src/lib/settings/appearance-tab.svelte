<script lang="ts">
  import { applyTheme } from "$lib/theme";

  let { theme = $bindable("system") }: { theme: string } = $props();
</script>

<p class="settings-description">
  Choose how Caffeine adapts to your light or dark color scheme preference.
</p>

<label class="form-field" for="themeSelect">
  <span>Theme</span>
  <select
    id="themeSelect"
    bind:value={theme}
    onchange={() => applyTheme(theme)}
  >
    <option value="system">System Default</option>
    <option value="dark">Dark Theme</option>
    <option value="light">Light Theme</option>
  </select>
</label>
