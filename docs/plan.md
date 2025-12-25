# VoiceInput 実装計画

## 概要

Windows 向け SuperWhisper 風ローカル音声入力アプリ。

### 主要機能

1. グローバルショートカットで任意アプリ上から音声入力
2. 日本語音声認識（Whisper large-v3-turbo、CUDA）
3. LLM による文章整形（Ollama 連携、カスタマイズ可能）
4. 完全ローカル処理（クラウド不要）
5. 音声入力ログ保存
6. Whisper モデル自動ダウンロード

---

## 完了済み

- **Phase 1: MVP** - 詳細は [`docs/done.md`](./done.md) を参照
- **Phase 2: UX 改善** - 詳細は [`docs/done.md`](./done.md) を参照
- **Phase 3: 精度向上・LLM 統合** - 詳細は [`docs/done.md`](./done.md) を参照

---

## Phase 4: UX 改善・最適化

**目標**: 使い勝手向上、柔軟性強化

> **注**: LLM は Ollama 連携のため、LLM 関連の管理は Ollama 側で行う

### 4.1 優先タスク (完了)

#### ショートカットキーカスタマイズ

- [x] 設定画面でショートカットキーを変更可能に
- [x] デフォルト: Ctrl+Space

#### 直接入力モード

- [x] クリップボードに残さず直接ペースト
- [x] 出力先選択（クリップボードのみ / 直接入力 / 両方）

#### Whisper モデル切り替え

- [x] 初期化済みでも別モデルを選択可能に
- [x] モデル切り替え時に前モデルをアンロード（VRAM 解放）

#### 履歴の自動更新

- [x] 録音完了時に履歴セクションを自動更新
- [x] 手動更新ボタン不要に

---

### 4.2 追加機能

#### 設定項目

- [x] 最大録音時間（デフォルト 5 分）

#### Windows スタートアップ登録

- [ ] Windows スタートアップ登録（自動起動）

---

## 参考リソース

- [whisper-rs](https://github.com/tazz4843/whisper-rs) - Whisper.cpp バインディング
- [Ollama](https://ollama.ai/) - ローカル LLM 実行環境
- [Tauri 2.0 Docs](https://v2.tauri.app/) - フレームワークドキュメント
