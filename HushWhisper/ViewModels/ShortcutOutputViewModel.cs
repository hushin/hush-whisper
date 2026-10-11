using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using HushWhisper.Models;
using HushWhisper.Services;

namespace HushWhisper.ViewModels;

public sealed partial class ShortcutOutputViewModel : ViewModelBase
{
    private readonly ISettingsService _settings;
    private readonly IGlobalHotkeyService _hotkeys;
    private readonly bool _loading;

    public ShortcutOutputViewModel(ISettingsService settings, IGlobalHotkeyService hotkeys)
    {
        _settings = settings;
        _hotkeys = hotkeys;
        _loading = true;

        var current = settings.Current;
        Shortcut = current.Shortcut.RecordingToggle;
        SelectedOutputMode = Options.OutputModes.FirstOrDefault(o => o.Value == current.OutputMode) ?? Options.OutputModes[0];
        Status = string.Empty;

        _loading = false;
    }

    public IReadOnlyList<OutputModeOption> OutputModes => Options.OutputModes;

    [ObservableProperty]
    public partial string Shortcut { get; set; }

    [ObservableProperty]
    public partial OutputModeOption? SelectedOutputMode { get; set; }

    [ObservableProperty]
    public partial string Status { get; set; }

    [RelayCommand]
    private void Save()
    {
        try
        {
            ShortcutParser.Parse(Shortcut);
        }
        catch (FormatException ex)
        {
            Status = $"ショートカットが不正です: {ex.Message}";
            return;
        }

        try
        {
            _hotkeys.Register(Shortcut);
        }
        catch (Exception ex)
        {
            Status = $"ショートカットを登録できませんでした: {ex.Message}";
            return;
        }

        _settings.Update(s => s.Shortcut.RecordingToggle = Shortcut);
        Status = $"ショートカット '{Shortcut}' を登録しました";
    }

    partial void OnSelectedOutputModeChanged(OutputModeOption? value)
    {
        if (_loading || value is null)
        {
            return;
        }

        _settings.Update(s => s.OutputMode = value.Value);
    }
}
