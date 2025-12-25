import { invoke } from "@tauri-apps/api/core";
import type {
  CategoryId,
  DownloadProgress,
  ModelInfo,
  Settings,
  PromptPreset,
  OutputMode,
  LogEntry,
} from "$lib/types";

class SettingsStore {
  // UI state
  activeCategory = $state<CategoryId>("model");

  // Whisper settings
  availableModels = $state<ModelInfo[]>([]);
  selectedModel = $state("large-v3-turbo");
  insertNewline = $state(true);
  maxRecordingSeconds = $state(300);
  currentLoadedModel = $state<string | null>(null);
  isModelInitialized = $state(false);

  // LLM settings
  llmEnabled = $state(false);
  llmOllamaUrl = $state("http://localhost:11434");
  llmModelName = $state("gpt-oss:20b");
  llmStatus = $state<"unknown" | "connected" | "disconnected">("unknown");
  isCheckingOllama = $state(false);
  isLlmRefining = $state(false);

  // Prompt settings
  promptPreset = $state<PromptPreset>("Default");
  customPrompt = $state("");
  presetPrompts = $state<Map<string, string>>(new Map());
  showPromptEditor = $state(false);

  // Output mode settings
  outputMode = $state<OutputMode>("DirectInput");

  // Shortcut settings
  shortcutKey = $state("Ctrl+Space");
  isEditingShortcut = $state(false);
  shortcutError = $state("");
  pendingShortcut = $state("");
  shortcutChanged = $state(false);

  // Autostart settings
  autostartEnabled = $state(false);
  isLoadingAutostart = $state(false);

  // Status
  isRecording = $state(false);
  isTranscribing = $state(false);
  isDownloading = $state(false);
  downloadProgress = $state<DownloadProgress | null>(null);
  transcriptionResult = $state("");
  statusMessage = $state("モデルを選択して初期化してください");
  errorMessage = $state("");

  // Log viewer
  showLogViewer = $state(false);
  logEntries = $state<LogEntry[]>([]);
  isLoadingLogs = $state(false);
  selectedLogEntry = $state<LogEntry | null>(null);
  logsNeedRefresh = $state(false);

  // Derived state
  isProcessing = $derived(
    this.isRecording || this.isTranscribing || this.isDownloading || this.isLlmRefining
  );

  // Methods
  setActiveCategory(category: CategoryId) {
    this.activeCategory = category;
  }

  formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  formatDuration(seconds: number): string {
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    if (minutes === 0) return `${secs}秒`;
    if (secs === 0) return `${minutes}分`;
    return `${minutes}分${secs}秒`;
  }

  async loadModels() {
    try {
      this.availableModels = await invoke("get_available_models");
    } catch (error) {
      console.error("Failed to load models:", error);
    }
  }

  async loadSettings(): Promise<boolean> {
    try {
      const settings: Settings = await invoke("get_settings");
      this.selectedModel = settings.whisper.model_name;
      this.insertNewline = settings.whisper.insert_newline ?? true;
      this.maxRecordingSeconds = settings.whisper.max_recording_seconds ?? 300;
      this.llmEnabled = settings.llm.enabled;
      this.llmOllamaUrl = settings.llm.ollama_url;
      this.llmModelName = settings.llm.model_name;
      this.promptPreset = settings.llm.preset || "Default";
      this.customPrompt = settings.llm.custom_prompt || "";
      this.outputMode = settings.output_mode || "Both";
      this.shortcutKey = settings.shortcut?.recording_toggle || "Ctrl+Space";
      console.log(
        "Loaded settings, model:",
        this.selectedModel,
        "llm:",
        settings.llm,
        "output_mode:",
        this.outputMode,
        "shortcut:",
        this.shortcutKey,
        "max_recording_seconds:",
        this.maxRecordingSeconds,
        "is_saved:",
        settings.is_saved
      );
      return settings.is_saved;
    } catch (error) {
      console.error("Failed to load settings:", error);
      return false;
    }
  }

  async saveModelSelection(modelName: string) {
    try {
      await invoke("save_model_selection", { modelName });
      this.selectedModel = modelName;
      console.log("Saved model selection:", modelName);
    } catch (error) {
      console.error("Failed to save model selection:", error);
    }
  }

  async saveInsertNewline() {
    try {
      await invoke("save_whisper_insert_newline", { insertNewline: this.insertNewline });
      console.log("Saved insert newline:", this.insertNewline);
    } catch (error) {
      console.error("Failed to save insert newline:", error);
    }
  }

  async saveMaxRecordingSeconds() {
    try {
      await invoke("save_max_recording_seconds", { maxSeconds: this.maxRecordingSeconds });
      console.log("Saved max recording seconds:", this.maxRecordingSeconds);
    } catch (error) {
      console.error("Failed to save max recording seconds:", error);
    }
  }

  async saveOutputMode() {
    try {
      await invoke("save_output_mode", { mode: this.outputMode });
      console.log("Saved output mode:", this.outputMode);
    } catch (error) {
      console.error("Failed to save output mode:", error);
    }
  }

  async saveLlmSettings() {
    try {
      await invoke("save_llm_settings", {
        enabled: this.llmEnabled,
        ollamaUrl: this.llmOllamaUrl,
        modelName: this.llmModelName,
      });
      console.log("Saved LLM settings");
    } catch (error) {
      console.error("Failed to save LLM settings:", error);
    }
  }

  async checkOllamaConnection() {
    this.isCheckingOllama = true;
    try {
      const isAvailable: boolean = await invoke("check_ollama_status", {
        ollamaUrl: this.llmOllamaUrl,
      });
      this.llmStatus = isAvailable ? "connected" : "disconnected";
      console.log("Ollama status:", this.llmStatus);
    } catch (error) {
      this.llmStatus = "disconnected";
      console.error("Failed to check Ollama status:", error);
    } finally {
      this.isCheckingOllama = false;
    }
  }

  async initializeWhisper() {
    try {
      this.errorMessage = "";
      this.statusMessage = "モデルを確認中...";

      await invoke("initialize_whisper", { modelName: this.selectedModel });
      this.isModelInitialized = true;
      this.currentLoadedModel = this.selectedModel;
      this.isDownloading = false;
      this.downloadProgress = null;
      this.statusMessage = `準備完了 - ${this.shortcutKey} で録音開始/停止`;

      // Save the selected model for next time
      await this.saveModelSelection(this.selectedModel);
    } catch (error) {
      this.errorMessage = `モデル初期化エラー: ${error}`;
      this.statusMessage = "エラー";
      this.isDownloading = false;
      this.downloadProgress = null;
      console.error(error);
    }
  }

  async toggleRecording() {
    try {
      this.errorMessage = "";
      const result = await invoke("toggle_recording");
      console.log(result);
    } catch (error) {
      this.errorMessage = `録音エラー: ${error}`;
      console.error(error);
    }
  }

  async loadPresetPrompts() {
    try {
      const presets: [string, string][] = await invoke("get_preset_prompts");
      this.presetPrompts = new Map(presets);
      console.log("Loaded preset prompts:", this.presetPrompts);
    } catch (error) {
      console.error("Failed to load preset prompts:", error);
    }
  }

  async loadAutostart() {
    try {
      this.isLoadingAutostart = true;
      this.autostartEnabled = await invoke("get_autostart_enabled");
      console.log("Loaded autostart enabled:", this.autostartEnabled);
    } catch (error) {
      console.error("Failed to load autostart setting:", error);
    } finally {
      this.isLoadingAutostart = false;
    }
  }

  async toggleAutostart() {
    try {
      this.isLoadingAutostart = true;
      await invoke("set_autostart_enabled", { enabled: !this.autostartEnabled });
      this.autostartEnabled = !this.autostartEnabled;
      console.log("Saved autostart enabled:", this.autostartEnabled);
    } catch (error) {
      console.error("Failed to save autostart setting:", error);
    } finally {
      this.isLoadingAutostart = false;
    }
  }

  saveShortcut() {
    if (!this.pendingShortcut) {
      this.shortcutError = "キーを入力してください";
      return;
    }
    return (async () => {
      try {
        this.shortcutError = "";
        await invoke("save_shortcut_setting", { shortcut: this.pendingShortcut });
        this.shortcutKey = this.pendingShortcut;
        this.isEditingShortcut = false;
        this.pendingShortcut = "";
        this.shortcutChanged = true;
        console.log("Saved shortcut:", this.shortcutKey);
      } catch (error) {
        this.shortcutError = String(error);
        console.error("Failed to save shortcut:", error);
      }
    })();
  }

  cancelShortcutEdit() {
    this.shortcutError = "";
    this.pendingShortcut = "";
    this.isEditingShortcut = false;
  }

  startShortcutEdit() {
    this.pendingShortcut = "";
    this.shortcutError = "";
    this.shortcutChanged = false;
    this.isEditingShortcut = true;
  }

  handleShortcutKeyDown(event: KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();

    // Ignore modifier-only keys
    if (["Control", "Shift", "Alt", "Meta"].includes(event.key)) {
      return;
    }

    const parts: string[] = [];

    if (event.ctrlKey) parts.push("Ctrl");
    if (event.altKey) parts.push("Alt");
    if (event.shiftKey) parts.push("Shift");
    if (event.metaKey) parts.push("Win");

    // Use event.code to get the physical key (avoids Shift+1 becoming "!")
    const code = event.code;
    let key: string;

    if (code === "Space") {
      key = "Space";
    } else if (code.startsWith("Key")) {
      key = code.slice(3); // KeyA -> A
    } else if (code.startsWith("Digit")) {
      key = code.slice(5); // Digit1 -> 1
    } else if (code.startsWith("Arrow")) {
      key = code.slice(5); // ArrowUp -> Up
    } else if (code.startsWith("F") && /^F\d+$/.test(code)) {
      key = code; // F1, F2, etc.
    } else if (
      code === "Backspace" ||
      code === "Delete" ||
      code === "Insert" ||
      code === "Home" ||
      code === "End" ||
      code === "PageUp" ||
      code === "PageDown" ||
      code === "Enter" ||
      code === "Tab" ||
      code === "Escape"
    ) {
      key = code;
    } else {
      // For other keys, use the code directly
      key = code;
    }

    parts.push(key);
    this.pendingShortcut = parts.join("+");
  }

  async savePromptSettings(preset: PromptPreset, customPrompt: string) {
    try {
      await invoke("save_llm_prompt_settings", { preset, customPrompt });
      this.promptPreset = preset;
      this.customPrompt = customPrompt;
      console.log("Saved prompt settings:", preset, customPrompt);
    } catch (error) {
      console.error("Failed to save prompt settings:", error);
    }
  }

  async loadLogs() {
    try {
      this.isLoadingLogs = true;
      this.logEntries = await invoke("get_recent_logs", { limit: 50 });
      this.logsNeedRefresh = false;
      console.log("Loaded logs:", this.logEntries.length);
    } catch (error) {
      console.error("Failed to load logs:", error);
    } finally {
      this.isLoadingLogs = false;
    }
  }

  async deleteLog(logId: string) {
    try {
      const deleted: boolean = await invoke("delete_log_entry", { id: logId });
      if (deleted) {
        // Remove from local list
        this.logEntries = this.logEntries.filter((entry) => entry.id !== logId);
        // Clear selected entry if it was deleted
        if (this.selectedLogEntry?.id === logId) {
          this.selectedLogEntry = null;
        }
        console.log("Deleted log entry:", logId);
      }
    } catch (error) {
      console.error("Failed to delete log:", error);
    }
  }

  selectLogEntry(entry: LogEntry | null) {
    this.selectedLogEntry = entry;
  }

  toggleLogViewer() {
    this.showLogViewer = !this.showLogViewer;
    if (this.showLogViewer && this.logEntries.length === 0) {
      this.loadLogs();
    }
  }
}

export const settingsStore = new SettingsStore();
