<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { presetDescriptions, type PromptPreset } from "$lib/types";

  function handlePresetChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    settingsStore.promptPreset = select.value as PromptPreset;
    // If switching to Custom and no custom prompt exists, use current preset's prompt as starting point
    if (settingsStore.promptPreset === "Custom" && !settingsStore.customPrompt) {
      const currentPresetPrompt = settingsStore.presetPrompts.get("Default") || "";
      settingsStore.customPrompt = currentPresetPrompt;
    }
    settingsStore.savePromptSettings(settingsStore.promptPreset, settingsStore.customPrompt);
  }

  function getCurrentPromptPreview(): string {
    if (settingsStore.promptPreset === "Custom") {
      return settingsStore.customPrompt || "(カスタムプロンプト未設定)";
    }
    return settingsStore.presetPrompts.get(settingsStore.promptPreset) || "";
  }

  function savePromptSettingsHandler() {
    settingsStore.savePromptSettings(settingsStore.promptPreset, settingsStore.customPrompt);
  }
</script>

<div class="section">
  <h2>LLM 整形設定 (Ollama)</h2>
  <div class="llm-toggle">
    <label class="switch">
      <input
        type="checkbox"
        bind:checked={settingsStore.llmEnabled}
        onchange={() => settingsStore.saveLlmSettings()}
      />
      <span class="slider"></span>
    </label>
    <span class="toggle-label">LLM による文章整形を有効にする</span>
  </div>

  {#if settingsStore.llmEnabled}
    <div class="llm-settings">
      <div class="input-group">
        <label for="ollama-url">Ollama URL</label>
        <div class="url-input-row">
          <input
            type="text"
            id="ollama-url"
            bind:value={settingsStore.llmOllamaUrl}
            onblur={() => settingsStore.saveLlmSettings()}
            placeholder="http://localhost:11434"
          />
          <button
            class="check-button"
            onclick={() => settingsStore.checkOllamaConnection()}
            disabled={settingsStore.isCheckingOllama}
          >
            {settingsStore.isCheckingOllama ? "確認中..." : "接続確認"}
          </button>
        </div>
        <div
          class="connection-status"
          class:connected={settingsStore.llmStatus === "connected"}
          class:disconnected={settingsStore.llmStatus === "disconnected"}
        >
          {#if settingsStore.llmStatus === "connected"}
            ✓ 接続済み
          {:else if settingsStore.llmStatus === "disconnected"}
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
          bind:value={settingsStore.llmModelName}
          onblur={() => settingsStore.saveLlmSettings()}
          placeholder="gpt-oss:20b"
        />
      </div>

      <div class="input-group">
        <label for="prompt-preset">プロンプトプリセット</label>
        <select
          id="prompt-preset"
          value={settingsStore.promptPreset}
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
            onclick={() => (settingsStore.showPromptEditor = !settingsStore.showPromptEditor)}
          >
            {settingsStore.showPromptEditor
              ? "閉じる"
              : settingsStore.promptPreset === "Custom"
                ? "編集"
                : "プレビュー"}
          </button>
        </div>
        {#if settingsStore.showPromptEditor}
          <div class="prompt-editor">
            {#if settingsStore.promptPreset === "Custom"}
              <textarea
                bind:value={settingsStore.customPrompt}
                onblur={savePromptSettingsHandler}
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

<style>
  /* コンポーネント固有のスタイル */
  .llm-toggle {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .llm-settings {
    margin-top: 1rem;
  }

  .url-input-row {
    display: flex;
    gap: 0.5rem;
  }

  .url-input-row input {
    flex: 1;
  }

  .check-button {
    padding: 0.5rem 1rem;
    background-color: #396cd8;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 0.9rem;
    cursor: pointer;
    transition: background-color 0.2s;
    white-space: nowrap;
  }

  .check-button:hover:not(:disabled) {
    background-color: #2854b8;
  }

  .check-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }

  .connection-status {
    margin-top: 0.5rem;
    font-size: 0.9rem;
    font-weight: 600;
  }

  .connection-status.connected {
    color: #4caf50;
  }

  .connection-status.disconnected {
    color: #f44336;
  }

  .prompt-preview-section {
    margin-top: 1rem;
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
    color: #333;
  }

  .toggle-preview-button {
    padding: 0.25rem 0.75rem;
    background-color: #f0f0f0;
    color: #333;
    border: none;
    border-radius: 4px;
    font-size: 0.85rem;
    cursor: pointer;
    transition: background-color 0.2s;
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
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.9rem;
    font-family: "Consolas", "Monaco", monospace;
    resize: vertical;
  }

  .prompt-preview {
    padding: 0.75rem;
    background-color: #f6f6f6;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.9rem;
    font-family: "Consolas", "Monaco", monospace;
    white-space: pre-wrap;
    word-wrap: break-word;
    margin: 0;
  }

  .prompt-hint {
    margin: 0.5rem 0 0 0;
    font-size: 0.85rem;
    color: #666;
  }

  .prompt-hint code {
    background-color: #f0f0f0;
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-family: "Consolas", "Monaco", monospace;
  }

  .llm-hint {
    font-size: 0.85rem;
    color: #666;
    margin: 1rem 0 0 0;
  }

  @media (prefers-color-scheme: dark) {
    .check-button {
      background-color: #90caf9;
      color: #000;
    }

    .check-button:hover:not(:disabled) {
      background-color: #64b5f6;
    }

    .check-button:disabled {
      background-color: #666;
      color: #999;
    }

    .preview-label {
      color: #f6f6f6;
    }

    .toggle-preview-button {
      background-color: #333;
      color: #f6f6f6;
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
      color: #aaa;
    }

    .prompt-hint code {
      background-color: #333;
      color: #f6f6f6;
    }

    .llm-hint {
      color: #aaa;
    }
  }
</style>
