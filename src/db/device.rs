use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub total_entries: u32,
}

impl Device {
    pub fn weight(&self, global_total: u32) -> f64 {
        if global_total == 0 {
            return 1.0;
        }
        self.total_entries as f64 / global_total as f64
    }
}
