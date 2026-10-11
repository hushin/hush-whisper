# 技術スタック

## フレームワーク・言語

| 領域      | 選定                            | 理由                                            |
| --------- | ------------------------------- | ----------------------------------------------- |
| Framework | WinUI 3 / Windows App SDK       | Windows ネイティブ UI、Mica/Fluent、単一言語(C#) |
| Language  | C# / .NET 9                     | 型安全、非同期処理、Win32 相互運用              |
| 配布形態  | Unpackaged (self-contained)     | ホットキー/トレイ/自動起動が実装容易、署名不要  |

> 旧実装は Tauri 2.0 (Rust + Svelte) だったが、WinUI 3 + Windows App SDK へ移植した。

## アーキテクチャ

| 領域        | 選定                                     | 理由                             |
| ----------- | ---------------------------------------- | -------------------------------- |
| UI パターン | MVVM (CommunityToolkit.Mvvm)             | 部分プロパティ/コマンド生成で簡潔 |
| DI          | Microsoft.Extensions.DependencyInjection | サービス/VM の疎結合             |
| シェル      | MicaBackdrop + NavigationView            | Windows 11 標準の外観            |

## 音声処理

| 領域          | 選定                        | 理由                                   |
| ------------- | --------------------------- | -------------------------------------- |
| Speech        | Whisper.net                 | whisper.cpp の .NET バインディング、CUDA 対応 |
| Audio Capture | NAudio (WASAPI)             | 低レイテンシー、リサンプラー内蔵       |
| Resampling    | NAudio WdlResamplingSampleProvider | マネージドで高品質             |
| VAD           | Whisper.net 内蔵 Silero VAD | 無音区間の除去、ハルシネーション防止   |

## LLM・文章整形

| 領域         | 選定                                   | 理由                        |
| ------------ | -------------------------------------- | --------------------------- |
| LLM 連携方式 | HTTP API                               | リンクエラー回避、柔軟性    |
| プロバイダー | Ollama / OpenAI 互換 API               | 複数の LLM ランタイムに対応 |
| Ollama       | http://localhost:11434/api/generate    | ローカル LLM 実行           |
| OpenAI 互換  | http://localhost:1234/v1/chat/completions | LM Studio, LocalAI, vLLM 等 |

## システム統合

| 領域            | 選定                              | 理由                       |
| --------------- | --------------------------------- | -------------------------- |
| Global Shortcut | Win32 `RegisterHotKey`            | OS ネイティブのグローバルホットキー |
| Clipboard       | Win32 Clipboard API               | 依存なし                   |
| Auto Paste      | Win32 `SendInput` (Shift+Insert)  | AHK 等のフック回避         |
| System Tray     | H.NotifyIcon.WinUI                | WinUI 向け実績ライブラリ   |
| Autostart       | レジストリ `HKCU\...\Run`         | OS 標準の自動起動          |

## VRAM 割り当て

- Whisper large-v3-turbo (CUDA): ~6GB
- VAD も同プロセスで GPU を使用
- LLM は外部プロセス（Ollama / LM Studio 等）で管理

## ビルド要件

- .NET 9 SDK
- (CUDA 使用時) NVIDIA GPU + CUDA 12.x / 13.x ドライバー
  - Whisper.net が CUDA 13 → CUDA 12 → Vulkan → CPU の順で自動選択
