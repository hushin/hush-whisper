using System.Runtime.InteropServices;
using System.Threading;
using HushWhisper.Interop;
using HushWhisper.Models;

namespace HushWhisper.Services;

public interface IClipboardService
{
    void OutputText(string text, OutputMode mode);

    void SetText(string text);

    string? GetText();
}

public sealed class ClipboardService : IClipboardService
{
    public void OutputText(string text, OutputMode mode)
    {
        switch (mode)
        {
            case OutputMode.ClipboardOnly:
                SetText(text);
                break;

            case OutputMode.DirectInput:
                PasteTemporary(text);
                break;

            case OutputMode.Both:
                SetText(text);
                Paste();
                break;
        }
    }

    public void SetText(string text)
    {
        for (var attempt = 0; attempt < 10; attempt++)
        {
            if (!NativeMethods.OpenClipboard(IntPtr.Zero))
            {
                Thread.Sleep(30);
                continue;
            }

            try
            {
                NativeMethods.EmptyClipboard();

                var bytes = (text.Length + 1) * sizeof(char);
                var hGlobal = NativeMethods.GlobalAlloc(NativeMethods.GMEM_MOVEABLE, (UIntPtr)bytes);
                if (hGlobal == IntPtr.Zero)
                {
                    continue;
                }

                var target = NativeMethods.GlobalLock(hGlobal);
                if (target == IntPtr.Zero)
                {
                    NativeMethods.GlobalFree(hGlobal);
                    continue;
                }

                try
                {
                    Marshal.Copy(text.ToCharArray(), 0, target, text.Length);
                    Marshal.WriteInt16(target, text.Length * sizeof(char), 0);
                }
                finally
                {
                    NativeMethods.GlobalUnlock(hGlobal);
                }

                // Ownership of hGlobal transfers to the clipboard on success.
                if (NativeMethods.SetClipboardData(NativeMethods.CF_UNICODETEXT, hGlobal) == IntPtr.Zero)
                {
                    NativeMethods.GlobalFree(hGlobal);
                }

                return;
            }
            finally
            {
                NativeMethods.CloseClipboard();
            }
        }
    }

    public string? GetText()
    {
        for (var attempt = 0; attempt < 10; attempt++)
        {
            if (!NativeMethods.IsClipboardFormatAvailable(NativeMethods.CF_UNICODETEXT) ||
                !NativeMethods.OpenClipboard(IntPtr.Zero))
            {
                Thread.Sleep(30);
                continue;
            }

            try
            {
                var handle = NativeMethods.GetClipboardData(NativeMethods.CF_UNICODETEXT);
                if (handle == IntPtr.Zero)
                {
                    return null;
                }

                var pointer = NativeMethods.GlobalLock(handle);
                if (pointer == IntPtr.Zero)
                {
                    return null;
                }

                try
                {
                    return Marshal.PtrToStringUni(pointer);
                }
                finally
                {
                    NativeMethods.GlobalUnlock(handle);
                }
            }
            finally
            {
                NativeMethods.CloseClipboard();
            }
        }

        return null;
    }

    private void PasteTemporary(string text)
    {
        var original = GetText();
        SetText(text);
        Paste();

        Thread.Sleep(100);

        if (!string.IsNullOrEmpty(original))
        {
            SetText(original);
        }
    }

    private static void Paste()
    {
        // Wait for the hotkey (Ctrl+Space) to be released before simulating input.
        Thread.Sleep(50);

        // Shift+Insert is the primary paste shortcut (works in most apps and avoids
        // AutoHotkey-style Ctrl+V hooks). Fall back to Ctrl+V if it does not land.
        if (!TrySend(KeyDown(NativeMethods.VK_SHIFT), KeyDown(NativeMethods.VK_INSERT),
                KeyUp(NativeMethods.VK_INSERT), KeyUp(NativeMethods.VK_SHIFT)))
        {
            TrySend(KeyDown(NativeMethods.VK_CONTROL), KeyDown(NativeMethods.VK_V),
                KeyUp(NativeMethods.VK_V), KeyUp(NativeMethods.VK_CONTROL));
        }
    }

    private static bool TrySend(params NativeMethods.INPUT[] inputs)
    {
        var sent = NativeMethods.SendInput((uint)inputs.Length, inputs, Marshal.SizeOf<NativeMethods.INPUT>());
        return sent == inputs.Length;
    }

    private static NativeMethods.INPUT KeyDown(ushort vk) => new()
    {
        type = NativeMethods.INPUT_KEYBOARD,
        u = new NativeMethods.INPUTUNION { ki = new NativeMethods.KEYBDINPUT { wVk = vk } },
    };

    private static NativeMethods.INPUT KeyUp(ushort vk) => new()
    {
        type = NativeMethods.INPUT_KEYBOARD,
        u = new NativeMethods.INPUTUNION
        {
            ki = new NativeMethods.KEYBDINPUT { wVk = vk, dwFlags = NativeMethods.KEYEVENTF_KEYUP },
        },
    };
}
