namespace HushWhisper.Models;

public enum ModelFamily
{
    Whisper,
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
