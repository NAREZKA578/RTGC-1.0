//! Player character structure and logic

use crate::game::save::{PlayerMoneyData, PlayerSkillsData};
use crate::game::settings::CameraMode;

use crate::network::protocol::PlayerInput;
use nalgebra::{UnitQuaternion, Vector3};

/// Inventory item (placeholder for now)
#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub name: String,
    pub count: u32,
    pub item_type: ItemType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    Tool,
    Consumable,
    Material,
    Other,
}

impl Default for InventoryItem {
    fn default() -> Self {
        Self {
            name: String::new(),
            count: 0,
            item_type: ItemType::Other,
        }
    }
}

/// Player state - where the player currently is
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    OnFoot,
    InVehicle {
        vehicle_index: usize,
        seat_index: u8,
    },
    InHelicopter {
        heli_index: usize,
        seat_index: u8,
    },
    InCrane,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::OnFoot
    }
}

/// Main player structure
#[derive(Debug, Clone)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub is_male: bool,
    pub height: f32,
    pub skin_color: [f32; 3],
    pub face_variant: u8,
    pub hair_style: u8,
    pub hair_color: [f32; 3],

    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub velocity: Vector3<f32>,

    pub state: PlayerState,

    pub stamina: f32,
    pub max_stamina: f32,
    pub health: f32,
    pub money: PlayerMoneyData,
    pub inventory: Vec<InventoryItem>,
    pub inventory_weight: f32,
    pub max_inventory_weight: f32,

    pub skills: PlayerSkillsData,

    pub capsule_body_id: Option<usize>,
    pub body_index: Option<usize>,

    pub camera_mode: CameraMode,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::from("Player"),
            is_male: true,
            height: 1.75,
            skin_color: [0.8, 0.65, 0.5],
            face_variant: 0,
            hair_style: 0,
            hair_color: [0.3, 0.2, 0.1],

            position: Vector3::new(0.0, 2.0, 0.0),
            rotation: UnitQuaternion::identity(),
            velocity: Vector3::zeros(),

            state: PlayerState::OnFoot,

            stamina: 1.0,
            max_stamina: 1.0,
            health: 100.0,
            money: PlayerMoneyData::default(),
            inventory: Vec::new(),
            inventory_weight: 0.0,
            max_inventory_weight: 50.0,

            skills: PlayerSkillsData::default(),

            capsule_body_id: None,
            body_index: None,

            camera_mode: CameraMode::ThirdPerson {
                distance: 4.0,
                yaw: 0.0,
                pitch: 0.3,
            },
        }
    }
}

impl Player {
    /// Create a new player with custom name
    pub fn new(name: String) -> Self {
        let mut player = Self::default();
        player.name = name;
        player.id = 1;
        player
    }

    /// Check if player is on foot
    pub fn is_on_foot(&self) -> bool {
        matches!(self.state, PlayerState::OnFoot)
    }

    /// Check if player is in any vehicle
    pub fn is_in_vehicle(&self) -> bool {
        matches!(
            self.state,
            PlayerState::InVehicle { .. } | PlayerState::InHelicopter { .. } | PlayerState::InCrane
        )
    }

    /// Get current vehicle ID if in vehicle
    pub fn get_vehicle_id(&self) -> Option<usize> {
        match self.state {
            PlayerState::InVehicle { vehicle_index, .. } => Some(vehicle_index),
            PlayerState::InHelicopter { heli_index, .. } => Some(heli_index),
            _ => None,
        }
    }

    /// Exit current vehicle
    pub fn exit_vehicle(&mut self) {
        self.state = PlayerState::OnFoot;
    }

    /// Enter a vehicle
    pub fn enter_vehicle(&mut self, vehicle_index: usize, seat_index: u8) {
        self.state = PlayerState::InVehicle {
            vehicle_index,
            seat_index,
        };
    }

    /// Update player position from physics body
    pub fn sync_from_physics(&mut self, new_pos: Vector3<f32>, new_vel: Vector3<f32>) {
        self.position = new_pos;
        self.velocity = new_vel;
    }

    pub fn update_stamina(&mut self, delta: f32) {
        self.stamina = (self.stamina + delta).clamp(0.0, self.max_stamina);
    }

    pub fn process_input(&mut self, input: &PlayerInput, dt: f32) {
        let speed = 5.0;
        if input.throttle > 0.0 {
            self.velocity.z = -speed * input.throttle;
        } else if input.throttle < 0.0 {
            self.velocity.z = speed * (-input.throttle);
        }
        if input.steering != 0.0 {
            self.velocity.x = speed * input.steering;
        }
    }

    pub fn process_input_with_physics(
        &mut self,
        input: &PlayerInput,
        physics_world: &mut crate::physics::PhysicsWorld,
        dt: f32,
    ) {
        self.process_input(input, dt);

        // Apply velocity to physics body if exists
        if let Some(body_idx) = self.body_index {
            if let Some(body) = physics_world.get_body_mut(body_idx) {
                body.velocity = self.velocity;
            }
        }
    }

    pub fn create_physics_body(&mut self, position: Vector3<f32>) -> crate::physics::RigidBody {
        self.position = position;
        crate::physics::RigidBody::new_capsule(self.position, 80.0, 0.4, 0.9)
    }
}
