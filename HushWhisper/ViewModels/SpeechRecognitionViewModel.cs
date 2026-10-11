using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using HushWhisper.Models;
using HushWhisper.Services;

namespace HushWhisper.ViewModels;

public sealed partial class SpeechRecognitionViewModel : ViewModelBase
{
    private readonly IRecordingCoordinator _coordinator;
    private readonly ISettingsService _settings;
    private readonly IAsrService _asr;
    private readonly bool _loading;

    public SpeechRecognitionViewModel(IRecordingCoordinator coordinator, ISettingsService settings, IAsrService asr)
    {
        _coordinator = coordinator;
        _settings = settings;
        _asr = asr;
        _loading = true;

        foreach (var model in asr.AvailableModels)
        {
            Models.Add(model);
        }

        var saved = settings.Current.Whisper;
        InsertNewline = saved.InsertNewline;
        MaxRecordingSeconds = saved.MaxRecordingSeconds;
        SelectedModel = Models.FirstOrDefault(m => m.Name == saved.ModelName) ?? Models.FirstOrDefault();
        Status = "待機中";

        _loading = false;

        _coordinator.StatusChanged += (_, message) => OnUi(() => Status = message);
        _coordinator.ModelDownloadProgress += (_, progress) => OnUi(() =>
        {
            DownloadPercentage = progress.Percentage;
            IsDownloading = true;
        });
        _coordinator.StateChanged += (_, state) => OnUi(() => IsRecording = state == RecordingState.Recording);
    }

    public ObservableCollection<ModelInfo> Models { get; } = [];

    [ObservableProperty]
    public partial ModelInfo? SelectedModel { get; set; }

    [ObservableProperty]
    public partial string Status { get; set; }

    [ObservableProperty]
    public partial bool IsBusy { get; set; }

    [ObservableProperty]
    public partial bool IsDownloading { get; set; }

    [ObservableProperty]
    public partial double DownloadPercentage { get; set; }

    [ObservableProperty]
    public partial bool InsertNewline { get; set; }

    [ObservableProperty]
    public partial double MaxRecordingSeconds { get; set; }

    [ObservableProperty]
    public partial bool IsRecording { get; set; }

    public string RecordingStateText => IsRecording ? "録音中" : "待機中";

    [RelayCommand]
    private async Task InitializeModelAsync()
    {
        if (SelectedModel is null || IsBusy)
        {
            return;
        }

        IsBusy = true;
        DownloadPercentage = 0;
        IsDownloading = !_asr.IsModelDownloaded(SelectedModel.Name);

        try
        {
            await _coordinator.InitializeModelAsync(SelectedModel.Name);
            Status = $"{SelectedModel.Name} を読み込みました";
        }
        catch (Exception ex)
        {
            Status = $"エラー: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
            IsDownloading = false;
        }
    }

    [RelayCommand]
    private async Task ToggleRecordingAsync()
    {
        await _coordinator.ToggleAsync();
    }

    partial void OnInsertNewlineChanged(bool value)
    {
        if (!_loading)
        {
            _settings.Update(s => s.Whisper.InsertNewline = value);
        }
    }

    partial void OnMaxRecordingSecondsChanged(double value)
    {
        if (!_loading)
        {
            _settings.Update(s => s.Whisper.MaxRecordingSeconds = (uint)value);
        }
    }

    partial void OnIsRecordingChanged(bool value) => OnPropertyChanged(nameof(RecordingStateText));
}
