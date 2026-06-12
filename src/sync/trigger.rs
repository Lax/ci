use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::adapter::rime;
use crate::config::Config;
use crate::db::device::DeviceRegistry;
use crate::db::merge;
use crate::db::store::FreqDb;
use crate::sync::git;

#[derive(Debug, Serialize, Deserialize)]
pub struct TriggerState {
    pub last_sync: Option<DateTime<Utc>>,
    pub cooldown_hours: u64,
}

impl TriggerState {
    pub fn new(cooldown_hours: u64) -> Self {
        Self {
            last_sync: None,
            cooldown_hours,
        }
    }

    pub fn load(path: &str) -> Result<Self> {
        let p = Path::new(path);
        if !p.exists() {
            return Ok(Self::new(12));
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

    pub fn should_sync(&self) -> bool {
        match self.last_sync {
            Some(last) => {
                let elapsed = Utc::now() - last;
                elapsed >= Duration::hours(self.cooldown_hours as i64)
            }
            None => true,
        }
    }

    pub fn next_sync(&self) -> Option<DateTime<Utc>> {
        self.last_sync
            .map(|last| last + Duration::hours(self.cooldown_hours as i64))
    }

    pub fn mark_synced(&mut self) {
        self.last_sync = Some(Utc::now());
    }

    /// Run one full sync cycle: import → sync → export.
    /// Returns true if work was done.
    pub fn run_cycle(cfg: &Config) -> Result<bool> {
        let dir = Path::new(&cfg.freq_db_dir);
        let db_path = dir.join("entries.yaml");

        if !db_path.exists() {
            anyhow::bail!("entries.yaml not found in {}", cfg.freq_db_dir);
        }

        // Import from Rime
        let mut db = FreqDb::load_yaml(&db_path.to_string_lossy())?;
        if let Err(e) = rime::import(&mut db, cfg) {
            log::warn!("import failed (skipping): {e:#}");
        }

        // Sync with remote
        if let Some(remote_yaml) = git::fetch_remote(&cfg.freq_db_dir)? {
            match serde_yaml::from_str::<FreqDb>(&remote_yaml) {
                Ok(remote) => {
                    db = merge::merge(&db, &remote);
                }
                Err(e) => {
                    log::warn!("failed to parse remote entries.yaml: {e}");
                }
            }
        }

        db.save_yaml(&db_path.to_string_lossy())?;

        // Update device registry
        let registry_path = dir.join("devices.yaml");
        let mut registry = DeviceRegistry::load(&registry_path.to_string_lossy())?;
        registry.upsert(&db.device.id, &db.device.name, db.device.total_entries);
        registry.save(&registry_path.to_string_lossy())?;

        // Git commit + push
        let n = db.entries.len();
        if let Err(e) = git::commit_and_push(&cfg.freq_db_dir, &format!("sync: {n} entries [ci]")) {
            log::warn!("git commit/push failed (skipping): {e:#}");
        }

        // Export to Rime
        if let Err(e) = rime::export(&db, cfg) {
            log::warn!("export failed (skipping): {e:#}");
        }

        log::info!("cycle complete ({n} entries)");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_trigger_should_sync_when_never_synced() {
        let state = TriggerState::new(12);
        assert!(state.should_sync());
    }

    #[test]
    fn test_trigger_should_not_sync_within_cooldown() {
        let mut state = TriggerState::new(12);
        state.mark_synced();
        assert!(!state.should_sync());
    }

    #[test]
    fn test_trigger_should_sync_after_cooldown() {
        let last = Utc::now() - Duration::hours(13);
        let state = TriggerState {
            last_sync: Some(last),
            cooldown_hours: 12,
        };
        assert!(state.should_sync());
    }

    #[test]
    fn test_trigger_mark_synced_updates_time() {
        let mut state = TriggerState::new(12);
        state.mark_synced();
        assert!(state.last_sync.is_some());
        let elapsed = Utc::now() - state.last_sync.unwrap();
        assert!(elapsed < Duration::seconds(2));
    }

    #[test]
    fn test_trigger_next_sync() {
        let mut state = TriggerState::new(12);
        state.mark_synced();
        let next = state.next_sync().unwrap();
        let diff = next - Utc::now();
        assert!(diff > Duration::hours(11) && diff < Duration::hours(13));
    }

    #[test]
    fn test_trigger_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("trigger.yaml").to_string_lossy().to_string();

        let mut state = TriggerState::new(6);
        state.mark_synced();
        state.save(&path).unwrap();

        let loaded = TriggerState::load(&path).unwrap();
        assert_eq!(loaded.cooldown_hours, 6);
        assert!(loaded.last_sync.is_some());
    }
}
