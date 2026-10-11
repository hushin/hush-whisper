using System.Net.Http;
using HushWhisper.Models;
using Whisper.net;

namespace HushWhisper.Services;

public sealed class WhisperAsrService : IAsrEngine
{
    private const string WhisperBaseUrl = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

    private static readonly HttpClient Http = new();

    private readonly object _lock = new();
    private readonly SemaphoreSlim _transcribeLock = new(1, 1);
    private readonly string _modelDirectory;
    private WhisperFactory? _factory;
    private WhisperProcessor? _processor;

    public WhisperAsrService()
    {
        _modelDirectory = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "voice-input",
            "models");
    }

    public ModelFamily Family => ModelFamily.Whisper;

    public bool IsLoaded
    {
        get
        {
            lock (_lock)
            {
                return _processor is not null;
            }
        }
    }

    public string? LoadedModel { get; private set; }

    public IReadOnlyList<ModelInfo> AvailableModels { get; } = ModelCatalog.InfoFor(ModelFamily.Whisper);

    public bool IsModelDownloaded(string modelName)
    {
        var def = GetModelDef(modelName);
        return def is not null && File.Exists(GetModelPath(def));
    }

    public async Task InitializeAsync(string modelName, IProgress<DownloadProgress>? progress = null, CancellationToken cancellationToken = default)
    {
        var def = GetModelDef(modelName)
                  ?? throw new ArgumentException($"Unknown model: {modelName}", nameof(modelName));

        lock (_lock)
        {
            if (LoadedModel == modelName && _processor is not null)
            {
                return;
            }
        }

        var path = GetModelPath(def);
        if (!File.Exists(path))
        {
            AppLog.Write($"Downloading model {modelName} from {WhisperBaseUrl}/{def.Source}");
            await DownloadModelAsync($"{WhisperBaseUrl}/{def.Source}", path, progress, cancellationToken);
        }

        // Build the new model without disturbing the currently loaded one.
        var factory = WhisperFactory.FromPath(path);
        var processor = factory.CreateBuilder()
            .WithLanguage("ja")
            .Build();

        // Swap under the transcription lock so an in-flight transcription can never
        // have its native processor disposed underneath it (which would crash).
        await _transcribeLock.WaitAsync(cancellationToken);
        try
        {
            lock (_lock)
            {
                DisposeModel();
                _factory = factory;
                _processor = processor;
                LoadedModel = modelName;
            }
        }
        finally
        {
            _transcribeLock.Release();
        }
    }

    public async Task<string> TranscribeAsync(float[] audio16k, bool insertNewline, CancellationToken cancellationToken = default)
    {
        // Acquire the transcription lock BEFORE reading the processor so a concurrent
        // model swap (which also holds this lock) can never dispose it mid-transcription.
        await _transcribeLock.WaitAsync(cancellationToken);
        try
        {
            WhisperProcessor processor;
            lock (_lock)
            {
                processor = _processor ?? throw new InvalidOperationException("ASR model is not initialized.");
            }

            var separator = insertNewline ? "\n" : string.Empty;
            var texts = new List<string>();

            await foreach (var segment in processor.ProcessAsync(audio16k, cancellationToken))
            {
                var text = segment.Text.Trim();
                if (text.Length > 0)
                {
                    texts.Add(text);
                }
            }

            return string.Join(separator, texts);
        }
        finally
        {
            _transcribeLock.Release();
        }
    }

    public void Unload()
    {
        lock (_lock)
        {
            DisposeModel();
            LoadedModel = null;
        }
    }

    private void DisposeModel()
    {
        _processor?.Dispose();
        _processor = null;
        _factory?.Dispose();
        _factory = null;
    }

    public void Dispose()
    {
        Unload();
        _transcribeLock.Dispose();
    }

    private static ModelDef? GetModelDef(string name)
    {
        var def = ModelCatalog.Find(name);
        return def is { Family: ModelFamily.Whisper } ? def : null;
    }

    private string GetModelPath(ModelDef def) => Path.Combine(_modelDirectory, def.Source);

    private static async Task DownloadModelAsync(string url, string targetPath, IProgress<DownloadProgress>? progress, CancellationToken cancellationToken)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(targetPath)!);

        var tempPath = targetPath + ".tmp";

        using var response = await Http.GetAsync(url, HttpCompletionOption.ResponseHeadersRead, cancellationToken);
        response.EnsureSuccessStatusCode();

        var total = response.Content.Headers.ContentLength ?? 0;
        var downloaded = 0L;

        await using (var source = await response.Content.ReadAsStreamAsync(cancellationToken))
        await using (var destination = File.Create(tempPath))
        {
            var buffer = new byte[81920];
            int read;
            while ((read = await source.ReadAsync(buffer, cancellationToken)) > 0)
            {
                await destination.WriteAsync(buffer.AsMemory(0, read), cancellationToken);
                downloaded += read;
                progress?.Report(new DownloadProgress(downloaded, total, total > 0 ? downloaded * 100.0 / total : 0));
            }
        }

        File.Move(tempPath, targetPath, overwrite: true);
    }
}
