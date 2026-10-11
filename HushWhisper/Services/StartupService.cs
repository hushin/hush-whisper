using Microsoft.Win32;

namespace HushWhisper.Services;

public interface IStartupService
{
    bool IsEnabled();

    void SetEnabled(bool enabled);
}

public sealed class StartupService : IStartupService
{
    private const string RunKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string ValueName = "hush-whisper";

    public bool IsEnabled()
    {
        using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath);
        return key?.GetValue(ValueName) is string;
    }

    public void SetEnabled(bool enabled)
    {
        using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: true)
                        ?? Registry.CurrentUser.CreateSubKey(RunKeyPath);

        if (enabled)
        {
            var exePath = Environment.ProcessPath
                          ?? throw new InvalidOperationException("Cannot determine executable path.");
            key.SetValue(ValueName, $"\"{exePath}\" --minimized");
        }
        else
        {
            key.DeleteValue(ValueName, throwOnMissingValue: false);
        }
    }
}
