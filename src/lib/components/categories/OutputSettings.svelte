<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { outputModeDescriptions } from "$lib/types";
</script>

<div class="section">
  <h2>出力設定</h2>
  <div class="input-group">
    <label for="output-mode">出力モード</label>
    <select
      id="output-mode"
      bind:value={settingsStore.outputMode}
      onchange={() => settingsStore.saveOutputMode()}
      class="output-select"
    >
      <option value="ClipboardOnly">{outputModeDescriptions.ClipboardOnly}</option>
      <option value="DirectInput">{outputModeDescriptions.DirectInput}</option>
      <option value="Both">{outputModeDescriptions.Both}</option>
    </select>
  </div>
  <p class="output-hint">
    直接入力 は認識結果を直接アクティブなアプリケーションに入力します
  </p>
</div>

<style>
  /* コンポーネント固有のスタイル */
  .output-select {
    cursor: pointer;
  }
</style>
