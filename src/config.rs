use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub rime_user_dir: String,
    pub freq_db_dir: String,
    pub device_id: String,
    pub device_name: String,
    pub sync_remote: String,
    pub cooldown_hours: u64,
    #[serde(default)]
    pub scan_repos: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rime_user_dir: "~/.local/share/fcitx5/rime".to_string(),
            freq_db_dir: ".".to_string(),
            device_id: String::new(),
            device_name: whoami::hostname().unwrap_or_else(|_| "unknown".into()),
            sync_remote: String::new(),
            cooldown_hours: 12,
            scan_repos: Vec::new(),
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config: {path}"))?;
        Ok(serde_yaml::from_str(&content)?)
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn rime_user_dir(&self) -> String {
        self.expand_tilde(&self.rime_user_dir)
    }

    pub fn expand_tilde(&self, s: &str) -> String {
        if s.starts_with("~/") {
            if let Some(home) = std::env::var("HOME").ok().or_else(|| {
                dirs_fallback()
            }) {
                return s.replacen("~", &home, 1);
            }
        }
        s.to_string()
    }
}

fn dirs_fallback() -> Option<String> {
    std::env::var("HOME").ok()
}

/// Find ci.yaml by searching current dir and parents
pub fn find_config() -> Result<String> {
    let mut dir = std::env::current_dir()?;
    loop {
        let candidate = dir.join("ci.yaml");
        if candidate.exists() {
            return Ok(candidate.to_string_lossy().to_string());
        }
        if !dir.pop() {
            anyhow::bail!("ci.yaml not found in current or parent directories");
        }
    }
}
