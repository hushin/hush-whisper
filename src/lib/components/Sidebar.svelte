<script lang="ts">
  import { categories, type CategoryId } from "$lib/types";
  import { settingsStore } from "$lib/stores/settings.svelte";

  function handleCategoryClick(categoryId: CategoryId) {
    settingsStore.setActiveCategory(categoryId);
  }
</script>

<aside class="sidebar">
  <div class="sidebar-menu">
    {#each categories as category}
      <div
        class="sidebar-item"
        class:active={settingsStore.activeCategory === category.id}
        onclick={() => handleCategoryClick(category.id)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            handleCategoryClick(category.id);
          }
        }}
        role="button"
        tabindex="0"
      >
        {category.label}
      </div>
    {/each}
  </div>

  <div class="sidebar-status">
    <div
      class="status-indicator"
      class:recording={settingsStore.isRecording}
      class:llm-processing={settingsStore.isLlmRefining}
    >
      <span class="status-icon">
        {#if settingsStore.isRecording}
          ●
        {:else if settingsStore.isLlmRefining}
          ⟳
        {:else}
          ○
        {/if}
      </span>
      <span class="status-text">
        {#if settingsStore.isRecording}
          録音中
        {:else if settingsStore.isLlmRefining}
          LLM処理中
        {:else}
          待機中
        {/if}
      </span>
    </div>
    <button
      class="record-button"
      class:recording={settingsStore.isRecording}
      onclick={() => settingsStore.toggleRecording()}
      disabled={!settingsStore.isModelInitialized}
      title={settingsStore.isRecording ? "録音停止" : "録音開始"}
    >
      {#if settingsStore.isRecording}
        <span class="pulse">●</span>
      {:else}
        ●
      {/if}
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: 200px;
    background-color: #f9f9f9;
    border-right: 1px solid #e0e0e0;
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .sidebar-menu {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 0;
  }

  .sidebar-item {
    display: flex;
    align-items: center;
    padding: 0.75rem 1rem;
    cursor: pointer;
    transition: background-color 0.2s;
    position: relative;
    color: #333;
    font-size: 0.95rem;
    user-select: none;
  }

  .sidebar-item:hover {
    background-color: #f0f0f0;
  }

  .sidebar-item.active {
    background-color: #e8f4ff;
    color: #396cd8;
    font-weight: 600;
  }

  .sidebar-item.active::before {
    content: "";
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background-color: #396cd8;
  }

  .sidebar-item:focus-visible {
    outline: 2px solid #396cd8;
    outline-offset: -2px;
  }

  .sidebar-status {
    padding: 1rem;
    border-top: 1px solid #e0e0e0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .record-button {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    padding: 0;
    font-size: 1rem;
    font-weight: 600;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .record-button:hover:not(:disabled) {
    transform: scale(1.05);
    box-shadow: 0 3px 6px rgba(0, 0, 0, 0.15);
  }

  .record-button:active:not(:disabled) {
    transform: scale(0.95);
  }

  .record-button:disabled {
    background: linear-gradient(135deg, #ccc 0%, #999 100%);
    cursor: not-allowed;
    opacity: 0.6;
  }

  .record-button.recording {
    background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
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

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: #666;
    transition: color 0.3s;
    flex: 1;
    min-width: 0;
  }

  .status-indicator.recording {
    color: #f5576c;
  }

  .status-indicator.llm-processing {
    color: #667eea;
  }

  .status-icon {
    font-size: 0.9rem;
    display: inline-block;
    flex-shrink: 0;
  }

  .status-indicator.recording .status-icon {
    animation: pulse 1.5s ease-in-out infinite;
  }

  .status-indicator.llm-processing .status-icon {
    animation: spin 1.5s linear infinite;
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  .status-text {
    font-weight: 600;
    white-space: nowrap;
  }

  @media (prefers-color-scheme: dark) {
    .sidebar {
      background-color: #1a1a1a;
      border-right-color: #333;
    }

    .sidebar-item {
      color: #f6f6f6;
    }

    .sidebar-item:hover {
      background-color: #2a2a2a;
    }

    .sidebar-item.active {
      background-color: #1a3a5c;
      color: #90caf9;
    }

    .sidebar-item.active::before {
      background-color: #90caf9;
    }

    .sidebar-status {
      border-top-color: #333;
    }

    .record-button {
      background: linear-gradient(135deg, #90caf9 0%, #a58de3 100%);
    }

    .record-button:hover:not(:disabled) {
      background: linear-gradient(135deg, #64b5f6 0%, #9575cd 100%);
    }

    .record-button:disabled {
      background: linear-gradient(135deg, #666 0%, #444 100%);
    }

    .record-button.recording {
      background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
    }

    .status-indicator {
      color: #aaa;
    }

    .status-indicator.recording {
      color: #f5576c;
    }

    .status-indicator.llm-processing {
      color: #90caf9;
    }
  }
</style>
