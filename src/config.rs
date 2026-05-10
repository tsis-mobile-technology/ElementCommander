use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ui: UiConfig,
    pub behavior: BehaviorConfig,
    pub bookmarks: Vec<Bookmark>,
    #[serde(default)]
    pub history: HistoryConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HistoryConfig {
    pub last_left_path: Option<PathBuf>,
    pub last_right_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileNote {
    pub memo: String,
    pub tags: Vec<String>,
    pub updated_at: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NotesStore {
    pub notes: std::collections::HashMap<String, FileNote>,
}

impl NotesStore {
    pub fn load() -> Self {
        let config_dir = dirs::config_dir()
            .map(|p| p.join("hermes_tail"))
            .unwrap_or_else(|| PathBuf::from(".hermes_tail"));
        let path = config_dir.join("notes.json");

        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .map(|p| p.join("hermes_tail"))
            .unwrap_or_else(|| PathBuf::from(".hermes_tail"));
        fs::create_dir_all(&config_dir)?;
        let path = config_dir.join("notes.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_note(&self, path: &str) -> Option<&FileNote> {
        self.notes.get(path)
    }

    pub fn set_note(&mut self, path: String, memo: String, tags: Vec<String>) {
        self.notes.insert(path, FileNote {
            memo,
            tags,
            updated_at: chrono::Local::now(),
        });
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub show_hidden: bool,
    pub use_icons: bool,
    pub color_scheme: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BehaviorConfig {
    pub confirm_delete: bool,
    pub default_sort: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bookmark {
    pub name: String,
    pub path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ui: UiConfig {
                show_hidden: false,
                use_icons: true,
                color_scheme: "default".to_string(),
            },
            behavior: BehaviorConfig {
                confirm_delete: true,
                default_sort: "name".to_string(),
            },
            bookmarks: Vec::new(),
            history: HistoryConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("hermes_tail");
        
        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(config_path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("hermes_tail");

        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}
