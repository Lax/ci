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
