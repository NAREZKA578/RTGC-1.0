//! Player module for RTGC-0.8
//! Handles player character, creation, appearance, and skills

mod player;

pub mod appearance;
pub mod character_creation;
pub mod skills;

pub use crate::game::economy::PlayerWallet;
pub use crate::game::settings::CameraMode;
pub use crate::network::protocol::PlayerInput;
pub use crate::physics::LAYER_PLAYER;
pub use appearance::Appearance;
pub use character_creation::CharacterCreation;
pub use player::{InventoryItem, ItemType, Player, PlayerState};
pub use skills::Skills;
