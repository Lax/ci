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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::device::Device;
    use crate::db::entry::EntrySource;
    use chrono::Utc;

    fn test_device() -> Device {
        Device {
            id: "test-id".into(),
            name: "test-dev".into(),
            total_entries: 0,
        }
    }

    #[test]
    fn test_freqdb_new() {
        let d = test_device();
        let db = FreqDb::new(d.clone());
        assert_eq!(db.version, 1);
        assert_eq!(db.device.name, "test-dev");
        assert!(db.entries.is_empty());
    }

    #[test]
    fn test_freqdb_yaml_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("entries.yaml").to_string_lossy().to_string();

        let mut db = FreqDb::new(test_device());
        db.entries.push(Entry {
            code: "ni hao".into(),
            word: "你好".into(),
            freq: 5,
            updated: Utc::now(),
            prev: None,
            source: EntrySource::Ime,
        });

        db.save_yaml(&path).unwrap();
        let loaded = FreqDb::load_yaml(&path).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].word, "你好");
    }
}
