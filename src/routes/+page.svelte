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

  type PromptPreset = "Default" | "Meeting" | "Memo" | "Chat" | "Custom";

  interface LlmSettings {
    enabled: boolean;
    ollama_url: string;
    model_name: string;
    preset: PromptPreset;
    custom_prompt: string;
  }

  interface Settings {
    whisper: WhisperSettings;
    llm: LlmSettings;
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

  // LLM settings
  let llmEnabled = $state(false);
  let llmOllamaUrl = $state("http://localhost:11434");
  let llmModelName = $state("gpt-oss:20b");
  let llmStatus = $state<"unknown" | "connected" | "disconnected">("unknown");
  let isCheckingOllama = $state(false);
  let isLlmRefining = $state(false);

  // Prompt settings
  let promptPreset = $state<PromptPreset>("Default");
  let customPrompt = $state("");
  let presetPrompts = $state<Map<string, string>>(new Map());
  let showPromptEditor = $state(false);

  const presetDescriptions: Record<PromptPreset, string> = {
    Default: "自然な日本語に整形",
    Meeting: "議事録形式で整理",
    Memo: "簡潔なメモに要約",
    Chat: "カジュアルなチャット文",
    Custom: "カスタムプロンプト",
  };

  // Log viewer
  interface LogEntry {
    id: string;
    timestamp: string;
    raw_text: string;
    refined_text: string | null;
    audio_duration_secs: number | null;
    llm_used: boolean;
    prompt_preset: string | null;
  }

  let showLogViewer = $state(false);
  let logEntries = $state<LogEntry[]>([]);
  let isLoadingLogs = $state(false);
  let selectedLogEntry = $state<LogEntry | null>(null);

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
      llmEnabled = settings.llm.enabled;
      llmOllamaUrl = settings.llm.ollama_url;
      llmModelName = settings.llm.model_name;
      promptPreset = settings.llm.preset || "Default";
      customPrompt = settings.llm.custom_prompt || "";
      console.log("Loaded settings, model:", selectedModel, "llm:", settings.llm, "is_saved:", settings.is_saved);
      return settings.is_saved;
    } catch (error) {
      console.error("Failed to load settings:", error);
      return false;
    }
  }

  async function loadPresetPrompts() {
    try {
      const presets: [string, string][] = await invoke("get_preset_prompts");
      presetPrompts = new Map(presets);
      console.log("Loaded preset prompts:", presetPrompts);
    } catch (error) {
      console.error("Failed to load preset prompts:", error);
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

  async function saveLlmSettings() {
    try {
      await invoke("save_llm_settings", {
        enabled: llmEnabled,
        ollamaUrl: llmOllamaUrl,
        modelName: llmModelName,
      });
      console.log("Saved LLM settings");
    } catch (error) {
      console.error("Failed to save LLM settings:", error);
    }
  }

  async function savePromptSettings() {
    try {
      await invoke("save_prompt_settings", {
        preset: promptPreset,
        customPrompt: customPrompt,
      });
      console.log("Saved prompt settings:", promptPreset);
    } catch (error) {
      console.error("Failed to save prompt settings:", error);
    }
  }

  function handlePresetChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    promptPreset = select.value as PromptPreset;
    // If switching to Custom and no custom prompt exists, use current preset's prompt as starting point
    if (promptPreset === "Custom" && !customPrompt) {
      const currentPresetPrompt = presetPrompts.get("Default") || "";
      customPrompt = currentPresetPrompt;
    }
    savePromptSettings();
  }

  function getCurrentPromptPreview(): string {
    if (promptPreset === "Custom") {
      return customPrompt || "(カスタムプロンプト未設定)";
    }
    return presetPrompts.get(promptPreset) || "";
  }

  // Log viewer functions
  async function loadRecentLogs() {
    isLoadingLogs = true;
    try {
      logEntries = await invoke("get_recent_logs", { limit: 50 });
      console.log("Loaded logs:", logEntries.length);
    } catch (error) {
      console.error("Failed to load logs:", error);
    } finally {
      isLoadingLogs = false;
    }
  }

  async function deleteLogEntry(id: string) {
    try {
      const deleted: boolean = await invoke("delete_log_entry", { id });
      if (deleted) {
        logEntries = logEntries.filter((entry) => entry.id !== id);
        if (selectedLogEntry?.id === id) {
          selectedLogEntry = null;
        }
        console.log("Deleted log entry:", id);
      }
    } catch (error) {
      console.error("Failed to delete log entry:", error);
    }
  }

  function formatLogTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleString("ja-JP", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function toggleLogViewer() {
    showLogViewer = !showLogViewer;
    if (showLogViewer && logEntries.length === 0) {
      loadRecentLogs();
    }
  }

  async function checkOllamaConnection() {
    isCheckingOllama = true;
    try {
      const isAvailable: boolean = await invoke("check_ollama_status", {
        ollamaUrl: llmOllamaUrl,
      });
      llmStatus = isAvailable ? "connected" : "disconnected";
      console.log("Ollama status:", llmStatus);
    } catch (error) {
      llmStatus = "disconnected";
      console.error("Failed to check Ollama status:", error);
    } finally {
      isCheckingOllama = false;
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
      await loadPresetPrompts();
      const hasSavedSettings = await loadSettings();
      if (hasSavedSettings) {
        console.log("Auto-initializing saved model:", selectedModel);
        await initializeWhisper();
      }
      // Check Ollama status if LLM is enabled
      if (llmEnabled) {
        await checkOllamaConnection();
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

    // Listen for LLM events
    const unlistenLlmStarted = listen("llm-refinement-started", () => {
      isLlmRefining = true;
      statusMessage = "LLM で整形中...";
      console.log("LLM refinement started");
    });

    const unlistenLlmComplete = listen<string>("llm-refinement-complete", (event) => {
      isLlmRefining = false;
      console.log("LLM refinement complete:", event.payload);
    });

    const unlistenLlmFailed = listen<string>("llm-refinement-failed", (event) => {
      isLlmRefining = false;
      console.warn("LLM refinement failed:", event.payload);
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
      unlistenLlmStarted.then((fn) => fn());
      unlistenLlmComplete.then((fn) => fn());
      unlistenLlmFailed.then((fn) => fn());
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
    <h2>LLM 整形設定 (Ollama)</h2>
    <div class="llm-toggle">
      <label class="switch">
        <input
          type="checkbox"
          bind:checked={llmEnabled}
          onchange={saveLlmSettings}
        />
        <span class="slider"></span>
      </label>
      <span class="toggle-label">LLM による文章整形を有効にする</span>
    </div>

    {#if llmEnabled}
      <div class="llm-settings">
        <div class="input-group">
          <label for="ollama-url">Ollama URL</label>
          <div class="url-input-row">
            <input
              type="text"
              id="ollama-url"
              bind:value={llmOllamaUrl}
              onblur={saveLlmSettings}
              placeholder="http://localhost:11434"
            />
            <button
              class="check-button"
              onclick={checkOllamaConnection}
              disabled={isCheckingOllama}
            >
              {isCheckingOllama ? "確認中..." : "接続確認"}
            </button>
          </div>
          <div class="connection-status" class:connected={llmStatus === "connected"} class:disconnected={llmStatus === "disconnected"}>
            {#if llmStatus === "connected"}
              ✓ 接続済み
            {:else if llmStatus === "disconnected"}
              ✗ 接続できません
            {:else}
              - 未確認
            {/if}
          </div>
        </div>

        <div class="input-group">
          <label for="llm-model">モデル名</label>
          <input
            type="text"
            id="llm-model"
            bind:value={llmModelName}
            onblur={saveLlmSettings}
            placeholder="gpt-oss:20b"
          />
        </div>

        <div class="input-group">
          <label for="prompt-preset">プロンプトプリセット</label>
          <select
            id="prompt-preset"
            value={promptPreset}
            onchange={handlePresetChange}
            class="preset-select"
          >
            <option value="Default">{presetDescriptions.Default}</option>
            <option value="Meeting">{presetDescriptions.Meeting}</option>
            <option value="Memo">{presetDescriptions.Memo}</option>
            <option value="Chat">{presetDescriptions.Chat}</option>
            <option value="Custom">{presetDescriptions.Custom}</option>
          </select>
        </div>

        <div class="prompt-preview-section">
          <div class="prompt-preview-header">
            <span class="preview-label">現在のプロンプト</span>
            <button
              class="toggle-preview-button"
              onclick={() => showPromptEditor = !showPromptEditor}
            >
              {showPromptEditor ? "閉じる" : promptPreset === "Custom" ? "編集" : "プレビュー"}
            </button>
          </div>
          {#if showPromptEditor}
            <div class="prompt-editor">
              {#if promptPreset === "Custom"}
                <textarea
                  bind:value={customPrompt}
                  onblur={savePromptSettings}
                  placeholder={"カスタムプロンプトを入力...\n{input} が音声認識結果に置換されます"}
                  class="prompt-textarea"
                  rows="8"
                ></textarea>
                <p class="prompt-hint">
                  <code>{"{input}"}</code> が音声認識結果に置換されます
                </p>
              {:else}
                <pre class="prompt-preview">{getCurrentPromptPreview()}</pre>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <p class="llm-hint">
      有効にすると、音声認識結果を LLM で整形します。Ollama を事前に起動してください。
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
      <p class="status" class:processing={isRecording || isTranscribing || isDownloading || isLlmRefining}>
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

  <div class="section">
    <div class="section-header">
      <h2>履歴</h2>
      <button class="toggle-logs-button" onclick={toggleLogViewer}>
        {showLogViewer ? "閉じる" : "履歴を表示"}
      </button>
    </div>

    {#if showLogViewer}
      <div class="log-viewer">
        {#if isLoadingLogs}
          <p class="loading">読み込み中...</p>
        {:else if logEntries.length === 0}
          <p class="no-logs">履歴がありません</p>
        {:else}
          <div class="log-list">
            {#each logEntries as entry (entry.id)}
              <div
                class="log-entry"
                class:selected={selectedLogEntry?.id === entry.id}
                onclick={() => selectedLogEntry = selectedLogEntry?.id === entry.id ? null : entry}
                onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectedLogEntry = selectedLogEntry?.id === entry.id ? null : entry; }}}
                role="button"
                tabindex="0"
              >
                <div class="log-entry-header">
                  <span class="log-timestamp">{formatLogTimestamp(entry.timestamp)}</span>
                  {#if entry.llm_used}
                    <span class="log-badge llm">LLM</span>
                  {/if}
                  <button
                    class="delete-button"
                    onclick={(e) => { e.stopPropagation(); deleteLogEntry(entry.id); }}
                    title="削除"
                  >
                    ×
                  </button>
                </div>
                <p class="log-text-preview">
                  {entry.refined_text || entry.raw_text}
                </p>
                {#if selectedLogEntry?.id === entry.id}
                  <div class="log-details">
                    <div class="detail-row">
                      <span class="detail-label">認識結果:</span>
                      <span class="detail-value">{entry.raw_text}</span>
                    </div>
                    {#if entry.refined_text}
                      <div class="detail-row">
                        <span class="detail-label">整形後:</span>
                        <span class="detail-value">{entry.refined_text}</span>
                      </div>
                    {/if}
                    {#if entry.prompt_preset}
                      <div class="detail-row">
                        <span class="detail-label">プリセット:</span>
                        <span class="detail-value">{entry.prompt_preset}</span>
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
          <button class="refresh-button" onclick={loadRecentLogs} disabled={isLoadingLogs}>
            更新
          </button>
        {/if}
      </div>
    {/if}
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

  .model-hint,
  .llm-hint {
    margin-top: 0.75rem;
    font-size: 0.85rem;
    color: #666;
  }

  /* LLM Settings Styles */
  .llm-toggle {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .toggle-label {
    font-size: 0.95rem;
  }

  .switch {
    position: relative;
    display: inline-block;
    width: 50px;
    height: 26px;
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
    border-radius: 26px;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 20px;
    width: 20px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
  }

  input:checked + .slider {
    background-color: #4caf50;
  }

  input:checked + .slider:before {
    transform: translateX(24px);
  }

  .llm-settings {
    margin-top: 1rem;
    padding: 1rem;
    background-color: #f9f9f9;
    border-radius: 8px;
    border: 1px solid #e0e0e0;
  }

  .input-group {
    margin-bottom: 1rem;
  }

  .input-group:last-child {
    margin-bottom: 0;
  }

  .input-group label {
    display: block;
    font-size: 0.9rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
    color: #555;
  }

  .input-group input[type="text"] {
    width: 100%;
    padding: 0.6rem 0.75rem;
    border: 2px solid #ddd;
    border-radius: 6px;
    font-size: 0.9rem;
    box-sizing: border-box;
  }

  .input-group input[type="text"]:focus {
    outline: none;
    border-color: #396cd8;
  }

  .preset-select {
    width: 100%;
    padding: 0.6rem 0.75rem;
    border: 2px solid #ddd;
    border-radius: 6px;
    font-size: 0.9rem;
    background-color: white;
    cursor: pointer;
  }

  .preset-select:focus {
    outline: none;
    border-color: #396cd8;
  }

  .prompt-preview-section {
    margin-top: 1rem;
    border-top: 1px solid #e0e0e0;
    padding-top: 1rem;
  }

  .prompt-preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .preview-label {
    font-size: 0.9rem;
    font-weight: 600;
    color: #555;
  }

  .toggle-preview-button {
    padding: 0.4rem 0.8rem;
    font-size: 0.8rem;
    background-color: #f0f0f0;
    color: #333;
    border: 1px solid #ddd;
    min-width: auto;
  }

  .toggle-preview-button:hover {
    background-color: #e0e0e0;
  }

  .prompt-editor {
    margin-top: 0.5rem;
  }

  .prompt-textarea {
    width: 100%;
    padding: 0.75rem;
    border: 2px solid #ddd;
    border-radius: 6px;
    font-size: 0.85rem;
    font-family: monospace;
    resize: vertical;
    box-sizing: border-box;
    line-height: 1.5;
  }

  .prompt-textarea:focus {
    outline: none;
    border-color: #396cd8;
  }

  .prompt-preview {
    background-color: #f5f5f5;
    border: 1px solid #ddd;
    border-radius: 6px;
    padding: 0.75rem;
    font-size: 0.8rem;
    font-family: monospace;
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    max-height: 200px;
    overflow-y: auto;
  }

  .prompt-hint {
    margin-top: 0.5rem;
    font-size: 0.8rem;
    color: #666;
  }

  .prompt-hint code {
    background-color: #f0f0f0;
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    font-family: monospace;
  }

  .url-input-row {
    display: flex;
    gap: 0.5rem;
  }

  .url-input-row input {
    flex: 1;
  }

  .check-button {
    padding: 0.6rem 1rem;
    background-color: #666;
    color: white;
    font-size: 0.85rem;
    min-width: 100px;
  }

  .check-button:hover:not(:disabled) {
    background-color: #555;
  }

  .connection-status {
    margin-top: 0.5rem;
    font-size: 0.85rem;
    color: #888;
  }

  .connection-status.connected {
    color: #4caf50;
  }

  .connection-status.disconnected {
    color: #f44336;
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

  /* Log viewer styles */
  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .section-header h2 {
    margin: 0;
  }

  .toggle-logs-button {
    padding: 0.4rem 0.8rem;
    font-size: 0.85rem;
    background-color: #f0f0f0;
    color: #333;
    border: 1px solid #ddd;
    min-width: auto;
  }

  .toggle-logs-button:hover {
    background-color: #e0e0e0;
  }

  .log-viewer {
    margin-top: 1rem;
  }

  .log-list {
    max-height: 400px;
    overflow-y: auto;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
  }

  .log-entry {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #e0e0e0;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .log-entry:last-child {
    border-bottom: none;
  }

  .log-entry:hover {
    background-color: #f5f5f5;
  }

  .log-entry.selected {
    background-color: #e8f4ff;
  }

  .log-entry-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .log-timestamp {
    font-size: 0.8rem;
    color: #666;
  }

  .log-badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    font-weight: 600;
  }

  .log-badge.llm {
    background-color: #e3f2fd;
    color: #1976d2;
  }

  .delete-button {
    margin-left: auto;
    padding: 0.2rem 0.5rem;
    font-size: 0.9rem;
    background-color: transparent;
    color: #999;
    border: none;
    cursor: pointer;
    min-width: auto;
  }

  .delete-button:hover {
    color: #f44336;
    background-color: transparent;
  }

  .log-text-preview {
    font-size: 0.9rem;
    color: #333;
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .log-details {
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px dashed #e0e0e0;
  }

  .detail-row {
    margin-bottom: 0.5rem;
  }

  .detail-row:last-child {
    margin-bottom: 0;
  }

  .detail-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: #666;
    display: block;
    margin-bottom: 0.25rem;
  }

  .detail-value {
    font-size: 0.85rem;
    color: #333;
    display: block;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .refresh-button {
    margin-top: 1rem;
    width: 100%;
    padding: 0.5rem;
    font-size: 0.9rem;
    background-color: #f0f0f0;
    color: #333;
    border: 1px solid #ddd;
  }

  .refresh-button:hover:not(:disabled) {
    background-color: #e0e0e0;
  }

  .loading,
  .no-logs {
    text-align: center;
    padding: 2rem;
    color: #666;
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

    .llm-hint {
      color: #888;
    }

    .llm-settings {
      background-color: #1a1a1a;
      border-color: #444;
    }

    .input-group label {
      color: #aaa;
    }

    .input-group input[type="text"] {
      background-color: #1a1a1a;
      color: #f6f6f6;
      border-color: #444;
    }

    .preset-select {
      background-color: #1a1a1a;
      color: #f6f6f6;
      border-color: #444;
    }

    .prompt-preview-section {
      border-color: #444;
    }

    .preview-label {
      color: #aaa;
    }

    .toggle-preview-button {
      background-color: #333;
      color: #f6f6f6;
      border-color: #555;
    }

    .toggle-preview-button:hover {
      background-color: #444;
    }

    .prompt-textarea {
      background-color: #1a1a1a;
      color: #f6f6f6;
      border-color: #444;
    }

    .prompt-preview {
      background-color: #1a1a1a;
      border-color: #444;
      color: #f6f6f6;
    }

    .prompt-hint {
      color: #888;
    }

    .prompt-hint code {
      background-color: #333;
      color: #f6f6f6;
    }

    /* Log viewer dark mode */
    .toggle-logs-button {
      background-color: #333;
      color: #f6f6f6;
      border-color: #555;
    }

    .toggle-logs-button:hover {
      background-color: #444;
    }

    .log-list {
      border-color: #444;
    }

    .log-entry {
      border-color: #444;
    }

    .log-entry:hover {
      background-color: #333;
    }

    .log-entry.selected {
      background-color: #1a3a5c;
    }

    .log-timestamp {
      color: #aaa;
    }

    .log-badge.llm {
      background-color: #1a3a5c;
      color: #90caf9;
    }

    .delete-button {
      color: #888;
    }

    .delete-button:hover {
      color: #f44336;
    }

    .log-text-preview {
      color: #f6f6f6;
    }

    .log-details {
      border-color: #444;
    }

    .detail-label {
      color: #aaa;
    }

    .detail-value {
      color: #f6f6f6;
    }

    .refresh-button {
      background-color: #333;
      color: #f6f6f6;
      border-color: #555;
    }

    .refresh-button:hover:not(:disabled) {
      background-color: #444;
    }

    .loading,
    .no-logs {
      color: #888;
    }
  }
</style>
