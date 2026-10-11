using HushWhisper.Models;

namespace HushWhisper.Models;

public sealed record OutputModeOption(OutputMode Value, string Label);

public sealed record LlmProviderOption(LlmProvider Value, string Label);

public sealed record PromptPresetOption(PromptPreset Value, string Label);

public static class Options
{
    public static IReadOnlyList<OutputModeOption> OutputModes { get; } =
    [
        new(OutputMode.ClipboardOnly, "クリップボードにコピーのみ"),
        new(OutputMode.DirectInput, "直接入力（クリップボード保持しない）"),
        new(OutputMode.Both, "コピー + 直接入力"),
    ];

    public static IReadOnlyList<LlmProviderOption> LlmProviders { get; } =
    [
        new(LlmProvider.Ollama, "Ollama"),
        new(LlmProvider.OpenAICompat, "OpenAI互換 (LM Studio 等)"),
    ];

    public static IReadOnlyList<PromptPresetOption> PromptPresets { get; } =
        Models.PromptPresets.Descriptions
            .Select(d => new PromptPresetOption(d.Preset, d.Description))
            .ToList();
}
