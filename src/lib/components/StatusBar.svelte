<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
</script>

<div class="section">
  <h2>ステータス</h2>
  <div class="status-display">
    <p class="status" class:processing={settingsStore.isProcessing}>
      {settingsStore.statusMessage}
    </p>
    {#if settingsStore.errorMessage}
      <p class="error">{settingsStore.errorMessage}</p>
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

  .status-display {
    padding: 1rem;
    background-color: #f6f6f6;
    border-radius: 4px;
  }

  .status {
    margin: 0;
    padding: 0.5rem;
    font-size: 1rem;
    color: #666;
  }

  .status.processing {
    color: #396cd8;
    font-weight: 600;
  }

  .error {
    margin: 0.5rem 0 0 0;
    padding: 0.5rem;
    color: #d32f2f;
    font-weight: 600;
  }

  @media (prefers-color-scheme: dark) {
    .section {
      background-color: #2a2a2a;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    }

    h2 {
      color: #f6f6f6;
    }

    .status-display {
      background-color: #1a1a1a;
    }

    .status {
      color: #aaa;
    }

    .status.processing {
      color: #90caf9;
    }

    .error {
      color: #f44336;
    }
  }
</style>
