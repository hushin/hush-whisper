<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
</script>

<div class="section">
  <h2>環境設定</h2>
  <div class="autostart-toggle">
    <label class="switch">
      <input
        type="checkbox"
        checked={settingsStore.autostartEnabled}
        onchange={() => settingsStore.toggleAutostart()}
        disabled={settingsStore.isLoadingAutostart}
      />
      <span class="slider"></span>
    </label>
    <span class="toggle-label">Windows起動時に自動的にアプリを起動する</span>
  </div>
  <p class="autostart-hint">
    有効にすると、Windows起動時に自動的にアプリが起動します
  </p>
</div>

<style>
  /* コンポーネント固有のスタイル */
  .autostart-toggle {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
  }

  input:disabled + .slider {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
