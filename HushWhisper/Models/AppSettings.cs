using System.Text.Json.Serialization;

namespace HushWhisper.Models;

/// <summary>Application settings, persisted to %APPDATA%/voice-input/config.json.</summary>
public sealed class AppSettings
{
    public WhisperSettings Whisper { get; set; } = new();

    public LlmSettings Llm { get; set; } = new();

    public OutputMode OutputMode { get; set; } = OutputMode.DirectInput;

    public ShortcutSettings Shortcut { get; set; } = new();

    /// <summary>Whether settings were loaded from a saved file (not defaults).</summary>
    [JsonIgnore]
    public bool IsSaved { get; set; }
}

public sealed class WhisperSettings
{
    public string ModelName { get; set; } = "large-v3-turbo";

    public bool InsertNewline { get; set; } = true;

    public uint MaxRecordingSeconds { get; set; } = 300;
}

public sealed class ShortcutSettings
{
    public string RecordingToggle { get; set; } = "Ctrl+Space";
}

public enum OutputMode
{
    ClipboardOnly,
    DirectInput,
    Both,
}

public enum LlmProvider
{
    Ollama,
    OpenAICompat,
}

public enum PromptPreset
{
    Default,
    Meeting,
    Memo,
    Chat,
    Custom,
}

public sealed class LlmSettings
{
    public bool Enabled { get; set; }

    public LlmProvider Provider { get; set; } = LlmProvider.Ollama;

    public string ApiUrl { get; set; } = "http://localhost:11434";

    public string ModelName { get; set; } = "gpt-oss:20b";

    public PromptPreset Preset { get; set; } = PromptPreset.Default;

    public string CustomPrompt { get; set; } = string.Empty;

    public string GetPromptTemplate()
    {
        if (Preset == PromptPreset.Custom)
        {
            return string.IsNullOrEmpty(CustomPrompt) ? PromptPresets.Get(PromptPreset.Default) : CustomPrompt;
        }

        return PromptPresets.Get(Preset);
    }
}

public static class PromptPresets
{
    public static string Get(PromptPreset preset) => preset switch
    {
        PromptPreset.Default =>
            """
            以下の音声認識結果を自然な日本語に整形してください。
            誤字脱字の修正、句読点の追加、文法の修正を行ってください。
            整形後のテキストのみを出力してください。余計な説明は不要です。

            入力: {input}

            出力:
            """,
        PromptPreset.Meeting =>
            """
            以下の音声認識結果を議事録形式で整形してください。
            - 発言内容を箇条書きで整理
            - 重要なポイントや決定事項を明確に
            - 誤字脱字を修正
            整形後のテキストのみを出力してください。

            入力: {input}

            出力:
            """,
        PromptPreset.Memo =>
            """
            以下の音声認識結果を簡潔なメモに整形してください。
            - 要点を短くまとめる
            - 不要な言葉を省く
            - 誤字脱字を修正
            整形後のテキストのみを出力してください。

            入力: {input}

            出力:
            """,
        PromptPreset.Chat =>
            """
            以下の音声認識結果をカジュアルなチャット文に整形してください。
            - 口語的な表現を維持
            - 適度な絵文字や句読点を追加
            - 誤字脱字のみ修正
            整形後のテキストのみを出力してください。

            入力: {input}

            出力:
            """,
        PromptPreset.Custom => string.Empty,
        _ => string.Empty,
    };

    public static IReadOnlyList<(PromptPreset Preset, string Description)> Descriptions { get; } =
    [
        (PromptPreset.Default, "自然な日本語に整形"),
        (PromptPreset.Meeting, "議事録形式で整理"),
        (PromptPreset.Memo, "簡潔なメモに要約"),
        (PromptPreset.Chat, "カジュアルなチャット文"),
        (PromptPreset.Custom, "カスタムプロンプト"),
    ];
}
