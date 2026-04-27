//! Interaction System for RTGC-0.9
//! Handles player interactions with doors, vehicles, objects, NPCs

use crate::game::inventory::InventoryItem;
use crate::game::events::{publish_event, GameEvent};
use crate::physics::PhysicsWorld;
use crate::physics::RaycastHit;
use crate::physics::{LAYER_INTERACTABLE_DOOR, LAYER_INTERACTABLE_OBJECT, LAYER_INTERACTABLE_VEHICLE};
use nalgebra::Vector3;

// Type alias for backwards compatibility
type Vec3 = Vector3<f32>;

/// Interaction layers bitmask (re-exported from physics module)
pub const LAYER_INTERACTABLE_NPC: u32 = 0b01000;
pub const LAYER_INTERACTABLE_ALL: u32 = LAYER_INTERACTABLE_DOOR | LAYER_INTERACTABLE_VEHICLE | LAYER_INTERACTABLE_OBJECT | LAYER_INTERACTABLE_NPC;

/// Maximum interaction distance (meters)
pub const MAX_INTERACTION_DISTANCE: f32 = 3.0;

/// Types of interactable objects
#[derive(Debug, Clone, PartialEq)]
pub enum InteractableType {
    /// Vehicle door (enter/exit)
    VehicleDoor {
        vehicle_id: u64,
        door_index: usize,
        is_open: bool,
    },
    /// Regular door (open/close)
    Door {
        door_id: u32,
        is_open: bool,
        locked: bool,
    },
    /// Pickable object
    PickableObject {
        object_id: u32,
        weight_kg: f32,
        name: String,
    },
    /// NPC to talk to
    NPC {
        npc_id: u32,
        name: String,
        dialogue_tree: String,
    },
    /// Bed/sleep location (for saving)
    Bed {
        bed_id: u32,
        location_name: String,
        is_owned: bool,
    },
    /// Workbench/crafting station
    Workbench {
        bench_id: u32,
        crafting_type: CraftingType,
    },
    /// Fuel pump
    FuelPump {
        pump_id: u32,
        fuel_type: FuelType,
        price_per_liter: f32,
    },
    /// Shop/trading post
    Shop {
        shop_id: u32,
        shop_type: ShopType,
        owner_name: String,
    },
}

/// Crafting types for workbenches
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CraftingType {
    Mechanics,
    Welding,
    Carpentry,
    Electronics,
}

/// Fuel types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FuelType {
    AI92,
    AI95,
    Diesel,
}

/// Shop types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShopType {
    GeneralStore,
    AutoParts,
    ConstructionSupplies,
    FoodMarket,
    Electronics,
}

/// Interaction result
#[derive(Debug, Clone)]
pub struct InteractionResult {
    pub success: bool,
    pub message: String,
    pub interactable: Option<InteractableType>,
}

/// Interaction manager
pub struct InteractionSystem {
    /// Currently highlighted interactable
    highlighted: Option<(InteractableType, f32)>,
    /// Interaction cooldown (prevent spam)
    interaction_cooldown: f32,
    /// Reference to player inventory for pickup operations
    inventory: Option<std::sync::Arc<std::sync::Mutex<crate::game::inventory::Inventory>>>,
}

impl InteractionSystem {
    pub fn new() -> Self {
        Self {
            highlighted: None,
            interaction_cooldown: 0.0,
            inventory: None,
        }
    }

    /// Set inventory reference for pickup operations
    pub fn set_inventory(&mut self, inv: std::sync::Arc<std::sync::Mutex<crate::game::inventory::Inventory>>) {
        self.inventory = Some(inv);
    }

    /// Update interaction system - raycast from player to find interactables
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vector3<f32>,
        player_forward: Vector3<f32>,
        camera_distance: f32,
        physics_world: &PhysicsWorld,
    ) {
        // Reduce cooldown
        if self.interaction_cooldown > 0.0 {
            self.interaction_cooldown -= dt;
        }

        // Raycast from camera position
        let ray_origin = player_pos + Vector3::new(0.0, 1.7, 0.0); // Eye height
        let ray_direction = player_forward.normalize();

        // Cast ray and find closest interactable
        let ray = crate::physics::Ray {
            origin: ray_origin.into(),
            direction: ray_direction,
        };
        let hit = physics_world.raycast(&ray).and_then(|hit| {
            if hit.distance <= MAX_INTERACTION_DISTANCE {
                Some(hit)
            } else {
                None
            }
        });

        self.highlighted = hit.and_then(|h| {
            if h.distance < MAX_INTERACTION_DISTANCE {
                self.identify_interactable(&h)
                    .map(|interactable| (interactable, h.distance))
            } else {
                None
            }
        });
    }

    /// Identify what type of interactable was hit
    fn identify_interactable(&self, hit: &RaycastHit) -> Option<InteractableType> {
        // This would check collision layers and object metadata
        // Placeholder implementation
        match hit.layer {
            LAYER_INTERACTABLE_DOOR => Some(InteractableType::Door {
                door_id: hit.object_id,
                is_open: false,
                locked: false,
            }),
            LAYER_INTERACTABLE_VEHICLE => Some(InteractableType::VehicleDoor {
                vehicle_id: hit.object_id as u64,
                door_index: 0,
                is_open: false,
            }),
            LAYER_INTERACTABLE_OBJECT => Some(InteractableType::PickableObject {
                object_id: hit.object_id,
                weight_kg: 5.0,
                name: "Unknown Object".to_string(),
            }),
            _ => None,
        }
    }

    /// Get currently highlighted interactable
    pub fn get_highlighted(&self) -> Option<&(InteractableType, f32)> {
        self.highlighted.as_ref()
    }

    /// Try to interact with highlighted object (F key)
    pub fn try_interact(
        &mut self,
        player_state: &mut crate::game::player::PlayerState,
    ) -> InteractionResult {
        if self.interaction_cooldown > 0.0 {
            return InteractionResult {
                success: false,
                message: "Too fast!".to_string(),
                interactable: None,
            };
        }

        if let Some((interactable, distance)) = &self.highlighted {
            if *distance > MAX_INTERACTION_DISTANCE {
                return InteractionResult {
                    success: false,
                    message: "Too far!".to_string(),
                    interactable: None,
                };
            }

            let result = self.perform_interaction(interactable, player_state);

            if result.success {
                self.interaction_cooldown = 0.3; // 300ms cooldown
            }

            result
        } else {
            InteractionResult {
                success: false,
                message: "Nothing to interact with".to_string(),
                interactable: None,
            }
        }
    }

    /// Perform specific interaction based on type
    fn perform_interaction(
        &self,
        interactable: &InteractableType,
        player_state: &mut crate::game::player::PlayerState,
    ) -> InteractionResult {
        match interactable {
            InteractableType::VehicleDoor {
                vehicle_id,
                door_index,
                ..
            } => {
                // Handle vehicle enter/exit
                self.handle_vehicle_interaction(*vehicle_id, *door_index, player_state)
            }
            InteractableType::Door {
                door_id,
                is_open,
                locked,
            } => self.handle_door_interaction(*door_id, *is_open, *locked, player_state),
            InteractableType::PickableObject {
                object_id,
                weight_kg,
                name,
            } => self.handle_pickup_interaction(*object_id, *weight_kg, name.clone(), player_state),
            InteractableType::Bed {
                bed_id,
                location_name,
                is_owned,
            } => {
                self.handle_bed_interaction(*bed_id, location_name.clone(), *is_owned, player_state)
            }
            _ => InteractionResult {
                success: false,
                message: "Interaction not implemented".to_string(),
                interactable: None,
            },
        }
    }

    /// Handle vehicle enter/exit
    fn handle_vehicle_interaction(
        &self,
        vehicle_id: u64,
        door_index: usize,
        player_state: &mut crate::game::player::PlayerState,
    ) -> InteractionResult {
        use crate::game::player::PlayerState as PState;

        match player_state {
            PState::OnFoot => {
                // Enter vehicle
                *player_state = PState::InVehicle {
                    vehicle_index: vehicle_id as usize,
                    seat_index: door_index as u8,
                };

                publish_event(GameEvent::PlayerEnteredVehicle {
                    player_name: "Player".to_string(),
                    vehicle_index: vehicle_id as usize,
                    vehicle_id,
                    seat_index: door_index,
                });

                InteractionResult {
                    success: true,
                    message: format!("Entered vehicle {}", vehicle_id),
                    interactable: None,
                }
            }
            PState::InHelicopter { .. } | PState::InCrane => InteractionResult {
                success: false,
                message: "Cannot enter vehicle while in vehicle/machine".to_string(),
                interactable: None,
            },
            PState::InVehicle {
                vehicle_index: current_vid,
                ..
            } => {
                if *current_vid == vehicle_id as usize {
                    // Exit current vehicle
                    *player_state = PState::OnFoot;

                    publish_event(GameEvent::PlayerExitedVehicle {
                        player_name: "Player".to_string(),
                        vehicle_index: 0,
                        vehicle_id: vehicle_id as u64,
                        exit_position: nalgebra::Vector3::zeros(),
                    });

                    InteractionResult {
                        success: true,
                        message: format!("Exited vehicle {}", vehicle_id),
                        interactable: None,
                    }
                } else {
                    InteractionResult {
                        success: false,
                        message: "Already in another vehicle".to_string(),
                        interactable: None,
                    }
                }
            }
        }
    }

    /// Handle door open/close
    fn handle_door_interaction(
        &self,
        door_id: u32,
        is_open: bool,
        locked: bool,
        _player_state: &mut crate::game::player::PlayerState,
    ) -> InteractionResult {
        if locked {
            return InteractionResult {
                success: false,
                message: "Door is locked".to_string(),
                interactable: None,
            };
        }

        let new_state = !is_open;
        publish_event(GameEvent::InteractionTriggered {
            interaction_type: if new_state {
                crate::game::events::InteractionType::OpenDoor
            } else {
                crate::game::events::InteractionType::CloseDoor
            },
            position: nalgebra::Vector3::zeros(),
            entity_index: None,
        });

        InteractionResult {
            success: true,
            message: if new_state {
                "Door opened"
            } else {
                "Door closed"
            }
            .to_string(),
            interactable: None,
        }
    }

    /// Handle object pickup
    fn handle_pickup_interaction(
        &self,
        object_id: u32,
        weight_kg: f32,
        name: String,
        player_state: &mut crate::game::player::PlayerState,
    ) -> InteractionResult {
        // Check inventory capacity (60kg limit from player.rs)
        let current_inventory_weight = match player_state {
            crate::game::player::PlayerState::OnFoot => {
                // Try to get actual inventory weight if available
                if let Some(ref inv_arc) = self.inventory {
                    match inv_arc.lock() { Ok(inv) => {
                        inv.get_total_weight()
                    } _ => {
                        0.0
                    }}
                } else {
                    0.0
                }
            }
            crate::game::player::PlayerState::InVehicle { .. } => 0.0,
            crate::game::player::PlayerState::InHelicopter { .. } => 0.0,
            crate::game::player::PlayerState::InCrane => 0.0,
        };

        if current_inventory_weight + weight_kg > 60.0 {
            return InteractionResult {
                success: false,
                message: "Inventory too heavy!".to_string(),
                interactable: None,
            };
        }

        // Add item to inventory if available
            if let Some(ref inv_arc) = self.inventory {
                if let Ok(mut inv) = inv_arc.lock() {
                    // Create a generic pickup item
                    let item_type = crate::game::inventory::ItemType::Resource(crate::game::inventory::ResourceType::Wood);
                    if let Err(e) = inv.add_item(InventoryItem::new("Wood", 1, item_type)) {
                        return InteractionResult {
                            success: false,
                            message: format!("Failed to add item: {}", e),
                            interactable: None,
                        };
                    }
                }
            }

        publish_event(GameEvent::InteractionTriggered {
            interaction_type: crate::game::events::InteractionType::PickUpItem,
            position: nalgebra::Vector3::zeros(),
            entity_index: None,
        });

        InteractionResult {
            success: true,
            message: format!("Picked up {} ({:.1} kg)", name, weight_kg),
            interactable: None,
        }
    }

    /// Handle bed interaction (sleep/save)
    fn handle_bed_interaction(
        &self,
        bed_id: u32,
        location_name: String,
        is_owned: bool,
        _player_state: &mut crate::game::player::PlayerState,
    ) -> InteractionResult {
        publish_event(GameEvent::InteractionTriggered {
            interaction_type: crate::game::events::InteractionType::UseMachine,
            position: nalgebra::Vector3::zeros(),
            entity_index: None,
        });

        InteractionResult {
            success: true,
            message: format!(
                "Sleeping at {} (Save point{})",
                location_name,
                if is_owned { " - Owned" } else { "" }
            ),
            interactable: None,
        }
    }

    /// Reset interaction state
    pub fn reset(&mut self) {
        self.highlighted = None;
        self.interaction_cooldown = 0.0;
    }
}

impl Default for InteractionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_system_creation() {
        let system = InteractionSystem::new();
        assert!(system.get_highlighted().is_none());
    }

    #[test]
    fn test_interaction_cooldown() {
        let mut system = InteractionSystem::new();

        // Simulate update
        system.update(0.1, Vector3::zeros(), Vector3::z(), 2.0);

        // Cooldown should be 0 initially
        assert_eq!(system.interaction_cooldown, 0.0);
    }
}
