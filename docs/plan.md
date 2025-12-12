# VoiceInput 実装計画

## 概要

Windows 向け SuperWhisper 風ローカル音声入力アプリ。

### 主要機能

1. グローバルショートカットで任意アプリ上から音声入力
2. 日本語音声認識（Whisper large-v3-turbo）
3. LLM による文脈理解・整形（カスタマイズ可能）
4. 完全オフライン処理
5. 音声入力ログ保存
6. モデル別途ダウンロード

### 技術スタック

| 領域      | 選定                | 理由                                    |
| --------- | ------------------- | --------------------------------------- |
| Framework | Tauri 2.0           | 軽量、Rust 統合、クロスプラットフォーム |
| Speech    | whisper-rs          | CUDA 対応、安定、GGML 形式              |
| LLM       | llama-cpp-rs        | GGUF 対応、成熟度高い                   |
| Model     | Qwen2.5-7B-Instruct | 日本語性能、128K コンテキスト           |
| Audio     | cpal + Silero VAD   | 低レイテンシー、音声区間検出            |
| Frontend  | Svelte 5            | 軽量、シンプル                          |

---

## Phase 1: MVP（基本動作）

**目標**: 録音開始/停止 で音声認識 → テキスト出力

### 1.1 プロジェクト初期化

```bash
pnpm create tauri-app voice-input --template svelte-ts
cd voice-input
```

**タスク**:

- [x] Tauri 2.0 プロジェクト作成
- [x] Cargo.toml 依存関係設定
- [x] 基本ディレクトリ構造作成

**Cargo.toml（初期）**:

```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
tauri-plugin-global-shortcut = "2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Audio
cpal = "0.15"

# Whisper (Phase 1) - CUDA enabled
whisper-rs = { version = "0.15", features = ["cuda"] }
# Note: Requires CMAKE_GENERATOR=Ninja and CMAKE_CUDA_FLAGS="-allow-unsupported-compiler" for VS 18
```

### 1.2 グローバルショートカット

**タスク**:

- [x] tauri-plugin-global-shortcut 設定
- [x] Ctrl+Space で 録音開始/停止

**実装済み**: `src-tauri/src/shortcuts/handler.rs`

### 1.3 音声キャプチャ

**タスク**:

- [x] cpal でマイク入力取得
- [x] 48kHz→16kHz リサンプリング（rubato）
- [x] f32 PCM データとしてバッファリング

**実装済み**: `src-tauri/src/audio/capture.rs`, `src-tauri/src/audio/resample.rs`

**注意点**:

- Whisper は 16kHz mono f32 を要求
- 最大録音時間 30 秒（初期設定）

### 1.4 Whisper 推論

**タスク**:

- [x] whisper-rs 初期化（CUDA版）
- [x] ggml-large-v3-turbo.bin 読み込み
- [x] 日本語指定で推論実行
- [x] 結果をフロントエンドへ送信

**実装済み**: `src-tauri/src/whisper/transcribe.rs`

**CUDA対応**:
- whisper-rs の `cuda` feature を有効化
- WhisperContextParameters で `use_gpu(true)` を設定
- RTX 4070 Ti (12GB VRAM) で動作確認済み

**ビルド環境設定** (`.cargo/config.toml`):
- `CMAKE_GENERATOR=Ninja`: Visual Studio 18 (2026) は cmake クレートでまだサポートされていないため
- `CMAKE_CUDA_FLAGS="-allow-unsupported-compiler"`: CUDA 13.0 は VS 2019-2022 のみ公式サポートのため

**モデルパス**: `%APPDATA%/voice-input/models/`

### 1.5 クリップボード出力

**タスク**:

- [x] 認識結果をクリップボードにコピー
- [ ] オプション: 自動ペースト（SendInput シミュレート）- 今後実装予定

**実装済み**: `src-tauri/src/clipboard/mod.rs`

### 1.6 基本 UI

**タスク**:

- [x] 録音インジケーター（波形 or アイコン）
- [x] 認識結果表示
- [ ] システムトレイ常駐 - 今後実装予定

**実装済み**: `src/routes/+page.svelte`

---

## Phase 2: 精度向上・LLM 統合

**目標**: VAD でハルシネーション抑制、LLM で文章整形

### 2.1 Silero VAD 統合

**タスク**:

- [ ] voice_activity_detector クレート追加
- [ ] 発話区間のみ Whisper へ送信
- [ ] 無音時の誤認識（「ご視聴ありがとう」等）防止

**追加依存**:

```toml
voice_activity_detector = "0.2"
```

### 2.2 LLM 後処理

**タスク**:

- [ ] llama-cpp-rs 設定（CUDA）
- [ ] Qwen2.5-7B-Instruct Q4_K_M ダウンロード
- [ ] 整形プロンプトテンプレート作成
- [ ] ストリーミング出力対応

**追加依存**:

```toml
llama_cpp = { git = "https://github.com/edgenai/llama_cpp-rs.git", features = ["cuda"] }
```

**プロンプト例**:

```
以下の音声認識結果を自然な日本語に整形してください。
誤字脱字の修正、句読点の追加、文法の修正を行ってください。

入力: {raw_text}

出力:
```

### 2.3 カスタムプロンプト

**タスク**:

- [ ] 設定画面でプロンプト編集可能に
- [ ] プリセット（議事録、メモ、チャット等）
- [ ] コンテキスト入力（直前の文章など）

### 2.4 ログ機能

**タスク**:

- [ ] SQLite or JSON でログ保存
- [ ] タイムスタンプ、元音声パス、認識結果、整形結果
- [ ] ログビューア UI

---

## Phase 3: 最適化・UX 改善

**目標**: レイテンシー最適化、使い勝手向上

### 3.1 ストリーミング認識

**タスク**:

- [ ] チャンク単位での Whisper 推論
- [ ] 中間結果のリアルタイム表示
- [ ] 確定結果の差分更新

### 3.2 VRAM 最適化

**タスク**:

- [ ] Whisper/LLM の動的ロード・アンロード
- [ ] INT8 量子化オプション
- [ ] 使用 VRAM モニタリング

### 3.3 モデル管理 UI

**タスク**:

- [ ] モデルダウンロード進捗表示
- [ ] モデルサイズ選択（tiny/base/small/medium/large）
- [ ] カスタム LLM モデル対応

**hf-hub によるダウンロード**:

```toml
hf-hub = { version = "0.3", features = ["tokio"] }
```

### 3.4 設定項目

- [ ] ショートカットキーカスタマイズ
- [ ] 出力先選択（クリップボード/直接入力/ファイル）
- [ ] 言語設定（日本語固定 or 自動検出）
- [ ] LLM 有効/無効切り替え
- [ ] 最大録音時間

### 3.5 自動起動・常駐

- [ ] Windows スタートアップ登録
- [ ] システムトレイメニュー
- [ ] バックグラウンド動作最適化

---

## ディレクトリ構造

```
voice-input/
├── CLAUDE.md
├── package.json
├── pnpm-lock.yaml
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
├── src/                          # Frontend
│   ├── app.html
│   ├── app.css
│   ├── lib/
│   │   ├── components/
│   │   │   ├── RecordingIndicator.svelte
│   │   │   ├── TranscriptionView.svelte
│   │   │   ├── Settings.svelte
│   │   │   └── ModelDownloader.svelte
│   │   └── stores/
│   │       ├── recording.ts
│   │       └── settings.ts
│   └── routes/
│       └── +page.svelte
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── build.rs
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── audio/
│       │   ├── mod.rs
│       │   ├── capture.rs         # cpal audio input
│       │   ├── resample.rs        # 48kHz→16kHz
│       │   └── vad.rs             # Silero VAD
│       ├── whisper/
│       │   ├── mod.rs
│       │   └── transcribe.rs      # whisper-rs wrapper
│       ├── llm/
│       │   ├── mod.rs
│       │   └── refine.rs          # llama-cpp-rs wrapper
│       ├── shortcuts/
│       │   ├── mod.rs
│       │   └── handler.rs         # Global hotkey
│       ├── models/
│       │   ├── mod.rs
│       │   └── download.rs        # hf-hub downloader
│       ├── clipboard/
│       │   └── mod.rs             # Clipboard operations
│       └── log/
│           ├── mod.rs
│           └── storage.rs         # Log persistence
└── docs/
    ├── plan.md                    # This file
    ├── audio-pipeline.md
    └── model-management.md
```

---

## VRAM 使用量見積もり（12GB 環境）

| コンポーネント   | モデル              | VRAM      |
| ---------------- | ------------------- | --------- |
| Whisper          | large-v3-turbo FP16 | ~6GB      |
| Whisper          | large-v3-turbo INT8 | ~3GB      |
| LLM              | Qwen2.5-7B Q4_K_M   | ~5GB      |
| Silero VAD       | -                   | ~50MB     |
| **合計（FP16）** |                     | **~11GB** |
| **合計（INT8）** |                     | **~8GB**  |

---

## レイテンシー目標

| 処理           | 目標              | 実現手段                |
| -------------- | ----------------- | ----------------------- |
| 音声キャプチャ | リアルタイム      | cpal 非同期ストリーム   |
| VAD            | <10ms/チャンク    | Silero ONNX             |
| Whisper 推論   | <500ms (3 秒音声) | CUDA, large-v3-turbo    |
| LLM 整形       | <1500ms           | Q4 量子化, 短プロンプト |
| **合計**       | **<3 秒**         |                         |

---

## 参考リソース

- [Epicenter Whispering](https://github.com/cjpais/epicenter-whispering) - Tauri + transcribe-rs 参考実装
- [whisper-rs](https://github.com/tazz4843/whisper-rs) - Whisper.cpp バインディング
- [llama_cpp-rs](https://github.com/edgenai/llama_cpp-rs) - llama.cpp バインディング
- [Tauri 2.0 Docs](https://v2.tauri.app/) - フレームワークドキュメント

---

## マイルストーン

| Phase   | 期間目安 | 成果物                 |
| ------- | -------- | ---------------------- |
| Phase 1 | 1-2 週間 | 基本音声入力動作       |
| Phase 2 | 1-2 週間 | VAD+LLM 統合、ログ機能 |
| Phase 3 | 1-2 週間 | 最適化、設定 UI 完成   |
