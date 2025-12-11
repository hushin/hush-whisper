<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let modelPath = $state("%APPDATA%\\voice-input\\models\\ggml-large-v3-turbo.bin");
  let isModelInitialized = $state(false);
  let isRecording = $state(false);
  let isTranscribing = $state(false);
  let transcriptionResult = $state("");
  let statusMessage = $state("モデルを初期化してください");
  let errorMessage = $state("");

  async function initializeWhisper() {
    try {
      errorMessage = "";
      statusMessage = "モデルを読み込み中...";

      // Pass the path as-is to Rust backend (environment variables will be expanded there if needed)
      await invoke("initialize_whisper", { modelPath });
      isModelInitialized = true;
      statusMessage = "準備完了 - Ctrl+Space で録音開始/停止";
    } catch (error) {
      errorMessage = `モデル初期化エラー: ${error}`;
      statusMessage = "エラー";
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
      unlistenRecordingStarted.then((fn) => fn());
      unlistenRecordingStopped.then((fn) => fn());
      unlistenTranscriptionStarted.then((fn) => fn());
      unlistenTranscriptionComplete.then((fn) => fn());
      unlistenRecordingToggle.then((fn) => fn());
    };
  });
</script>

<main class="container">
  <h1>🎤 VoiceInput</h1>
  <p class="subtitle">ローカル音声入力アプリ</p>

  <div class="section">
    <h2>モデル設定</h2>
    <div class="model-setup">
      <input
        type="text"
        bind:value={modelPath}
        placeholder="Whisperモデルのパス"
        disabled={isModelInitialized}
        class="model-path-input"
      />
      <button
        onclick={initializeWhisper}
        disabled={isModelInitialized}
        class="init-button"
      >
        {isModelInitialized ? "✓ 初期化済み" : "モデルを読み込む"}
      </button>
    </div>
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
      <p class="status" class:processing={isRecording || isTranscribing}>
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

  .model-path-input {
    flex: 1;
    padding: 0.75rem;
    border: 2px solid #ddd;
    border-radius: 8px;
    font-size: 0.9rem;
    font-family: "Consolas", monospace;
  }

  .model-path-input:focus {
    outline: none;
    border-color: #396cd8;
  }

  .model-path-input:disabled {
    background-color: #f0f0f0;
    color: #999;
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

    .model-path-input {
      background-color: #1a1a1a;
      color: #f6f6f6;
      border-color: #444;
    }

    .model-path-input:disabled {
      background-color: #333;
      color: #666;
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
