//! Scene modules

pub mod loading;
pub mod main_menu;
pub mod open_world;
pub mod pause;

pub use loading::LoadingScene;
pub use main_menu::MainMenuScene;
pub use open_world::OpenWorldScene;
pub use pause::PauseScene;
