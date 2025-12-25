<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
</script>

<div class="section">
  <h2>環境設定</h2>
  <div class="autostart-toggle">
    <label class="switch">
      <input
        type="checkbox"
        bind:checked={settingsStore.autostartEnabled}
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
  .section {
    background-color: white;
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  h2 {
    margin: 0 0 1rem 0;
    color: #333;
    font-size: 1.25rem;
  }

  .autostart-toggle {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
  }

  .toggle-label {
    font-size: 0.95rem;
    color: #333;
  }

  .switch {
    position: relative;
    display: inline-block;
    width: 50px;
    height: 24px;
  }

  .switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: #ccc;
    transition: 0.3s;
    border-radius: 24px;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }

  input:checked + .slider {
    background-color: #396cd8;
  }

  input:checked + .slider:before {
    transform: translateX(26px);
  }

  input:disabled + .slider {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .autostart-hint {
    font-size: 0.85rem;
    color: #666;
    margin: 0;
  }

  @media (prefers-color-scheme: dark) {
    .section {
      background-color: #2a2a2a;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    }

    h2 {
      color: #f6f6f6;
    }

    .toggle-label {
      color: #f6f6f6;
    }

    .slider {
      background-color: #666;
    }

    input:checked + .slider {
      background-color: #90caf9;
    }

    .autostart-hint {
      color: #aaa;
    }
  }
</style>
