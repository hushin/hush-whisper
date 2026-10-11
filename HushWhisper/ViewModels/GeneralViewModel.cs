using System.Diagnostics;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using HushWhisper.Services;

namespace HushWhisper.ViewModels;

public sealed partial class GeneralViewModel : ViewModelBase
{
    private readonly IStartupService _startup;
    private readonly bool _loading;

    public GeneralViewModel(ISettingsService settings, IStartupService startup)
    {
        _startup = startup;
        _loading = true;

        AutostartEnabled = startup.IsEnabled();
        ConfigPath = settings.ConfigPath;
        Status = string.Empty;

        _loading = false;
    }

    [ObservableProperty]
    public partial bool AutostartEnabled { get; set; }

    [ObservableProperty]
    public partial string ConfigPath { get; set; }

    [ObservableProperty]
    public partial string Status { get; set; }

    partial void OnAutostartEnabledChanged(bool value)
    {
        if (_loading)
        {
            return;
        }

        try
        {
            _startup.SetEnabled(value);
            Status = value ? "スタートアップに登録しました" : "スタートアップから削除しました";
        }
        catch (Exception ex)
        {
            Status = $"エラー: {ex.Message}";
        }
    }

    [RelayCommand]
    private void OpenConfigFolder()
    {
        try
        {
            var directory = Path.GetDirectoryName(ConfigPath);
            if (!string.IsNullOrEmpty(directory) && Directory.Exists(directory))
            {
                Process.Start(new ProcessStartInfo("explorer.exe", directory) { UseShellExecute = true });
            }
        }
        catch (Exception ex)
        {
            Status = $"エラー: {ex.Message}";
        }
    }
}
