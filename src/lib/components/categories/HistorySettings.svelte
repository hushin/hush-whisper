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
