<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
</script>

<div class="section">
  <h2>録音</h2>
  <div class="recording-controls">
    <button
      onclick={() => settingsStore.toggleRecording()}
      disabled={!settingsStore.isModelInitialized}
      class="record-button"
      class:recording={settingsStore.isRecording}
    >
      {#if settingsStore.isRecording}
        <span class="pulse">●</span> 録音中...
      {:else}
        ● 録音開始
      {/if}
    </button>
  </div>
</div>

<div class="section">
  <h2>録音設定</h2>

  <div class="whisper-toggle">
    <label class="switch">
      <input
        type="checkbox"
        bind:checked={settingsStore.insertNewline}
        onchange={() => settingsStore.saveInsertNewline()}
      />
      <span class="slider"></span>
    </label>
    <span class="toggle-label">セグメント間に改行を入れる</span>
  </div>

  <div class="max-recording-setting">
    <label for="max-recording">最大録音時間</label>
    <div class="max-recording-input-row">
      <select
        id="max-recording"
        bind:value={settingsStore.maxRecordingSeconds}
        onchange={() => settingsStore.saveMaxRecordingSeconds()}
        class="max-recording-select"
      >
        <option value={60}>1分</option>
        <option value={120}>2分</option>
        <option value={180}>3分</option>
        <option value={300}>5分 (デフォルト)</option>
        <option value={600}>10分</option>
        <option value={900}>15分</option>
        <option value={0}>無制限</option>
      </select>
    </div>
    <p class="max-recording-hint">
      設定時間を超えると自動的に録音を停止します
    </p>
  </div>
</div>

<style>
  /* コンポーネント固有のスタイル */
  .recording-controls {
    display: flex;
    justify-content: center;
  }

  .record-button {
    padding: 1rem 2rem;
    font-size: 1.125rem;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .record-button:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 6px 12px rgba(0, 0, 0, 0.15);
  }

  .record-button:disabled {
    background: linear-gradient(135deg, #ccc 0%, #999 100%);
    cursor: not-allowed;
    opacity: 0.6;
  }

  .record-button.recording {
    background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  }

  .pulse {
    display: inline-block;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.3;
    }
  }

  .whisper-toggle {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .max-recording-setting {
    margin-top: 1.5rem;
  }

  .max-recording-setting label {
    display: block;
    margin-bottom: 0.5rem;
    font-size: 0.95rem;
    font-weight: 600;
    color: #333;
  }

  .max-recording-input-row {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .max-recording-select {
    flex: 1;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.95rem;
    background-color: white;
    cursor: pointer;
  }

  .max-recording-hint {
    margin: 0.5rem 0 0 0;
    font-size: 0.85rem;
    color: #666;
  }

  @media (prefers-color-scheme: dark) {
    .max-recording-setting label {
      color: #f6f6f6;
    }

    .max-recording-select {
      background-color: #1a1a1a;
      color: #f6f6f6;
      border-color: #444;
    }

    .max-recording-hint {
      color: #aaa;
    }
  }
</style>
