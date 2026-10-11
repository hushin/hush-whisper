using HushWhisper.Models;

namespace HushWhisper.Services;

public interface IRecordingCoordinator
{
    RecordingState State { get; }

    bool IsRecording { get; }

    event EventHandler<RecordingState>? StateChanged;

    event EventHandler<string>? TranscriptionCompleted;

    event EventHandler<string>? StatusChanged;

    event EventHandler<DownloadProgress>? ModelDownloadProgress;

    Task InitializeModelAsync(string modelName, CancellationToken cancellationToken = default);

    Task ToggleAsync();

    Task StartAsync();

    Task StopAsync();
}

public sealed class RecordingCoordinator : IRecordingCoordinator
{
    private readonly ISettingsService _settings;
    private readonly IAudioCaptureService _audio;
    private readonly IAsrService _asr;
    private readonly ILlmService _llm;
    private readonly IVadService _vad;
    private readonly IClipboardService _clipboard;
    private readonly ILogService _log;

    private readonly SemaphoreSlim _gate = new(1, 1);
    private CancellationTokenSource? _autoStopCts;
    private RecordingState _state = RecordingState.Idle;

    public RecordingCoordinator(
        ISettingsService settings,
        IAudioCaptureService audio,
        IAsrService asr,
        ILlmService llm,
        IVadService vad,
        IClipboardService clipboard,
        ILogService log)
    {
        _settings = settings;
        _audio = audio;
        _asr = asr;
        _llm = llm;
        _vad = vad;
        _clipboard = clipboard;
        _log = log;
    }

    public RecordingState State => _state;

    public bool IsRecording => _audio.IsRecording;

    public event EventHandler<RecordingState>? StateChanged;

    public event EventHandler<string>? TranscriptionCompleted;

    public event EventHandler<string>? StatusChanged;

    public event EventHandler<DownloadProgress>? ModelDownloadProgress;

    public async Task InitializeModelAsync(string modelName, CancellationToken cancellationToken = default)
    {
        var progress = new Progress<DownloadProgress>(p => ModelDownloadProgress?.Invoke(this, p));
        var isDownloaded = _asr.IsModelDownloaded(modelName);

        StatusChanged?.Invoke(this, isDownloaded ? "モデルを読み込み中..." : "モデルをダウンロード中...");

        await _asr.InitializeAsync(modelName, progress, cancellationToken);

        _settings.Update(s => s.Whisper.ModelName = modelName);
        StatusChanged?.Invoke(this, "モデルの準備が完了しました");
    }

    public async Task ToggleAsync()
    {
        if (_state == RecordingState.Idle)
        {
            await StartAsync();
        }
        else if (_state == RecordingState.Recording)
        {
            await StopAsync();
        }
    }

    public async Task StartAsync()
    {
        await _gate.WaitAsync();
        try
        {
            if (_state != RecordingState.Idle)
            {
                return;
            }

            _audio.Start();
            SetState(RecordingState.Recording);

            var maxSeconds = _settings.Current.Whisper.MaxRecordingSeconds;
            if (maxSeconds > 0)
            {
                _autoStopCts = new CancellationTokenSource();
                var token = _autoStopCts.Token;
                _ = Task.Run(async () =>
                {
                    try
                    {
                        await Task.Delay(TimeSpan.FromSeconds(maxSeconds), token);
                        if (_state == RecordingState.Recording)
                        {
                            await StopAsync();
                        }
                    }
                    catch (OperationCanceledException)
                    {
                        // Manual stop cancelled the timer.
                    }
                });
            }
        }
        finally
        {
            _gate.Release();
        }
    }

    public async Task StopAsync()
    {
        await _gate.WaitAsync();
        try
        {
            if (_state != RecordingState.Recording)
            {
                return;
            }

            _autoStopCts?.Cancel();
            _autoStopCts?.Dispose();
            _autoStopCts = null;

            var raw = _audio.Stop();
            var captureRate = _audio.CaptureSampleRate;
            SetState(RecordingState.Processing);

            if (raw.Length == 0)
            {
                StatusChanged?.Invoke(this, "音声が取得できませんでした");
                SetState(RecordingState.Idle);
                return;
            }

            if (!_asr.IsLoaded)
            {
                StatusChanged?.Invoke(this, "モデルが読み込まれていません。音声認識の設定からモデルを選択してください。");
                SetState(RecordingState.Idle);
                return;
            }

            var resampled = AudioResampler.Resample(raw, captureRate);
            var durationSecs = resampled.Length / (float)AudioResampler.TargetSampleRate;
            AppLog.Write($"Captured {raw.Length} samples @ {captureRate}Hz -> {resampled.Length} samples @16kHz ({durationSecs:F2}s)");

            var speech = await _vad.ExtractSpeechAsync(resampled);
            AppLog.Write($"After VAD: {speech.Length} samples");

            if (speech.Length < 1600)
            {
                StatusChanged?.Invoke(this, "音声が検出されませんでした");
                SetState(RecordingState.Idle);
                return;
            }

            StatusChanged?.Invoke(this, "文字起こし中...");
            var text = await _asr.TranscribeAsync(speech, _settings.Current.Whisper.InsertNewline);
            AppLog.Write($"Transcribed: \"{text}\"");

            var settings = _settings.Current;
            var finalText = text;
            string? refined = null;

            if (settings.Llm.Enabled && !string.IsNullOrWhiteSpace(text))
            {
                StatusChanged?.Invoke(this, "LLMで整形中...");
                try
                {
                    refined = await _llm.RefineAsync(text, settings.Llm);
                    if (!string.IsNullOrEmpty(refined))
                    {
                        finalText = refined;
                    }
                }
                catch (Exception ex)
                {
                    AppLog.Write("LLM refinement failed", ex);
                    StatusChanged?.Invoke(this, $"LLM整形に失敗しました: {ex.Message}");
                }
            }

            if (!string.IsNullOrWhiteSpace(finalText))
            {
                AppLog.Write($"Output mode={settings.OutputMode}, text=\"{finalText}\"");
                _clipboard.OutputText(finalText, settings.OutputMode);

                _log.Add(
                    text,
                    settings.Llm.Enabled ? refined : null,
                    durationSecs,
                    settings.Llm.Enabled,
                    settings.Llm.Enabled ? settings.Llm.Preset.ToString() : null);
            }

            TranscriptionCompleted?.Invoke(this, finalText);
            SetState(RecordingState.Idle);
            StatusChanged?.Invoke(this, "待機中");
        }
        catch (Exception ex)
        {
            AppLog.Write("StopAsync failed", ex);
            StatusChanged?.Invoke(this, $"エラー: {ex.Message}");
            SetState(RecordingState.Idle);
        }
        finally
        {
            _gate.Release();
        }
    }

    private void SetState(RecordingState state)
    {
        _state = state;
        StateChanged?.Invoke(this, state);
    }
}
