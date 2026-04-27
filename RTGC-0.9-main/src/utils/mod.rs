//! Utils module for RTGC-0.8

pub mod math;
pub mod time;
pub mod logger;
pub mod random;
pub mod console;
pub mod hot_reload;
pub mod terrain;
pub mod path;

pub use math::*;
pub use time::{TimeManager, FpsCounter};
pub use logger::{init_logger, init_logger_with_level};
pub use random::Rng;
pub use console::{Console, ConsoleKey};
pub use hot_reload::{HotReloadManager, HotReloadConfig};
pub use terrain::{compute_terrain_normal, compute_terrain_normal_from_heightmap, get_height_from_heightmap};
pub use path::{sanitize_path, validate_path, create_safe_save_path};
