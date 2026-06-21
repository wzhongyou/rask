use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::{RaskError, Result};
use crate::paths::config_path;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub default_model: Option<String>,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path().ok_or_else(|| RaskError::Config("cannot find config dir".into()))?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)?;
        toml::from_str(&text).map_err(|e| RaskError::Config(e.to_string()))
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path().ok_or_else(|| RaskError::Config("cannot find config dir".into()))?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        let text = toml::to_string_pretty(self).map_err(|e| RaskError::Config(e.to_string()))?;
        std::fs::write(&path, text)?;
        Ok(())
    }

    pub fn default_model(&self) -> &str {
        self.default_model.as_deref().unwrap_or("gpt-4o-mini")
    }

    pub fn provider(&self, name: &str) -> Option<ProviderConfig> {
        self.providers.get(name).cloned()
    }

    /// `rask config set default_model gpt-4o`
    /// `rask config set providers.openai.api_key sk-...`
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let parts: Vec<&str> = key.splitn(3, '.').collect();
        match parts.as_slice() {
            ["default_model"] => self.default_model = Some(value.into()),
            ["providers", name, "api_key"] => {
                self.providers.entry(name.to_string()).or_insert_with(|| ProviderConfig {
                    api_key: String::new(),
                    base_url: None,
                }).api_key = value.into();
            }
            ["providers", name, "base_url"] => {
                self.providers.entry(name.to_string()).or_insert_with(|| ProviderConfig {
                    api_key: String::new(),
                    base_url: None,
                }).base_url = Some(value.into());
            }
            _ => return Err(RaskError::Config(format!("unknown key: {key}"))),
        }
        Ok(())
    }
}

