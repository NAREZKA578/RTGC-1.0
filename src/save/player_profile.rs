use crate::core::app_state::CharacterData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub character: CharacterData,
    pub playtime_hours: f64,
    pub created_at: String,
    pub last_saved: String,
}

impl PlayerProfile {
    pub fn new(character: CharacterData) -> Self {
        let now = chrono_now();
        Self {
            character,
            playtime_hours: 0.0,
            created_at: now.clone(),
            last_saved: now,
        }
    }
}

fn chrono_now() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    format!(
        "2026-{:02}-{:02} {:02}:{:02}:{:02}",
        days / 30 + 1,
        days % 30 + 1,
        hours,
        mins,
        secs
    )
}
