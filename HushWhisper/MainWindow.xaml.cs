using HushWhisper.Interop;
using HushWhisper.Models;
using HushWhisper.Services;
using HushWhisper.Views;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.Graphics;

namespace HushWhisper;

public sealed partial class MainWindow : Window
{
    private readonly IRecordingCoordinator _coordinator = App.GetService<IRecordingCoordinator>();
    private readonly ISettingsService _settings = App.GetService<ISettingsService>();
    private readonly IGlobalHotkeyService _hotkeys = App.GetService<IGlobalHotkeyService>();
    private readonly DispatcherQueue _dispatcher;
    private readonly DispatcherQueueTimer _infoBarTimer;

    private bool _isExiting;

    public MainWindow()
    {
        InitializeComponent();

        _dispatcher = DispatcherQueue;

        ExtendsContentIntoTitleBar = true;
        SetTitleBar(AppTitleBar);
        Title = "HushWhisper";

        ConfigureWindow();

        _infoBarTimer = _dispatcher.CreateTimer();
        _infoBarTimer.IsRepeating = false;
        _infoBarTimer.Tick += (_, _) => StatusInfoBar.IsOpen = false;

        TrayIcon.Icon = TrayIconFactory.ForState(RecordingState.Idle);
        TrayIcon.LeftClickCommand = new CommunityToolkit.Mvvm.Input.RelayCommand(ShowWindow);
        TrayIcon.ContextFlyout = BuildTrayMenu();
        TrayIcon.ForceCreate(true);

        AppWindow.Closing += OnAppWindowClosing;

        _coordinator.StateChanged += OnStateChanged;
        _coordinator.StatusChanged += OnStatusChanged;
        _hotkeys.Pressed += OnHotkeyPressed;

        RegisterHotkey(_settings.Current.Shortcut.RecordingToggle);

        NavView.SelectedItem = NavView.MenuItems[0];

        // Restore the previously used model in the background.
        _ = LoadDefaultModelAsync();
    }

    public IRecordingCoordinator Coordinator => _coordinator;

    private async Task LoadDefaultModelAsync()
    {
        var asr = App.GetService<IAsrService>();
        var model = _settings.Current.Whisper.ModelName;

        if (asr.IsLoaded || !asr.IsModelDownloaded(model))
        {
            return;
        }

        try
        {
            AppLog.Write($"Auto-loading model '{model}' at startup");
            await _coordinator.InitializeModelAsync(model);
        }
        catch (Exception ex)
        {
            AppLog.Write("Auto model load failed", ex);
        }
    }

    private void OnHotkeyPressed()
    {
        try
        {
            _ = _coordinator.ToggleAsync();
        }
        catch (Exception ex)
        {
            AppLog.Write("Hotkey toggle failed", ex);
        }
    }

    private void RegisterHotkey(string shortcut)
    {
        try
        {
            _hotkeys.Register(shortcut);
            AppLog.Write($"Global shortcut '{shortcut}' registered");
        }
        catch (Exception ex)
        {
            AppLog.Write("Shortcut registration failed", ex);
            ShowInfo($"ショートカットの登録に失敗しました: {ex.Message}", InfoBarSeverity.Warning);
        }
    }

    public void ReregisterHotkey(string shortcut) => RegisterHotkey(shortcut);

    private void ConfigureWindow()
    {
        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
        var dpi = NativeMethods.GetDpiForWindow(hwnd);
        var scale = dpi == 0 ? 1.0 : dpi / 96.0;

        var width = (int)(1180 * scale);
        var height = (int)(820 * scale);

        var displayArea = DisplayArea.GetFromWindowId(AppWindow.Id, DisplayAreaFallback.Primary);
        var work = displayArea.WorkArea;
        var x = work.X + Math.Max(0, (work.Width - width) / 2);
        var y = work.Y + Math.Max(0, (work.Height - height) / 2);

        AppWindow.MoveAndResize(new RectInt32(x, y, width, height));
    }

    private void OnStateChanged(object? sender, RecordingState state)
    {
        OnUi(() =>
        {
            TrayIcon.Icon = TrayIconFactory.ForState(state);
            TrayIcon.ToolTipText = state switch
            {
                RecordingState.Recording => "HushWhisper - 録音中...",
                RecordingState.Processing => "HushWhisper - 処理中...",
                _ => "HushWhisper - 待機中",
            };
        });
    }

    private void OnStatusChanged(object? sender, string message)
    {
        // Routine states are shown inline on the page and by the tray icon; the
        // InfoBar is reserved for noteworthy/actionable messages.
        if (!ShouldShowInInfoBar(message))
        {
            return;
        }

        var severity = ClassifySeverity(message);
        var autoClose = severity is InfoBarSeverity.Informational or InfoBarSeverity.Success;
        OnUi(() => ShowInfo(message, severity, autoClose));
    }

    private static bool ShouldShowInInfoBar(string message) =>
        message is not ("待機中" or "録音中" or "文字起こし中..." or "LLMで整形中...");

    private static InfoBarSeverity ClassifySeverity(string message)
    {
        if (message.Contains("エラー") || message.Contains("失敗") || message.Contains("できません"))
        {
            return InfoBarSeverity.Warning;
        }

        if (message.Contains("完了") || message.Contains("読み込みました"))
        {
            return InfoBarSeverity.Success;
        }

        return InfoBarSeverity.Informational;
    }

    private void OnUi(Action action)
    {
        void Invoke()
        {
            try
            {
                action();
            }
            catch (Exception ex)
            {
                // Exceptions escaping a DispatcherQueue callback crash the app with a
                // stowed exception, so contain them here.
                AppLog.Write("MainWindow UI update failed", ex);
            }
        }

        if (_dispatcher.HasThreadAccess)
        {
            Invoke();
        }
        else
        {
            _dispatcher.TryEnqueue(Invoke);
        }
    }

    private void ShowInfo(string message, InfoBarSeverity severity, bool autoClose = false)
    {
        StatusInfoBar.Message = message;
        StatusInfoBar.Severity = severity;
        StatusInfoBar.IsOpen = true;

        _infoBarTimer.Stop();
        if (autoClose)
        {
            _infoBarTimer.Interval = TimeSpan.FromSeconds(5);
            _infoBarTimer.Start();
        }
    }

    private void ShowWindow()
    {
        AppWindow.Show();

        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
        NativeMethods.ShowWindow(hwnd, NativeMethods.SW_RESTORE);
        Activate();
        NativeMethods.SetForegroundWindow(hwnd);
    }

    private void OnAppWindowClosing(AppWindow sender, AppWindowClosingEventArgs args)
    {
        if (_isExiting)
        {
            return;
        }

        AppLog.Write("Window close requested - hiding to tray");

        // Keep the app alive in the system tray instead of exiting.
        args.Cancel = true;
        sender.Hide();
    }

    private MenuFlyout BuildTrayMenu()
    {
        // H.NotifyIcon's default PopupMenu mode invokes the ICommand bound to each
        // MenuFlyoutItem; it does NOT raise the Click event. So menus must use
        // commands, never Click handlers.
        var showItem = new MenuFlyoutItem
        {
            Text = "ウィンドウを表示",
            Command = new CommunityToolkit.Mvvm.Input.RelayCommand(ShowWindow),
        };
        var exitItem = new MenuFlyoutItem
        {
            Text = "終了",
            Command = new CommunityToolkit.Mvvm.Input.RelayCommand(ExitApp),
        };

        var menu = new MenuFlyout();
        menu.Items.Add(showItem);
        menu.Items.Add(new MenuFlyoutSeparator());
        menu.Items.Add(exitItem);
        return menu;
    }

    private void ExitApp()
    {
        AppLog.Write("Tray exit requested");
        _isExiting = true;

        try
        {
            _hotkeys.Dispose();
        }
        catch (Exception ex)
        {
            AppLog.Write("Hotkey dispose failed", ex);
        }

        try
        {
            TrayIcon.Dispose();
        }
        catch (Exception ex)
        {
            AppLog.Write("Tray dispose failed", ex);
        }

        // Application.Exit() can be ignored, so defer it and force-terminate if the
        // process is somehow still alive shortly after.
        _dispatcher.TryEnqueue(() =>
        {
            try
            {
                Application.Current.Exit();
            }
            catch (Exception ex)
            {
                AppLog.Write("Application.Exit failed", ex);
            }

            _ = Task.Delay(2000).ContinueWith(_ => Environment.Exit(0));
        });
    }

    private void NavView_SelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args)
    {
        if (args.SelectedItem is not NavigationViewItem item || item.Tag is not string tag)
        {
            return;
        }

        var pageType = tag switch
        {
            "speech" => typeof(SpeechRecognitionPage),
            "shortcut" => typeof(ShortcutOutputPage),
            "llm" => typeof(LlmPage),
            "general" => typeof(GeneralPage),
            "history" => typeof(HistoryPage),
            "about" => typeof(AboutPage),
            _ => typeof(SpeechRecognitionPage),
        };

        if (ContentFrame.CurrentSourcePageType != pageType)
        {
            ContentFrame.Navigate(pageType);
        }
    }
}
