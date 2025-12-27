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

  async function handleDeleteAll() {
    if (settingsStore.logEntries.length === 0) {
      return;
    }

    const confirmed = confirm(
      `すべての履歴(${settingsStore.logEntries.length}件)を削除しますか?\nこの操作は元に戻せません。`
    );

    if (confirmed) {
      try {
        await settingsStore.deleteAllLogs();
      } catch (error) {
        alert("履歴の削除に失敗しました: " + error);
      }
    }
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
      <div class="button-row">
        <button
          class="refresh-button"
          onclick={() => settingsStore.loadLogs()}
          disabled={settingsStore.isLoadingLogs}
        >
          更新
        </button>
        <button
          class="delete-all-button"
          onclick={handleDeleteAll}
          disabled={settingsStore.isLoadingLogs || settingsStore.logEntries.length === 0}
        >
          全削除
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  /* コンポーネント固有のスタイル */
  .log-list {
    max-height: 300px;
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

  @media (prefers-color-scheme: dark) {
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
  }
</style>
