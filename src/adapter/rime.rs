use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;

use crate::config::Config;
use crate::db::entry::{Entry, EntrySource};
use crate::db::merge;
use crate::db::store::FreqDb;

/// Trigger a Rime sync, using FFI (librime) if available, otherwise shelling out.
fn trigger_sync(cfg: &Config) -> Result<()> {
    // Try FFI first (direct librime call, no process spawn)
    if let Ok(()) = crate::ffi::sync_user_data(cfg) {
        return Ok(());
    }
    log::info!("FFI sync unavailable, falling back to rime_dict_manager -s");
    let output = std::process::Command::new("rime_dict_manager")
        .arg("-s")
        .output()
        .context("failed to run rime_dict_manager -s (install librime or rime_dict_manager)")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("rime_dict_manager -s failed: {stderr}");
    }
    Ok(())
}

/// Run trigger_sync, then read synced .userdb.txt files into `local`.
pub fn import(local: &mut FreqDb, cfg: &Config) -> Result<()> {
    let rime_user_dir = cfg.rime_user_dir();
    trigger_sync(cfg)?;

    let sync_dir = Path::new(&rime_user_dir).join("sync");
    if !sync_dir.exists() {
        log::warn!("sync directory not found: {}", sync_dir.display());
        return Ok(());
    }

    let mut remote = FreqDb::new(local.device.clone());

    for entry in std::fs::read_dir(&sync_dir).context("failed to read sync dir")? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let inst_dir = entry.path();
        for file in std::fs::read_dir(&inst_dir).context("failed to read inst dir")? {
            let file = file?;
            let fpath = file.path();
            if fpath.extension().map_or(true, |e| e != "txt") {
                continue;
            }
            log::info!("parsing {}", fpath.display());
            let entries = parse_userdb_txt(&fpath)?;
            remote.entries.extend(entries);
        }
    }

    remote.device.total_entries = remote.entries.len() as u32;

    if remote.entries.is_empty() {
        log::warn!("no entries found in Rime sync data");
        return Ok(());
    }

    let merged = merge::merge(local, &remote);
    *local = merged;
    log::info!("imported {} entries from Rime", remote.entries.len());
    Ok(())
}

/// Generate a .userdb.txt from `db`, write it to the sync dir for another
/// device, then trigger sync to merge it into Rime.
pub fn export(db: &FreqDb, cfg: &Config) -> Result<()> {
    let rime_user_dir = cfg.rime_user_dir();

    // Write export file into a temp pseudo-device sync dir
    let export_id = format!("ci_export_{}", db.device.id.replace('-', ""));
    let export_dir = Path::new(&rime_user_dir)
        .join("sync")
        .join(&export_id);
    std::fs::create_dir_all(&export_dir)
        .context("failed to create export sync dir")?;

    let export_path = export_dir.join("ci_export.userdb.txt");
    let mut content = String::new();
    content.push_str("#@/db_name=ci_export\n");
    content.push_str(&format!("#@/user_id={}\n", db.device.id.replace('-', "")));

    for entry in &db.entries {
        let ts = entry.updated.timestamp();
        content.push_str(&format!(
            "{}\t{}\tc={} d=0 t={ts}\n",
            entry.code, entry.word, entry.freq
        ));
    }

    std::fs::write(&export_path, &content)
        .with_context(|| format!("failed to write {}", export_path.display()))?;

    // Trigger sync — librime will find the file in our pseudo-device dir
    // and merge it into the userdb
    trigger_sync(cfg)?;

    // Cleanup
    std::fs::remove_file(&export_path).ok();
    std::fs::remove_dir(&export_dir).ok();

    log::info!("exported {} entries to Rime", db.entries.len());
    Ok(())
}

fn parse_userdb_txt(path: &Path) -> Result<Vec<Entry>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let re = Regex::new(
        r"^([^\t]+)\t([^\t]+)\tc=(\d+)\s+d=(\d+)\s+t=(\d+)",
    )
    .expect("invalid regex");

    let mut entries = Vec::new();
    for line in content.lines() {
        if line.starts_with('#') {
            continue;
        }
        if let Some(caps) = re.captures(line) {
            let code = caps[1].to_string();
            let word = caps[2].to_string();
            let freq: u32 = caps[3].parse().unwrap_or(1);
            let ts: i64 = caps[5].parse().unwrap_or(0);
            let updated = DateTime::from_timestamp(ts, 0).unwrap_or(Utc::now());

            entries.push(Entry {
                code,
                word,
                freq,
                updated,
                prev: None,
                source: EntrySource::Ime,
            });
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_userdb_txt() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.userdb.txt");
        let content = "\
#@/db_name=luna_pinyin
#@/user_id=test-device
ni hao\t你好\tc=5 d=0 t=1700000000
shi jie\t世界\tc=3 d=0 t=1700000001
";
        std::fs::write(&path, content).unwrap();
        let entries = parse_userdb_txt(&path).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].code, "ni hao");
        assert_eq!(entries[0].word, "你好");
        assert_eq!(entries[0].freq, 5);
        assert_eq!(entries[1].freq, 3);
    }

    #[test]
    fn test_parse_userdb_txt_skips_comments() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.userdb.txt");
        let content = "\
# comment line
#@/metadata
ni hao\t你好\tc=5 d=0 t=1700000000
";
        std::fs::write(&path, content).unwrap();
        let entries = parse_userdb_txt(&path).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_parse_userdb_txt_invalid_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.userdb.txt");
        let content = "this is not valid tsv format\n";
        std::fs::write(&path, content).unwrap();
        let entries = parse_userdb_txt(&path).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_userdb_txt_missing_file() {
        let result = parse_userdb_txt(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }
}
