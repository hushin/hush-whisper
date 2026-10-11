namespace HushWhisper.Models;

public enum ModelFamily
{
    Whisper,
    Qwen3,
}

public sealed record ModelDef(
    string Name,
    ModelFamily Family,
    string Source,
    string SizeHint);

public sealed record ModelInfo(
    string Name,
    string Filename,
    string SizeHint,
    string Family);

public sealed record DownloadProgress(
    long Downloaded,
    long Total,
    double Percentage);

/// <summary>Registry of every ASR model the app can load, across all engine families.</summary>
public static class ModelCatalog
{
    public static readonly IReadOnlyList<ModelDef> Models =
    [
        new("large-v3-turbo", ModelFamily.Whisper, "ggml-large-v3-turbo.bin", "~1.5GB (Whisper 推奨)"),
        new("large-v3-turbo-q8_0", ModelFamily.Whisper, "ggml-large-v3-turbo-q8_0.bin", "~820MB"),
        new("medium", ModelFamily.Whisper, "ggml-medium.bin", "~1.5GB"),
        new("small", ModelFamily.Whisper, "ggml-small.bin", "~500MB"),
        new("base", ModelFamily.Whisper, "ggml-base.bin", "~150MB"),
        new("tiny", ModelFamily.Whisper, "ggml-tiny.bin", "~77MB"),
        new("qwen3-asr-0.6b", ModelFamily.Qwen3, "sherpa-onnx-qwen3-asr-0.6B-int8-2026-03-25.tar.bz2", "~940MB (Qwen3-ASR 日本語高精度)"),
    ];

    public static ModelDef? Find(string name) => Models.FirstOrDefault(m => m.Name == name);

    public static IReadOnlyList<ModelInfo> InfoFor(ModelFamily family) => Models
        .Where(m => m.Family == family)
        .Select(m => new ModelInfo(m.Name, m.Source, m.SizeHint, FamilyName(m.Family)))
        .ToList();

    public static string FamilyName(ModelFamily family) => family switch
    {
        ModelFamily.Qwen3 => "qwen3",
        _ => "whisper",
    };
}
