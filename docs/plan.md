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

## Phase 4: 最適化・高度な機能

**目標**: レイテンシー最適化、使い勝手向上

> **注**: LLM は Ollama 連携のため、LLM 関連の VRAM 管理・モデル管理は Ollama 側で行う

### 4.1 ストリーミング認識

- [ ] チャンク単位での Whisper 推論
- [ ] 中間結果のリアルタイム表示
- [ ] 確定結果の差分更新

### 4.2 Whisper モデル管理

- [ ] モデルダウンロード進捗表示の改善
- [ ] Whisper モデルの動的アンロード（VRAM 解放）
- [ ] 使用 VRAM モニタリング

### 4.3 設定項目

- [ ] ショートカットキーカスタマイズ
- [ ] 出力先選択（クリップボード/直接入力/ファイル）
- [ ] 言語設定（日本語固定 or 自動検出）
- [ ] 最大録音時間

### 4.4 コンテキスト入力

- [ ] 直前の文章をプロンプトに含める
- [ ] コンテキスト履歴の管理

### 4.5 Windows スタートアップ登録

- [ ] Windows スタートアップ登録（自動起動）

---

## 参考リソース

- [whisper-rs](https://github.com/tazz4843/whisper-rs) - Whisper.cpp バインディング
- [Ollama](https://ollama.ai/) - ローカル LLM 実行環境
- [Tauri 2.0 Docs](https://v2.tauri.app/) - フレームワークドキュメント
