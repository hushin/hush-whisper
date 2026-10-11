using System.Text.Json.Serialization;

namespace HushWhisper.Models;

/// <summary>A single transcription log entry.</summary>
public sealed class LogEntry
{
    public string Id { get; set; } = string.Empty;

    public DateTimeOffset Timestamp { get; set; }

    public string RawText { get; set; } = string.Empty;

    public string? RefinedText { get; set; }

    public float? AudioDurationSecs { get; set; }

    public bool LlmUsed { get; set; }

    public string? PromptPreset { get; set; }

    [JsonIgnore]
    public string DisplayText => string.IsNullOrEmpty(RefinedText) ? RawText : RefinedText!;

    [JsonIgnore]
    public string TimestampText => Timestamp.LocalDateTime.ToString("yyyy-MM-dd HH:mm:ss");
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.SnakeCaseLower, WriteIndented = true)]
[JsonSerializable(typeof(LogEntry[]))]
internal partial class LogJsonContext : JsonSerializerContext;
