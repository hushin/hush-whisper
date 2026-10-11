using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using HushWhisper.Models;
using HushWhisper.Services;

namespace HushWhisper.ViewModels;

public sealed partial class LlmViewModel : ViewModelBase
{
    private readonly ISettingsService _settings;
    private readonly ILlmService _llm;

    public LlmViewModel(ISettingsService settings, ILlmService llm)
    {
        _settings = settings;
        _llm = llm;

        var current = settings.Current.Llm;
        Enabled = current.Enabled;
        ApiUrl = current.ApiUrl;
        ModelName = current.ModelName;
        CustomPrompt = current.CustomPrompt;
        SelectedProvider = Options.LlmProviders.FirstOrDefault(p => p.Value == current.Provider);
        SelectedPreset = Options.PromptPresets.FirstOrDefault(p => p.Value == current.Preset);
        Status = string.Empty;
    }

    public IReadOnlyList<LlmProviderOption> Providers => Options.LlmProviders;

    public IReadOnlyList<PromptPresetOption> Presets => Options.PromptPresets;

    [ObservableProperty]
    public partial bool Enabled { get; set; }

    [ObservableProperty]
    public partial LlmProviderOption? SelectedProvider { get; set; }

    [ObservableProperty]
    public partial string ApiUrl { get; set; }

    [ObservableProperty]
    public partial string ModelName { get; set; }

    [ObservableProperty]
    public partial PromptPresetOption? SelectedPreset { get; set; }

    [ObservableProperty]
    public partial string CustomPrompt { get; set; }

    [ObservableProperty]
    public partial string Status { get; set; }

    public bool IsCustomPreset => SelectedPreset?.Value == PromptPreset.Custom;

    [RelayCommand]
    private void Save()
    {
        _settings.Update(s =>
        {
            s.Llm.Enabled = Enabled;
            s.Llm.Provider = SelectedProvider?.Value ?? LlmProvider.Ollama;
            s.Llm.ApiUrl = ApiUrl;
            s.Llm.ModelName = ModelName;
            s.Llm.Preset = SelectedPreset?.Value ?? PromptPreset.Default;
            s.Llm.CustomPrompt = CustomPrompt;
        });

        Status = "保存しました";
    }

    [RelayCommand]
    private async Task TestConnectionAsync()
    {
        Status = "接続確認中...";
        var ok = await _llm.IsAvailableAsync(ApiUrl, SelectedProvider?.Value ?? LlmProvider.Ollama);
        Status = ok ? "接続OK" : "接続できませんでした";
    }

    partial void OnSelectedPresetChanged(PromptPresetOption? value) => OnPropertyChanged(nameof(IsCustomPreset));
}
