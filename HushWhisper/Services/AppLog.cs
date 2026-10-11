using System.Text;

namespace HushWhisper.Services;

/// <summary>Appends diagnostic messages to %APPDATA%/voice-input/logs/app.log.</summary>
internal static class AppLog
{
    private static readonly object Lock = new();

    private static readonly string LogPath = System.IO.Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
        "voice-input",
        "logs",
        "app.log");

    public static string LogFilePath => LogPath;

    public static void Write(string message)
    {
        try
        {
            lock (Lock)
            {
                var directory = System.IO.Path.GetDirectoryName(LogPath);
                if (!string.IsNullOrEmpty(directory))
                {
                    Directory.CreateDirectory(directory);
                }

                var line = new StringBuilder()
                    .Append(DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss.fff"))
                    .Append(" [")
                    .Append(Environment.CurrentManagedThreadId)
                    .Append("] ")
                    .Append(message)
                    .Append('\n')
                    .ToString();

                File.AppendAllText(LogPath, line);
            }
        }
        catch
        {
            // Logging must never throw.
        }
    }

    public static void Write(string message, Exception ex) =>
        Write($"{message}: {ex.GetType().Name}: {ex.Message}\n{ex.StackTrace}");
}
