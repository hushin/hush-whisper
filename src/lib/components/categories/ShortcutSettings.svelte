<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
</script>

<div class="section">
  <h2>ショートカット設定</h2>
  <div class="shortcut-setting">
    <span class="shortcut-label">録音開始/停止: </span>
    {#if settingsStore.isEditingShortcut}
      <input
        type="text"
        readonly
        value={settingsStore.pendingShortcut || "キーを押してください..."}
        class="shortcut-input"
        class:placeholder={!settingsStore.pendingShortcut}
        onkeydown={(e) => settingsStore.handleShortcutKeyDown(e)}
      />
      <button
        class="shortcut-save-button"
        onclick={() => settingsStore.saveShortcut()}
        disabled={!settingsStore.pendingShortcut}
      >
        保存
      </button>
      <button class="shortcut-cancel-button" onclick={() => settingsStore.cancelShortcutEdit()}>
        キャンセル
      </button>
    {:else}
      <kbd>{settingsStore.shortcutKey}</kbd>
      <button class="shortcut-edit-button" onclick={() => settingsStore.startShortcutEdit()}>
        変更
      </button>
    {/if}
  </div>
  {#if settingsStore.shortcutError}
    <p class="shortcut-error">{settingsStore.shortcutError}</p>
  {/if}
  {#if settingsStore.shortcutChanged}
    <p class="shortcut-notice">ショートカットを変更しました。</p>
  {/if}
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

  .shortcut-setting {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .shortcut-label {
    font-weight: 600;
    color: #333;
  }

  kbd {
    display: inline-block;
    padding: 0.4rem 0.8rem;
    background-color: #f6f6f6;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-family: "Consolas", "Monaco", monospace;
    font-size: 0.9rem;
    font-weight: 600;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.1);
  }

  .shortcut-input {
    padding: 0.4rem 0.8rem;
    border: 1px solid #396cd8;
    border-radius: 4px;
    font-family: "Consolas", "Monaco", monospace;
    font-size: 0.9rem;
    min-width: 200px;
    background-color: white;
  }

  .shortcut-input.placeholder {
    color: #999;
    font-style: italic;
  }

  .shortcut-edit-button,
  .shortcut-save-button,
  .shortcut-cancel-button {
    padding: 0.4rem 1rem;
    border: none;
    border-radius: 4px;
    font-size: 0.9rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .shortcut-edit-button {
    background-color: #396cd8;
    color: white;
  }

  .shortcut-edit-button:hover {
    background-color: #2854b8;
  }

  .shortcut-save-button {
    background-color: #4caf50;
    color: white;
  }

  .shortcut-save-button:hover:not(:disabled) {
    background-color: #45a049;
  }

  .shortcut-save-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }

  .shortcut-cancel-button {
    background-color: #f0f0f0;
    color: #333;
  }

  .shortcut-cancel-button:hover {
    background-color: #e0e0e0;
  }

  .shortcut-error {
    color: #f44336;
    font-size: 0.9rem;
    margin: 0.5rem 0 0 0;
  }

  .shortcut-notice {
    color: #4caf50;
    font-size: 0.9rem;
    margin: 0.5rem 0 0 0;
  }

  @media (prefers-color-scheme: dark) {
    .section {
      background-color: #2a2a2a;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    }

    h2 {
      color: #f6f6f6;
    }

    .shortcut-label {
      color: #f6f6f6;
    }

    kbd {
      background-color: #333;
      border-color: #555;
      color: #f6f6f6;
    }

    .shortcut-input {
      background-color: #1a1a1a;
      color: #f6f6f6;
      border-color: #90caf9;
    }

    .shortcut-edit-button {
      background-color: #90caf9;
      color: #000;
    }

    .shortcut-edit-button:hover {
      background-color: #64b5f6;
    }

    .shortcut-save-button {
      background-color: #66bb6a;
    }

    .shortcut-save-button:hover:not(:disabled) {
      background-color: #4caf50;
    }

    .shortcut-save-button:disabled {
      background-color: #666;
      color: #999;
    }

    .shortcut-cancel-button {
      background-color: #333;
      color: #f6f6f6;
    }

    .shortcut-cancel-button:hover {
      background-color: #444;
    }
  }
</style>
