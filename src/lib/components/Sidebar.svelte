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
    <div class="status-indicator" class:recording={settingsStore.isRecording}>
      <span class="status-icon">{settingsStore.isRecording ? "●" : "○"}</span>
      <span class="status-text">
        {settingsStore.isRecording ? "録音中" : "待機中"}
      </span>
    </div>
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
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: #666;
    transition: color 0.3s;
  }

  .status-indicator.recording {
    color: #f5576c;
  }

  .status-icon {
    font-size: 1rem;
    display: inline-block;
  }

  .status-indicator.recording .status-icon {
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.3;
    }
  }

  .status-text {
    font-weight: 600;
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

    .status-indicator {
      color: #aaa;
    }

    .status-indicator.recording {
      color: #f5576c;
    }
  }
</style>
