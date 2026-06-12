use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TriggerState {
    pub last_sync: Option<DateTime<Utc>>,
    pub cooldown_hours: u64,
}

impl TriggerState {
    pub fn new(cooldown_hours: u64) -> Self {
        Self {
            last_sync: None,
            cooldown_hours,
        }
    }

    pub fn should_sync(&self) -> bool {
        match self.last_sync {
            Some(last) => {
                let elapsed = Utc::now() - last;
                elapsed >= Duration::hours(self.cooldown_hours as i64)
            }
            None => true,
        }
    }

    pub fn mark_synced(&mut self) {
        self.last_sync = Some(Utc::now());
    }
}
