<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
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
  }
</style>
