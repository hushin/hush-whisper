# 実装済み機能

## Phase 1: MVP（基本動作）✅

Phase 1 の全機能が実装完了しました。

### 1.1 プロジェクト初期化 ✅

- Tauri 2.0 プロジェクト作成
- Cargo.toml 依存関係設定
- 基本ディレクトリ構造作成

**実装**: プロジェクトルート

### 1.2 グローバルショートカット ✅

- tauri-plugin-global-shortcut 設定
- Ctrl+Space で録音開始/停止

**実装**: `src-tauri/src/shortcuts/handler.rs`

### 1.3 音声キャプチャ ✅

- cpal でマイク入力取得
- 48kHz→16kHz リサンプリング（rubato）
- f32 PCM データとしてバッファリング
- Stereo → Mono 変換

**実装**:

- `src-tauri/src/audio/capture.rs`
- `src-tauri/src/audio/resample.rs`

**注意点**:

- Whisper は 16kHz mono f32 を要求
- 最大録音時間の制限なし（現在）

### 1.4 Whisper 推論 ✅

- whisper-rs 初期化（CUDA 版）
- ggml-large-v3-turbo.bin 読み込み
- 日本語指定で推論実行
- 結果をフロントエンドへ送信

**実装**: `src-tauri/src/whisper/transcribe.rs`

**CUDA 対応**:

- whisper-rs の `cuda` feature を有効化
- RTX 4070 Ti (12GB VRAM) で動作確認済み

**ビルド環境設定** (`.cargo/config.toml`):

- `LIBCLANG_PATH`: LLVM の libclang パス（bindgen が使用）
- `BINDGEN_EXTRA_CLANG_ARGS`: VS のヘッダパスを指定（stdbool.h 等の解決用）

### 1.5 クリップボード出力 ✅

- 認識結果をクリップボードにコピー（arboard）
- 自動ペースト（enigo で Ctrl+V シミュレート）

**実装**: `src-tauri/src/clipboard/mod.rs`

### 1.6 基本 UI ✅

- 録音インジケーター（波形 or アイコン）
- 認識結果表示

**実装**: `src/routes/+page.svelte`

**未実装**: システムトレイ常駐（Phase 3 で実装予定）

---

## 実装されたワークフロー

```
┌────────────────────────────────────────────────────────┐
│ ユーザーが Ctrl+Space を押下                            │
└──────────────────┬─────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ AudioCapture: マイク入力開始 (cpal)                     │
│ - 任意サンプルレート (通常 48kHz)                       │
│ - Stereo → Mono 変換                                   │
└──────────────────┬─────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ ユーザーが再度 Ctrl+Space を押下                        │
└──────────────────┬─────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ AudioCapture: 録音停止、バッファ取得                    │
└──────────────────┬─────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ Resampler: 16kHz にリサンプリング (rubato)              │
└──────────────────┬─────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ Whisper: 音声認識実行 (CUDA)                            │
│ - Model: large-v3-turbo                                │
│ - Language: Japanese                                   │
└──────────────────┬─────────────────────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────┐
│ Clipboard: 結果をクリップボードにコピー                 │
│ - arboard: クリップボード操作                           │
│ - enigo: Ctrl+V で自動ペースト                         │
└────────────────────────────────────────────────────────┘
```

---

## 技術的成果

### CUDA アクセラレーション

- Whisper 推論を GPU で高速化
- RTX 4070 Ti で動作確認済み

### クロスプラットフォーム設計

- Tauri 2.0 により Windows/macOS/Linux 対応可能な基盤
- 現在は Windows をターゲットとして開発

### 低レイテンシー

- cpal による低遅延音声キャプチャ
- リアルタイムリサンプリング
- CUDA による高速推論

---

## Phase 2: UX 改善 ✅

### 2.1 システムトレイ・状態表示 ✅

- システムトレイアイコン常駐
- 録音中は赤い丸を表示（視覚的フィードバック）
- トレイメニュー（ウィンドウを表示、終了）
- 左クリックでウィンドウ表示
- ウィンドウ閉じボタンでシステムトレイに常駐（アプリ終了しない）

**実装**: `src-tauri/src/tray/`

### 2.2 モデル選択の永続化 ✅

- デフォルトモデルを ggml-large-v3-turbo.bin に変更
- ユーザーが選択したモデルを設定ファイルに保存
- 次回起動時に前回選択したモデルを自動ロード
- 設定 UI でモデル選択を保存

**実装**:

- 設定ファイル: `%APPDATA%/voice-input/config.json`
- Rust側: `src-tauri/src/config/mod.rs` で設定の読み書き
- Tauriコマンド: `get_settings`, `save_model_selection`
- フロントエンド: 起動時に設定読み込み、モデル初期化時に保存

---

## Phase 3: 精度向上・LLM 統合 ✅

### 3.1 Silero VAD 統合 ✅

- voice_activity_detector クレート追加
- 発話区間のみ Whisper へ送信
- 無音時の誤認識（「ご視聴ありがとう」等）防止

**実装**:

- `src-tauri/src/audio/vad.rs` - VAD 処理モジュール
- Silero VAD V5 モデルで発話区間を検出
- 512 サンプル (32ms) ごとに判定、パディング付き

### 3.2 LLM 後処理（Ollama / OpenAI 互換 API 連携）✅

> **注意**: whisper-rs-sys と llama-cpp-sys-2 の両方が ggml ライブラリを静的リンクするため、
> リンク時に重複シンボルエラー（LNK2005）が発生する問題を回避するため、HTTP API 経由で外部 LLM と連携。

**対応プロバイダー**:

- **Ollama**: ローカル LLM 実行環境
  - デフォルト URL: `http://localhost:11434`
  - API エンドポイント: `/api/generate`
- **OpenAI 互換 API**: LM Studio, LocalAI, vLLM 等
  - デフォルト URL: `http://localhost:1234`
  - API エンドポイント: `/v1/chat/completions`

**実装**:

- `src-tauri/src/llm/mod.rs` - LLM クライアントモジュール
- HTTP API 経由でリクエスト
- 設定で LLM 有効/無効切り替え
- 設定でプロバイダー選択、URL・モデル名変更可能

### 3.3 カスタムプロンプト ✅

- 設定画面でプロンプト編集可能
- プリセット（自然整形、議事録、メモ、チャット）
- カスタムプロンプト入力

**実装**:

- `src-tauri/src/config/mod.rs` - PromptPreset enum とプリセット定義
- フロントエンド: プリセット選択ドロップダウン、カスタムプロンプト編集 UI
- 設定ファイルでプロンプト設定を永続化

### 3.4 ログ機能 ✅

- JSON でログ保存（日付ごとにファイル分割）
- タイムスタンプ、認識結果、整形結果、使用プリセットを保存
- ログビューア UI（履歴表示、詳細表示、削除機能）

**実装**:

- `src-tauri/src/log/mod.rs` - ログ管理モジュール
- ログ保存先: `%APPDATA%/voice-input/logs/YYYY-MM-DD.json`
- フロントエンド: 履歴セクション

---

## Phase 4: UX 改善・最適化 ✅

**目標**: 使い勝手向上、柔軟性強化

> **注**: LLM は Ollama 連携のため、LLM 関連の管理は Ollama 側で行う

### 4.1 優先タスク ✅

#### ショートカットキーカスタマイズ ✅

- 設定画面でショートカットキーを変更可能
- デフォルト: Ctrl+Space

**実装**:

- `src-tauri/src/config/mod.rs` - 設定でショートカットキーを保存
- `src-tauri/src/shortcuts/handler.rs` - 動的ショートカット登録
- フロントエンド: ショートカットキー設定 UI

#### 直接入力モード ✅

- クリップボードに残さず直接ペースト
- 出力先選択（クリップボードのみ / 直接入力 / 両方）

**実装**:

- `src-tauri/src/config/mod.rs` - OutputMode enum（Clipboard / DirectInput / Both）
- `src-tauri/src/clipboard/mod.rs` - 出力モードに応じた処理
- フロントエンド: 出力先選択 UI

#### Whisper モデル切り替え ✅

- 初期化済みでも別モデルを選択可能
- モデル切り替え時に前モデルをアンロード（VRAM 解放）

**実装**:

- `src-tauri/src/whisper/transcribe.rs` - モデル再初期化ロジック
- フロントエンド: モデル切り替え UI

#### 履歴の自動更新 ✅

- 録音完了時に履歴セクションを自動更新
- 手動更新ボタン不要

**実装**:

- Tauri イベントシステムで録音完了を通知
- フロントエンド: イベントリスナーで自動更新

### 4.2 追加機能 ✅

#### 設定項目 ✅

- 最大録音時間（デフォルト 5 分）

**実装**:

- `src-tauri/src/config/mod.rs` - max_recording_duration 設定
- `src-tauri/src/audio/capture.rs` - 録音時間チェック
- フロントエンド: 最大録音時間設定 UI

#### Windows スタートアップ登録 ✅

- Windows スタートアップ登録（自動起動）

**実装**:

- `src-tauri/src/startup/mod.rs` - スタートアップ登録/解除
- Windows レジストリ操作（`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`）
- フロントエンド: スタートアップ設定トグル
