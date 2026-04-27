//! Physics Constraints Module
//! Implements spring constraints, joints, and other constraint types for vehicle suspension
//! 
//! # Безопасность
//! - Все вычисления проверяются на NaN/Inf
//! - При обнаружении некорректных значений применяется безопасное состояние

use crate::physics::physics_module::PhysicsWorld;
use nalgebra::{Isometry3, Point3, UnitQuaternion, Vector3};

/// Spring constraint for vehicle suspension (B1)
#[derive(Debug, Clone)]
pub struct SpringConstraint {
    pub body_a: usize,          // chassis body index
    pub body_b: usize,          // wheel body index (if wheels are separate bodies)
    pub anchor_a: Vector3<f32>, // attachment point on chassis (local space)
    pub anchor_b: Vector3<f32>, // attachment point on wheel (local space)
    pub rest_length: f32,
    pub stiffness: f32, // spring constant (N/m)
    pub damping: f32,   // damping coefficient (N·s/m)
    pub max_force: f32, // maximum force the spring can apply
}

impl SpringConstraint {
    /// Validates that all physical quantities are finite (not NaN or Inf)
    pub fn validate_state(&self) -> bool {
        self.rest_length.is_finite()
            && self.stiffness.is_finite()
            && self.damping.is_finite()
            && self.max_force.is_finite()
            && self.anchor_a.x.is_finite()
            && self.anchor_a.y.is_finite()
            && self.anchor_a.z.is_finite()
            && self.anchor_b.x.is_finite()
            && self.anchor_b.y.is_finite()
            && self.anchor_b.z.is_finite()
    }

    /// Resets constraint to a safe state when invalid values are detected
    pub fn reset_to_safe_state(&mut self) {
        if !self.rest_length.is_finite() || self.rest_length <= 0.0 {
            self.rest_length = 0.5;
        }
        if !self.stiffness.is_finite() || self.stiffness <= 0.0 {
            self.stiffness = 75000.0;
        }
        if !self.damping.is_finite() || self.damping < 0.0 {
            self.damping = 5000.0;
        }
        if !self.max_force.is_finite() || self.max_force <= 0.0 {
            self.max_force = 50000.0;
        }
        if !self.anchor_a.x.is_finite() || !self.anchor_a.y.is_finite() || !self.anchor_a.z.is_finite() {
            self.anchor_a = Vector3::new(0.0, 0.0, 0.0);
        }
        if !self.anchor_b.x.is_finite() || !self.anchor_b.y.is_finite() || !self.anchor_b.z.is_finite() {
            self.anchor_b = Vector3::new(0.0, -1.0, 0.0);
        }
    }

    pub fn new(
        body_a: usize,
        body_b: usize,
        anchor_a: Vector3<f32>,
        anchor_b: Vector3<f32>,
        rest_length: f32,
        stiffness: f32,
        damping: f32,
    ) -> Self {
        let mut constraint = Self {
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            rest_length,
            stiffness,
            damping,
            max_force: 50000.0, // Default max force
        };
        
        // Validate initial state
        if !constraint.validate_state() {
            tracing::warn!(target: "physics", "SpringConstraint created with invalid state, resetting to safe values");
            constraint.reset_to_safe_state();
        }
        
        constraint
    }

    /// Calculate the current length of the spring with NaN protection
    pub fn current_length(&self, world: &PhysicsWorld) -> f32 {
        if let (Some(body_a), Some(body_b)) = (
            world.rigid_bodies.get_by_index(self.body_a),
            world.rigid_bodies.get_by_index(self.body_b),
        ) {
            // Transform local anchors to world space
            let world_anchor_a = body_a.position + body_a.rotation * self.anchor_a;
            let world_anchor_b = body_b.position + body_b.rotation * self.anchor_b;

            let length = (world_anchor_b - world_anchor_a).norm();
            
            // Protect against NaN
            if !length.is_finite() {
                tracing::warn!(target: "physics", "NaN detected in spring current_length, returning rest_length");
                return self.rest_length;
            }
            
            length
        } else {
            self.rest_length
        }
    }

    /// Calculate the velocity along the spring axis with NaN protection
    pub fn relative_velocity(&self, world: &PhysicsWorld) -> f32 {
        if let (Some(body_a), Some(body_b)) = (
            world.rigid_bodies.get_by_index(self.body_a),
            world.rigid_bodies.get_by_index(self.body_b),
        ) {
            let world_anchor_a = body_a.position + body_a.rotation * self.anchor_a;
            let world_anchor_b = body_b.position + body_b.rotation * self.anchor_b;

            let axis = (world_anchor_b - world_anchor_a).normalize();

            // Get velocities at anchor points
            let vel_a = body_a.velocity
                + body_a
                    .angular_velocity
                    .cross(&(world_anchor_a - body_a.position));
            let vel_b = body_b.velocity
                + body_b
                    .angular_velocity
                    .cross(&(world_anchor_b - body_b.position));

            let relative_vel = vel_b - vel_a;
            let result = relative_vel.dot(&axis);
            
            // Protect against NaN
            if !result.is_finite() {
                tracing::warn!(target: "physics", "NaN detected in spring relative_velocity, returning 0.0");
                return 0.0;
            }
            
            result
        } else {
            0.0
        }
    }
}

/// Raycast-based suspension (simpler alternative for alpha)
/// Used when wheels are not separate rigid bodies
#[derive(Debug, Clone)]
pub struct RaycastSuspension {
    pub chassis_body: usize,
    pub wheel_position_local: Vector3<f32>, // Position relative to chassis
    pub wheel_radius: f32,
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub max_travel: f32,
}

impl RaycastSuspension {
    /// Validates that all physical quantities are finite
    pub fn validate_state(&self) -> bool {
        self.wheel_radius.is_finite()
            && self.rest_length.is_finite()
            && self.stiffness.is_finite()
            && self.damping.is_finite()
            && self.max_travel.is_finite()
            && self.wheel_position_local.x.is_finite()
            && self.wheel_position_local.y.is_finite()
            && self.wheel_position_local.z.is_finite()
    }

    /// Resets to safe state when invalid values detected
    pub fn reset_to_safe_state(&mut self) {
        if !self.wheel_radius.is_finite() || self.wheel_radius <= 0.0 {
            self.wheel_radius = 0.5;
        }
        if !self.rest_length.is_finite() || self.rest_length <= 0.0 {
            self.rest_length = 0.45;
        }
        if !self.stiffness.is_finite() || self.stiffness <= 0.0 {
            self.stiffness = 75000.0;
        }
        if !self.damping.is_finite() || self.damping < 0.0 {
            self.damping = 5000.0;
        }
        if !self.max_travel.is_finite() || self.max_travel <= 0.0 {
            self.max_travel = self.rest_length * 0.8;
        }
        if !self.wheel_position_local.x.is_finite()
            || !self.wheel_position_local.y.is_finite()
            || !self.wheel_position_local.z.is_finite()
        {
            self.wheel_position_local = Vector3::new(-1.0, -0.5, 1.5);
        }
    }

    pub fn new(
        chassis_body: usize,
        wheel_position_local: Vector3<f32>,
        wheel_radius: f32,
        rest_length: f32,
        stiffness: f32,
        damping: f32,
    ) -> Self {
        let mut suspension = Self {
            chassis_body,
            wheel_position_local,
            wheel_radius,
            rest_length,
            stiffness,
            damping,
            max_travel: rest_length * 0.8,
        };

        // Validate initial state
        if !suspension.validate_state() {
            tracing::warn!(target: "physics", "RaycastSuspension created with invalid state, resetting");
            suspension.reset_to_safe_state();
        }

        suspension
    }

    /// Get the world position of the wheel attachment point with NaN protection
    pub fn get_attachment_point_world(&self, world: &PhysicsWorld) -> Option<Vector3<f32>> {
        world
            .rigid_bodies
            .get_by_index(self.chassis_body)
            .map(|body| {
                let point = body.position + body.rotation * self.wheel_position_local;
                if !point.x.is_finite() || !point.y.is_finite() || !point.z.is_finite() {
                    tracing::warn!(target: "physics", "NaN in suspension attachment point, returning safe value");
                    body.position
                } else {
                    point
                }
            })
    }

    /// Calculate suspension compression with NaN protection
    pub fn compression(&self, ground_height: f32, attachment_point: Vector3<f32>) -> f32 {
        // Validate inputs
        if !ground_height.is_finite() || !attachment_point.y.is_finite() || !self.rest_length.is_finite() || self.rest_length <= 0.0 {
            tracing::warn!(target: "physics", "Invalid inputs to suspension compression, returning 0.0");
            return 0.0;
        }

        let current_length = attachment_point.y - ground_height - self.wheel_radius;
        
        if !current_length.is_finite() {
            tracing::warn!(target: "physics", "NaN in suspension current_length, returning 0.0");
            return 0.0;
        }

        let compression = 1.0 - (current_length / self.rest_length);
        compression.clamp(0.0, 1.0)
    }

    /// Calculate spring force with NaN protection
    pub fn spring_force(&self, compression: f32, velocity: f32) -> f32 {
        // Validate inputs
        if !compression.is_finite() || !velocity.is_finite() {
            tracing::warn!(target: "physics", "Invalid inputs to spring_force, returning 0.0");
            return 0.0;
        }

        if !self.stiffness.is_finite() || !self.rest_length.is_finite() {
            tracing::warn!(target: "physics", "Invalid suspension parameters in spring_force, returning 0.0");
            return 0.0;
        }

        let spring_force = self.stiffness * compression * self.rest_length;
        let damping_force = self.damping * velocity;
        let total = spring_force + damping_force;
        
        if !total.is_finite() {
            tracing::warn!(target: "physics", "NaN in spring force calculation, returning 0.0");
            return 0.0;
        }

        let max_force = self.max_force();
        total.clamp(-max_force, max_force)
    }

    fn max_force(&self) -> f32 {
        let force = self.stiffness * self.max_travel;
        if !force.is_finite() {
            tracing::warn!(target: "physics", "NaN in max_force calculation, returning default");
            return 50000.0;
        }
        force
    }
}

impl PhysicsWorld {
    /// Add a spring constraint to the physics world
    pub fn add_spring_constraint(&mut self, constraint: SpringConstraint) -> usize {
        self.spring_constraints.push(constraint);
        self.spring_constraints.len() - 1
    }

    /// Solve all spring constraints
    pub fn solve_spring_constraints(&mut self, _dt: f32) {
        let mut constraints_to_apply: Vec<(usize, Vector3<f32>)> = Vec::new();

        for constraint in &self.spring_constraints {
            if let (Some(body_a), Some(body_b)) = (
                self.rigid_bodies.get_by_index(constraint.body_a),
                self.rigid_bodies.get_by_index(constraint.body_b),
            ) {
                if body_a.is_static && body_b.is_static {
                    continue;
                }

                // Get world space anchor positions
                let world_anchor_a = body_a.position + body_a.rotation * constraint.anchor_a;
                let world_anchor_b = body_b.position + body_b.rotation * constraint.anchor_b;

                // Calculate spring vector
                let delta = world_anchor_b - world_anchor_a;
                let distance = delta.norm();

                if distance < 0.001 {
                    continue;
                }

                let direction = delta.normalize();

                // Calculate spring force (Hooke's law + damping)
                let displacement = distance - constraint.rest_length;
                let spring_force_magnitude = -constraint.stiffness * displacement;

                // Calculate relative velocity along spring axis
                let vel_a = body_a.velocity
                    + body_a
                        .angular_velocity
                        .cross(&(world_anchor_a - body_a.position));
                let vel_b = body_b.velocity
                    + body_b
                        .angular_velocity
                        .cross(&(world_anchor_b - body_b.position));
                let relative_velocity = (vel_b - vel_a).dot(&direction);
                let damping_force_magnitude = -constraint.damping * relative_velocity;

                let total_force = spring_force_magnitude + damping_force_magnitude;
                let force_magnitude =
                    total_force.clamp(-constraint.max_force, constraint.max_force);

                let force = direction * force_magnitude;

                // Apply forces to bodies
                if !body_a.is_static {
                    constraints_to_apply.push((constraint.body_a, -force));
                }
                if !body_b.is_static {
                    constraints_to_apply.push((constraint.body_b, force));
                }
            }
        }

        // Apply accumulated forces
        for (body_idx, force) in constraints_to_apply {
            if let Some(body) = self.rigid_bodies.get_mut_by_index(body_idx) {
                if !body.is_static {
                    body.apply_force(force);
                }
            }
        }
    }

    /// Add a raycast suspension
    pub fn add_raycast_suspension(&mut self, suspension: RaycastSuspension) -> usize {
        self.raycast_suspensions.push(suspension);
        self.raycast_suspensions.len() - 1
    }
}

// Extend PhysicsWorld with constraint storage

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::physics_module::{PhysicsConfig, RigidBody};

    #[test]
    fn test_spring_constraint_creation() {
        let constraint = SpringConstraint::new(
            0,
            1,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            0.5,
            75000.0,
            5000.0,
        );

        assert_eq!(constraint.body_a, 0);
        assert_eq!(constraint.body_b, 1);
        assert_eq!(constraint.rest_length, 0.5);
        assert_eq!(constraint.stiffness, 75000.0);
    }

    #[test]
    fn test_raycast_suspension() {
        let suspension = RaycastSuspension::new(
            0,
            Vector3::new(-1.0, -0.5, 1.5),
            0.53,
            0.45,
            75000.0,
            5000.0,
        );

        assert_eq!(suspension.wheel_radius, 0.53);
        assert_eq!(suspension.rest_length, 0.45);
    }

    #[test]
    fn test_spring_constraint_validates_state() {
        // Test with valid values
        let constraint = SpringConstraint::new(
            0, 1,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            0.5, 75000.0, 5000.0,
        );
        assert!(constraint.validate_state());

        // Test with NaN - should reset to safe state
        let mut bad_constraint = SpringConstraint {
            body_a: 0, body_b: 1,
            anchor_a: Vector3::new(f32::NAN, 0.0, 0.0),
            anchor_b: Vector3::new(0.0, -1.0, 0.0),
            rest_length: 0.5,
            stiffness: 75000.0,
            damping: 5000.0,
            max_force: 50000.0,
        };
        assert!(!bad_constraint.validate_state());
        bad_constraint.reset_to_safe_state();
        assert!(bad_constraint.validate_state());
        assert!(bad_constraint.anchor_a.x.is_finite());
    }

    #[test]
    fn test_raycast_suspension_validates_state() {
        // Test with valid values
        let suspension = RaycastSuspension::new(
            0, Vector3::new(-1.0, -0.5, 1.5),
            0.53, 0.45, 75000.0, 5000.0,
        );
        assert!(suspension.validate_state());

        // Test with Inf - should reset
        let mut bad_suspension = RaycastSuspension {
            chassis_body: 0,
            wheel_position_local: Vector3::new(-1.0, -0.5, 1.5),
            wheel_radius: f32::INFINITY,
            rest_length: 0.45,
            stiffness: 75000.0,
            damping: 5000.0,
            max_travel: 0.36,
        };
        assert!(!bad_suspension.validate_state());
        bad_suspension.reset_to_safe_state();
        assert!(bad_suspension.validate_state());
        assert!(bad_suspension.wheel_radius.is_finite());
    }

    #[test]
    fn test_spring_force_with_nan_protection() {
        let suspension = RaycastSuspension::new(
            0, Vector3::new(-1.0, -0.5, 1.5),
            0.53, 0.45, 75000.0, 5000.0,
        );

        // Normal case
        let force = suspension.spring_force(0.5, 1.0);
        assert!(force.is_finite());

        // NaN compression
        let nan_force = suspension.spring_force(f32::NAN, 1.0);
        assert_eq!(nan_force, 0.0);

        // NaN velocity
        let nan_force2 = suspension.spring_force(0.5, f32::NAN);
        assert_eq!(nan_force2, 0.0);
    }
}
