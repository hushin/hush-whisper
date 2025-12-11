# VoiceInput

Windows向けローカル音声入力アプリ（開発中）

## 現在の状況

Phase 1（基本動作）の実装が完了しました。

### ✅ 実装済み機能

- **グローバルショートカット**: Ctrl+Space で録音開始/停止
- **音声キャプチャ**: cpal でマイク入力取得
- **リサンプリング**: 48kHz → 16kHz 変換（Whisper用）
- **Whisper音声認識**: whisper-rs による日本語音声認識
- **クリップボード**: 認識結果の自動コピー
- **基本UI**: 録音インジケーター、結果表示

## セットアップ

### 必要な環境

- Node.js 18+
- Rust 1.70+
- pnpm
- Visual Studio 2022 (C++ ビルドツール)
- **Ninja** (CMake generator)
- CMake

### Ninja のインストール

```bash
# winget
winget install Ninja-build.Ninja

# または scoop
scoop install ninja
```

### Whisper モデルのダウンロード

```bash
# モデルディレクトリを作成
mkdir -p %APPDATA%\voice-input\models

# Hugging Face からモデルをダウンロード（例: large-v3-turbo）
# https://huggingface.co/ggerganov/whisper.cpp/tree/main
# ggml-large-v3-turbo.bin を上記ディレクトリに配置
```

### インストール

```bash
# 依存関係のインストール
pnpm install

# 開発モードで起動
pnpm tauri dev

# ビルド
pnpm tauri build
```

## 使い方

1. アプリを起動
2. Whisper モデルのパスを入力し「モデルを読み込む」をクリック
3. Ctrl+Space を押して録音開始
4. もう一度 Ctrl+Space を押して録音停止
5. 認識結果がクリップボードに自動コピーされます

## 技術スタック

- **Framework**: Tauri 2.0
- **Frontend**: Svelte 5 + TypeScript
- **Audio**: cpal + rubato
- **Speech Recognition**: whisper-rs (whisper.cpp)
- **Clipboard**: arboard

## ディレクトリ構造

```
src-tauri/
  .cargo/
    config.toml  # CMAKE_GENERATOR=Ninja の設定
  src/
    audio/       # 音声キャプチャ・リサンプリング
    clipboard/   # クリップボード操作
    shortcuts/   # グローバルホットキー
    whisper/     # 音声認識
src/             # Svelte フロントエンド
docs/            # 実装計画・詳細
```

## 次のステップ (Phase 2)

1. VAD（Voice Activity Detection）の実装
2. LLM統合による文章整形
3. システムトレイ常駐
4. 設定画面の実装

詳細は `docs/plan.md` を参照してください。

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
