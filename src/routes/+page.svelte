<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  interface DownloadProgress {
    downloaded: number;
    total: number;
    percentage: number;
  }

  interface ModelInfo {
    name: string;
    filename: string;
    size_hint: string;
  }

  interface WhisperSettings {
    model_name: string;
  }

  interface Settings {
    whisper: WhisperSettings;
    is_saved: boolean;
  }

  let availableModels = $state<ModelInfo[]>([]);
  let selectedModel = $state("large-v3-turbo");
  let isModelInitialized = $state(false);
  let isRecording = $state(false);
  let isTranscribing = $state(false);
  let isDownloading = $state(false);
  let downloadProgress = $state<DownloadProgress | null>(null);
  let transcriptionResult = $state("");
  let statusMessage = $state("モデルを選択して初期化してください");
  let errorMessage = $state("");

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  async function loadModels() {
    try {
      availableModels = await invoke("get_available_models");
    } catch (error) {
      console.error("Failed to load models:", error);
    }
  }

  async function loadSettings(): Promise<boolean> {
    try {
      const settings: Settings = await invoke("get_settings");
      selectedModel = settings.whisper.model_name;
      console.log("Loaded settings, model:", selectedModel, "is_saved:", settings.is_saved);
      return settings.is_saved;
    } catch (error) {
      console.error("Failed to load settings:", error);
      return false;
    }
  }

  async function saveModelSelection(modelName: string) {
    try {
      await invoke("save_model_selection", { modelName });
      console.log("Saved model selection:", modelName);
    } catch (error) {
      console.error("Failed to save model selection:", error);
    }
  }

  async function initializeWhisper() {
    try {
      errorMessage = "";
      statusMessage = "モデルを確認中...";

      await invoke("initialize_whisper", { modelName: selectedModel });
      isModelInitialized = true;
      isDownloading = false;
      downloadProgress = null;
      statusMessage = "準備完了 - Ctrl+Space で録音開始/停止";

      // Save the selected model for next time
      await saveModelSelection(selectedModel);
    } catch (error) {
      errorMessage = `モデル初期化エラー: ${error}`;
      statusMessage = "エラー";
      isDownloading = false;
      downloadProgress = null;
      console.error(error);
    }
  }

  async function toggleRecording() {
    try {
      errorMessage = "";
      const result = await invoke("toggle_recording");
      console.log(result);
    } catch (error) {
      errorMessage = `録音エラー: ${error}`;
      console.error(error);
    }
  }

  onMount(() => {
    // Load available models first, then settings, then auto-initialize if saved
    (async () => {
      await loadModels();
      const hasSavedSettings = await loadSettings();
      if (hasSavedSettings) {
        console.log("Auto-initializing saved model:", selectedModel);
        await initializeWhisper();
      }
    })();

    // Listen for download events
    const unlistenDownloadStarted = listen("download-started", () => {
      isDownloading = true;
      downloadProgress = null;
      statusMessage = "モデルをダウンロード中...";
      console.log("Download started");
    });

    const unlistenDownloadProgress = listen<DownloadProgress>(
      "download-progress",
      (event) => {
        downloadProgress = event.payload;
        statusMessage = `ダウンロード中... ${event.payload.percentage.toFixed(1)}%`;
      }
    );

    const unlistenDownloadComplete = listen("download-complete", () => {
      isDownloading = false;
      statusMessage = "ダウンロード完了 - モデルを読み込み中...";
      console.log("Download complete");
    });

    // Listen for recording events
    const unlistenRecordingStarted = listen("recording-started", () => {
      isRecording = true;
      statusMessage = "録音中...";
      console.log("Recording started");
    });

    const unlistenRecordingStopped = listen("recording-stopped", () => {
      isRecording = false;
      statusMessage = "録音停止 - 音声認識中...";
      console.log("Recording stopped");
    });

    const unlistenTranscriptionStarted = listen("transcription-started", () => {
      isTranscribing = true;
      statusMessage = "音声認識中...";
      console.log("Transcription started");
    });

    const unlistenTranscriptionComplete = listen<string>(
      "transcription-complete",
      (event) => {
        isTranscribing = false;
        transcriptionResult = event.payload;
        statusMessage = "認識完了 - クリップボードにコピーしました";
        console.log("Transcription complete:", event.payload);
      }
    );

    const unlistenRecordingToggle = listen("recording-toggle", () => {
      console.log("Recording toggle event received");
    });

    // Cleanup listeners on unmount
    return () => {
      unlistenDownloadStarted.then((fn) => fn());
      unlistenDownloadProgress.then((fn) => fn());
      unlistenDownloadComplete.then((fn) => fn());
      unlistenRecordingStarted.then((fn) => fn());
      unlistenRecordingStopped.then((fn) => fn());
      unlistenTranscriptionStarted.then((fn) => fn());
      unlistenTranscriptionComplete.then((fn) => fn());
      unlistenRecordingToggle.then((fn) => fn());
    };
  });
</script>

<main class="container">
  <h1>VoiceInput</h1>
  <p class="subtitle">ローカル音声入力アプリ</p>

  <div class="section">
    <h2>モデル設定</h2>
    <div class="model-setup">
      <select
        bind:value={selectedModel}
        disabled={isModelInitialized || isDownloading}
        class="model-select"
      >
        {#each availableModels as model}
          <option value={model.name}>
            {model.name} ({model.size_hint})
          </option>
        {/each}
      </select>
      <button
        onclick={initializeWhisper}
        disabled={isModelInitialized || isDownloading}
        class="init-button"
      >
        {#if isDownloading}
          ダウンロード中...
        {:else if isModelInitialized}
          初期化済み
        {:else}
          モデルを読み込む
        {/if}
      </button>
    </div>

    {#if isDownloading && downloadProgress}
      <div class="download-progress">
        <div class="progress-bar">
          <div
            class="progress-fill"
            style="width: {downloadProgress.percentage}%"
          ></div>
        </div>
        <p class="progress-text">
          {formatBytes(downloadProgress.downloaded)} / {formatBytes(downloadProgress.total)}
          ({downloadProgress.percentage.toFixed(1)}%)
        </p>
      </div>
    {/if}

    <p class="model-hint">
      モデルが存在しない場合は自動的にダウンロードされます
    </p>
  </div>

  <div class="section">
    <h2>録音</h2>
    <div class="recording-controls">
      <button
        onclick={toggleRecording}
        disabled={!isModelInitialized}
        class="record-button"
        class:recording={isRecording}
      >
        {#if isRecording}
          <span class="pulse">●</span> 録音中...
        {:else}
          ● 録音開始
        {/if}
      </button>
      <p class="shortcut-hint">グローバルショートカット: <kbd>Ctrl+Space</kbd></p>
    </div>
  </div>

  <div class="section">
    <h2>ステータス</h2>
    <div class="status-display">
      <p class="status" class:processing={isRecording || isTranscribing || isDownloading}>
        {statusMessage}
      </p>
      {#if errorMessage}
        <p class="error">{errorMessage}</p>
      {/if}
    </div>
  </div>

  <div class="section">
    <h2>認識結果</h2>
    <div class="result-display">
      {#if transcriptionResult}
        <p class="result">{transcriptionResult}</p>
      {:else}
        <p class="placeholder">認識結果がここに表示されます</p>
      {/if}
    </div>
  </div>
</main>

<style>
  :root {
    font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
  }

  h1 {
    text-align: center;
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
  }

  .subtitle {
    text-align: center;
    color: #666;
    margin-bottom: 2rem;
  }

  .section {
    background: white;
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  h2 {
    font-size: 1.2rem;
    margin-top: 0;
    margin-bottom: 1rem;
    color: #333;
  }

  .model-setup {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .model-select {
    flex: 1;
    padding: 0.75rem;
    border: 2px solid #ddd;
    border-radius: 8px;
    font-size: 0.9rem;
    background-color: white;
    cursor: pointer;
  }

  .model-select:focus {
    outline: none;
    border-color: #396cd8;
  }

  .model-select:disabled {
    background-color: #f0f0f0;
    color: #999;
    cursor: not-allowed;
  }

  .model-hint {
    margin-top: 0.75rem;
    font-size: 0.85rem;
    color: #666;
  }

  .download-progress {
    margin-top: 1rem;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background-color: #e0e0e0;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background-color: #396cd8;
    transition: width 0.3s ease;
  }

  .progress-text {
    margin-top: 0.5rem;
    font-size: 0.85rem;
    color: #666;
    text-align: center;
  }

  button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .init-button {
    background-color: #396cd8;
    color: white;
    min-width: 160px;
  }

  .init-button:hover:not(:disabled) {
    background-color: #2d5ab8;
  }

  .recording-controls {
    text-align: center;
  }

  .record-button {
    background-color: #4caf50;
    color: white;
    font-size: 1.2rem;
    padding: 1rem 2rem;
    min-width: 200px;
  }

  .record-button:hover:not(:disabled) {
    background-color: #45a049;
  }

  .record-button.recording {
    background-color: #f44336;
  }

  .record-button.recording:hover {
    background-color: #da190b;
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

  .shortcut-hint {
    margin-top: 1rem;
    color: #666;
    font-size: 0.9rem;
  }

  kbd {
    background-color: #f4f4f4;
    border: 1px solid #ccc;
    border-radius: 4px;
    padding: 0.2rem 0.5rem;
    font-family: monospace;
    font-size: 0.85rem;
  }

  .status-display {
    text-align: center;
  }

  .status {
    font-size: 1.1rem;
    color: #333;
    margin: 0;
  }

  .status.processing {
    color: #396cd8;
    font-weight: 600;
  }

  .error {
    color: #f44336;
    margin-top: 0.5rem;
    font-weight: 600;
  }

  .result-display {
    min-height: 100px;
    padding: 1rem;
    background-color: #f9f9f9;
    border-radius: 8px;
    border: 2px solid #e0e0e0;
  }

  .result {
    font-size: 1.1rem;
    line-height: 1.6;
    color: #333;
    margin: 0;
    white-space: pre-wrap;
  }

  .placeholder {
    color: #999;
    font-style: italic;
    text-align: center;
    margin: 2rem 0;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #1a1a1a;
    }

    .section {
      background: #2a2a2a;
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    }

    h2 {
      color: #f6f6f6;
    }

    .subtitle {
      color: #aaa;
    }

    .model-select {
      background-color: #1a1a1a;
      color: #f6f6f6;
      border-color: #444;
    }

    .model-select:disabled {
      background-color: #333;
      color: #666;
    }

    .model-hint {
      color: #888;
    }

    .progress-bar {
      background-color: #444;
    }

    .progress-text {
      color: #aaa;
    }

    .result-display {
      background-color: #1a1a1a;
      border-color: #444;
    }

    .result {
      color: #f6f6f6;
    }

    kbd {
      background-color: #333;
      border-color: #555;
      color: #f6f6f6;
    }
  }
</style>
