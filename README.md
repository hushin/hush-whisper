# VoiceInput

Windows向けローカル音声入力アプリ（開発中）

## 現在の状況

Phase 1（基本動作）の実装を進行中です。

### ✅ 実装済み機能

- **グローバルショートカット**: Ctrl+Space で録音開始/停止
- **音声キャプチャ**: cpal でマイク入力取得
- **リサンプリング**: 48kHz → 16kHz 変換（Whisper用）
- **クリップボード**: 認識結果の自動コピー
- **基本UI**: 録音インジケーター、結果表示

### ⏸️ 保留中

- **Whisper統合**: Visual Studio 18の cmake サポート問題により一時保留
  - 現在はダミーモードで動作確認可能
  - cmake問題解決後に実装予定

## セットアップ

### 必要な環境

- Node.js 18+
- Rust 1.70+
- pnpm
- Visual Studio 2022 (C++ ビルドツール)

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
2. 「モデルを読み込む」をクリック（現在はダミーモード）
3. Ctrl+Space を押して録音開始
4. もう一度 Ctrl+Space を押して録音停止
5. 認識結果（現在はダミーテキスト）がクリップボードにコピーされます

## 技術スタック

- **Framework**: Tauri 2.0
- **Frontend**: Svelte 5 + TypeScript
- **Audio**: cpal + rubato
- **Speech Recognition**: whisper-rs (実装予定)
- **Clipboard**: arboard

## ディレクトリ構造

```
src-tauri/
  src/
    audio/       # 音声キャプチャ・リサンプリング
    clipboard/   # クリップボード操作
    shortcuts/   # グローバルホットキー
    whisper/     # 音声認識（実装予定）
src/             # Svelte フロントエンド
docs/            # 実装計画・詳細
```

## 次のステップ

1. cmake問題の解決とWhisper統合
2. VAD（Voice Activity Detection）の実装
3. LLM統合による文章整形
4. システムトレイ常駐

詳細は `docs/plan.md` を参照してください。

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
