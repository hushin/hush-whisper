// Category types
export type CategoryId =
  | 'speech-recognition'
  | 'result-history'
  | 'shortcut-output'
  | 'llm'
  | 'general'
  | 'about';

export interface Category {
  id: CategoryId;
  label: string;
}

export const categories: Category[] = [
  { id: 'speech-recognition', label: '音声認識' },
  { id: 'shortcut-output', label: 'ショートカット・出力' },
  { id: 'llm', label: 'LLM' },
  { id: 'general', label: '環境設定' },
  { id: 'result-history', label: '結果・履歴' },
  { id: 'about', label: 'About' },
];

// Model types
export interface DownloadProgress {
  downloaded: number;
  total: number;
  percentage: number;
}

export interface ModelInfo {
  name: string;
  filename: string;
  size_hint: string;
}

// Settings types
export interface WhisperSettings {
  model_name: string;
  insert_newline: boolean;
  max_recording_seconds: number;
}

export type PromptPreset = 'Default' | 'Meeting' | 'Memo' | 'Chat' | 'Custom';

export type LlmProvider = 'Ollama' | 'OpenAICompat';

export type OutputMode = 'ClipboardOnly' | 'DirectInput' | 'Both';

export interface LlmSettings {
  enabled: boolean;
  provider: LlmProvider;
  api_url: string;
  model_name: string;
  preset: PromptPreset;
  custom_prompt: string;
}

export const llmProviderDescriptions: Record<LlmProvider, string> = {
  Ollama: 'Ollama',
  OpenAICompat: 'OpenAI互換 (LM Studio等)',
};

export const llmProviderDefaultUrls: Record<LlmProvider, string> = {
  Ollama: 'http://localhost:11434',
  OpenAICompat: 'http://localhost:1234',
};

export interface ShortcutSettings {
  recording_toggle: string;
}

export interface Settings {
  whisper: WhisperSettings;
  llm: LlmSettings;
  output_mode: OutputMode;
  shortcut: ShortcutSettings;
  is_saved: boolean;
}

// Log types
export interface LogEntry {
  id: string;
  timestamp: string;
  raw_text: string;
  refined_text: string | null;
  audio_duration_secs: number | null;
  llm_used: boolean;
  prompt_preset: string | null;
}

// Description types
export const presetDescriptions: Record<PromptPreset, string> = {
  Default: '自然な日本語に整形',
  Meeting: '議事録形式で整理',
  Memo: '簡潔なメモに要約',
  Chat: 'カジュアルなチャット文',
  Custom: 'カスタムプロンプト',
};

export const outputModeDescriptions: Record<OutputMode, string> = {
  ClipboardOnly: 'クリップボードにコピーのみ',
  DirectInput: '直接入力（クリップボード保持しない）',
  Both: 'コピー + 直接入力',
};
