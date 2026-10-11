using System.Collections.Concurrent;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Threading;
using HushWhisper.Interop;

namespace HushWhisper.Services;

public interface IGlobalHotkeyService : IDisposable
{
    event Action? Pressed;

    /// <summary>Registers (or re-registers) the global hotkey for the given shortcut string.</summary>
    void Register(string shortcut);

    void Unregister();
}

/// <summary>
/// Registers a system-wide hotkey using thread hotkeys (<c>RegisterHotKey</c> with a NULL
/// window handle). WM_HOTKEY is delivered to a dedicated message-loop thread, so no hidden
/// window is required.
/// </summary>
public sealed class GlobalHotkeyService : IGlobalHotkeyService
{
    private const int HotkeyId = 0x4857; // 'HW'
    private const uint WmAppRegister = 0x8000 + 1; // WM_APP + 1

    private readonly ManualResetEventSlim _ready = new(false);
    private readonly ConcurrentQueue<HotkeyRequest> _requests = new();
    private readonly Thread _thread;

    private uint _threadId;
    private bool _unregisterRequested;

    public event Action? Pressed;

    public GlobalHotkeyService()
    {
        _thread = new Thread(MessageLoop)
        {
            IsBackground = true,
            Name = "HushWhisper.Hotkey",
        };
        _thread.Start();
        _ready.Wait();
    }

    public void Register(string shortcut)
    {
        var (modifiers, key) = ShortcutParser.Parse(shortcut);
        Send(new HotkeyRequest { Register = true, Modifiers = modifiers, Key = key });
    }

    public void Unregister()
    {
        Send(new HotkeyRequest { Register = false });
    }

    public void Dispose()
    {
        try
        {
            NativeMethods.PostThreadMessage(_threadId, 0x0012 /* WM_QUIT */, IntPtr.Zero, IntPtr.Zero);
        }
        catch
        {
            // Thread may already be gone.
        }
    }

    private void Send(HotkeyRequest request)
    {
        _requests.Enqueue(request);
        NativeMethods.PostThreadMessage(_threadId, WmAppRegister, IntPtr.Zero, IntPtr.Zero);

        // Wait (bounded) for the hotkey thread to process the request so callers can
        // observe registration failures synchronously.
        if (!request.Done.Wait(TimeSpan.FromSeconds(5)))
        {
            throw new InvalidOperationException("Hotkey thread did not respond.");
        }

        if (request.Error is not null)
        {
            throw request.Error;
        }
    }

    private void MessageLoop()
    {
        _threadId = NativeMethods.GetCurrentThreadId();
        _ready.Set();

        while (NativeMethods.GetMessage(out var msg, IntPtr.Zero, 0, 0) > 0)
        {
            if (msg.message == WmAppRegister)
            {
                ProcessRequests();
            }
            else if (msg.message == NativeMethods.WM_HOTKEY && msg.wParam.ToInt32() == HotkeyId)
            {
                try
                {
                    Pressed?.Invoke();
                }
                catch (Exception ex)
                {
                    AppLog.Write("Hotkey callback failed", ex);
                }
            }
        }
    }

    private void ProcessRequests()
    {
        if (_unregisterRequested)
        {
            NativeMethods.UnregisterHotKey(IntPtr.Zero, HotkeyId);
            _unregisterRequested = false;
        }

        while (_requests.TryDequeue(out var request))
        {
            try
            {
                NativeMethods.UnregisterHotKey(IntPtr.Zero, HotkeyId);

                if (request.Register)
                {
                    if (!NativeMethods.RegisterHotKey(
                            IntPtr.Zero,
                            HotkeyId,
                            request.Modifiers | NativeMethods.MOD_NOREPEAT,
                            request.Key))
                    {
                        var error = Marshal.GetLastWin32Error();
                        request.Error = new InvalidOperationException(
                            $"ショートカットを登録できませんでした (Win32 error {error})。他のアプリが使用している可能性があります。");
                    }
                }
            }
            catch (Exception ex)
            {
                request.Error = ex;
            }
            finally
            {
                request.Done.Set();
            }
        }
    }

    private sealed class HotkeyRequest
    {
        public bool Register { get; init; }

        public uint Modifiers { get; init; }

        public uint Key { get; init; }

        public Exception? Error { get; set; }

        public ManualResetEventSlim Done { get; } = new(false);
    }
}
