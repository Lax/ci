use std::path::Path;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::db::device::Device;
use crate::db::store::FreqDb;

pub fn run(dir: &str) -> Result<()> {
    let path = Path::new(dir);
    std::fs::create_dir_all(path.join("split"))
        .with_context(|| format!("failed to create {}", path.join("split").display()))?;

    std::fs::write(path.join(".gitignore"), "/target\nentries.json\n")
        .context("failed to write .gitignore")?;

    let device_id = uuid::Uuid::new_v4().to_string();
    let device_name = whoami::hostname().unwrap_or_else(|_| "unknown".into());

    let config = Config {
        freq_db_dir: dir.to_string(),
        device_id: device_id.clone(),
        device_name: device_name.clone(),
        ..Default::default()
    };
    config
        .save(&path.join("ci.yaml").to_string_lossy())
        .context("failed to save ci.yaml")?;

    git2::Repository::init(path).context("failed to init git repo")?;

    let device = Device {
        id: device_id,
        name: device_name,
        total_entries: 0,
    };
    let db = FreqDb::new(device);
    db.save_yaml(&path.join("entries.yaml").to_string_lossy())
        .context("failed to save initial entries.yaml")?;

    log::info!("initialized freq-db in {}", dir);
    Ok(())
}
