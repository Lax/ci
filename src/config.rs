use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub rime_user_dir: String,
    pub freq_db_dir: String,
    pub device_id: String,
    pub device_name: String,
    pub sync_remote: String,
    pub cooldown_hours: u64,
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
        }
    }
}
