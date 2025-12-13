# 音声処理パイプライン

## 概要

現在の音声処理フローは以下の通り：

```
┌─────────────┐    ┌──────────────┐    ┌───────────┐    ┌─────────────┐
│ Microphone  │───▶│  Resample    │───▶│  Silero   │───▶│  whisper-rs │
│   (cpal)    │    │ 48k→16k mono │    │    VAD    │    │   (CUDA)    │
└─────────────┘    └──────────────┘    └───────────┘    └─────────────┘
     任意Hz             16kHz            speech           transcription
  stereo/mono          mono             chunks
```

---

## 1. 音声キャプチャ (cpal)

### 実装概要

**ファイル**: `src-tauri/src/audio/capture.rs`

cpal を使用してマイク入力を取得。以下の機能を実装：

- デフォルト入力デバイスの自動選択
- 複数のサンプルフォーマットに対応（F32, I16, U16）
- Stereo → Mono 変換（各チャンネルの平均）
- バッファリング

### 設計のポイント

1. **サンプルフォーマットの正規化**

   - すべての入力を f32 に変換して処理を統一
   - デバイス依存の差異を吸収

2. **Mono 変換**

   - Whisper は mono 音声を要求
   - Stereo の場合は各フレームの平均値を計算

3. **エラーハンドリング**
   - デバイスが見つからない場合のエラー処理
   - ストリームエラーのロギング

### 使用例

```rust
let capture = AudioCapture::new()?;
let stream = capture.start_recording()?;

// ... 録音中 ...

drop(stream); // ストリームを停止
let samples = capture.stop_recording(); // バッファを取得
```

---

## 2. リサンプリング (rubato)

### 実装概要

**ファイル**: `src-tauri/src/audio/resample.rs`

Whisper は **16kHz mono f32** を要求するため、一般的なマイク（48kHz など）の出力をリサンプリング。

### 設計のポイント

1. **高品質リサンプリング**

   - `SincFixedIn` を使用（Sinc 補間）
   - BlackmanHarris2 ウィンドウ関数で高品質な変換

2. **同一サンプルレートの最適化**

   - 入力と出力が同じ場合はリサンプリングをスキップ

3. **柔軟な設計**
   - 任意のサンプルレート変換に対応
   - チャンク単位での処理が可能

### パラメータ

| パラメータ          | 値              | 説明                     |
| ------------------- | --------------- | ------------------------ |
| sinc_len            | 256             | Sinc フィルタの長さ      |
| f_cutoff            | 0.95            | カットオフ周波数         |
| interpolation       | Linear          | 補間タイプ               |
| oversampling_factor | 256             | オーバーサンプリング係数 |
| window              | BlackmanHarris2 | ウィンドウ関数           |

### 使用例

```rust
let resampler = Resampler::new(16000); // 16kHz にリサンプリング
let output = resampler.resample(&input, 48000)?; // 48kHz から変換
```

---

## 3. Voice Activity Detection (VAD)

無音区間を除去して Whisper への入力を最適化。ハルシネーション防止にも効果的。

### 実装概要

**ファイル**: `src-tauri/src/audio/vad.rs`

- **ライブラリ**: `voice_activity_detector` クレート (v0.2)
- **モデル**: Silero VAD V5
- **処理**: 発話区間のみを Whisper に送信

### パラメータ

| パラメータ       | 値    | 説明                                |
| ---------------- | ----- | ----------------------------------- |
| sample_rate      | 16000 | 入力サンプルレート                  |
| chunk_size       | 512   | 処理チャンクサイズ (32ms)           |
| threshold        | 0.5   | 発話判定閾値                        |
| padding_chunks   | 3     | 発話前後のパディング (~96ms)        |

### 処理フロー

1. 16kHz にリサンプリングされた音声を 512 サンプルごとに分割
2. 各チャンクの発話確率を Silero VAD で推定
3. 確率 > 0.5 のチャンクを Speech として抽出
4. 80% 以上が発話の場合は元音声をそのまま使用（品質保持）
5. 発話が検出されない場合は空配列を返す（ハルシネーション防止）

### 効果

1. **ハルシネーション防止**

   - 無音時の誤認識（「ご視聴ありがとう」等）を防止

2. **処理効率化**

   - 不要な音声データを除外して推論時間を短縮

3. **精度向上**
   - 音声区間に集中することで認識精度が向上

---

## パフォーマンス考慮事項

1. **バッファサイズ**

   - cpal のデフォルトバッファサイズで安定性とレイテンシーのバランスを確保

2. **メモリ使用量**

   - 長時間録音時のバッファサイズに注意
   - 必要に応じて最大録音時間の制限を実装

3. **CPU/GPU 使用率**
   - リサンプリングは CPU で処理
   - Whisper 推論は GPU (CUDA) で高速化

---

## 関連ファイル

- `src-tauri/src/audio/capture.rs` - 音声キャプチャ
- `src-tauri/src/audio/resample.rs` - リサンプリング
- `src-tauri/src/audio/vad.rs` - Voice Activity Detection
- `src-tauri/src/audio/mod.rs` - モジュール定義
- `src-tauri/src/whisper/transcribe.rs` - Whisper 推論

---

## 参考リソース

- [cpal Documentation](https://docs.rs/cpal/) - クロスプラットフォーム音声 I/O
- [rubato Documentation](https://docs.rs/rubato/) - 高品質リサンプリング
- [voice_activity_detector](https://docs.rs/voice_activity_detector/) - Silero VAD V5 Rust バインディング
- [Silero VAD](https://github.com/snakers4/silero-vad) - 音声活動検出モデル
