# HushWhisper

Windows 向けローカル音声入力アプリ。グローバルショートカットで任意のアプリに日本語音声入力。

## Tech Stack

- **Framework**: WinUI 3 / Windows App SDK (C# / .NET 9, unpackaged)
- **UI パターン**: MVVM (CommunityToolkit.Mvvm) + DI (Microsoft.Extensions.DependencyInjection)
- **Speech Recognition**: Whisper.net (whisper.cpp bindings, CUDA 13/12, Vulkan, CPU を自動選択)
- **VAD**: Whisper.net 内蔵 Silero VAD
- **Audio**: NAudio (WASAPI + WDL リサンプラー)
- **UI Shell**: MicaBackdrop + NavigationView
- **LLM**: Ollama / OpenAI 互換 API (LM Studio, LocalAI 等)

## Project Structure

```
HushWhisper/          # WinUI 3 アプリ本体
  Interop/            # Win32 P/Invoke
  Models/             # 設定・モデル定義・ログ型
  Services/           # 音声, ASR, VAD, LLM, 出力, ホットキー, トレイ, ログ, 設定
  ViewModels/         # 各ページの ViewModel
  Views/              # 各設定ページ (XAML)
  Assets/             # アイコン
docs/                 # 実装計画・詳細 (必要に応じて参照)
```

## Commands

```bash
# ビルド (ビルドチェックはこれを使用)
dotnet build HushWhisper/HushWhisper.csproj -c Debug -p:Platform=x64

# 実行
dotnet run --project HushWhisper/HushWhisper.csproj -c Debug -p:Platform=x64

# リリースビルド
dotnet publish HushWhisper/HushWhisper.csproj -c Release -r win-x64 -p:Platform=x64

# フォーマット
dotnet format HushWhisper/HushWhisper.csproj
```

## Key Constraints

- Whisper: ~6GB VRAM (CUDA, large-v3-turbo)。VAD も同プロセスで GPU を使う
- LLM は外部プロセスで管理（Ollama / LM Studio 等）
- No Python runtime
- Models downloaded separately (Whisper: HuggingFace, VAD: Whisper.net downloader)
- Target latency: <3 seconds end-to-end
- 設定形式は `%APPDATA%/voice-input/config.json`（旧 Tauri 版と互換）
- `LangVersion` は MVVM Toolkit 8.4 の部分プロパティ生成のため `preview` を指定

## Documentation

- `docs/plan.md` - 実装計画
- `docs/done.md` - 実装済み機能
- `docs/tech-stack.md` - 技術スタックと選定理由
