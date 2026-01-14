<script lang="ts">
  import { onMount } from "svelte";

  interface LicenseEntry {
    name: string;
    version: string;
    license: string;
    repository: string;
    authors?: string;
    publisher?: string;
  }

  interface LicenseData {
    generated: string;
    npm: LicenseEntry[];
    rust: LicenseEntry[];
  }

  let licenseData: LicenseData | null = $state(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let activeTab: "rust" | "npm" = $state("rust");

  onMount(async () => {
    try {
      const response = await fetch("/licenses/licenses.json");
      if (!response.ok) {
        throw new Error("Failed to load license data");
      }
      licenseData = await response.json();
    } catch (e) {
      error = e instanceof Error ? e.message : "Unknown error";
    } finally {
      loading = false;
    }
  });

  const currentLicenses = $derived(() => {
    if (!licenseData) return [];
    return activeTab === "rust" ? licenseData.rust : licenseData.npm;
  });

  const licenseCount = $derived(() => {
    if (!licenseData) return { rust: 0, npm: 0 };
    return {
      rust: licenseData.rust.length,
      npm: licenseData.npm.length,
    };
  });
</script>

<div class="section">
  <h2>About</h2>
  <div class="app-info">
    <p class="app-name">HushWhisper</p>
    <p class="app-version">Version 0.1.1</p>
    <p class="app-description">
      ローカル音声入力アプリ
    </p>
    <a
      href="https://github.com/hushin/hush-whisper"
      target="_blank"
      rel="noopener noreferrer"
      class="app-repository"
    >
      GitHub Repository
    </a>
  </div>
</div>

<div class="section">
  <h2>Third-Party Licenses</h2>

  {#if loading}
    <p class="loading">Loading licenses...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if licenseData}
    <div class="tabs">
      <button
        class="tab"
        class:active={activeTab === "rust"}
        onclick={() => (activeTab = "rust")}
      >
        Rust ({licenseCount().rust})
      </button>
      <button
        class="tab"
        class:active={activeTab === "npm"}
        onclick={() => (activeTab = "npm")}
      >
        npm ({licenseCount().npm})
      </button>
    </div>

    <div class="license-list">
      {#each currentLicenses() as license}
        <div class="license-item">
          <div class="license-header">
            <span class="license-name">{license.name}</span>
            <span class="license-version">{license.version}</span>
          </div>
          <div class="license-details">
            <span class="license-type">{license.license}</span>
            {#if license.repository}
              <a
                href={license.repository}
                target="_blank"
                rel="noopener noreferrer"
                class="license-repo"
              >
                Repository
              </a>
            {/if}
          </div>
        </div>
      {:else}
        <p class="no-results">No licenses found</p>
      {/each}
    </div>

  {/if}
</div>

<style>
  .app-info {
    text-align: center;
    margin-bottom: 1rem;
  }

  .app-name {
    font-size: 1.5rem;
    font-weight: bold;
    margin: 0;
  }

  .app-version {
    color: var(--text-secondary);
    margin: 0.25rem 0;
  }

  .app-description {
    margin: 0.5rem 0 0;
  }

  .app-repository {
    display: inline-block;
    margin-top: 0.75rem;
    color: var(--accent-color, #007aff);
    text-decoration: none;
    font-size: 0.875rem;
  }

  .app-repository:hover {
    text-decoration: underline;
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .tab {
    padding: 0.5rem 1rem;
    border: 1px solid var(--border-color);
    background: var(--bg-secondary);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .tab:hover {
    background: var(--bg-hover);
  }

  .tab.active {
    background: var(--accent-color, #007aff);
    color: white;
    border-color: var(--accent-color, #007aff);
  }

  .license-list {
    max-height: 250px;
    overflow-y: auto;
    border: 1px solid var(--border-color);
    border-radius: 4px;
  }

  .license-item {
    padding: 0.75rem;
    border-bottom: 1px solid var(--border-color);
  }

  .license-item:last-child {
    border-bottom: none;
  }

  .license-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.25rem;
  }

  .license-name {
    font-weight: 500;
  }

  .license-version {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .license-details {
    display: flex;
    gap: 1rem;
    font-size: 0.875rem;
  }

  .license-type {
    color: var(--text-secondary);
  }

  .license-repo {
    color: var(--accent-color);
    text-decoration: none;
  }

  .license-repo:hover {
    text-decoration: underline;
  }

  .loading,
  .error,
  .no-results {
    padding: 1rem;
    text-align: center;
    color: var(--text-secondary);
  }

  .error {
    color: var(--error-color, #e74c3c);
  }
</style>
