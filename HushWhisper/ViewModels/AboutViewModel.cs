using System.Reflection;
using CommunityToolkit.Mvvm.ComponentModel;

namespace HushWhisper.ViewModels;

public sealed partial class AboutViewModel : ViewModelBase
{
    public AboutViewModel()
    {
        var version = Assembly.GetExecutingAssembly().GetName().Version;
        Version = version is null ? "0.2.1" : $"{version.Major}.{version.Minor}.{version.Build}";
    }

    public string Version { get; }

    public string Description => "Windows 向けローカル音声入力アプリ";

    public string FeatureSummary =>
        "グローバルショートカットで任意のアプリに日本語音声入力。Whisper (CUDA) による音声認識と、Ollama / OpenAI互換 API による文章整形に対応。";
}
