using System.Text.Json;
using System.Text.Json.Serialization;
using HushWhisper.Models;

namespace HushWhisper.Services;

public interface ISettingsService
{
    AppSettings Current { get; }

    string ConfigPath { get; }

    AppSettings Reload();

    void Save(AppSettings settings);

    void Update(Action<AppSettings> mutate);
}

public sealed class SettingsService : ISettingsService
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        WriteIndented = true,
        Converters = { new JsonStringEnumConverter() },
    };

    private readonly object _lock = new();
    private AppSettings _current;

    public SettingsService()
    {
        ConfigPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "voice-input",
            "config.json");
        _current = LoadFromDisk();
    }

    public string ConfigPath { get; }

    public AppSettings Current
    {
        get
        {
            lock (_lock)
            {
                return _current;
            }
        }
    }

    public AppSettings Reload()
    {
        lock (_lock)
        {
            _current = LoadFromDisk();
            return _current;
        }
    }

    public void Save(AppSettings settings)
    {
        lock (_lock)
        {
            _current = settings;
            SaveToDisk(settings);
        }
    }

    public void Update(Action<AppSettings> mutate)
    {
        lock (_lock)
        {
            mutate(_current);
            SaveToDisk(_current);
        }
    }

    private AppSettings LoadFromDisk()
    {
        try
        {
            if (!File.Exists(ConfigPath))
            {
                return new AppSettings();
            }

            var json = File.ReadAllText(ConfigPath);
            var settings = JsonSerializer.Deserialize<AppSettings>(json, JsonOptions) ?? new AppSettings();
            settings.IsSaved = true;
            return settings;
        }
        catch
        {
            return new AppSettings();
        }
    }

    private void SaveToDisk(AppSettings settings)
    {
        var directory = Path.GetDirectoryName(ConfigPath);
        if (!string.IsNullOrEmpty(directory))
        {
            Directory.CreateDirectory(directory);
        }

        File.WriteAllText(ConfigPath, JsonSerializer.Serialize(settings, JsonOptions));
    }
}
