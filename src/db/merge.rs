use super::device::Device;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::device::Device;
    use chrono::{TimeZone, Utc};

    fn make_device(id: &str, name: &str, n: u32) -> Device {
        Device {
            id: id.into(),
            name: name.into(),
            total_entries: n,
        }
    }

    fn make_entry(code: &str, word: &str, freq: u32, ts: i64) -> Entry {
        Entry {
            code: code.into(),
            word: word.into(),
            freq,
            updated: Utc.timestamp_opt(ts, 0).unwrap(),
            prev: None,
            source: crate::db::entry::EntrySource::Ime,
        }
    }

    #[test]
    fn test_merge_new_entries_from_remote() {
        let local = FreqDb {
            version: 1,
            device: make_device("local", "A", 1),
            entries: vec![make_entry("a", "阿", 5, 100)],
        };
        let remote = FreqDb {
            version: 1,
            device: make_device("remote", "B", 1),
            entries: vec![make_entry("b", "吧", 3, 200)],
        };
        let merged = merge(&local, &remote);
        assert_eq!(merged.entries.len(), 2);
    }

    #[test]
    fn test_merge_existing_entry_freq_averaged() {
        let local = FreqDb {
            version: 1,
            device: make_device("local", "A", 1),
            entries: vec![make_entry("a", "阿", 10, 100)],
        };
        let remote = FreqDb {
            version: 1,
            device: make_device("remote", "B", 1),
            entries: vec![make_entry("a", "阿", 20, 200)],
        };
        let merged = merge(&local, &remote);
        assert_eq!(merged.entries.len(), 1);
        // equal weights (1:1) => avg = 15
        assert_eq!(merged.entries[0].freq, 15);
    }

    #[test]
    fn test_merge_preserves_higher_version() {
        let local = FreqDb {
            version: 1,
            device: make_device("local", "A", 0),
            entries: vec![],
        };
        let remote = FreqDb {
            version: 3,
            device: make_device("remote", "B", 0),
            entries: vec![],
        };
        let merged = merge(&local, &remote);
        assert_eq!(merged.version, 3);
    }

    #[test]
    fn test_merge_updates_prev_context() {
        let local = FreqDb {
            version: 1,
            device: make_device("local", "A", 1),
            entries: vec![Entry {
                code: "a".into(),
                word: "阿".into(),
                freq: 5,
                updated: Utc.timestamp_opt(100, 0).unwrap(),
                prev: Some("old".into()),
                source: crate::db::entry::EntrySource::Ime,
            }],
        };
        let remote = FreqDb {
            version: 1,
            device: make_device("remote", "B", 1),
            entries: vec![Entry {
                code: "a".into(),
                word: "阿".into(),
                freq: 5,
                updated: Utc.timestamp_opt(200, 0).unwrap(),
                prev: None,
                source: crate::db::entry::EntrySource::Ime,
            }],
        };
        let merged = merge(&local, &remote);
        // local.prev is Some, remote.prev is None: merge keeps local.prev
        assert_eq!(merged.entries[0].prev, Some("old".into()));
    }
}
