using System.Text;
using HushWhisper.ViewModels;
using Microsoft.UI.Input;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Windows.System;

namespace HushWhisper.Views;

public sealed partial class ShortcutOutputPage : Page
{
    public ShortcutOutputPage()
    {
        ViewModel = App.GetService<ShortcutOutputViewModel>();
        InitializeComponent();
    }

    public ShortcutOutputViewModel ViewModel { get; }

    private void ShortcutBox_KeyDown(object sender, KeyRoutedEventArgs e)
    {
        var key = e.Key;
        if (key is VirtualKey.None)
        {
            return;
        }

        e.Handled = true;

        if (IsModifier(key))
        {
            return;
        }

        var shortcut = BuildShortcut(key);
        if (shortcut is null)
        {
            return;
        }

        ViewModel.Shortcut = shortcut;
        ShortcutBox.Text = shortcut;
    }

    private static bool IsModifier(VirtualKey key) => key is
        VirtualKey.Control or VirtualKey.Shift or VirtualKey.Menu or
        VirtualKey.LeftWindows or VirtualKey.RightWindows;

    private static string? BuildShortcut(VirtualKey key)
    {
        var parts = new List<string>();

        if (IsDown(VirtualKey.Control))
        {
            parts.Add("Ctrl");
        }

        if (IsDown(VirtualKey.Menu))
        {
            parts.Add("Alt");
        }

        if (IsDown(VirtualKey.Shift))
        {
            parts.Add("Shift");
        }

        if (IsDown(VirtualKey.LeftWindows) || IsDown(VirtualKey.RightWindows))
        {
            parts.Add("Win");
        }

        var keyName = KeyName(key);
        if (keyName is null)
        {
            return null;
        }

        parts.Add(keyName);
        return string.Join("+", parts);
    }

    private static bool IsDown(VirtualKey key)
    {
        var state = InputKeyboardSource.GetKeyStateForCurrentThread(key);
        return (state & Windows.UI.Core.CoreVirtualKeyStates.Down) == Windows.UI.Core.CoreVirtualKeyStates.Down;
    }

    private static string? KeyName(VirtualKey key)
    {
        if (key is >= VirtualKey.A and <= VirtualKey.Z)
        {
            return ((char)('A' + (key - VirtualKey.A))).ToString();
        }

        if (key is >= VirtualKey.Number0 and <= VirtualKey.Number9)
        {
            return ((char)('0' + (key - VirtualKey.Number0))).ToString();
        }

        if (key is >= VirtualKey.F1 and <= VirtualKey.F12)
        {
            return $"F{key - VirtualKey.F1 + 1}";
        }

        return key switch
        {
            VirtualKey.Space => "Space",
            VirtualKey.Enter => "Enter",
            VirtualKey.Tab => "Tab",
            VirtualKey.Escape => "Escape",
            VirtualKey.Back => "Backspace",
            VirtualKey.Delete => "Delete",
            VirtualKey.Insert => "Insert",
            VirtualKey.Home => "Home",
            VirtualKey.End => "End",
            VirtualKey.PageUp => "PageUp",
            VirtualKey.PageDown => "PageDown",
            VirtualKey.Up => "Up",
            VirtualKey.Down => "Down",
            VirtualKey.Left => "Left",
            VirtualKey.Right => "Right",
            _ => null,
        };
    }
}
