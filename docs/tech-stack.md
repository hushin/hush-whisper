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

## LLM・文章整形

| 領域         | 選定                                   | 理由                                          |
| ------------ | -------------------------------------- | --------------------------------------------- |
| LLM 連携方式 | HTTP API                               | リンクエラー回避、柔軟性                      |
| プロバイダー | Ollama / OpenAI 互換 API               | 複数の LLM ランタイムに対応                   |
| Ollama       | http://localhost:11434/api/generate    | ローカル LLM 実行、シンプルな API             |
| OpenAI 互換  | http://localhost:1234/v1/chat/completions | LM Studio, LocalAI, vLLM 等に対応         |

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

## VRAM 割り当て

- Whisper large-v3-turbo (CUDA): ~6GB
- LLM は外部プロセス（Ollama / LM Studio 等）で管理
  - VRAM 使用量は選択するモデルに依存
  - Whisper と LLM を同時実行する場合は VRAM 容量に注意

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
