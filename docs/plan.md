# VoiceInput 実装計画

## 概要

Windows 向けローカル音声入力アプリ

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
- **Phase 4: UX 改善・最適化** - 詳細は [`docs/done.md`](./done.md) を参照

---

## 参考リソース

- [whisper-rs](https://github.com/tazz4843/whisper-rs) - Whisper.cpp バインディング
- [Ollama](https://ollama.ai/) - ローカル LLM 実行環境
- [Tauri 2.0 Docs](https://v2.tauri.app/) - フレームワークドキュメント
