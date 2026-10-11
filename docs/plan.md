# HushWhisper 実装計画

## 概要

Windows 向けローカル音声入力アプリ

### 主要機能

1. グローバルショートカットで任意アプリ上から音声入力
2. 日本語音声認識（Whisper large-v3-turbo、CUDA）
3. LLM による文章整形（Ollama / LM Studio 等の OpenAI 互換 API 対応、カスタマイズ可能）
4. 完全ローカル処理（クラウド不要）
5. 音声入力ログ保存
6. Whisper モデル自動ダウンロード

---

## 完了済み

- **Phase 1: MVP** - 詳細は [`docs/done.md`](./done.md) を参照
- **Phase 2: UX 改善** - 詳細は [`docs/done.md`](./done.md) を参照
- **Phase 3: 精度向上・LLM 統合** - 詳細は [`docs/done.md`](./done.md) を参照
- **Phase 4: UX 改善・最適化** - 詳細は [`docs/done.md`](./done.md) を参照
- **Phase 5: WinUI 3 移植** - Tauri (Rust + Svelte) から WinUI 3 + Windows App SDK へ移植。詳細は [`docs/done.md`](./done.md) を参照

---

## 参考リソース

- [Whisper.net](https://github.com/sandrohanea/whisper.net) - whisper.cpp の .NET バインディング
- [Windows App SDK](https://learn.microsoft.com/windows/apps/windows-app-sdk/) - WinUI 3 プラットフォーム
- [CommunityToolkit.Mvvm](https://learn.microsoft.com/dotnet/communitytoolkit/mvvm/) - MVVM Toolkit
- [Ollama](https://ollama.ai/) - ローカル LLM 実行環境
