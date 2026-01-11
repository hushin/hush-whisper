<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  import Sidebar from "$lib/components/Sidebar.svelte";

  import SpeechRecognitionSettings from "$lib/components/categories/SpeechRecognitionSettings.svelte";
  import ResultAndHistory from "$lib/components/categories/ResultAndHistory.svelte";
  import ShortcutAndOutput from "$lib/components/categories/ShortcutAndOutput.svelte";
  import LlmSettings from "$lib/components/categories/LlmSettings.svelte";
  import GeneralSettings from "$lib/components/categories/GeneralSettings.svelte";
  import About from "$lib/components/categories/About.svelte";

  import { settingsStore } from "$lib/stores/settings.svelte";
  import type { DownloadProgress } from "$lib/types";

  onMount(() => {
    // Load available models first, then settings, then auto-initialize if saved
    (async () => {
      await settingsStore.loadModels();
      await settingsStore.loadPresetPrompts();
      await settingsStore.loadAutostart();
      const hasSavedSettings = await settingsStore.loadSettings();
      if (hasSavedSettings) {
        console.log("Auto-initializing saved model:", settingsStore.selectedModel);
        await settingsStore.initializeWhisper();
      }
      // Check Ollama status if LLM is enabled
      if (settingsStore.llmEnabled) {
        await settingsStore.checkOllamaConnection();
      }
    })();

    // Listen for download events
    const unlistenDownloadStarted = listen("download-started", () => {
      settingsStore.isDownloading = true;
      settingsStore.downloadProgress = null;
      settingsStore.statusMessage = "モデルをダウンロード中...";
      console.log("Download started");
    });

    const unlistenDownloadProgress = listen<DownloadProgress>(
      "download-progress",
      (event) => {
        settingsStore.downloadProgress = event.payload;
        settingsStore.statusMessage = `ダウンロード中... ${event.payload.percentage.toFixed(1)}%`;
      }
    );

    const unlistenDownloadComplete = listen("download-complete", () => {
      settingsStore.isDownloading = false;
      settingsStore.statusMessage = "ダウンロード完了 - モデルを読み込み中...";
      console.log("Download complete");
    });

    // Listen for recording events
    const unlistenRecordingStarted = listen("recording-started", () => {
      settingsStore.isRecording = true;
      settingsStore.statusMessage = "録音中...";
      console.log("Recording started");
    });

    const unlistenRecordingStopped = listen("recording-stopped", () => {
      settingsStore.isRecording = false;
      settingsStore.statusMessage = "録音停止 - 音声認識中...";
      console.log("Recording stopped");
    });

    const unlistenTranscriptionStarted = listen("transcription-started", () => {
      settingsStore.isTranscribing = true;
      settingsStore.statusMessage = "音声認識中...";
      console.log("Transcription started");
    });

    const unlistenTranscriptionComplete = listen<string>(
      "transcription-complete",
      async (event) => {
        settingsStore.isTranscribing = false;
        settingsStore.transcriptionResult = event.payload;
        settingsStore.statusMessage = "認識完了 - クリップボードにコピーしました";
        console.log("Transcription complete:", event.payload);

        // Auto-refresh logs if result-history category is active
        if (settingsStore.activeCategory === "result-history") {
          await settingsStore.loadLogs();
        } else {
          settingsStore.logsNeedRefresh = true;
        }
      }
    );

    const unlistenRecordingToggle = listen("recording-toggle", () => {
      console.log("Recording toggle event received");
    });

    const unlistenRecordingAutoStopped = listen("recording-auto-stopped", () => {
      console.log("Recording auto-stopped due to max time limit");
      settingsStore.statusMessage = "最大録音時間に達したため自動停止しました";
    });

    // Listen for LLM events
    const unlistenLlmStarted = listen("llm-refinement-started", () => {
      settingsStore.isLlmRefining = true;
      settingsStore.statusMessage = "LLM で整形中...";
      console.log("LLM refinement started");
    });

    const unlistenLlmComplete = listen<string>("llm-refinement-complete", (event) => {
      settingsStore.isLlmRefining = false;
      console.log("LLM refinement complete:", event.payload);
    });

    const unlistenLlmFailed = listen<string>("llm-refinement-failed", (event) => {
      settingsStore.isLlmRefining = false;
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
      unlistenRecordingAutoStopped.then((fn) => fn());
      unlistenLlmStarted.then((fn) => fn());
      unlistenLlmComplete.then((fn) => fn());
      unlistenLlmFailed.then((fn) => fn());
    };
  });
</script>

<div class="app-container">
  <Sidebar />

  <div class="main-area">
    <div class="category-content">
      {#if settingsStore.activeCategory === "speech-recognition"}
        <SpeechRecognitionSettings />
      {:else if settingsStore.activeCategory === "result-history"}
        <ResultAndHistory />
      {:else if settingsStore.activeCategory === "shortcut-output"}
        <ShortcutAndOutput />
      {:else if settingsStore.activeCategory === "llm"}
        <LlmSettings />
      {:else if settingsStore.activeCategory === "general"}
        <GeneralSettings />
      {:else if settingsStore.activeCategory === "about"}
        <About />
      {/if}
    </div>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
      "Helvetica Neue", Arial, sans-serif;
  }

  .app-container {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .main-area {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
    background-color: #f6f6f6;
  }

  .category-content {
    max-width: 800px;
    margin: 0 auto;
  }

  @media (prefers-color-scheme: dark) {
    .main-area {
      background-color: #1a1a1a;
    }
  }
</style>
