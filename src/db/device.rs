use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub total_entries: u32,
}

impl Device {
    pub fn weight(&self, global_total: u32) -> f64 {
        if global_total == 0 {
            return 1.0;
        }
        self.total_entries as f64 / global_total as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub total_entries: u32,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceRegistry {
    pub devices: Vec<DeviceInfo>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    pub fn load(path: &str) -> Result<Self> {
        let p = Path::new(path);
        if !p.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(p)
            .with_context(|| format!("failed to read {path}"))?;
        Ok(serde_yaml::from_str(&content)?)
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn upsert(&mut self, id: &str, name: &str, total_entries: u32) {
        let now = Utc::now();
        if let Some(d) = self.devices.iter_mut().find(|d| d.id == id) {
            d.name = name.to_string();
            d.total_entries = total_entries;
            d.last_seen = now;
        } else {
            self.devices.push(DeviceInfo {
                id: id.to_string(),
                name: name.to_string(),
                total_entries,
                last_seen: now,
            });
        }
    }

    pub fn get(&self, id: &str) -> Option<&DeviceInfo> {
        self.devices.iter().find(|d| d.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_device_weight_equal() {
        let d = Device {
            id: "a".into(),
            name: "dev-a".into(),
            total_entries: 50,
        };
        assert!((d.weight(100) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_device_weight_zero_global() {
        let d = Device {
            id: "a".into(),
            name: "dev-a".into(),
            total_entries: 50,
        };
        assert!((d.weight(0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_registry_upsert_new() {
        let mut reg = DeviceRegistry::new();
        reg.upsert("id-1", "dev-one", 10);
        assert_eq!(reg.devices.len(), 1);
        assert_eq!(reg.devices[0].name, "dev-one");
        assert_eq!(reg.devices[0].total_entries, 10);
    }

    #[test]
    fn test_registry_upsert_existing() {
        let mut reg = DeviceRegistry::new();
        reg.upsert("id-1", "dev-one", 10);
        reg.upsert("id-1", "dev-one-updated", 20);
        assert_eq!(reg.devices.len(), 1);
        assert_eq!(reg.devices[0].name, "dev-one-updated");
        assert_eq!(reg.devices[0].total_entries, 20);
    }

    #[test]
    fn test_registry_get() {
        let mut reg = DeviceRegistry::new();
        reg.upsert("id-1", "dev-one", 10);
        let d = reg.get("id-1").unwrap();
        assert_eq!(d.name, "dev-one");
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("devices.yaml").to_string_lossy().to_string();

        let mut reg = DeviceRegistry::new();
        reg.upsert("id-1", "dev-one", 10);
        reg.save(&path).unwrap();

        let loaded = DeviceRegistry::load(&path).unwrap();
        assert_eq!(loaded.devices.len(), 1);
        assert_eq!(loaded.devices[0].name, "dev-one");
    }

    #[test]
    fn test_registry_load_nonexistent() {
        let reg = DeviceRegistry::load("/nonexistent/path.yaml").unwrap();
        assert!(reg.devices.is_empty());
    }
}
