# 技術スタック

## フレームワーク・言語

| 領域      | 選定      | 理由                                    |
| --------- | --------- | --------------------------------------- |
| Framework | Tauri 2.0 | 軽量、Rust 統合、クロスプラットフォーム |
| Frontend  | Svelte 5  | 軽量、シンプル                          |
| Backend   | Rust      | パフォーマンス、メモリ安全性            |

## 音声処理

| 領域          | 選定       | 理由                                   |
| ------------- | ---------- | -------------------------------------- |
| Speech        | whisper-rs | CUDA 対応、安定、GGML 形式             |
| Audio Capture | cpal       | クロスプラットフォーム、低レイテンシー |
| Resampling    | rubato     | 高品質リサンプリング                   |
| VAD (計画中)  | Silero VAD | 音声区間検出、ハルシネーション防止     |

## LLM・モデル管理（計画中）

| 領域           | 選定                | 理由                          |
| -------------- | ------------------- | ----------------------------- |
| LLM Runtime    | llama-cpp-rs        | GGUF 対応、成熟度高い         |
| Model          | Qwen2.5-7B-Instruct | 日本語性能、128K コンテキスト |
| Model Download | hf-hub              | HuggingFace 公式クライアント  |

## システム統合

| 領域            | 選定                         | 理由                               |
| --------------- | ---------------------------- | ---------------------------------- |
| Global Shortcut | tauri-plugin-global-shortcut | ショートカット統合                 |
| Clipboard       | arboard                      | クロスプラットフォーム対応         |
| Auto Paste      | enigo                        | キーボードイベントシミュレーション |

## 開発ツール

| 領域            | 選定      | 理由                           |
| --------------- | --------- | ------------------------------ |
| Package Manager | pnpm      | 高速、効率的なディスク使用     |
| Build Tool      | Tauri CLI | Rust + Frontend 統合ビルド     |
| Logging         | tracing   | 構造化ロギング、パフォーマンス |

## VRAM 割り当て（想定）

- Whisper large-v3-turbo (CUDA): ~6GB
- Qwen2.5-7B-Instruct Q4_K_M: ~5GB
- **合計**: ~11GB (RTX 4070 Ti 12GB で動作)

## ビルド要件

### Windows

- Visual Studio 2022 (C++ ビルドツール)
- CMake
- LLVM (`winget install LLVM.LLVM`)
- CUDA Toolkit 12.x+ (GPU アクセラレーション使用時)

### 注意事項

- whisper-rs のビルドには bindgen が必要で、LLVM の libclang を使用
- `.cargo/config.toml` で `LIBCLANG_PATH` と `BINDGEN_EXTRA_CLANG_ARGS` を設定
- CUDA 統合: VS 2022 Build Tools インストール後、CUDA の MSBuildExtensions を手動コピーが必要な場合あり
