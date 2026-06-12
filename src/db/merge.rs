use super::entry::Entry;
use super::store::FreqDb;

pub fn merge(local: &FreqDb, remote: &FreqDb) -> FreqDb {
    let total = local.device.total_entries + remote.device.total_entries;
    let local_w = local.device.weight(total);
    let remote_w = remote.device.weight(total);

    let mut merged_entries: Vec<Entry> = local.entries.clone();

    for remote_entry in &remote.entries {
        let idx = merged_entries.iter().position(|e| {
            e.code == remote_entry.code && e.word == remote_entry.word
        });

        match idx {
            Some(i) => {
                let local_entry = &merged_entries[i];
                let merged_freq = ((local_entry.freq as f64 * local_w
                    + remote_entry.freq as f64 * remote_w)
                    / (local_w + remote_w))
                    .round() as u32;
                let merged_updated = local_entry.updated.max(remote_entry.updated);
                let merged_prev = if remote_entry.prev.is_some() && local_entry.prev.is_none() {
                    remote_entry.prev.clone()
                } else {
                    local_entry.prev.clone()
                };
                merged_entries[i].freq = merged_freq;
                merged_entries[i].updated = merged_updated;
                merged_entries[i].prev = merged_prev;
            }
            None => {
                merged_entries.push(remote_entry.clone());
            }
        }
    }

    let merged_device = Device {
        id: local.device.id.clone(),
        name: local.device.name.clone(),
        total_entries: merged_entries.len() as u32,
    };

    FreqDb {
        version: local.version.max(remote.version),
        device: merged_device,
        entries: merged_entries,
    }
}

use super::device::Device;
