use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntrySource {
    Ime,
    Scan { repo: String, path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub code: String,
    pub word: String,
    pub freq: u32,
    pub updated: DateTime<Utc>,
    pub prev: Option<String>,
    pub source: EntrySource,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_entry_creation() {
        let e = Entry {
            code: "ni hao".into(),
            word: "你好".into(),
            freq: 5,
            updated: Utc.timestamp_opt(1700000000, 0).unwrap(),
            prev: None,
            source: EntrySource::Ime,
        };
        assert_eq!(e.code, "ni hao");
        assert_eq!(e.word, "你好");
        assert_eq!(e.freq, 5);
        assert!(e.prev.is_none());
    }

    #[test]
    fn test_entry_with_prev() {
        let e = Entry {
            code: "shi jie".into(),
            word: "世界".into(),
            freq: 3,
            updated: Utc.timestamp_opt(1700000001, 0).unwrap(),
            prev: Some("hello".into()),
            source: EntrySource::Scan {
                repo: "test-repo".into(),
                path: "/path/to/file.md".into(),
            },
        };
        assert_eq!(e.prev.unwrap(), "hello");
        match e.source {
            EntrySource::Scan { repo, path } => {
                assert_eq!(repo, "test-repo");
                assert_eq!(path, "/path/to/file.md");
            }
            _ => panic!("expected Scan source"),
        }
    }

    #[test]
    fn test_entry_serialize_roundtrip() {
        let e = Entry {
            code: "ce shi".into(),
            word: "测试".into(),
            freq: 2,
            updated: Utc.timestamp_opt(1700000003, 0).unwrap(),
            prev: Some("hello".into()),
            source: EntrySource::Ime,
        };
        let yaml = serde_yaml::to_string(&e).unwrap();
        let deserialized: Entry = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(e.code, deserialized.code);
        assert_eq!(e.freq, deserialized.freq);
        assert_eq!(e.prev, deserialized.prev);
    }
}
