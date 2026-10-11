using HushWhisper.Models;

namespace HushWhisper.Services;

/// <summary>A single ASR backend (Whisper, Qwen3-ASR, ...) selected by model name.</summary>
public interface IAsrEngine : IDisposable
{
    ModelFamily Family { get; }

    bool IsLoaded { get; }

    string? LoadedModel { get; }

    IReadOnlyList<ModelInfo> AvailableModels { get; }

    bool IsModelDownloaded(string modelName);

    Task InitializeAsync(string modelName, IProgress<DownloadProgress>? progress = null, CancellationToken cancellationToken = default);

    Task<string> TranscribeAsync(float[] audio16k, bool insertNewline, CancellationToken cancellationToken = default);

    void Unload();
}
