# モデル管理

## 概要

アプリ本体とモデルを分離し、必要なモデルのみダウンロード。バンドルサイズ削減とモデル更新の柔軟性を確保。

**現状**: モデルは手動でダウンロード・配置する必要があります。
**Phase 2**: UI からモデルをダウンロード・管理できるようにする予定です。

---

## モデル保存場所

```
%APPDATA%/voice-input/
├── models/
│   ├── whisper/
│   │   ├── ggml-large-v3-turbo.bin    # ~1.6GB
│   │   ├── ggml-medium.bin             # ~1.5GB (optional)
│   │   └── ggml-small.bin              # ~466MB (optional)
│   └── llm/  (Phase 2 で実装予定)
│       ├── qwen2.5-7b-instruct-q4_k_m.gguf  # ~4.7GB
│       └── qwen2.5-3b-instruct-q4_k_m.gguf  # ~2.0GB (optional)
├── config.json
└── logs/
```

---

## 対応予定のモデル

### Whisper モデル

| モデル名       | サイズ | 説明                | 用途               |
| -------------- | ------ | ------------------- | ------------------ |
| Large v3 Turbo | ~1.6GB | 最高精度、CUDA 推奨 | デフォルト         |
| Medium         | ~1.5GB | バランス型          | 中程度の精度と速度 |
| Small          | ~466MB | 軽量、CPU 可        | 低 VRAM 環境       |

**リポジトリ**: `ggerganov/whisper.cpp`

### LLM モデル（Phase 2）

| モデル名            | サイズ | 説明                            | 用途         |
| ------------------- | ------ | ------------------------------- | ------------ |
| Qwen2.5 7B Instruct | ~4.7GB | 日本語高精度、128K コンテキスト | デフォルト   |
| Qwen2.5 3B Instruct | ~2.0GB | 軽量、低 VRAM 環境向け          | 低 VRAM 環境 |

**リポジトリ**: `Qwen/Qwen2.5-*-Instruct-GGUF`

---

## 実装計画（Phase 2）

### 1. モデルダウンロード機能

**ライブラリ**: `hf-hub` (HuggingFace 公式クライアント)

#### 主要機能

- HuggingFace Hub からモデルを自動ダウンロード
- ダウンロード進捗の表示
- キャッシュ管理
- モデルの検証（ハッシュチェックなど）

#### UI フロー（予定）

```
┌─────────────────────────────────────────┐
│           Welcome to VoiceInput         │
├─────────────────────────────────────────┤
│                                         │
│  音声認識モデルをダウンロードしてください  │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │ ◉ Large v3 Turbo (推奨)        │   │
│  │   1.6GB - 最高精度              │   │
│  ├─────────────────────────────────┤   │
│  │ ○ Medium                        │   │
│  │   1.5GB - バランス型            │   │
│  ├─────────────────────────────────┤   │
│  │ ○ Small                         │   │
│  │   466MB - 軽量                  │   │
│  └─────────────────────────────────┘   │
│                                         │
│  □ LLMによる文章整形を有効にする         │
│    (追加で4.7GBダウンロード)            │
│                                         │
│           [ダウンロード開始]             │
│                                         │
└─────────────────────────────────────────┘
```

### 2. モデル管理 UI

#### 機能

- ダウンロード済みモデルの一覧表示
- モデルの削除
- モデルの切り替え
- ディスク使用量の表示

#### 設定項目

- 選択中の Whisper モデル
- 選択中の LLM モデル（オプション）
- LLM の有効/無効切り替え
- モデル保存先ディレクトリ

---

## 現在のモデル配置方法（手動）

Phase 2 実装までは、以下の手順で手動配置：

### 1. モデルのダウンロード

#### Whisper モデル

HuggingFace から直接ダウンロード：

- [ggerganov/whisper.cpp](https://huggingface.co/ggerganov/whisper.cpp/tree/main)
  - `ggml-large-v3-turbo.bin` を推奨

### 2. 配置

ダウンロードしたモデルを以下の場所に配置：

```
%APPDATA%/voice-input/models/whisper/ggml-large-v3-turbo.bin
```

または、アプリの UI でモデルパスを指定。

---

## 技術的な考慮事項

### VRAM 管理

- Whisper large-v3-turbo: ~6GB
- Qwen2.5-7B Q4_K_M: ~5GB
- **合計**: ~11GB（RTX 4070 Ti 12GB で動作）

### ディスク容量

- 最小構成（Whisper Small）: ~500MB
- 推奨構成（Large v3 Turbo）: ~1.6GB
- フル構成（Whisper + LLM）: ~6.3GB

### ダウンロード時間（目安）

- 100Mbps 接続の場合
  - Whisper Large v3 Turbo: ~2 分
  - Qwen2.5 7B: ~6 分

---

## 参考リソース

### HuggingFace モデル URL

| モデル                 | リポジトリ                    | ファイル                        |
| ---------------------- | ----------------------------- | ------------------------------- |
| Whisper Large v3 Turbo | ggerganov/whisper.cpp         | ggml-large-v3-turbo.bin         |
| Whisper Medium         | ggerganov/whisper.cpp         | ggml-medium.bin                 |
| Whisper Small          | ggerganov/whisper.cpp         | ggml-small.bin                  |
| Qwen2.5 7B             | Qwen/Qwen2.5-7B-Instruct-GGUF | qwen2.5-7b-instruct-q4_k_m.gguf |
| Qwen2.5 3B             | Qwen/Qwen2.5-3B-Instruct-GGUF | qwen2.5-3b-instruct-q4_k_m.gguf |

### ライブラリ

- [hf-hub](https://docs.rs/hf-hub/) - HuggingFace Hub クライアント（Rust）
- [HuggingFace Hub](https://huggingface.co/) - モデルリポジトリ
