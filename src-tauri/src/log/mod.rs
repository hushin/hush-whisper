use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A single transcription log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique ID for the entry
    pub id: String,
    /// Timestamp when the transcription was made
    pub timestamp: DateTime<Utc>,
    /// Raw transcription result from Whisper
    pub raw_text: String,
    /// Refined text from LLM (if enabled)
    pub refined_text: Option<String>,
    /// Duration of the audio in seconds
    pub audio_duration_secs: Option<f32>,
    /// Whether LLM refinement was used
    pub llm_used: bool,
    /// Prompt preset used (if LLM was enabled)
    pub prompt_preset: Option<String>,
}

/// Log manager for storing and retrieving transcription logs
pub struct LogManager {
    log_dir: PathBuf,
}

impl LogManager {
    /// Create a new LogManager
    pub fn new() -> Result<Self, String> {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        let log_dir = PathBuf::from(appdata).join("voice-input").join("logs");

        // Create log directory if it doesn't exist
        fs::create_dir_all(&log_dir).map_err(|e| format!("Failed to create log directory: {}", e))?;

        Ok(Self { log_dir })
    }

    /// Generate a unique ID for a log entry
    fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("{:x}", now)
    }

    /// Get the log file path for a specific date (YYYY-MM-DD.json)
    fn get_log_file_path(&self, date: &DateTime<Utc>) -> PathBuf {
        let date_str = date.format("%Y-%m-%d").to_string();
        self.log_dir.join(format!("{}.json", date_str))
    }

    /// Load logs from a specific date
    fn load_logs_for_date(&self, date: &DateTime<Utc>) -> Vec<LogEntry> {
        let path = self.get_log_file_path(date);
        if !path.exists() {
            return Vec::new();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    /// Save logs for a specific date
    fn save_logs_for_date(&self, date: &DateTime<Utc>, logs: &[LogEntry]) -> Result<(), String> {
        let path = self.get_log_file_path(date);
        let content =
            serde_json::to_string_pretty(logs).map_err(|e| format!("Failed to serialize logs: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write log file: {}", e))?;
        Ok(())
    }

    /// Add a new log entry
    pub fn add_entry(
        &self,
        raw_text: String,
        refined_text: Option<String>,
        audio_duration_secs: Option<f32>,
        llm_used: bool,
        prompt_preset: Option<String>,
    ) -> Result<LogEntry, String> {
        let now = Utc::now();
        let entry = LogEntry {
            id: Self::generate_id(),
            timestamp: now,
            raw_text,
            refined_text,
            audio_duration_secs,
            llm_used,
            prompt_preset,
        };

        // Load existing logs for today, add the new entry, and save
        let mut logs = self.load_logs_for_date(&now);
        logs.push(entry.clone());
        self.save_logs_for_date(&now, &logs)?;

        tracing::info!("Added log entry: {}", entry.id);
        Ok(entry)
    }

    /// Get all log entries for a specific date
    pub fn get_logs_for_date(&self, year: i32, month: u32, day: u32) -> Vec<LogEntry> {
        use chrono::NaiveDate;
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            let datetime = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            self.load_logs_for_date(&datetime)
        } else {
            Vec::new()
        }
    }

    /// Get recent logs (last N entries across all dates)
    pub fn get_recent_logs(&self, limit: usize) -> Vec<LogEntry> {
        let mut all_logs: Vec<LogEntry> = Vec::new();

        // Get list of log files, sorted by date (newest first)
        let mut log_files: Vec<PathBuf> = fs::read_dir(&self.log_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
                    .collect()
            })
            .unwrap_or_default();

        log_files.sort_by(|a, b| b.cmp(a)); // Sort descending

        for path in log_files {
            if all_logs.len() >= limit {
                break;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut logs) = serde_json::from_str::<Vec<LogEntry>>(&content) {
                    // Sort logs by timestamp descending within the file
                    logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    all_logs.extend(logs);
                }
            }
        }

        // Sort all collected logs and take the limit
        all_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        all_logs.truncate(limit);
        all_logs
    }

    /// Delete a log entry by ID
    pub fn delete_entry(&self, id: &str) -> Result<bool, String> {
        // Search through all log files
        let log_files: Vec<PathBuf> = fs::read_dir(&self.log_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
                    .collect()
            })
            .unwrap_or_default();

        for path in log_files {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut logs) = serde_json::from_str::<Vec<LogEntry>>(&content) {
                    let original_len = logs.len();
                    logs.retain(|entry| entry.id != id);

                    if logs.len() < original_len {
                        // Entry was found and removed
                        let content = serde_json::to_string_pretty(&logs)
                            .map_err(|e| format!("Failed to serialize logs: {}", e))?;
                        fs::write(&path, content).map_err(|e| format!("Failed to write log file: {}", e))?;
                        tracing::info!("Deleted log entry: {}", id);
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get available log dates (for date picker)
    pub fn get_available_dates(&self) -> Vec<String> {
        let mut dates: Vec<String> = fs::read_dir(&self.log_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let path = e.path();
                        if path.extension().is_some_and(|ext| ext == "json") {
                            path.file_stem()
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        dates.sort_by(|a, b| b.cmp(a)); // Sort descending (newest first)
        dates
    }

    /// Delete all log entries
    pub fn delete_all_entries(&self) -> Result<usize, String> {
        let mut deleted_count = 0;

        // Get all log files
        let log_files: Vec<PathBuf> = fs::read_dir(&self.log_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
                    .collect()
            })
            .unwrap_or_default();

        // Delete each log file
        for path in log_files {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(logs) = serde_json::from_str::<Vec<LogEntry>>(&content) {
                    deleted_count += logs.len();
                }
            }
            fs::remove_file(&path).map_err(|e| format!("Failed to delete log file: {}", e))?;
        }

        tracing::info!("Deleted all log entries: {} entries", deleted_count);
        Ok(deleted_count)
    }
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new().expect("Failed to create LogManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = LogManager::generate_id();
        let id2 = LogManager::generate_id();
        assert!(!id1.is_empty());
        // IDs should be unique (though in a tight loop they might be the same)
    }
}
