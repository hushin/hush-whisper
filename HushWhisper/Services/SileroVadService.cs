using Whisper.net;
using Whisper.net.Ggml;

namespace HushWhisper.Services;

public interface IVadService
{
    /// <summary>
    /// Removes non-speech segments from 16 kHz mono audio. Returns the original
    /// audio when most of it is speech, or an empty array when no speech is found.
    /// </summary>
    Task<float[]> ExtractSpeechAsync(float[] audio16k, CancellationToken cancellationToken = default);
}

public sealed class SileroVadService : IVadService
{
    private const int SampleRate = 16000;
    private const float SpeechThreshold = 0.5f;

    private static readonly HttpClient Http = new();

    private readonly string _modelPath;
    private readonly SemaphoreSlim _initLock = new(1, 1);
    private WhisperVadFactory? _factory;

    public SileroVadService()
    {
        _modelPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "voice-input",
            "models",
            "ggml-silero-vad.bin");
    }

    public async Task<float[]> ExtractSpeechAsync(float[] audio16k, CancellationToken cancellationToken = default)
    {
        if (audio16k.Length < SampleRate / 2)
        {
            return audio16k;
        }

        try
        {
            var factory = await EnsureFactoryAsync(cancellationToken);

            using var vad = factory.CreateBuilder()
                .WithThreshold(SpeechThreshold)
                .WithUseGpu(false)
                .Build();

            var segments = vad.DetectSpeech(audio16k);
            if (segments.Count == 0)
            {
                return [];
            }

            var totalSpeechSamples = segments.Sum(s => (s.End - s.Start).TotalSeconds * SampleRate);
            if (totalSpeechSamples > audio16k.Length * 0.8)
            {
                return audio16k;
            }

            var output = new List<float>((int)totalSpeechSamples + SampleRate);
            foreach (var segment in segments)
            {
                var start = Math.Clamp((int)(segment.Start.TotalSeconds * SampleRate), 0, audio16k.Length);
                var end = Math.Clamp((int)(segment.End.TotalSeconds * SampleRate), 0, audio16k.Length);
                if (end > start)
                {
                    output.AddRange(audio16k.AsSpan(start, end - start).ToArray());
                }
            }

            return output.Count == 0 ? audio16k : output.ToArray();
        }
        catch
        {
            // VAD is a best-effort enhancement; fall back to the original audio.
            return audio16k;
        }
    }

    private async Task<WhisperVadFactory> EnsureFactoryAsync(CancellationToken cancellationToken)
    {
        if (_factory is not null)
        {
            return _factory;
        }

        await _initLock.WaitAsync(cancellationToken);
        try
        {
            if (_factory is not null)
            {
                return _factory;
            }

            if (!File.Exists(_modelPath))
            {
                await DownloadVadModelAsync(cancellationToken);
            }

            _factory = WhisperVadFactory.FromPath(_modelPath);
            return _factory;
        }
        finally
        {
            _initLock.Release();
        }
    }

    private async Task DownloadVadModelAsync(CancellationToken cancellationToken)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(_modelPath)!);

        var tempPath = _modelPath + ".tmp";
        try
        {
            using var modelStream = await WhisperGgmlDownloader.Default.GetGgmlSileroVadModelAsync(
                SileroVadType.V5_1_2,
                cancellationToken);
            await using (var fileWriter = File.Create(tempPath))
            {
                await modelStream.CopyToAsync(fileWriter, cancellationToken);
            }

            File.Move(tempPath, _modelPath, overwrite: true);
        }
        catch
        {
            try
            {
                File.Delete(tempPath);
            }
            catch
            {
                // ignore
            }

            throw;
        }
    }
}
