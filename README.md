# HushWhisper

ローカル音声入力アプリ

## 🎯 主要機能

- グローバルショートカットで任意アプリ上に音声入力
- 音声認識（Whisper）
- LLM による文章整形（Ollama 連携、カスタマイズ可能）
- 完全ローカル処理（クラウド不要）
- システムトレイに常駐
- 履歴保存

## 対応 OS

- Windows

## インストール

1. [リリース](https://github.com/hushin/hush-whisper/releases) から `.msi` インストーラーをダウンロード
1. ダブルクリックしてインストーラを実行
1. Windows Defender が表示された場合：「詳細情報」→「実行」をクリック

## 使い方

1. アプリを起動
2. Whisper モデルを選択し、「モデルを読み込む」をクリック
3. ショートカット を押して録音開始
4. もう一度 ショートカット を押して録音停止
5. 認識結果が貼り付けられる

## スクリーンショット

![](image-1.png)

## 開発環境セットアップ

### 必要な環境

- Node.js 22+
- Rust 1.90+
- pnpm
- Visual Studio 2022 (C++ ビルドツール)
- CMake
- LLVM
- CUDA Toolkit 12.x+ (GPU アクセラレーション使用時)

```PowerShell
winget install --id Kitware.CMake -e
winget install --id LLVM.LLVM -e
winget install --id Microsoft.VisualStudio.2022.Community -e
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --quiet --wait"
```

```PowerShell
# 管理者権限の PowerShell で実行
Copy-Item "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.0\extras\visual_studio_integration\MSBuildExtensions\*" "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\MSBuild\Microsoft\VC\v170\BuildCustomizations\" -Force
```

### インストール

```bash
# 依存関係のインストール
pnpm install

# Cargo 設定ファイルをコピー（パスは環境に合わせて編集）
cp src-tauri/.cargo/config.toml.example src-tauri/.cargo/config.toml

# 開発モードで起動
pnpm tauri dev

# ビルド
pnpm tauri build
```

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
    config.toml.example  # bindgen 用の設定テンプレート
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
  model-management.md  # モデル管理
```

## License

[MIT](LICENSE)
