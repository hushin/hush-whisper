using System.Drawing;
using System.Drawing.Drawing2D;
using HushWhisper.Models;

namespace HushWhisper.Services;

/// <summary>Creates tray icons with a colored status dot, mirroring the original Tauri app.</summary>
internal static class TrayIconFactory
{
    private static readonly Lazy<Icon> Idle = new(() => CreateIcon(null));
    private static readonly Lazy<Icon> Recording = new(() => CreateIcon(Color.FromArgb(220, 53, 69)));
    private static readonly Lazy<Icon> Processing = new(() => CreateIcon(Color.FromArgb(102, 126, 234)));

    public static Icon ForState(RecordingState state) => state switch
    {
        RecordingState.Recording => Recording.Value,
        RecordingState.Processing => Processing.Value,
        _ => Idle.Value,
    };

    private static Icon CreateIcon(Color? dotColor)
    {
        var iconPath = Path.Combine(AppContext.BaseDirectory, "Assets", "AppIcon.ico");

        using var baseIcon = File.Exists(iconPath)
            ? new Icon(iconPath, 32, 32)
            : SystemIcons.Application;

        using var bitmap = new Bitmap(32, 32);
        using (var graphics = Graphics.FromImage(bitmap))
        {
            graphics.DrawIcon(baseIcon, new Rectangle(0, 0, 32, 32));
        }

        if (dotColor is { } color)
        {
            using var graphics = Graphics.FromImage(bitmap);
            graphics.SmoothingMode = SmoothingMode.AntiAlias;
            using var brush = new SolidBrush(color);
            graphics.FillEllipse(brush, 32 - 12, 32 - 12, 11, 11);
        }

        // The HICON returned by GetHicon stays valid after the bitmap is disposed.
        return Icon.FromHandle(bitmap.GetHicon());
    }
}
