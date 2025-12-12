# VoiceInput

Windows 向けローカル音声入力アプリ。グローバルショートカットで任意のアプリに日本語音声入力。

## Tech Stack

- **Framework**: Tauri 2.0 (Rust backend + Web frontend)
- **Speech Recognition**: whisper-rs (whisper.cpp bindings, CUDA)
- **LLM**: llama-cpp-rs (Qwen2.5-7B-Instruct Q4_K_M)
- **Audio**: cpal + Silero VAD
- **Frontend**: Svelte 5 + TypeScript

## Project Structure

```
src-tauri/       # Rust backend
  src/
    audio/       # Audio capture, VAD, resampling
    whisper/     # Speech recognition
    llm/         # Text refinement
    shortcuts/   # Global hotkey handling
src/             # Svelte frontend
models/          # Downloaded models (git-ignored)
docs/            # Implementation details (read as needed)
```

## Commands

```bash
# Development
pnpm tauri dev

# Build
pnpm tauri build

# Test Rust
cargo test --manifest-path src-tauri/Cargo.toml

# Lint & Format
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml
pnpm lint
```

## Key Constraints

- 12GB VRAM budget: Whisper (~6GB) + LLM (~5GB)
- No Python runtime - all inference in Rust
- Models downloaded separately via hf-hub
- Target latency: <3 seconds end-to-end

## Documentation

詳細な実装ガイドは `docs/` 配下を参照:

- `docs/plan.md` - 実装計画とフェーズ
- `docs/audio-pipeline.md` - 音声処理パイプライン
- `docs/model-management.md` - モデルダウンロード・管理
