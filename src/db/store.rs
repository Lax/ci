use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::device::Device;
use super::entry::Entry;

#[derive(Debug, Serialize, Deserialize)]
pub struct FreqDb {
    pub version: u32,
    pub device: Device,
    pub entries: Vec<Entry>,
}

impl FreqDb {
    pub fn new(device: Device) -> Self {
        Self {
            version: 1,
            device,
            entries: Vec::new(),
        }
    }

    pub fn load_yaml(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&content)?)
    }

    pub fn save_yaml(&self, path: &str) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load_json(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save_json(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
