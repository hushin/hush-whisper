using HushWhisper.Interop;

namespace HushWhisper.Services;

/// <summary>Parses shortcut strings like "Ctrl+Space" into Win32 modifier/key codes.</summary>
internal static class ShortcutParser
{
    public static (uint Modifiers, uint VirtualKey) Parse(string shortcut)
    {
        uint modifiers = 0;
        uint? virtualKey = null;

        foreach (var rawPart in shortcut.Split('+', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries))
        {
            switch (rawPart.ToUpperInvariant())
            {
                case "CTRL":
                case "CONTROL":
                    modifiers |= NativeMethods.MOD_CONTROL;
                    break;
                case "ALT":
                    modifiers |= NativeMethods.MOD_ALT;
                    break;
                case "SHIFT":
                    modifiers |= NativeMethods.MOD_SHIFT;
                    break;
                case "SUPER":
                case "WIN":
                case "CMD":
                case "META":
                    modifiers |= NativeMethods.MOD_WIN;
                    break;
                default:
                    virtualKey = ParseKey(rawPart);
                    break;
            }
        }

        if (virtualKey is null)
        {
            throw new FormatException($"No key specified in shortcut: {shortcut}");
        }

        return (modifiers, virtualKey.Value);
    }

    private static uint ParseKey(string key)
    {
        var upper = key.ToUpperInvariant();

        if (upper.Length == 1)
        {
            var c = upper[0];
            if (c is >= 'A' and <= 'Z')
            {
                return c;
            }

            if (c is >= '0' and <= '9')
            {
                return c;
            }
        }

        return upper switch
        {
            "SPACE" => 0x20,
            "ENTER" or "RETURN" => 0x0D,
            "TAB" => 0x09,
            "ESCAPE" or "ESC" => 0x1B,
            "BACKSPACE" => 0x08,
            "DELETE" => 0x2E,
            "INSERT" => 0x2D,
            "HOME" => 0x24,
            "END" => 0x23,
            "PAGEUP" => 0x21,
            "PAGEDOWN" => 0x22,
            "UP" => 0x26,
            "DOWN" => 0x28,
            "LEFT" => 0x25,
            "RIGHT" => 0x27,
            "F1" => 0x70,
            "F2" => 0x71,
            "F3" => 0x72,
            "F4" => 0x73,
            "F5" => 0x74,
            "F6" => 0x75,
            "F7" => 0x76,
            "F8" => 0x77,
            "F9" => 0x78,
            "F10" => 0x79,
            "F11" => 0x7A,
            "F12" => 0x7B,
            "EQUAL" => 0xBB,
            "MINUS" => 0xBD,
            "BRACKETLEFT" => 0xDB,
            "BRACKETRIGHT" => 0xDD,
            "SEMICOLON" => 0xBA,
            "QUOTE" => 0xDE,
            "BACKQUOTE" => 0xC0,
            "BACKSLASH" => 0xDC,
            "COMMA" => 0xBC,
            "PERIOD" => 0xBE,
            "SLASH" => 0xBF,
            "NUMPAD0" => 0x60,
            "NUMPAD1" => 0x61,
            "NUMPAD2" => 0x62,
            "NUMPAD3" => 0x63,
            "NUMPAD4" => 0x64,
            "NUMPAD5" => 0x65,
            "NUMPAD6" => 0x66,
            "NUMPAD7" => 0x67,
            "NUMPAD8" => 0x68,
            "NUMPAD9" => 0x69,
            "NUMPADMULTIPLY" => 0x6A,
            "NUMPADADD" => 0x6B,
            "NUMPADSUBTRACT" => 0x6D,
            "NUMPADDECIMAL" => 0x6E,
            "NUMPADDIVIDE" => 0x6F,
            _ => throw new FormatException($"Unknown key: {key}"),
        };
    }
}
