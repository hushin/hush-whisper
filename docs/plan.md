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

---

## 完了済み

- **Phase 1: MVP** - 詳細は [`docs/done.md`](./done.md) を参照
- **Phase 2: UX 改善** - 詳細は [`docs/done.md`](./done.md) を参照
- **Phase 3: 精度向上・LLM 統合** - 詳細は [`docs/done.md`](./done.md) を参照

---

## Phase 4: 最適化・高度な機能

**目標**: レイテンシー最適化、モデル管理強化

### 4.1 ストリーミング認識

**タスク**:

- [ ] チャンク単位での Whisper 推論
- [ ] 中間結果のリアルタイム表示
- [ ] 確定結果の差分更新

### 4.2 VRAM 最適化

**タスク**:

- [ ] Whisper/LLM の動的ロード・アンロード
- [ ] INT8 量子化オプション
- [ ] 使用 VRAM モニタリング

### 4.3 モデル管理 UI

**タスク**:

- [ ] モデルダウンロード進捗表示
- [ ] モデルサイズ選択（tiny/base/small/medium/large）
- [ ] カスタム LLM モデル対応

**hf-hub によるダウンロード**:

```toml
hf-hub = { version = "0.3", features = ["tokio"] }
```

### 4.4 設定項目

- [ ] ショートカットキーカスタマイズ
- [ ] 出力先選択（クリップボード/直接入力/ファイル）
- [ ] 言語設定（日本語固定 or 自動検出）
- [ ] 最大録音時間

### 4.5 コンテキスト入力

- [ ] 直前の文章をプロンプトに含める
- [ ] コンテキスト履歴の管理

### 4.6 Windows スタートアップ登録

- [ ] Windows スタートアップ登録（自動起動）

---

## 参考リソース

- [whisper-rs](https://github.com/tazz4843/whisper-rs) - Whisper.cpp バインディング
- [llama_cpp-rs](https://github.com/edgenai/llama_cpp-rs) - llama.cpp バインディング
- [Tauri 2.0 Docs](https://v2.tauri.app/) - フレームワークドキュメント
