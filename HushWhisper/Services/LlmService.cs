using System.Net.Http;
using System.Net.Http.Json;
using System.Text.Json.Serialization;
using HushWhisper.Models;

namespace HushWhisper.Services;

public interface ILlmService
{
    Task<string> RefineAsync(string rawText, LlmSettings settings, CancellationToken cancellationToken = default);

    Task<bool> IsAvailableAsync(string apiUrl, LlmProvider provider, CancellationToken cancellationToken = default);
}

public sealed class LlmService : ILlmService
{
    private static readonly HttpClient Http = new() { Timeout = TimeSpan.FromMinutes(5) };

    public async Task<string> RefineAsync(string rawText, LlmSettings settings, CancellationToken cancellationToken = default)
    {
        var prompt = settings.GetPromptTemplate().Replace("{input}", rawText);
        var baseUrl = settings.ApiUrl.TrimEnd('/');

        return settings.Provider switch
        {
            LlmProvider.OpenAICompat => await RefineWithOpenAiCompatAsync(baseUrl, settings.ModelName, prompt, cancellationToken),
            _ => await RefineWithOllamaAsync(baseUrl, settings.ModelName, prompt, cancellationToken),
        };
    }

    public async Task<bool> IsAvailableAsync(string apiUrl, LlmProvider provider, CancellationToken cancellationToken = default)
    {
        var baseUrl = apiUrl.TrimEnd('/');
        var url = provider == LlmProvider.OpenAICompat ? $"{baseUrl}/v1/models" : $"{baseUrl}/api/tags";

        try
        {
            using var response = await Http.GetAsync(url, cancellationToken);
            return response.IsSuccessStatusCode;
        }
        catch
        {
            return false;
        }
    }

    private static async Task<string> RefineWithOllamaAsync(string baseUrl, string model, string prompt, CancellationToken cancellationToken)
    {
        var url = $"{baseUrl}/api/generate";
        var request = new OllamaGenerateRequest(model, prompt, false);

        using var response = await Http.PostAsJsonAsync(url, request, cancellationToken);
        if (!response.IsSuccessStatusCode)
        {
            var body = await response.Content.ReadAsStringAsync(cancellationToken);
            throw new InvalidOperationException($"Ollama API error ({(int)response.StatusCode}): {body}");
        }

        var result = await response.Content.ReadFromJsonAsync<OllamaGenerateResponse>(cancellationToken);
        return result?.Response?.Trim() ?? string.Empty;
    }

    private static async Task<string> RefineWithOpenAiCompatAsync(string baseUrl, string model, string prompt, CancellationToken cancellationToken)
    {
        var url = $"{baseUrl}/v1/chat/completions";
        var request = new OpenAiChatRequest(model, [new OpenAiMessage("user", prompt)], false);

        using var response = await Http.PostAsJsonAsync(url, request, cancellationToken);
        if (!response.IsSuccessStatusCode)
        {
            var body = await response.Content.ReadAsStringAsync(cancellationToken);
            throw new InvalidOperationException($"OpenAI-compatible API error ({(int)response.StatusCode}): {body}");
        }

        var result = await response.Content.ReadFromJsonAsync<OpenAiChatResponse>(cancellationToken);
        return result?.Choices?.FirstOrDefault()?.Message.Content?.Trim() ?? string.Empty;
    }

    private sealed record OllamaGenerateRequest(
        [property: JsonPropertyName("model")] string Model,
        [property: JsonPropertyName("prompt")] string Prompt,
        [property: JsonPropertyName("stream")] bool Stream);

    private sealed record OllamaGenerateResponse(
        [property: JsonPropertyName("response")] string? Response);

    private sealed record OpenAiChatRequest(
        [property: JsonPropertyName("model")] string Model,
        [property: JsonPropertyName("messages")] IReadOnlyList<OpenAiMessage> Messages,
        [property: JsonPropertyName("stream")] bool Stream);

    private sealed record OpenAiMessage(
        [property: JsonPropertyName("role")] string Role,
        [property: JsonPropertyName("content")] string Content);

    private sealed record OpenAiChatResponse(
        [property: JsonPropertyName("choices")] IReadOnlyList<OpenAiChoice>? Choices);

    private sealed record OpenAiChoice(
        [property: JsonPropertyName("message")] OpenAiMessage Message);
}
