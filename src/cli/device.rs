use anyhow::Result;

use crate::config::{self, Config};
use crate::db::device::DeviceRegistry;

pub fn run_list() -> Result<()> {
    let config_path = config::find_config()?;
    let cfg = Config::load(&config_path)?;

    let registry_path = std::path::Path::new(&cfg.freq_db_dir).join("devices.yaml");
    let registry = DeviceRegistry::load(&registry_path.to_string_lossy())?;

    if registry.devices.is_empty() {
        println!("no devices registered");
        return Ok(());
    }

    println!("{:<20} {:<20} {:>10}  {}", "ID", "NAME", "ENTRIES", "LAST SEEN");
    for d in &registry.devices {
        let short_id = if d.id.len() > 8 { &d.id[..8] } else { &d.id };
        println!(
            "{:<20} {:<20} {:>10}  {}",
            short_id,
            d.name,
            d.total_entries,
            d.last_seen.format("%Y-%m-%d %H:%M"),
        );
    }
    Ok(())
}

pub fn run_add(name: &str) -> Result<()> {
    let config_path = config::find_config()?;
    let cfg = Config::load(&config_path)?;

    let registry_path = std::path::Path::new(&cfg.freq_db_dir).join("devices.yaml");
    let mut registry = DeviceRegistry::load(&registry_path.to_string_lossy())?;

    let id = uuid::Uuid::new_v4().to_string();
    registry.upsert(&id, name, 0);
    registry.save(&registry_path.to_string_lossy())?;

    println!("added device: {name} ({})", &id[..8]);
    Ok(())
}
