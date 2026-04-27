use nalgebra::{Isometry3, Matrix3, Point3, UnitQuaternion, Vector3};
use tracing;

use crate::physics::arena_allocator::ArenaAllocator;
use crate::physics::spatial_hash::SpatialHash;
use crate::physics::thread_pool::ThreadPool;
use crate::physics::constraints::{SpringConstraint, RaycastSuspension};
use super::rigid_body::{RigidBody, Shape, ContactEvent, PhysicsStats, Aabb, LAYER_WORLD};

#[derive(Debug, Clone)]
struct Contact {
    body_a: usize,
    body_b: usize,
    contact_point: Vector3<f32>,
    normal: Vector3<f32>,
    penetration_depth: f32,
    restitution: f32,
    friction: f32,
}
