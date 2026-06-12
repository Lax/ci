use std::path::Path;

use anyhow::{Context, Result};

use crate::adapter::rime;
use crate::config::{self, Config};
use crate::db::store::FreqDb;

pub fn run() -> Result<()> {
    let config_path = config::find_config()?;
    let cfg = Config::load(&config_path)?;

    let db_path = Path::new(&cfg.freq_db_dir).join("entries.yaml");
    let mut db = if db_path.exists() {
        FreqDb::load_yaml(&db_path.to_string_lossy())?
    } else {
        anyhow::bail!("entries.yaml not found in {}", cfg.freq_db_dir);
    };

    rime::import(&mut db, &cfg)?;

    db.save_yaml(&db_path.to_string_lossy())
        .context("failed to save entries.yaml")?;

    log::info!("import complete");
    Ok(())
}
