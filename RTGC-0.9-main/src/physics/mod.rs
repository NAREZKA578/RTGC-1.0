//! Physics Module for RTGC-0.9
//! Provides rigid body dynamics, collision detection, constraints, and vehicle physics

pub mod advanced_vehicle;
pub mod arena_allocator;
pub mod async_physics;
pub mod constraints;
pub mod crane_arm;
pub mod deformable_terrain;
pub mod fracture_component;
pub mod helicopter;
pub mod physics_module;
pub mod spatial_hash;
pub mod thread_pool;
pub mod tracked_vehicle;
pub mod vehicle;

// Core types
pub use physics_module::{
    Aabb, PhysicsStats, PhysicsWorld, Ray, RaycastHit, RigidBody, Shape,
    ContactEvent,
};

// Collision layers
pub use physics_module::{
    LAYER_CARGO, LAYER_PLAYER, LAYER_TRIGGER, LAYER_VEHICLE, LAYER_WORLD,
    LAYER_INTERACTABLE_DOOR, LAYER_INTERACTABLE_OBJECT, LAYER_INTERACTABLE_VEHICLE,
};

// Vehicles
pub use advanced_vehicle::{AdvancedSuspension, AdvancedVehicle, AdvancedWheel};
pub use vehicle::{Vehicle, VehicleConfig, VehicleControls, WheelState};

// Helicopter
pub use helicopter::{
    Helicopter, HelicopterConfig, HelicopterControls, HelicopterState,
    MainRotor, TailRotor, TurboshaftEngine,
};

// Constraints
pub use constraints::{RaycastSuspension, SpringConstraint};

// Other
pub use arena_allocator::ArenaAllocator;
pub use async_physics::AsyncPhysicsEngine;
pub use crane_arm::{CraneArm, CraneConfig, CraneState};
pub use deformable_terrain::{DeformableTerrainComponent, DeformableTerrainInterface, DeformationType};
pub use fracture_component::FractureComponent;
pub use spatial_hash::SpatialHash;
pub use thread_pool::ThreadPool;
pub use tracked_vehicle::{
    TrackedControls, TrackedVehicle, TrackedVehicleState, TrackedVehicleType,
};
