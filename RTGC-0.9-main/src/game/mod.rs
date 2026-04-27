//! Game Module for RTGC-0.8
//! Contains gameplay systems: missions, cargo, weather, day/night cycle

pub mod base_builder;
pub mod cargo;
pub mod character_creation;
pub mod debug_menu;
pub mod economy;
pub mod events;
pub mod first_mission;
pub mod interaction;
pub mod inventory;
pub mod loading_manager;
pub mod asset_manager;
pub mod main_menu;
pub mod map_system;
pub mod mission_generator;
pub mod mission_save;
pub mod player;
pub mod save;
pub mod scene;
pub mod scenes;
pub mod settings;
pub mod skills;
pub mod storage;
pub mod ui;
pub mod vehicle_parts;
pub mod weather;
pub mod winch;

pub use crate::network::protocol::PlayerInput;
pub use crate::physics::LAYER_PLAYER;
pub use base_builder::{
    BaseBuildingSystem, BaseCapability, BaseType, BuiltStructure, PlayerBase, ResourceRequirements,
    StructureType, GRID_CELL_SIZE, MAX_BASES_PER_PLAYER, MAX_STRUCTURES_PER_BASE,
    MIN_BASE_DISTANCE,
};
pub use cargo::Cargo;
pub use character_creation::CharacterCreation;
pub use debug_menu::DebugMenu;
pub use economy::{
    calculate_wage, get_base_salary, BuyOrder, ContractJob, EconomySystem, JobBoard, MarketPrice,
    PlayerWallet, Shop, ShopItem, ShopType, BASE_SALARIES,
};
pub use events::{
    init_events, poll_events, publish_event, EventManager, EventSubscriber, GameEvent,
};
pub use first_mission::{FirstMission, FirstMissionManager, FirstMissionState, PhoneNotification};
pub use interaction::{
    InteractableType, InteractionResult, InteractionSystem, MAX_INTERACTION_DISTANCE,
};
pub use inventory::{
    Inventory, InventoryItem, InventorySlot, ItemType, MAX_INVENTORY_SLOTS, MAX_INVENTORY_WEIGHT,
};
pub use main_menu::{MainMenu, MenuAction, MenuButton, MenuState};
pub use map_system::{MapMarker, MapSystem, MarkerType};
pub use mission_generator::{CargoType, Mission, MissionGenerator};
pub use mission_save::{MissionSaveManager, SaveGame};
pub use player::{CameraMode, Player, PlayerState};
pub use save::{SaveData, SaveLocationType, SaveMetadata, SaveSystem, MAX_SAVE_SLOTS};
pub use scene::{
    Scene, SceneId, SceneManager, SceneManagerConfig, SceneState, SceneType, TransitionEffect,
};
pub use scenes::{LoadingScene, MainMenuScene, OpenWorldScene, PauseScene};
pub use settings::{
    AudioSettings, ControlsSettings, DebugSettings, DisplaySettings, GameSettings,
    GameplaySettings, GraphicsSettings, HudSettings, NetworkSettings, PerformanceSettings,
    SettingsManager,
};
pub use skills::{PlayerSkills, Skill, SkillType};
pub use storage::{
    ContainerType, ItemDimensions, StorageContainer, StorageSlot, StorageSystem, StoredItem,
    MAX_STORAGE_HEIGHT, MAX_STORAGE_SLOTS, MAX_STORAGE_WIDTH,
};
pub use ui::{
    HUDData, MinimapData, Notification, NotificationType, UIManager, UIVisibility,
    WaypointType,
};
pub use vehicle_parts::{
    PartCategory, PartDiagnostic, VehiclePart, VehiclePartsSystem, MAX_INTEGRITY,
    MIN_FUNCTIONAL_INTEGRITY,
};
pub use weather::WeatherState;
pub use winch::Winch;
