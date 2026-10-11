using System.Formats.Tar;
using System.Net.Http;
using HushWhisper.Models;
using SharpCompress.Compressors;
using SharpCompress.Compressors.BZip2;
using SherpaOnnx;

namespace HushWhisper.Services;

/// <summary>
/// Qwen3-ASR transcriber backed by sherpa-onnx (ONNX Runtime). Models are
/// downloaded from the official sherpa-onnx release archives and extracted
/// under %APPDATA%/voice-input/models/qwen3.
/// </summary>
public sealed class Qwen3AsrService : IAsrEngine
{
    private const string ReleaseBaseUrl = "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models";

    private static readonly string[] RequiredFiles =
    [
        "conv_frontend.onnx",
        "encoder.int8.onnx",
        "decoder.int8.onnx",
        Path.Combine("tokenizer", "vocab.json"),
        Path.Combine("tokenizer", "merges.txt"),
        Path.Combine("tokenizer", "tokenizer_config.json"),
    ];

    private static readonly HttpClient Http = new();

    private readonly object _lock = new();
    private readonly SemaphoreSlim _transcribeLock = new(1, 1);
    private readonly string _modelRoot;
    private OfflineRecognizer? _recognizer;

    public Qwen3AsrService()
    {
        _modelRoot = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "voice-input",
            "models",
            "qwen3");
    }

    public ModelFamily Family => ModelFamily.Qwen3;

    public bool IsLoaded
    {
        get
        {
            lock (_lock)
            {
                return _recognizer is not null;
            }
        }
    }

    public string? LoadedModel { get; private set; }

    public IReadOnlyList<ModelInfo> AvailableModels { get; } = ModelCatalog.InfoFor(ModelFamily.Qwen3);

    public bool IsModelDownloaded(string modelName)
    {
        var def = GetModelDef(modelName);
        if (def is null)
        {
            return false;
        }

        var dir = GetModelDir(def);
        return RequiredFiles.All(f => File.Exists(Path.Combine(dir, f)));
    }

    public async Task InitializeAsync(string modelName, IProgress<DownloadProgress>? progress = null, CancellationToken cancellationToken = default)
    {
        var def = GetModelDef(modelName)
                  ?? throw new ArgumentException($"Unknown Qwen3-ASR model: {modelName}", nameof(modelName));

        lock (_lock)
        {
            if (LoadedModel == modelName && _recognizer is not null)
            {
                return;
            }
        }

        var dir = GetModelDir(def);
        if (!IsModelDownloaded(modelName))
        {
            AppLog.Write($"Downloading Qwen3-ASR model {modelName} from {ReleaseBaseUrl}/{def.Source}");
            await DownloadAndExtractAsync(def, progress, cancellationToken);
        }

        // Native recognizer creation does blocking IO (mmap + graph init), so keep it
        // off the UI thread.
        var recognizer = await Task.Run(() => CreateRecognizer(dir), cancellationToken);

        await _transcribeLock.WaitAsync(cancellationToken);
        try
        {
            lock (_lock)
            {
                _recognizer?.Dispose();
                _recognizer = recognizer;
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
        await _transcribeLock.WaitAsync(cancellationToken);
        try
        {
            OfflineRecognizer recognizer;
            lock (_lock)
            {
                recognizer = _recognizer ?? throw new InvalidOperationException("ASR model is not initialized.");
            }

            using var stream = recognizer.CreateStream();
            stream.AcceptWaveform(16000, audio16k);
            recognizer.Decode(stream);

            return CleanTranscript(stream.Result.Text);
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
            _recognizer?.Dispose();
            _recognizer = null;
            LoadedModel = null;
        }
    }

    public void Dispose()
    {
        Unload();
        _transcribeLock.Dispose();
    }

    private static OfflineRecognizer CreateRecognizer(string dir)
    {
        var config = new OfflineRecognizerConfig();
        config.ModelConfig.Qwen3Asr.ConvFrontend = Path.Combine(dir, "conv_frontend.onnx");
        config.ModelConfig.Qwen3Asr.Encoder = Path.Combine(dir, "encoder.int8.onnx");
        config.ModelConfig.Qwen3Asr.Decoder = Path.Combine(dir, "decoder.int8.onnx");
        config.ModelConfig.Qwen3Asr.Tokenizer = Path.Combine(dir, "tokenizer");
        config.ModelConfig.Qwen3Asr.MaxNewTokens = 512;
        config.ModelConfig.Qwen3Asr.Hotwords = "";
        config.ModelConfig.Tokens = "";
        config.ModelConfig.NumThreads = Math.Clamp(Environment.ProcessorCount / 2, 1, 8);

        return new OfflineRecognizer(config);
    }

    /// <summary>
    /// Qwen3-ASR may emit a language tag followed by an <c>&lt;asr_text&gt;</c> marker
    /// before the transcript. sherpa-onnx usually strips it, but guard against any
    /// leftover so only the spoken text is returned.
    /// </summary>
    private static string CleanTranscript(string? raw)
    {
        if (string.IsNullOrEmpty(raw))
        {
            return string.Empty;
        }

        const string marker = "<asr_text>";
        var index = raw.LastIndexOf(marker, StringComparison.Ordinal);
        return (index >= 0 ? raw[(index + marker.Length)..] : raw).Trim();
    }

    private static ModelDef? GetModelDef(string name)
    {
        var def = ModelCatalog.Find(name);
        return def is { Family: ModelFamily.Qwen3 } ? def : null;
    }

    private string GetModelDir(ModelDef def)
    {
        // The release archive has a single top-level directory named after the asset
        // (minus the .tar.bz2 suffix); the model files live inside it.
        var folder = Path.GetFileNameWithoutExtension(Path.GetFileNameWithoutExtension(def.Source));
        return Path.Combine(_modelRoot, def.Name, folder);
    }

    private async Task DownloadAndExtractAsync(ModelDef def, IProgress<DownloadProgress>? progress, CancellationToken cancellationToken)
    {
        var extractDir = Path.Combine(_modelRoot, def.Name);
        Directory.CreateDirectory(extractDir);

        var archivePath = Path.Combine(extractDir, def.Source);
        await DownloadFileAsync($"{ReleaseBaseUrl}/{def.Source}", archivePath, progress, cancellationToken);

        await Task.Run(() => ExtractTarBz2(archivePath, extractDir), cancellationToken);

        File.Delete(archivePath);
    }

    private static void ExtractTarBz2(string archivePath, string destinationDirectory)
    {
        using var file = File.OpenRead(archivePath);
        using var decompressed = BZip2Stream.Create(file, CompressionMode.Decompress, decompressConcatenated: false);
        TarFile.ExtractToDirectory(decompressed, destinationDirectory, overwriteFiles: true);
    }

    private static async Task DownloadFileAsync(string url, string targetPath, IProgress<DownloadProgress>? progress, CancellationToken cancellationToken)
    {
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
