<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
</script>

<div class="section">
  <h2>認識結果</h2>
  <div class="result-display">
    {#if settingsStore.transcriptionResult}
      <p class="result">{settingsStore.transcriptionResult}</p>
    {:else}
      <p class="placeholder">認識結果がここに表示されます</p>
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

  .result-display {
    padding: 1rem;
    background-color: #f6f6f6;
    border-radius: 4px;
    min-height: 4rem;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .result {
    margin: 0;
    font-size: 1rem;
    color: #333;
    line-height: 1.6;
  }

  .placeholder {
    margin: 0;
    color: #999;
    font-style: italic;
  }

  @media (prefers-color-scheme: dark) {
    .section {
      background-color: #2a2a2a;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    }

    h2 {
      color: #f6f6f6;
    }

    .result-display {
      background-color: #1a1a1a;
    }

    .result {
      color: #f6f6f6;
    }

    .placeholder {
      color: #666;
    }
  }
</style>
