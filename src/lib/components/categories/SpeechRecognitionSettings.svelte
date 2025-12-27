<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
</script>

<div class="app-header">
  <h1>VoiceInput</h1>
  <p class="app-subtitle">ローカル音声入力アプリ</p>
</div>

<div class="section">
  <h2>モデル設定</h2>
  <div class="model-setup">
    <select
      bind:value={settingsStore.selectedModel}
      disabled={settingsStore.isDownloading}
      class="model-select"
    >
      {#each settingsStore.availableModels as model}
        <option value={model.name}>
          {model.name} ({model.size_hint})
        </option>
      {/each}
    </select>
    <button
      onclick={() => settingsStore.initializeWhisper()}
      disabled={settingsStore.isDownloading ||
        (settingsStore.isModelInitialized &&
          settingsStore.selectedModel === settingsStore.currentLoadedModel)}
      class="init-button"
    >
      {#if settingsStore.isDownloading}
        ダウンロード中...
      {:else if settingsStore.isModelInitialized && settingsStore.selectedModel === settingsStore.currentLoadedModel}
        読み込み済み
      {:else if settingsStore.isModelInitialized}
        モデルを切り替える
      {:else}
        モデルを読み込む
      {/if}
    </button>
  </div>

  {#if settingsStore.isDownloading && settingsStore.downloadProgress}
    <div class="download-progress">
      <div class="progress-bar">
        <div
          class="progress-fill"
          style="width: {settingsStore.downloadProgress.percentage}%"
        ></div>
      </div>
      <p class="progress-text">
        {settingsStore.formatBytes(settingsStore.downloadProgress.downloaded)} /
        {settingsStore.formatBytes(settingsStore.downloadProgress.total)}
        ({settingsStore.downloadProgress.percentage.toFixed(1)}%)
      </p>
    </div>
  {/if}

  <p class="model-hint">
    モデルが存在しない場合は自動的にダウンロードされます
  </p>
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
  .app-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .app-header h1 {
    font-size: 2.5rem;
    margin: 0 0 0.5rem 0;
    color: #333;
  }

  .app-subtitle {
    color: #666;
    margin: 0 0 0 0;
    font-size: 1.1rem;
  }

  .model-setup {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .model-select {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.95rem;
    background-color: white;
    cursor: pointer;
  }

  .model-select:disabled {
    background-color: #f0f0f0;
    cursor: not-allowed;
  }

  .init-button {
    padding: 0.75rem 1.5rem;
    background-color: #396cd8;
    color: white;
    border: none;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
    white-space: nowrap;
  }

  .init-button:hover:not(:disabled) {
    background-color: #2854b8;
  }

  .init-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }

  .download-progress {
    margin: 1rem 0;
  }

  .progress-bar {
    width: 100%;
    height: 24px;
    background-color: #f0f0f0;
    border-radius: 12px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #396cd8 0%, #667eea 100%);
    transition: width 0.3s ease;
  }

  .progress-text {
    text-align: center;
    font-size: 0.9rem;
    color: #666;
    margin: 0;
  }

  .model-hint {
    font-size: 0.85rem;
    color: #666;
    margin: 0.5rem 0 0 0;
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
    .app-header h1 {
      color: #f6f6f6;
    }

    .app-subtitle {
      color: #aaa;
    }

    .model-select {
      background-color: #1a1a1a;
      color: #f6f6f6;
      border-color: #444;
    }

    .model-select:disabled {
      background-color: #333;
    }

    .init-button {
      background-color: #90caf9;
      color: #000;
    }

    .init-button:hover:not(:disabled) {
      background-color: #64b5f6;
    }

    .init-button:disabled {
      background-color: #666;
      color: #999;
    }

    .progress-bar {
      background-color: #333;
    }

    .progress-text {
      color: #aaa;
    }

    .model-hint {
      color: #aaa;
    }

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
