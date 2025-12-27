<script lang="ts">
  import { onMount } from "svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";

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

  onMount(() => {
    // 履歴カテゴリーが開かれたときに自動的にログを読み込む
    if (settingsStore.logEntries.length === 0) {
      settingsStore.loadLogs();
    }
  });
</script>

<div class="section">
  <h2>ステータス・認識結果</h2>

  <div class="status-row">
    <span class="status-label">状態:</span>
    <span class="status-value" class:processing={settingsStore.isProcessing}>
      {settingsStore.statusMessage}
    </span>
  </div>

  {#if settingsStore.errorMessage}
    <div class="error-row">
      <span class="error-icon">⚠</span>
      <span class="error-text">{settingsStore.errorMessage}</span>
    </div>
  {/if}

  {#if settingsStore.transcriptionResult}
    <div class="result-area">
      <div class="result-header">認識結果:</div>
      <div class="result-text">{settingsStore.transcriptionResult}</div>
    </div>
  {/if}
</div>

<div class="section">
  <h2>履歴</h2>

  <div class="log-viewer">
    {#if settingsStore.isLoadingLogs}
      <p class="loading">読み込み中...</p>
    {:else if settingsStore.logEntries.length === 0}
      <p class="no-logs">履歴がありません</p>
    {:else}
      <div class="log-list">
        {#each settingsStore.logEntries as entry (entry.id)}
          <div
            class="log-entry"
            class:selected={settingsStore.selectedLogEntry?.id === entry.id}
            onclick={() =>
              settingsStore.selectLogEntry(
                settingsStore.selectedLogEntry?.id === entry.id ? null : entry
              )}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                settingsStore.selectLogEntry(
                  settingsStore.selectedLogEntry?.id === entry.id ? null : entry
                );
              }
            }}
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
                onclick={(e) => {
                  e.stopPropagation();
                  settingsStore.deleteLog(entry.id);
                }}
                title="削除"
              >
                ×
              </button>
            </div>
            <p class="log-text-preview">
              {entry.refined_text || entry.raw_text}
            </p>
            {#if settingsStore.selectedLogEntry?.id === entry.id}
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
      <button
        class="refresh-button"
        onclick={() => settingsStore.loadLogs()}
        disabled={settingsStore.isLoadingLogs}
      >
        更新
      </button>
    {/if}
  </div>
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

  .status-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background-color: #f6f6f6;
    border-radius: 4px;
    margin-bottom: 0.75rem;
  }

  .status-label {
    font-weight: 600;
    color: #666;
    font-size: 0.9rem;
  }

  .status-value {
    color: #666;
    font-size: 0.9rem;
  }

  .status-value.processing {
    color: #396cd8;
    font-weight: 600;
  }

  .error-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    background-color: #fff5f5;
    border: 1px solid #ffcccc;
    border-radius: 4px;
    margin-bottom: 0.75rem;
  }

  .error-icon {
    color: #d32f2f;
    font-size: 1.1rem;
  }

  .error-text {
    color: #d32f2f;
    font-weight: 600;
    font-size: 0.9rem;
    flex: 1;
  }

  .result-area {
    background-color: #f6f6f6;
    border-radius: 4px;
    padding: 0.75rem;
  }

  .result-header {
    font-weight: 600;
    color: #666;
    font-size: 0.9rem;
    margin-bottom: 0.5rem;
  }

  .result-text {
    color: #333;
    font-size: 0.95rem;
    line-height: 1.6;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .log-viewer {
    margin-top: 1rem;
  }

  .loading,
  .no-logs {
    text-align: center;
    color: #666;
    padding: 2rem;
    font-style: italic;
  }

  .log-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1rem;
    max-height: 500px;
    overflow-y: auto;
  }

  .log-entry {
    padding: 1rem;
    background-color: #f6f6f6;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .log-entry:hover {
    background-color: #f0f0f0;
    border-color: #ccc;
  }

  .log-entry.selected {
    background-color: #e8f4ff;
    border-color: #396cd8;
  }

  .log-entry:focus-visible {
    outline: 2px solid #396cd8;
    outline-offset: 2px;
  }

  .log-entry-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .log-timestamp {
    font-size: 0.85rem;
    color: #666;
    font-weight: 600;
  }

  .log-badge {
    padding: 0.2rem 0.5rem;
    font-size: 0.75rem;
    font-weight: 600;
    border-radius: 3px;
  }

  .log-badge.llm {
    background-color: #667eea;
    color: white;
  }

  .delete-button {
    margin-left: auto;
    padding: 0.2rem 0.5rem;
    background-color: transparent;
    color: #f44336;
    border: none;
    border-radius: 3px;
    font-size: 1.5rem;
    line-height: 1;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .delete-button:hover {
    background-color: rgba(244, 67, 54, 0.1);
  }

  .log-text-preview {
    margin: 0;
    font-size: 0.95rem;
    color: #333;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    line-height: 1.4;
  }

  .log-details {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #ddd;
  }

  .detail-row {
    margin-bottom: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .detail-label {
    font-size: 0.85rem;
    font-weight: 600;
    color: #666;
  }

  .detail-value {
    font-size: 0.9rem;
    color: #333;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .refresh-button {
    width: 100%;
    padding: 0.75rem;
    background-color: #f0f0f0;
    color: #333;
    border: none;
    border-radius: 4px;
    font-size: 0.95rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .refresh-button:hover:not(:disabled) {
    background-color: #e0e0e0;
  }

  .refresh-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @media (prefers-color-scheme: dark) {
    .section {
      background-color: #2a2a2a;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    }

    h2 {
      color: #f6f6f6;
    }

    .status-row {
      background-color: #1a1a1a;
    }

    .status-label {
      color: #aaa;
    }

    .status-value {
      color: #aaa;
    }

    .status-value.processing {
      color: #90caf9;
    }

    .error-row {
      background-color: #2a1a1a;
      border-color: #663333;
    }

    .error-icon,
    .error-text {
      color: #f44336;
    }

    .result-area {
      background-color: #1a1a1a;
    }

    .result-header {
      color: #aaa;
    }

    .result-text {
      color: #f6f6f6;
    }

    .loading,
    .no-logs {
      color: #aaa;
    }

    .log-entry {
      background-color: #1a1a1a;
      border-color: #444;
    }

    .log-entry:hover {
      background-color: #222;
      border-color: #555;
    }

    .log-entry.selected {
      background-color: #1a3a5c;
      border-color: #90caf9;
    }

    .log-timestamp {
      color: #aaa;
    }

    .log-text-preview {
      color: #f6f6f6;
    }

    .log-details {
      border-top-color: #444;
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
    }

    .refresh-button:hover:not(:disabled) {
      background-color: #444;
    }
  }
</style>
