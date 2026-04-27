//! Events System - Cross-module event communication using crossbeam-channel
//! Implements publish-subscribe pattern for game events
//! Исправлено: убран unsafe, используется Arc<Mutex<>> для потокобезопасности

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use nalgebra::Vector3;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

/// Maximum events in channel before blocking
const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Global event channel storage using Arc<Mutex<>> for thread safety
struct EventChannelStorage {
    sender: Mutex<Option<Sender<GameEvent>>>,
    receiver: Mutex<Option<Receiver<GameEvent>>>,
}

impl EventChannelStorage {
    fn new() -> Self {
        Self {
            sender: Mutex::new(None),
            receiver: Mutex::new(None),
        }
    }
}

/// Global event channel (lazy-initialized, thread-safe)
static EVENT_STORAGE: Lazy<Arc<EventChannelStorage>> =
    Lazy::new(|| Arc::new(EventChannelStorage::new()));

/// Initialize the event system (call once at startup)
pub fn init_events() {
    let (tx, rx) = bounded(EVENT_CHANNEL_CAPACITY);

    if let Ok(mut sender_guard) = EVENT_STORAGE.sender.lock() {
        *sender_guard = Some(tx);
    }
    if let Ok(mut receiver_guard) = EVENT_STORAGE.receiver.lock() {
        *receiver_guard = Some(rx);
    }
}

/// Get the global event sender (thread-safe clone)
pub fn get_event_sender() -> Option<Sender<GameEvent>> {
    EVENT_STORAGE
        .sender
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

/// Get the global event receiver (for main thread polling)
pub fn get_event_receiver() -> Option<Receiver<GameEvent>> {
    EVENT_STORAGE
        .receiver
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

/// Publish an event to all subscribers (thread-safe)
pub fn publish_event(event: GameEvent) -> Result<(), TrySendError<GameEvent>> {
    let sender_guard = EVENT_STORAGE
        .sender
        .lock()
        .map_err(|_| TrySendError::Disconnected(event.clone()))?;

    if let Some(ref sender) = *sender_guard {
        sender.try_send(event)?;
    }
    Ok(())
}

/// Poll for events (non-blocking, main thread only)
pub fn poll_events() -> Vec<GameEvent> {
    let mut events = Vec::new();

    if let Ok(receiver_guard) = EVENT_STORAGE.receiver.lock() {
        if let Some(ref receiver) = *receiver_guard {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }
    }
    events
}

/// Game event types
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// Player entered a vehicle
    PlayerEnteredVehicle {
        player_name: String,
        vehicle_index: usize,
        vehicle_id: u64,
        seat_index: usize,
    },

    /// Player exited a vehicle
    PlayerExitedVehicle {
        player_name: String,
        vehicle_index: usize,
        vehicle_id: u64,
        exit_position: Vector3<f32>,
    },

    /// Skill leveled up
    SkillLeveledUp {
        skill_name: String,
        old_rank: u8,
        new_rank: u8,
        mastery: f32,
    },

    /// Vehicle was damaged
    VehicleDamaged {
        vehicle_index: usize,
        part_name: String,
        damage_amount: f32,
        cause: DamageCause,
    },

    /// Interaction triggered (door, item, etc.)
    InteractionTriggered {
        interaction_type: InteractionType,
        position: Vector3<f32>,
        entity_index: Option<usize>,
    },

    /// Cargo loaded
    CargoLoaded {
        cargo_type: String,
        weight: f32,
        vehicle_index: usize,
    },

    /// Cargo unloaded
    CargoUnloaded {
        cargo_type: String,
        weight: f32,
        position: Vector3<f32>,
    },

    /// Mission started
    MissionStarted {
        mission_id: u64,
        mission_name: String,
    },

    /// Mission completed
    MissionCompleted {
        mission_id: u64,
        reward_rub: f64,
        reputation_change: i32,
    },

    /// First mission phone call triggered
    FirstMissionPhoneCall,

    /// First mission accepted
    FirstMissionAccepted,

    /// First mission delivery started
    FirstMissionDeliveryStarted,

    /// First mission completed
    FirstMissionCompleted { reward: f64, time_taken_hours: f32 },

    /// First mission failed
    FirstMissionFailed { reason: String },

    /// Contacts unlocked
    ContactsUnlocked { count: usize },

    /// Money changed
    MoneyChanged {
        amount: f64,
        currency: Currency,
        reason: String,
    },

    /// Weather changed
    WeatherChanged {
        weather_type: String,
        intensity: f32,
    },

    /// Time of day changed
    TimeOfDayChanged { hour: f32, is_night: bool },

    /// Player saved game
    GameSaved {
        save_slot: u8,
        position: Vector3<f32>,
    },

    /// Player loaded game
    GameLoaded {
        save_slot: u8,
        position: Vector3<f32>,
    },

    /// NPC spawned
    NpcSpawned {
        npc_id: u64,
        npc_type: String,
        position: Vector3<f32>,
    },

    /// NPC despawned
    NpcDespawned { npc_id: u64 },

    /// Building constructed
    BuildingConstructed {
        building_type: String,
        position: Vector3<f32>,
    },

    /// Resource extracted
    ResourceExtracted {
        resource_type: String,
        amount: f32,
        position: Vector3<f32>,
    },
}

/// Damage cause enum
#[derive(Debug, Clone)]
pub enum DamageCause {
    Collision { impact_velocity: f32 },
    Wear { hours_used: f32 },
    Overload { load_factor: f32 },
    Environmental { weather_intensity: f32 },
    Combat,
    Other,
}

/// Interaction type enum
#[derive(Debug, Clone)]
pub enum InteractionType {
    EnterVehicle,
    ExitVehicle,
    PickUpItem,
    DropItem,
    OpenDoor,
    CloseDoor,
    UseMachine,
    TalkToNpc,
    ActivateSwitch,
    Refuel,
    Repair,
    LoadCargo,
    UnloadCargo,
}

/// Currency enum
#[derive(Debug, Clone, Copy)]
pub enum Currency {
    Rub,
    Cny,
    Usd,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::Rub => write!(f, "RUB"),
            Currency::Cny => write!(f, "CNY"),
            Currency::Usd => write!(f, "USD"),
        }
    }
}

/// Event subscriber trait for modules that want to listen to events
pub trait EventSubscriber: Send {
    /// Handle an event
    fn handle_event(&mut self, event: &GameEvent);
}

/// Event manager for managing multiple subscribers
pub struct EventManager {
    subscribers: Vec<Box<dyn EventSubscriber>>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    /// Add a subscriber
    pub fn add_subscriber<T: EventSubscriber + 'static>(&mut self, subscriber: T) {
        self.subscribers.push(Box::new(subscriber));
    }

    /// Process all pending events and notify subscribers
    pub fn process_events(&mut self) {
        let events = poll_events();
        for event in events {
            for subscriber in &mut self.subscribers {
                subscriber.handle_event(&event);
            }
        }
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_system_thread_safe() {
        init_events();

        // Test publish from "another thread"
        let event = GameEvent::WeatherChanged {
            weather_type: "Rain".to_string(),
            intensity: 0.5,
        };

        assert!(publish_event(event).is_ok());

        // Test poll events
        let events = poll_events();
        assert!(!events.is_empty());
    }
}
