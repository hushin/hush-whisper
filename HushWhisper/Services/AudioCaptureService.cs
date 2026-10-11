using NAudio.CoreAudioApi;
using NAudio.Wave;
using NAudio.Wave.SampleProviders;

namespace HushWhisper.Services;

public interface IAudioCaptureService
{
    bool IsRecording { get; }

    /// <summary>Sample rate of the last recording session, in Hz.</summary>
    int CaptureSampleRate { get; }

    void Start();

    /// <summary>Stops recording and returns mono float samples at <see cref="CaptureSampleRate"/>.</summary>
    float[] Stop();
}

public sealed class AudioCaptureService : IAudioCaptureService
{
    private readonly object _lock = new();
    private readonly List<float> _buffer = [];
    private WasapiCapture? _capture;
    private WaveFormat? _format;
    private bool _recording;

    public bool IsRecording
    {
        get
        {
            lock (_lock)
            {
                return _recording;
            }
        }
    }

    public int CaptureSampleRate
    {
        get
        {
            lock (_lock)
            {
                return _sampleRate;
            }
        }
    }

    private int _sampleRate = 16000;

    public void Start()
    {
        lock (_lock)
        {
            _buffer.Clear();
            _recording = true;
        }

        var capture = new WasapiCapture();
        _capture = capture;
        _format = capture.WaveFormat;

        lock (_lock)
        {
            _sampleRate = _format.SampleRate == 0 ? 16000 : _format.SampleRate;
        }

        capture.DataAvailable += OnDataAvailable;
        capture.StartRecording();
    }

    public float[] Stop()
    {
        var capture = _capture;
        _capture = null;

        if (capture is not null)
        {
            capture.DataAvailable -= OnDataAvailable;
            try
            {
                capture.StopRecording();
            }
            catch
            {
                // Ignore stop errors; we already have the captured audio.
            }
        }

        float[] result;
        lock (_lock)
        {
            _recording = false;
            result = _buffer.ToArray();
        }

        capture?.Dispose();
        _format = null;
        return result;
    }

    private void OnDataAvailable(object? sender, WaveInEventArgs e)
    {
        var format = _format;
        if (format is null)
        {
            return;
        }

        var samples = ConvertToMono(
            e.Buffer,
            e.BytesRecorded,
            format.BitsPerSample,
            format.Channels,
            format.Encoding == WaveFormatEncoding.IeeeFloat);

        lock (_lock)
        {
            _buffer.AddRange(samples);
        }
    }

    private static float[] ConvertToMono(byte[] buffer, int bytesRecorded, int bitsPerSample, int channels, bool isFloat)
    {
        if (channels <= 0)
        {
            channels = 1;
        }

        var bytesPerSample = bitsPerSample / 8;
        if (bytesPerSample <= 0)
        {
            bytesPerSample = 4;
        }

        var frameSize = bytesPerSample * channels;
        if (frameSize <= 0)
        {
            return [];
        }

        var frames = bytesRecorded / frameSize;
        var output = new float[frames];

        for (var f = 0; f < frames; f++)
        {
            var frameOffset = f * frameSize;
            var sum = 0f;
            for (var c = 0; c < channels; c++)
            {
                sum += ReadSample(buffer, frameOffset + (c * bytesPerSample), bitsPerSample, isFloat);
            }

            output[f] = sum / channels;
        }

        return output;
    }

    private static float ReadSample(byte[] buffer, int offset, int bitsPerSample, bool isFloat)
    {
        if (isFloat && bitsPerSample == 32)
        {
            return BitConverter.ToSingle(buffer, offset);
        }

        return bitsPerSample switch
        {
            8 => (buffer[offset] - 128) / 128f,
            16 => BitConverter.ToInt16(buffer, offset) / 32768f,
            24 => (buffer[offset] | (buffer[offset + 1] << 8) | ((sbyte)buffer[offset + 2] << 16)) / 8388608f,
            32 => BitConverter.ToInt32(buffer, offset) / 2147483648f,
            _ => 0f,
        };
    }
}

/// <summary>Resamples mono float audio to 16 kHz using NAudio's WDL resampler.</summary>
internal static class AudioResampler
{
    public const int TargetSampleRate = 16000;

    public static float[] Resample(float[] input, int inputSampleRate)
    {
        if (inputSampleRate == TargetSampleRate || input.Length == 0)
        {
            return input;
        }

        var source = new ArraySampleProvider(input, inputSampleRate);
        var resampler = new WdlResamplingSampleProvider(source, TargetSampleRate);

        var output = new List<float>((int)(input.Length * (double)TargetSampleRate / inputSampleRate) + 1024);
        var buffer = new float[4096];
        int read;
        while ((read = resampler.Read(buffer, 0, buffer.Length)) > 0)
        {
            output.AddRange(buffer.AsSpan(0, read).ToArray());
        }

        return output.ToArray();
    }

    private sealed class ArraySampleProvider : ISampleProvider
    {
        private readonly float[] _data;
        private int _position;

        public ArraySampleProvider(float[] data, int sampleRate)
        {
            _data = data;
            WaveFormat = WaveFormat.CreateIeeeFloatWaveFormat(sampleRate, 1);
        }

        public WaveFormat WaveFormat { get; }

        public int Read(float[] buffer, int offset, int count)
        {
            var remaining = _data.Length - _position;
            var toCopy = Math.Min(remaining, count);
            Array.Copy(_data, _position, buffer, offset, toCopy);
            _position += toCopy;
            return toCopy;
        }
    }
}
