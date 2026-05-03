use crate::core::app_state::{CharacterData, Gender};
use serde::{Deserialize, Serialize};

pub fn default_character() -> CharacterData {
    CharacterData {
        id: uuid::Uuid::new_v4().to_string(),
        gender: Gender::Male,
        height_m: 1.80,
        skin_color: 0,
        face_index: 0,
        hair_index: 0,
        hair_color: [0.2, 0.15, 0.1],
        university_id: String::new(),
        specialty: String::new(),
        start_capital: 50000.0,
        uaz_color: [0.9, 0.9, 0.9],
        start_region: String::from("center"),
        start_pos: [0.0, 0.0, 0.0],
    }
}
