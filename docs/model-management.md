# モデル管理

## 概要

アプリ本体とモデルを分離し、必要なモデルのみダウンロード。バンドルサイズ削減とモデル更新の柔軟性を確保。

## モデル保存場所

```
%APPDATA%/voice-input/
├── models/
│   ├── whisper/
│   │   ├── ggml-large-v3-turbo.bin    # ~1.6GB
│   │   ├── ggml-medium.bin             # ~1.5GB (optional)
│   │   └── ggml-small.bin              # ~466MB (optional)
│   └── llm/
│       ├── qwen2.5-7b-instruct-q4_k_m.gguf  # ~4.7GB
│       └── qwen2.5-3b-instruct-q4_k_m.gguf  # ~2.0GB (optional)
├── config.json
└── logs/
```

## 1. モデル定義

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub filename: String,
    pub size_bytes: u64,
    pub model_type: ModelType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    Whisper,
    Llm,
}

/// 利用可能なモデルカタログ
pub fn get_available_models() -> Vec<ModelInfo> {
    vec![
        // Whisper models
        ModelInfo {
            id: "whisper-large-v3-turbo".into(),
            name: "Whisper Large v3 Turbo".into(),
            repo: "ggerganov/whisper.cpp".into(),
            filename: "ggml-large-v3-turbo.bin".into(),
            size_bytes: 1_600_000_000,
            model_type: ModelType::Whisper,
            description: "最高精度、CUDA推奨".into(),
        },
        ModelInfo {
            id: "whisper-medium".into(),
            name: "Whisper Medium".into(),
            repo: "ggerganov/whisper.cpp".into(),
            filename: "ggml-medium.bin".into(),
            size_bytes: 1_500_000_000,
            model_type: ModelType::Whisper,
            description: "バランス型".into(),
        },
        ModelInfo {
            id: "whisper-small".into(),
            name: "Whisper Small".into(),
            repo: "ggerganov/whisper.cpp".into(),
            filename: "ggml-small.bin".into(),
            size_bytes: 466_000_000,
            model_type: ModelType::Whisper,
            description: "軽量、CPU可".into(),
        },
        // LLM models
        ModelInfo {
            id: "qwen2.5-7b-instruct".into(),
            name: "Qwen2.5 7B Instruct".into(),
            repo: "Qwen/Qwen2.5-7B-Instruct-GGUF".into(),
            filename: "qwen2.5-7b-instruct-q4_k_m.gguf".into(),
            size_bytes: 4_700_000_000,
            model_type: ModelType::Llm,
            description: "日本語高精度、128Kコンテキスト".into(),
        },
        ModelInfo {
            id: "qwen2.5-3b-instruct".into(),
            name: "Qwen2.5 3B Instruct".into(),
            repo: "Qwen/Qwen2.5-3B-Instruct-GGUF".into(),
            filename: "qwen2.5-3b-instruct-q4_k_m.gguf".into(),
            size_bytes: 2_000_000_000,
            model_type: ModelType::Llm,
            description: "軽量、低VRAM環境向け".into(),
        },
    ]
}
```

## 2. ダウンローダー (hf-hub)

```rust
use hf_hub::api::tokio::{Api, ApiBuilder};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded: u64,
    pub total: u64,
    pub percent: f32,
    pub status: DownloadStatus,
}

#[derive(Clone, serde::Serialize)]
pub enum DownloadStatus {
    Starting,
    Downloading,
    Completed,
    Failed(String),
}

pub struct ModelDownloader {
    api: Api,
    models_dir: PathBuf,
}

impl ModelDownloader {
    pub fn new(app_data_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let models_dir = app_data_dir.join("models");
        std::fs::create_dir_all(&models_dir)?;

        let api = ApiBuilder::new()
            .with_cache_dir(&models_dir)
            .build()?;

        Ok(Self { api, models_dir })
    }

    /// モデルがダウンロード済みか確認
    pub fn is_downloaded(&self, model: &ModelInfo) -> bool {
        let subdir = match model.model_type {
            ModelType::Whisper => "whisper",
            ModelType::Llm => "llm",
        };
        self.models_dir
            .join(subdir)
            .join(&model.filename)
            .exists()
    }

    /// モデルのローカルパスを取得
    pub fn get_model_path(&self, model: &ModelInfo) -> PathBuf {
        let subdir = match model.model_type {
            ModelType::Whisper => "whisper",
            ModelType::Llm => "llm",
        };
        self.models_dir.join(subdir).join(&model.filename)
    }

    /// モデルをダウンロード（進捗コールバック付き）
    pub async fn download(
        &self,
        model: &ModelInfo,
        progress_tx: mpsc::Sender<DownloadProgress>,
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let model_id = model.id.clone();
        
        progress_tx.send(DownloadProgress {
            model_id: model_id.clone(),
            downloaded: 0,
            total: model.size_bytes,
            percent: 0.0,
            status: DownloadStatus::Starting,
        }).await?;

        let repo = self.api.model(model.repo.clone());
        
        // hf-hubはキャッシュを自動管理
        // ダウンロード済みならキャッシュから返す
        let cached_path = repo.get(&model.filename).await?;

        // 目的のディレクトリにコピー/シンボリックリンク
        let target_path = self.get_model_path(model);
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if !target_path.exists() {
            // Windowsではコピー、Unix系ではハードリンク
            #[cfg(windows)]
            std::fs::copy(&cached_path, &target_path)?;
            #[cfg(not(windows))]
            std::fs::hard_link(&cached_path, &target_path)?;
        }

        progress_tx.send(DownloadProgress {
            model_id,
            downloaded: model.size_bytes,
            total: model.size_bytes,
            percent: 100.0,
            status: DownloadStatus::Completed,
        }).await?;

        Ok(target_path)
    }

    /// ダウンロード済みモデル一覧
    pub fn list_downloaded(&self) -> Vec<ModelInfo> {
        get_available_models()
            .into_iter()
            .filter(|m| self.is_downloaded(m))
            .collect()
    }

    /// モデルを削除
    pub fn delete(&self, model: &ModelInfo) -> Result<(), std::io::Error> {
        let path = self.get_model_path(model);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}
```

## 3. Tauriコマンド

```rust
use tauri::ipc::Channel;
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn list_models() -> Vec<ModelInfo> {
    get_available_models()
}

#[tauri::command]
pub async fn get_downloaded_models(
    downloader: State<'_, Arc<Mutex<ModelDownloader>>>,
) -> Result<Vec<ModelInfo>, String> {
    let dl = downloader.lock().await;
    Ok(dl.list_downloaded())
}

#[tauri::command]
pub async fn download_model(
    model_id: String,
    on_progress: Channel<DownloadProgress>,
    downloader: State<'_, Arc<Mutex<ModelDownloader>>>,
) -> Result<String, String> {
    let model = get_available_models()
        .into_iter()
        .find(|m| m.id == model_id)
        .ok_or("Model not found")?;

    let dl = downloader.lock().await;
    
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    
    // 進捗をChannelに転送
    let on_progress_clone = on_progress.clone();
    tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            on_progress_clone.send(progress).ok();
        }
    });

    let path = dl.download(&model, tx)
        .await
        .map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn delete_model(
    model_id: String,
    downloader: State<'_, Arc<Mutex<ModelDownloader>>>,
) -> Result<(), String> {
    let model = get_available_models()
        .into_iter()
        .find(|m| m.id == model_id)
        .ok_or("Model not found")?;

    let dl = downloader.lock().await;
    dl.delete(&model).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_model_path(
    model_id: String,
    downloader: State<'_, Arc<Mutex<ModelDownloader>>>,
) -> Result<String, String> {
    let model = get_available_models()
        .into_iter()
        .find(|m| m.id == model_id)
        .ok_or("Model not found")?;

    let dl = downloader.lock().await;
    
    if !dl.is_downloaded(&model) {
        return Err("Model not downloaded".into());
    }
    
    Ok(dl.get_model_path(&model).to_string_lossy().to_string())
}
```

## 4. フロントエンド連携

### TypeScript型定義

```typescript
interface ModelInfo {
  id: string;
  name: string;
  repo: string;
  filename: string;
  size_bytes: number;
  model_type: 'Whisper' | 'Llm';
  description: string;
}

interface DownloadProgress {
  model_id: string;
  downloaded: number;
  total: number;
  percent: number;
  status: 'Starting' | 'Downloading' | 'Completed' | { Failed: string };
}
```

### Svelte使用例

```svelte
<script lang="ts">
  import { invoke, Channel } from '@tauri-apps/api/core';
  
  let models: ModelInfo[] = [];
  let downloadProgress: Record<string, DownloadProgress> = {};
  
  async function loadModels() {
    models = await invoke('list_models');
  }
  
  async function downloadModel(modelId: string) {
    const onProgress = new Channel<DownloadProgress>();
    
    onProgress.onmessage = (progress) => {
      downloadProgress[progress.model_id] = progress;
    };
    
    try {
      await invoke('download_model', { modelId, onProgress });
    } catch (e) {
      console.error('Download failed:', e);
    }
  }
</script>
```

## 5. 設定ファイル

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub selected_whisper_model: String,
    pub selected_llm_model: Option<String>,
    pub llm_enabled: bool,
    pub shortcut: String,
    pub max_recording_seconds: u32,
    pub auto_paste: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            selected_whisper_model: "whisper-large-v3-turbo".into(),
            selected_llm_model: Some("qwen2.5-7b-instruct".into()),
            llm_enabled: true,
            shortcut: "ctrl+shift+;".into(),
            max_recording_seconds: 30,
            auto_paste: false,
        }
    }
}

impl AppConfig {
    pub fn load(app_data_dir: &PathBuf) -> Self {
        let config_path = app_data_dir.join("config.json");
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, app_data_dir: &PathBuf) -> Result<(), std::io::Error> {
        let config_path = app_data_dir.join("config.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, json)
    }
}
```

## 6. 初回起動フロー

```
┌─────────────────────────────────────────┐
│           Welcome to VoiceInput         │
├─────────────────────────────────────────┤
│                                         │
│  音声認識モデルをダウンロードしてください  │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │ ◉ Large v3 Turbo (推奨)        │   │
│  │   1.6GB - 最高精度              │   │
│  ├─────────────────────────────────┤   │
│  │ ○ Medium                        │   │
│  │   1.5GB - バランス型            │   │
│  ├─────────────────────────────────┤   │
│  │ ○ Small                         │   │
│  │   466MB - 軽量                  │   │
│  └─────────────────────────────────┘   │
│                                         │
│  □ LLMによる文章整形を有効にする         │
│    (追加で4.7GBダウンロード)            │
│                                         │
│           [ダウンロード開始]             │
│                                         │
└─────────────────────────────────────────┘
```

## 7. ディスク使用量管理

```rust
pub fn get_models_disk_usage(models_dir: &PathBuf) -> u64 {
    walkdir::WalkDir::new(models_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
```

## HuggingFace モデルURL一覧

| モデル | リポジトリ | ファイル |
|--------|-----------|---------|
| Whisper Large v3 Turbo | ggerganov/whisper.cpp | ggml-large-v3-turbo.bin |
| Whisper Medium | ggerganov/whisper.cpp | ggml-medium.bin |
| Whisper Small | ggerganov/whisper.cpp | ggml-small.bin |
| Qwen2.5 7B | Qwen/Qwen2.5-7B-Instruct-GGUF | qwen2.5-7b-instruct-q4_k_m.gguf |
| Qwen2.5 3B | Qwen/Qwen2.5-3B-Instruct-GGUF | qwen2.5-3b-instruct-q4_k_m.gguf |
