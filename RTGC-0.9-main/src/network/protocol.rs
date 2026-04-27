// Network Protocol Module for RTGC
// Defines serializable structures for multiplayer game state synchronization
// Note: This file only contains data structures - no network implementation yet

use serde::{Serialize, Deserialize};

/// Unique identifier for a networked entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

impl EntityId {
    pub const fn null() -> Self {
        Self(0)
    }
    
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }
}

/// Entity type for replication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Vehicle,
    Helicopter,
    Player,
    NPC,
    Cargo,
    Building,
    Prop,
    Projectile,
    Custom(u32),
}

/// Replication priority for entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReplicationPriority {
    Low = 0,      // Static props, distant buildings
    Medium = 1,   // NPCs, cargo
    High = 2,     // Local vehicles, players
    Critical = 3, // Player's own vehicle, active projectiles
}

/// Component types that can be replicated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicatedComponent {
    Transform {
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    },
    Velocity {
        linear: [f32; 3],
        angular: [f32; 3],
    },
    VehicleState {
        throttle: f32,
        brake: f32,
        steering: f32,
        gear: i32,
        engine_rpm: f32,
        fuel: f32,
        health: f32,
        diff_lock_rear: bool,
        diff_lock_front: bool,
        low_range: bool,
    },
    HelicopterState {
        collective: f32,
        cyclic_longitudinal: f32,
        cyclic_lateral: f32,
        tail_rotor: f32,
        throttle: f32,
        main_rotor_rpm: f32,
        tail_rotor_rpm: f32,
    },
    Health {
        current: f32,
        max: f32,
    },
    Inventory {
        items: Vec<(String, u32)>,
    },
    Animation {
        state: String,
        progress: f32,
        blend_weight: f32,
    },
}

/// Replicated entity state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicatedEntity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub priority: ReplicationPriority,
    pub components: Vec<ReplicatedComponent>,
    pub tick_created: u64,
    pub tick_updated: u64,
}

impl ReplicatedEntity {
    pub fn new(id: EntityId, entity_type: EntityType, priority: ReplicationPriority) -> Self {
        let now_tick = 0; // Will be set by server
        Self {
            id,
            entity_type,
            priority,
            components: Vec::new(),
            tick_created: now_tick,
            tick_updated: now_tick,
        }
    }
    
    pub fn with_transform(mut self, position: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> Self {
        self.components.push(ReplicatedComponent::Transform { position, rotation, scale });
        self
    }
}

/// World chunk data for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkData {
    pub x: i32,
    pub y: i32,
    pub terrain_heights: Vec<f32>,
    pub terrain_materials: Vec<u8>,
    pub prop_ids: Vec<EntityId>,
    pub building_ids: Vec<EntityId>,
}

/// Complete game state for network synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// World seed for deterministic generation
    pub world_seed: u64,
    
    /// Current server tick
    pub server_tick: u64,
    
    /// Time delta for this tick
    pub tick_delta: f32,
    
    /// Player vehicle position
    pub vehicle_position: [f32; 3],
    
    /// Player vehicle rotation (quaternion)
    pub vehicle_rotation: [f32; 4],
    
    /// Vehicle velocity
    pub vehicle_velocity: [f32; 3],
    
    /// Vehicle angular velocity
    pub vehicle_angular_velocity: [f32; 3],
    
    /// Vehicle fuel level (0.0 - 1.0)
    pub vehicle_fuel: f32,
    
    /// Vehicle health (0.0 - 1.0)
    pub vehicle_health: f32,
    
    /// Vehicle engine RPM
    pub vehicle_rpm: f32,
    
    /// Current gear (-1 = reverse, 0 = neutral, 1+ = forward)
    pub vehicle_gear: i32,
    
    /// Active cargo mission ID (if any)
    pub current_mission_id: Option<String>,
    
    /// Cargo weight in kg (if attached)
    pub cargo_weight_kg: Option<f32>,
    
    /// Time of day (0.0 - 24.0)
    pub time_of_day: f32,
    
    /// Weather seed for deterministic weather
    pub weather_seed: u64,
    
    /// Player reputation/score
    pub reputation: i32,
    
    /// Completed mission IDs
    pub completed_missions: Vec<String>,
    
    /// Helicopter position (if active)
    pub helicopter_position: Option<[f32; 3]>,
    
    /// Helicopter rotation (if active)
    pub helicopter_rotation: Option<[f32; 4]>,
    
    /// Replicated entities in vicinity
    pub replicated_entities: Vec<ReplicatedEntity>,
    
    /// Loaded chunks around player
    pub loaded_chunks: Vec<ChunkData>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            world_seed: 0,
            server_tick: 0,
            tick_delta: 0.016,
            vehicle_position: [0.0; 3],
            vehicle_rotation: [0.0, 0.0, 0.0, 1.0], // identity quaternion
            vehicle_velocity: [0.0; 3],
            vehicle_angular_velocity: [0.0; 3],
            vehicle_fuel: 1.0,
            vehicle_health: 1.0,
            vehicle_rpm: 0.0,
            vehicle_gear: 0,
            current_mission_id: None,
            cargo_weight_kg: None,
            time_of_day: 12.0,
            weather_seed: 0,
            reputation: 0,
            completed_missions: Vec::new(),
            helicopter_position: None,
            helicopter_rotation: None,
            replicated_entities: Vec::new(),
            loaded_chunks: Vec::new(),
        }
    }
}

/// Network message types for client-server communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Client requests to join server
    JoinRequest {
        player_name: String,
        client_version: String,
    },
    
    /// Server accepts client join
    JoinAccepted {
        client_id: u64,
        server_tick: u64,
        initial_state: GameState,
    },
    
    /// Server rejects client join
    JoinRejected {
        reason: String,
    },
    
    /// Client sends input state
    InputUpdate {
        tick: u64,
        inputs: PlayerInput,
    },
    
    /// Server broadcasts game state update
    StateUpdate {
        game_state: GameState,
        tick: u64,
        entity_updates: Vec<EntityUpdate>,
        entity_spawns: Vec<ReplicatedEntity>,
        entity_despawns: Vec<EntityId>,
    },
    
    /// Client acknowledges state
    StateAck {
        tick: u64,
        latency_ms: f32,
    },
    
    /// Mission started
    MissionStart {
        mission_id: String,
        pickup_location: [f32; 3],
        delivery_location: [f32; 3],
        cargo_type: String,
        reward: i32,
    },
    
    /// Mission completed
    MissionComplete {
        mission_id: String,
        success: bool,
        reward_earned: i32,
    },
    
    /// Chat message
    ChatMessage {
        sender_id: u64,
        sender_name: String,
        message: String,
    },
    
    /// RPC call to server
    RpcRequest {
        method: String,
        args: serde_json::Value,
    },
    
    /// RPC response from server
    RpcResponse {
        method: String,
        result: Result<serde_json::Value, String>,
    },
    
    /// Heartbeat / keepalive
    Heartbeat {
        timestamp: u64,
    },
    
    /// Disconnect notification
    Disconnect {
        reason: String,
    },
}

/// Incremental entity update for efficient replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityUpdate {
    pub id: EntityId,
    pub tick: u64,
    pub updated_components: Vec<ReplicatedComponent>,
}

/// Player input state for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub throttle: f32,
    pub brake: f32,
    pub steering: f32,
    pub handbrake: bool,
    pub diff_lock_rear: bool,
    pub diff_lock_front: bool,
    pub low_range: bool,
    pub winch_active: bool,
    
    // Helicopter controls
    pub collective: f32,
    pub cyclic_x: f32,
    pub cyclic_y: f32,
    pub yaw: f32,
    pub heli_throttle: f32,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            brake: 0.0,
            steering: 0.0,
            handbrake: false,
            diff_lock_rear: false,
            diff_lock_front: false,
            low_range: false,
            winch_active: false,
            collective: 0.0,
            cyclic_x: 0.0,
            cyclic_y: 0.0,
            yaw: 0.0,
            heli_throttle: 0.0,
        }
    }
}

/// Network statistics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub average_latency_ms: f32,
    pub packet_loss_percent: f32,
    pub entities_replicated: u32,
    pub chunks_streamed: u32,
}

/// Client information on server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub client_id: u64,
    pub player_name: String,
    pub connection_time_secs: u64,
    pub last_input_tick: u64,
    pub last_ack_tick: u64,
    pub average_latency_ms: f32,
    pub packet_loss_percent: f32,
}
