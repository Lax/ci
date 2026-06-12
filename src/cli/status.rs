use anyhow::Result;

use crate::config::{self, Config};
use crate::db::device::DeviceRegistry;
use crate::db::entry::EntrySource;
use crate::db::store::FreqDb;

pub fn run() -> Result<()> {
    let config_path = config::find_config()?;
    let cfg = Config::load(&config_path)?;

    let dir = std::path::Path::new(&cfg.freq_db_dir);
    let db_path = dir.join("entries.yaml");
    let db = FreqDb::load_yaml(&db_path.to_string_lossy())?;

    let mut ime_count = 0u32;
    let mut scan_count = 0u32;
    for e in &db.entries {
        match e.source {
            EntrySource::Ime => ime_count += 1,
            EntrySource::Scan { .. } => scan_count += 1,
        }
    }

    println!("device:         {} ({})", db.device.name, &db.device.id[..8]);
    println!("version:        {}", db.version);
    println!("total entries:  {}", db.entries.len());
    println!("  from IME:     {ime_count}");
    println!("  from scan:    {scan_count}");

    let registry_path = dir.join("devices.yaml");
    let registry = DeviceRegistry::load(&registry_path.to_string_lossy())?;
    println!("known devices:  {}", registry.devices.len());

    for d in &registry.devices {
        let short = if d.id.len() > 8 { &d.id[..8] } else { &d.id };
        println!(
            "  {short}  {:<20}  {:>6} entries  {}",
            d.name,
            d.total_entries,
            d.last_seen.format("%Y-%m-%d"),
        );
    }

    Ok(())
}
