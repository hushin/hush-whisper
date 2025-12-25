<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { outputModeDescriptions } from "$lib/types";
</script>

<div class="section">
  <h2>出力設定</h2>
  <div class="input-group">
    <label for="output-mode">出力モード</label>
    <select
      id="output-mode"
      bind:value={settingsStore.outputMode}
      onchange={() => settingsStore.saveOutputMode()}
      class="output-select"
    >
      <option value="ClipboardOnly">{outputModeDescriptions.ClipboardOnly}</option>
      <option value="DirectInput">{outputModeDescriptions.DirectInput}</option>
      <option value="Both">{outputModeDescriptions.Both}</option>
    </select>
  </div>
  <p class="output-hint">
    直接入力 は認識結果を直接アクティブなアプリケーションに入力します
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

  .input-group {
    margin-bottom: 1rem;
  }

  .input-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-size: 0.95rem;
    font-weight: 600;
    color: #333;
  }

  .output-select {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.95rem;
    background-color: white;
    cursor: pointer;
  }

  .output-select:focus {
    outline: 2px solid #396cd8;
    outline-offset: 2px;
  }

  .output-hint {
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

    .input-group label {
      color: #f6f6f6;
    }

    .output-select {
      background-color: #1a1a1a;
      color: #f6f6f6;
      border-color: #444;
    }

    .output-hint {
      color: #aaa;
    }
  }
</style>
