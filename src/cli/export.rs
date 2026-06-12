use std::path::Path;

use anyhow::Result;

use crate::adapter::rime;
use crate::config::{self, Config};
use crate::db::store::FreqDb;

pub fn run() -> Result<()> {
    let config_path = config::find_config()?;
    let cfg = Config::load(&config_path)?;

    let db_path = Path::new(&cfg.freq_db_dir).join("entries.yaml");
    let db = FreqDb::load_yaml(&db_path.to_string_lossy())?;

    rime::export(&db, &cfg)?;

    log::info!("export complete");
    Ok(())
}
