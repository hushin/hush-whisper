using System.Globalization;
using System.Text.Json;
using System.Text.Json.Serialization;
using HushWhisper.Models;

namespace HushWhisper.Services;

public interface ILogService
{
    string LogDirectory { get; }

    LogEntry Add(string rawText, string? refinedText, float? audioDurationSecs, bool llmUsed, string? promptPreset);

    IReadOnlyList<LogEntry> GetRecent(int limit);

    IReadOnlyList<LogEntry> GetForDate(DateOnly date);

    IReadOnlyList<DateOnly> GetAvailableDates();

    bool Delete(string id);

    int DeleteAll();
}

public sealed class LogService : ILogService
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        WriteIndented = true,
    };

    private readonly object _lock = new();

    public LogService()
    {
        LogDirectory = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "voice-input",
            "logs");
        Directory.CreateDirectory(LogDirectory);
    }

    public string LogDirectory { get; }

    public LogEntry Add(string rawText, string? refinedText, float? audioDurationSecs, bool llmUsed, string? promptPreset)
    {
        lock (_lock)
        {
            var now = DateTimeOffset.Now;
            var entry = new LogEntry
            {
                Id = GenerateId(),
                Timestamp = now,
                RawText = rawText,
                RefinedText = refinedText,
                AudioDurationSecs = audioDurationSecs,
                LlmUsed = llmUsed,
                PromptPreset = promptPreset,
            };

            var entries = LoadForDate(DateOnly.FromDateTime(now.LocalDateTime)).ToList();
            entries.Add(entry);
            SaveForDate(DateOnly.FromDateTime(now.LocalDateTime), entries);
            return entry;
        }
    }

    public IReadOnlyList<LogEntry> GetRecent(int limit)
    {
        lock (_lock)
        {
            var result = new List<LogEntry>();
            foreach (var file in EnumerateLogFilesDescending())
            {
                if (result.Count >= limit)
                {
                    break;
                }

                result.AddRange(ReadEntries(file));
            }

            return result
                .OrderByDescending(e => e.Timestamp)
                .Take(limit)
                .ToList();
        }
    }

    public IReadOnlyList<LogEntry> GetForDate(DateOnly date)
    {
        lock (_lock)
        {
            return LoadForDate(date)
                .OrderByDescending(e => e.Timestamp)
                .ToList();
        }
    }

    public IReadOnlyList<DateOnly> GetAvailableDates()
    {
        lock (_lock)
        {
            var dates = new List<DateOnly>();
            foreach (var file in Directory.EnumerateFiles(LogDirectory, "*.json"))
            {
                var name = Path.GetFileNameWithoutExtension(file);
                if (DateOnly.TryParseExact(name, "yyyy-MM-dd", CultureInfo.InvariantCulture, DateTimeStyles.None, out var date))
                {
                    dates.Add(date);
                }
            }

            return dates.OrderByDescending(d => d).ToList();
        }
    }

    public bool Delete(string id)
    {
        lock (_lock)
        {
            foreach (var file in Directory.EnumerateFiles(LogDirectory, "*.json"))
            {
                var entries = ReadEntries(file).ToList();
                var removed = entries.RemoveAll(e => e.Id == id);
                if (removed > 0)
                {
                    File.WriteAllText(file, JsonSerializer.Serialize(entries, JsonOptions));
                    return true;
                }
            }

            return false;
        }
    }

    public int DeleteAll()
    {
        lock (_lock)
        {
            var count = 0;
            foreach (var file in Directory.EnumerateFiles(LogDirectory, "*.json"))
            {
                count += ReadEntries(file).Count;
                File.Delete(file);
            }

            return count;
        }
    }

    private IEnumerable<string> EnumerateLogFilesDescending() =>
        Directory.EnumerateFiles(LogDirectory, "*.json")
            .OrderByDescending(f => f, StringComparer.OrdinalIgnoreCase);

    private IEnumerable<LogEntry> LoadForDate(DateOnly date)
    {
        var path = GetLogFilePath(date);
        if (!File.Exists(path))
        {
            return [];
        }

        return ReadEntries(path);
    }

    private void SaveForDate(DateOnly date, IEnumerable<LogEntry> entries)
    {
        File.WriteAllText(GetLogFilePath(date), JsonSerializer.Serialize(entries, JsonOptions));
    }

    private static IReadOnlyList<LogEntry> ReadEntries(string path)
    {
        try
        {
            var json = File.ReadAllText(path);
            return JsonSerializer.Deserialize<List<LogEntry>>(json, JsonOptions) ?? [];
        }
        catch
        {
            return [];
        }
    }

    private string GetLogFilePath(DateOnly date) =>
        Path.Combine(LogDirectory, $"{date:yyyy-MM-dd}.json");

    private static string GenerateId() =>
        DateTimeOffset.UtcNow.ToUnixTimeMilliseconds().ToString("x");
}
