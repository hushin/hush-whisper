using HushWhisper.Services;
using HushWhisper.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;

namespace HushWhisper;

public partial class App : Application
{
    public static IServiceProvider Services { get; private set; } = null!;

    private Window? _window;

    public App()
    {
        InitializeComponent();
        Services = ConfigureServices();

        AppLog.Write("App started");

        // Crash diagnostics.
        UnhandledException += (_, e) =>
        {
            AppLog.Write("XAML UnhandledException", e.Exception);
        };
        AppDomain.CurrentDomain.UnhandledException += (_, e) =>
        {
            AppLog.Write($"AppDomain UnhandledException (terminating={e.IsTerminating})", e.ExceptionObject as Exception ?? new Exception(e.ExceptionObject?.ToString()));
        };
        TaskScheduler.UnobservedTaskException += (_, e) =>
        {
            AppLog.Write("UnobservedTaskException", e.Exception);
        };
        AppDomain.CurrentDomain.FirstChanceException += (_, e) =>
        {
            // Log COM/WinRT exceptions as they are thrown; these are the usual
            // source of native stowed-exception crashes (0xc000027b).
            if (e.Exception is System.Runtime.InteropServices.COMException)
            {
                AppLog.Write("FirstChance COMException", e.Exception);
            }
        };
    }

    private static ServiceProvider ConfigureServices()
    {
        var services = new ServiceCollection();

        // Core services
        services.AddSingleton<ISettingsService, SettingsService>();
        services.AddSingleton<ILogService, LogService>();
        services.AddSingleton<IAudioCaptureService, AudioCaptureService>();
        services.AddSingleton<IAsrService, WhisperAsrService>();
        services.AddSingleton<ILlmService, LlmService>();
        services.AddSingleton<IClipboardService, ClipboardService>();
        services.AddSingleton<IStartupService, StartupService>();
        services.AddSingleton<IVadService, SileroVadService>();
        services.AddSingleton<IGlobalHotkeyService, GlobalHotkeyService>();
        services.AddSingleton<IRecordingCoordinator, RecordingCoordinator>();

        // ViewModels
        services.AddTransient<SpeechRecognitionViewModel>();
        services.AddTransient<ShortcutOutputViewModel>();
        services.AddTransient<LlmViewModel>();
        services.AddTransient<GeneralViewModel>();
        services.AddTransient<HistoryViewModel>();
        services.AddTransient<AboutViewModel>();

        return services.BuildServiceProvider();
    }

    public static T GetService<T>() where T : notnull => Services.GetRequiredService<T>();

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        var minimized = Environment.GetCommandLineArgs()
            .Any(a => string.Equals(a, "--minimized", StringComparison.OrdinalIgnoreCase));

        var window = new MainWindow();
        _window = window;

        if (!minimized)
        {
            window.Activate();
        }
    }
}
