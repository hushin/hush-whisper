# VoiceInput

Windows 向けローカル音声入力アプリ。グローバルショートカットで任意のアプリに日本語音声入力。

## 🎯 主要機能

### ✅ 実装済み（Phase 1 完了）

- **グローバルショートカット**: Ctrl+Space で録音開始/停止
- **音声キャプチャ**: cpal でマイク入力取得、Stereo → Mono 変換
- **リサンプリング**: 任意サンプルレート → 16kHz 変換（Whisper 用）
- **Whisper 音声認識**: whisper-rs (CUDA) による日本語音声認識
- **クリップボード**: 認識結果の自動コピー・ペースト
- **基本 UI**: 録音インジケーター、結果表示

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
5. 認識結果がクリップボードに自動コピーされ、貼り付けされます

## 技術スタック

- **Framework**: Tauri 2.0 (Rust + Svelte)
- **Frontend**: Svelte 5 + TypeScript
- **Audio**: cpal (キャプチャ) + rubato (リサンプリング)
- **Speech Recognition**: whisper-rs (whisper.cpp bindings, CUDA 対応)
- **Clipboard**: arboard (コピー) + enigo (自動ペースト)

詳細は [`docs/tech-stack.md`](docs/tech-stack.md) を参照。

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
  plan.md        # 実装計画
  done.md        # 実装済み機能
  tech-stack.md  # 技術スタックと選定理由
  audio-pipeline.md  # 音声処理パイプライン
  model-management.md  # モデル管理（Phase 2 で実装予定）
```
