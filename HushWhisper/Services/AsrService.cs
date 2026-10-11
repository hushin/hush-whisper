using HushWhisper.Models;

namespace HushWhisper.Services;

public interface IAsrService
{
    bool IsLoaded { get; }

    string? LoadedModel { get; }

    IReadOnlyList<ModelInfo> AvailableModels { get; }

    bool IsModelDownloaded(string modelName);

    Task InitializeAsync(string modelName, IProgress<DownloadProgress>? progress = null, CancellationToken cancellationToken = default);

    Task<string> TranscribeAsync(float[] audio16k, bool insertNewline, CancellationToken cancellationToken = default);

    void Unload();
}

/// <summary>
/// Routes ASR requests to the engine (Whisper / Qwen3-ASR) that owns the requested
/// model. Only one model is kept loaded at a time so GPU/VRAM use stays bounded.
/// </summary>
public sealed class AsrService : IAsrService
{
    private readonly IReadOnlyList<IAsrEngine> _engines;

    public AsrService(WhisperAsrService whisper, Qwen3AsrService qwen3)
    {
        _engines = [whisper, qwen3];
        AvailableModels = _engines.SelectMany(e => e.AvailableModels).ToList();
    }

    public IReadOnlyList<ModelInfo> AvailableModels { get; }

    public bool IsLoaded => _engines.Any(e => e.IsLoaded);

    public string? LoadedModel => _engines.FirstOrDefault(e => e.IsLoaded)?.LoadedModel;

    public bool IsModelDownloaded(string modelName) =>
        GetEngine(modelName)?.IsModelDownloaded(modelName) ?? false;

    public async Task InitializeAsync(string modelName, IProgress<DownloadProgress>? progress = null, CancellationToken cancellationToken = default)
    {
        var engine = GetEngine(modelName)
                     ?? throw new ArgumentException($"Unknown model: {modelName}", nameof(modelName));

        // Free the other engine before loading the new model (Whisper and Qwen3 both
        // want the GPU, and Qwen3 alone needs well over a gigabyte).
        foreach (var other in _engines)
        {
            if (!ReferenceEquals(other, engine))
            {
                other.Unload();
            }
        }

        await engine.InitializeAsync(modelName, progress, cancellationToken);
    }

    public Task<string> TranscribeAsync(float[] audio16k, bool insertNewline, CancellationToken cancellationToken = default)
    {
        var engine = _engines.FirstOrDefault(e => e.IsLoaded)
                     ?? throw new InvalidOperationException("ASR model is not initialized.");

        return engine.TranscribeAsync(audio16k, insertNewline, cancellationToken);
    }

    public void Unload()
    {
        foreach (var engine in _engines)
        {
            engine.Unload();
        }
    }

    private IAsrEngine? GetEngine(string modelName) =>
        _engines.FirstOrDefault(e => e.AvailableModels.Any(m => m.Name == modelName));
}
