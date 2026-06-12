use std::path::Path;

use anyhow::{Context, Result};

use crate::adapter::scan;
use crate::config::{self, Config};
use crate::db::merge;
use crate::db::store::FreqDb;

pub fn run() -> Result<()> {
    let config_path = config::find_config()?;
    let cfg = Config::load(&config_path)?;

    let dir = Path::new(&cfg.freq_db_dir);
    let db_path = dir.join("entries.yaml");
    let mut db = FreqDb::load_yaml(&db_path.to_string_lossy())?;

    if cfg.scan_repos.is_empty() {
        log::warn!("no scan_repos configured in ci.yaml");
        return Ok(());
    }

    for repo in &cfg.scan_repos {
        let expanded = cfg.expand_tilde(repo);
        match scan::scan_repo(&expanded) {
            Ok(scan_result) => {
                if scan_result.entries.is_empty() {
                    continue;
                }
                let before = db.entries.len();
                db = merge::merge(&db, &scan_result);
                let added = db.entries.len() - before;
                log::info!("merged {added} new entries from {repo}");
            }
            Err(e) => {
                log::error!("failed to scan {repo}: {e:#}");
            }
        }
    }

    db.save_yaml(&db_path.to_string_lossy())
        .context("failed to save entries.yaml")?;

    log::info!("scan complete ({} total entries)", db.entries.len());
    Ok(())
}
