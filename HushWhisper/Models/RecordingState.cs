namespace HushWhisper.Models;

/// <summary>High-level state of the recording/transcription pipeline.</summary>
public enum RecordingState
{
    Idle,
    Recording,
    Processing,
}
