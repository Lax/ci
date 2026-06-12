use std::path::Path;

use anyhow::{Context, Result};

use crate::config::{self, Config};
use crate::db::device::DeviceRegistry;
use crate::db::merge;
use crate::db::store::FreqDb;
use crate::sync::git;

pub fn run() -> Result<()> {
    let config_path = config::find_config()?;
    let cfg = Config::load(&config_path)?;
    let dir = Path::new(&cfg.freq_db_dir);

    let db_path = dir.join("entries.yaml");

    // Load local db
    let mut local = FreqDb::load_yaml(&db_path.to_string_lossy())?;

    // Fetch remote and merge if available
    if let Some(remote_yaml) = git::fetch_remote(&cfg.freq_db_dir)? {
        let remote: FreqDb = serde_yaml::from_str(&remote_yaml)
            .context("failed to parse remote entries.yaml")?;
        log::info!(
            "merging remote device '{}' ({} entries)",
            remote.device.name,
            remote.entries.len()
        );
        local = merge::merge(&local, &remote);
        local.save_yaml(&db_path.to_string_lossy())?;
    }

    // Update device registry
    let registry_path = dir.join("devices.yaml");
    let mut registry = DeviceRegistry::load(&registry_path.to_string_lossy())?;
    registry.upsert(&local.device.id, &local.device.name, local.device.total_entries);
    registry.save(&registry_path.to_string_lossy())?;

    // Commit and push
    let n = local.entries.len();
    git::commit_and_push(
        &cfg.freq_db_dir,
        &format!("sync: {n} entries [ci]"),
    )?;

    log::info!("sync complete ({n} entries)");
    Ok(())
}
