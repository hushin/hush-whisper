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

## Phase 1: MVP（基本動作）✅ 完了

Phase 1 の全機能が実装完了しました。詳細は [`docs/done.md`](./done.md) を参照。

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

## 参考リソース

- [whisper-rs](https://github.com/tazz4843/whisper-rs) - Whisper.cpp バインディング
- [llama_cpp-rs](https://github.com/edgenai/llama_cpp-rs) - llama.cpp バインディング
- [Tauri 2.0 Docs](https://v2.tauri.app/) - フレームワークドキュメント
