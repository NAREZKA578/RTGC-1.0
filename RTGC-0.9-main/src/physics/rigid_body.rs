use nalgebra::{Isometry3, Matrix3, Point3, UnitQuaternion, Vector3};
use tracing;

// Collision layers (B4)
pub const LAYER_WORLD: u32 = 0b0001;
pub const LAYER_VEHICLE: u32 = 0b0010;
pub const LAYER_CARGO: u32 = 0b0100;
pub const LAYER_TRIGGER: u32 = 0b1000;
pub const LAYER_PLAYER: u32 = 0b10000;
// Interactable object layers (aliases for LAYER_CARGO and LAYER_VEHICLE)
pub const LAYER_INTERACTABLE_DOOR: u32 = LAYER_WORLD;
pub const LAYER_INTERACTABLE_VEHICLE: u32 = LAYER_VEHICLE;
pub const LAYER_INTERACTABLE_OBJECT: u32 = LAYER_CARGO;

/// Contact event for sound and effects (B6)
#[derive(Debug, Clone)]
pub struct ContactEvent {
    pub body_a: usize,
    pub body_b: usize,
    pub impact_velocity: f32,
    pub contact_point: Vector3<f32>,
    pub normal: Vector3<f32>,
}

/// Physics profiling statistics (C4)
#[derive(Debug, Clone, Default)]
pub struct PhysicsStats {
    pub active_bodies: usize,
    pub sleeping_bodies: usize,
    pub broadphase_pairs: usize,
    pub contacts_resolved: usize,
    pub step_time_us: u64,
    pub collision_events: usize,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Sphere {
        radius: f32,
    },
    Box {
        half_extents: Vector3<f32>,
    },
    Capsule {
        radius: f32,
        height: f32,
    },
    Terrain {
        height_map: Vec<Vec<f32>>,
        scale: Vector3<f32>,
    },
    Mesh {
        vertices: Vec<Vector3<f32>>,
        indices: Vec<u32>,
    },
}

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Self {
        Self { origin, direction }
    }
}

#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub point: Point3<f32>,
    pub normal: Vector3<f32>,
    pub distance: f32,
    pub body_index: usize,
    pub layer: u32,
    pub object_id: u32,
}

#[derive(Debug, Clone)]
pub struct RigidBody {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
    pub mass: f32,
    pub inverse_mass: f32,
    pub inertia_tensor: Matrix3<f32>,
    pub inverse_inertia_tensor: Matrix3<f32>,
    pub restitution: f32,
    pub friction: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub shape: Shape,
    pub is_static: bool,
    pub bounds: Aabb,
    pub forces: Vector3<f32>,
    pub torques: Vector3<f32>,
    pub center_of_mass: Vector3<f32>,
    pub drag_coefficient: f32,
    pub lift_coefficient: f32,
    pub reference_area: f32,
    // Sleep system fields (A3)
    pub idle_timer: f32,
    pub is_sleeping: bool,
    // Collision layers (B4)
    pub collision_layer: u32,
    pub collision_mask: u32,
    // Trigger volume (B5)
    pub is_trigger: bool,
    // CCD flag (B3)
    pub enable_ccd: bool,
}

impl RigidBody {
    pub fn new_sphere(position: Vector3<f32>, mass: f32, radius: f32) -> Self {
        let inverse_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };

        // Inertia tensor for a sphere: (2/5) * m * r^2
        let inertia_scalar = (2.0 / 5.0) * mass * radius * radius;
        let inertia_tensor = if mass > 0.0 {
            Matrix3::new(
                inertia_scalar, 0.0, 0.0, 0.0, inertia_scalar, 0.0, 0.0, 0.0, inertia_scalar,
            )
        } else {
            Matrix3::zeros()
        };

        let inverse_inertia_tensor = if mass > 0.0 && inertia_scalar != 0.0 {
            inertia_tensor.try_inverse().unwrap_or(Matrix3::zeros())
        } else {
            Matrix3::zeros()
        };

        Self {
            position,
            rotation: UnitQuaternion::identity(),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            mass,
            inverse_mass,
            inertia_tensor,
            inverse_inertia_tensor,
            restitution: 0.5,
            friction: 0.1,
            linear_damping: 0.99,
            angular_damping: 0.99,
            shape: Shape::Sphere { radius },
            is_static: mass <= 0.0,
            bounds: Aabb::from_shape_and_transform(
                &Shape::Sphere { radius },
                &Isometry3::from_parts(position.into(), UnitQuaternion::identity()),
            ),
            forces: Vector3::zeros(),
            torques: Vector3::zeros(),
            center_of_mass: Vector3::zeros(),
            drag_coefficient: 0.47,
            lift_coefficient: 0.0,
            reference_area: std::f32::consts::PI * radius * radius,
            // Sleep system (A3)
            idle_timer: 0.0,
            is_sleeping: false,
            // Collision layers (B4) - default to world layer
            collision_layer: LAYER_WORLD,
            collision_mask: LAYER_VEHICLE | LAYER_CARGO | LAYER_TRIGGER,
            // Trigger volume (B5)
            is_trigger: false,
            // CCD flag (B3)
            enable_ccd: false,
        }
    }

    pub fn new_box(position: Vector3<f32>, mass: f32, half_extents: Vector3<f32>) -> Self {
        let inverse_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };

        // Inertia tensor for a box:
        // Ixx = (1/12) * m * (h^2 + d^2)
        // Iyy = (1/12) * m * (w^2 + d^2)
        // Izz = (1/12) * m * (w^2 + h^2)
        let w = half_extents.x * 2.0;
        let h = half_extents.y * 2.0;
        let d = half_extents.z * 2.0;

        let inertia_tensor = if mass > 0.0 {
            Matrix3::new(
                (1.0 / 12.0) * mass * (h * h + d * d),
                0.0,
                0.0,
                0.0,
                (1.0 / 12.0) * mass * (w * w + d * d),
                0.0,
                0.0,
                0.0,
                (1.0 / 12.0) * mass * (w * w + h * h),
            )
        } else {
            Matrix3::zeros()
        };

        let inverse_inertia_tensor = if mass > 0.0 {
            inertia_tensor.try_inverse().unwrap_or(Matrix3::zeros())
        } else {
            Matrix3::zeros()
        };

        Self {
            position,
            rotation: UnitQuaternion::identity(),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            mass,
            inverse_mass,
            inertia_tensor,
            inverse_inertia_tensor,
            restitution: 0.5,
            friction: 0.1,
            linear_damping: 0.99,
            angular_damping: 0.99,
            shape: Shape::Box { half_extents },
            is_static: mass <= 0.0,
            bounds: Aabb::from_shape_and_transform(
                &Shape::Box { half_extents },
                &Isometry3::from_parts(position.into(), UnitQuaternion::identity()),
            ),
            forces: Vector3::zeros(),
            torques: Vector3::zeros(),
            center_of_mass: Vector3::zeros(),
            drag_coefficient: 1.05,
            lift_coefficient: 0.0,
            reference_area: 4.0 * half_extents.x * half_extents.y,
            // Sleep system (A3)
            idle_timer: 0.0,
            is_sleeping: false,
            // Collision layers (B4)
            collision_layer: LAYER_WORLD,
            collision_mask: LAYER_VEHICLE | LAYER_CARGO | LAYER_TRIGGER,
            // Trigger volume (B5)
            is_trigger: false,
            // CCD flag (B3)
            enable_ccd: false,
        }
    }

    pub fn new_capsule(position: Vector3<f32>, mass: f32, radius: f32, height: f32) -> Self {
        let inverse_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };

        // Approximate inertia tensor for a capsule
        // A capsule can be approximated as a cylinder with two hemispheres
        let cylinder_volume = std::f32::consts::PI * radius * radius * height;
        let hemisphere_volume = (2.0 / 3.0) * std::f32::consts::PI * radius * radius * radius;
        let total_volume = cylinder_volume + 2.0 * hemisphere_volume;
        let density = if total_volume > 0.0 {
            mass / total_volume
        } else {
            0.0
        };

        let cylinder_mass = density * cylinder_volume;
        let hemisphere_mass = density * hemisphere_volume;

        // Moment of inertia for cylinder around z-axis: (1/2) * m * r^2
        // Moment of inertia for cylinder around x,y-axis: (1/12) * m * h^2 + (1/4) * m * r^2
        // Moment of inertia for hemisphere around z-axis: (2/5) * m * r^2
        // Moment of inertia for hemisphere around x,y-axis: (2/5) * m * r^2 + (3/8)^2 * m * h^2 (parallel axis theorem)

        let cylinder_i_z = (1.0 / 2.0) * cylinder_mass * radius * radius;
        let cylinder_i_xy = (1.0 / 12.0) * cylinder_mass * height * height
            + (1.0 / 4.0) * cylinder_mass * radius * radius;

        let hemisphere_i_z = (2.0 / 5.0) * hemisphere_mass * radius * radius;
        let hemisphere_i_xy = (2.0 / 5.0) * hemisphere_mass * radius * radius;

        let total_i_z = cylinder_i_z + 2.0 * hemisphere_i_z;
        let total_i_xy =
            cylinder_i_xy + 2.0 * (hemisphere_i_xy + hemisphere_mass * (height / 2.0).powi(2));

        let inertia_tensor = if mass > 0.0 {
            Matrix3::new(
                total_i_xy, 0.0, 0.0, 0.0, total_i_xy, 0.0, 0.0, 0.0, total_i_z,
            )
        } else {
            Matrix3::zeros()
        };

        let inverse_inertia_tensor = if mass > 0.0 {
            inertia_tensor.try_inverse().unwrap_or(Matrix3::zeros())
        } else {
            Matrix3::zeros()
        };

        Self {
            position,
            rotation: UnitQuaternion::identity(),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            mass,
            inverse_mass,
            inertia_tensor,
            inverse_inertia_tensor,
            restitution: 0.5,
            friction: 0.1,
            linear_damping: 0.99,
            angular_damping: 0.99,
            shape: Shape::Capsule { radius, height },
            is_static: mass <= 0.0,
            bounds: Aabb::from_shape_and_transform(
                &Shape::Capsule { radius, height },
                &Isometry3::from_parts(position.into(), UnitQuaternion::identity()),
            ),
            forces: Vector3::zeros(),
            torques: Vector3::zeros(),
            center_of_mass: Vector3::zeros(),
            drag_coefficient: 0.82,
            lift_coefficient: 0.0,
            reference_area: std::f32::consts::PI * radius * radius,
            // Sleep system (A3)
            idle_timer: 0.0,
            is_sleeping: false,
            // Collision layers (B4)
            collision_layer: LAYER_VEHICLE,
            collision_mask: LAYER_WORLD | LAYER_CARGO,
            // Trigger volume (B5)
            is_trigger: false,
            // CCD flag (B3) - enable for vehicles by default
            enable_ccd: true,
        }
    }

    pub fn new_terrain(
        position: Vector3<f32>,
        height_map: Vec<Vec<f32>>,
        scale: Vector3<f32>,
    ) -> Self {
        // Вычисляем реальные bounds terrain
        let rows = height_map.len();
        let cols = if rows > 0 { height_map[0].len() } else { 0 };
        let terrain_width = if cols > 0 {
            cols as f32 * scale.x
        } else {
            scale.x
        };
        let terrain_depth = if rows > 0 {
            rows as f32 * scale.z
        } else {
            scale.z
        };

        // Находим min/max высоты
        let (min_h, max_h) = if rows > 0 && cols > 0 {
            let mut min_h = f32::MAX;
            let mut max_h = f32::MIN;
            for row in &height_map {
                for &h in row {
                    if h < min_h {
                        min_h = h;
                    }
                    if h > max_h {
                        max_h = h;
                    }
                }
            }
            (min_h * scale.y, max_h * scale.y)
        } else {
            (0.0, 0.0)
        };

        let half_size = Vector3::new(
            terrain_width / 2.0,
            (max_h - min_h).abs().max(1.0) / 2.0,
            terrain_depth / 2.0,
        );
        let center = position + Vector3::new(0.0, (min_h + max_h) * scale.y / 2.0, 0.0);
        let bounds = Aabb {
            min: center - half_size,
            max: center + half_size,
        };

        Self {
            position,
            rotation: UnitQuaternion::identity(),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            mass: 0.0,
            inverse_mass: 0.0,
            inertia_tensor: Matrix3::zeros(),
            inverse_inertia_tensor: Matrix3::zeros(),
            restitution: 0.3,
            friction: 0.7,
            linear_damping: 1.0,
            angular_damping: 1.0,
            shape: Shape::Terrain { height_map, scale },
            is_static: true,
            bounds,
            forces: Vector3::zeros(),
            torques: Vector3::zeros(),
            center_of_mass: Vector3::zeros(),
            drag_coefficient: 0.0,
            lift_coefficient: 0.0,
            reference_area: 0.0,
            // Sleep system (A3) - terrain never sleeps
            idle_timer: 0.0,
            is_sleeping: false,
            // Collision layers (B4) - world layer
            collision_layer: LAYER_WORLD,
            collision_mask: LAYER_VEHICLE | LAYER_CARGO,
            // Trigger volume (B5)
            is_trigger: false,
            // CCD flag (B3) - terrain doesn't need CCD
            enable_ccd: false,
        }
    }

    pub fn new_mesh(
        position: Vector3<f32>,
        mass: f32,
        vertices: Vec<Vector3<f32>>,
        indices: Vec<u32>,
    ) -> Self {
        let inverse_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };

        // For mesh objects, we'll calculate a rough approximation of the inertia tensor
        // based on the bounding box of the mesh
        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

        for vertex in &vertices {
            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);

            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);
        }

        let size = max - min;
        let half_extents = size / 2.0;

        // Calculate inertia tensor as if it were a box
        let w = half_extents.x * 2.0;
        let h = half_extents.y * 2.0;
        let d = half_extents.z * 2.0;

        let inertia_tensor = if mass > 0.0 {
            Matrix3::new(
                (1.0 / 12.0) * mass * (h * h + d * d),
                0.0,
                0.0,
                0.0,
                (1.0 / 12.0) * mass * (w * w + d * d),
                0.0,
                0.0,
                0.0,
                (1.0 / 12.0) * mass * (w * w + h * h),
            )
        } else {
            Matrix3::zeros()
        };

        let inverse_inertia_tensor = if mass > 0.0 {
            inertia_tensor.try_inverse().unwrap_or(Matrix3::zeros())
        } else {
            Matrix3::zeros()
        };

        Self {
            position,
            rotation: UnitQuaternion::identity(),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            mass,
            inverse_mass,
            inertia_tensor,
            inverse_inertia_tensor,
            restitution: 0.4,
            friction: 0.6,
            linear_damping: 0.99,
            angular_damping: 0.99,
            shape: Shape::Mesh { vertices, indices },
            is_static: mass <= 0.0,
            bounds: Aabb::from_shape_and_transform(
                &Shape::Mesh {
                    vertices: vec![],
                    indices: vec![],
                },
                &Isometry3::from_parts(position.into(), UnitQuaternion::identity()),
            ),
            forces: Vector3::zeros(),
            torques: Vector3::zeros(),
            center_of_mass: Vector3::zeros(),
            drag_coefficient: 1.2,
            lift_coefficient: 0.0,
            reference_area: size.x * size.y,
            // Sleep system (A3)
            idle_timer: 0.0,
            is_sleeping: false,
            // Collision layers (B4)
            collision_layer: LAYER_WORLD,
            collision_mask: LAYER_VEHICLE | LAYER_CARGO,
            // Trigger volume (B5)
            is_trigger: false,
            // CCD flag (B3)
            enable_ccd: false,
        }
    }

    pub fn apply_force(&mut self, force: Vector3<f32>) {
        if !self.is_static {
            // Validate force to prevent NaN propagation
            if force.x.is_finite() && force.y.is_finite() && force.z.is_finite() {
                self.forces += force;
            } else {
                tracing::warn!(target: "physics", "NaN force applied to rigid body, ignoring");
            }
        }
    }

    pub fn apply_force_at_point(&mut self, force: Vector3<f32>, point: Vector3<f32>) {
        if !self.is_static {
            // Validate inputs
            if !force.x.is_finite() || !force.y.is_finite() || !force.z.is_finite() {
                tracing::warn!(target: "physics", "NaN force in apply_force_at_point, ignoring");
                return;
            }
            if !point.x.is_finite() || !point.y.is_finite() || !point.z.is_finite() {
                tracing::warn!(target: "physics", "NaN point in apply_force_at_point, using position");
                self.forces += force;
                return;
            }

            self.forces += force;
            let r = point - self.position - self.center_of_mass;
            let torque = r.cross(&force);
            self.torques += torque;
        }
    }

    pub fn apply_impulse(&mut self, impulse: Vector3<f32>) {
        if !self.is_static {
            // Validate impulse
            if impulse.x.is_finite() && impulse.y.is_finite() && impulse.z.is_finite() {
                self.velocity += impulse * self.inverse_mass;
            } else {
                tracing::warn!(target: "physics", "NaN impulse applied, ignoring");
            }
        }
    }

    pub fn apply_impulse_at_point(&mut self, impulse: Vector3<f32>, point: Vector3<f32>) {
        if !self.is_static {
            // Validate inputs
            if !impulse.x.is_finite() || !impulse.y.is_finite() || !impulse.z.is_finite() {
                tracing::warn!(target: "physics", "NaN impulse in apply_impulse_at_point, ignoring");
                return;
            }

            self.velocity += impulse * self.inverse_mass;

            if point.x.is_finite() && point.y.is_finite() && point.z.is_finite() {
                let r = point - self.position - self.center_of_mass;
                let angular_impulse = r.cross(&impulse);
                self.angular_velocity += self.inverse_inertia_tensor * angular_impulse;
            }
        }
    }

    pub fn apply_torque(&mut self, torque: Vector3<f32>) {
        if !self.is_static {
            // Validate torque
            if torque.x.is_finite() && torque.y.is_finite() && torque.z.is_finite() {
                self.torques += torque;
            } else {
                tracing::warn!(target: "physics", "NaN torque applied, ignoring");
            }
        }
    }

    pub fn apply_angular_impulse(&mut self, impulse: Vector3<f32>) {
        if !self.is_static {
            // Validate impulse
            if impulse.x.is_finite() && impulse.y.is_finite() && impulse.z.is_finite() {
                self.angular_velocity += self.inverse_inertia_tensor * impulse;
            } else {
                tracing::warn!(target: "physics", "NaN angular impulse applied, ignoring");
            }
        }
    }

    pub fn set_center_of_mass(&mut self, com: Vector3<f32>) {
        self.center_of_mass = com;
    }

    pub fn get_center_of_mass_world(&self) -> Vector3<f32> {
        self.position + self.rotation.transform_vector(&self.center_of_mass)
    }

    pub fn clear_forces(&mut self) {
        self.forces = Vector3::zeros();
        self.torques = Vector3::zeros();
    }

    pub fn update(&mut self, dt: f32) {
        if !self.is_static {
            // Apply aerodynamic forces (drag and lift)
            let air_density = 1.225; // kg/m^3 at sea level
            let velocity_magnitude = self.velocity.magnitude();

            if velocity_magnitude > 0.001 {
                // Drag force: F_drag = -0.5 * rho * v^2 * Cd * A * direction
                let drag_magnitude = 0.5
                    * air_density
                    * velocity_magnitude
                    * velocity_magnitude
                    * self.drag_coefficient
                    * self.reference_area;
                let drag_direction = -self.velocity.normalize();
                let drag_force = drag_direction * drag_magnitude;
                self.forces += drag_force;

                // Lift force: F_lift = 0.5 * rho * v^2 * Cl * A * up_direction
                if self.lift_coefficient != 0.0 {
                    let lift_magnitude = 0.5
                        * air_density
                        * velocity_magnitude
                        * velocity_magnitude
                        * self.lift_coefficient
                        * self.reference_area;
                    let up_direction = self.rotation * Vector3::y();
                    let lift_force = up_direction * lift_magnitude;
                    self.forces += lift_force;
                }
            }

            // SYMPLECTIC EULER INTEGRATOR (more stable than explicit Euler)
            // Step 1: Update velocity using forces (semi-implicit)
            let linear_acceleration = self.forces * self.inverse_mass;
            self.velocity += linear_acceleration * dt;

            // Transform inertia tensor to world space for correct torque application
            let rotation_matrix = self.rotation.to_rotation_matrix();
            let world_inertia_tensor =
                rotation_matrix * self.inertia_tensor * rotation_matrix.transpose();
            let world_inverse_inertia_tensor = world_inertia_tensor
                .try_inverse()
                .unwrap_or(self.inverse_inertia_tensor);

            // Step 2: Update angular velocity using torques
            let angular_acceleration = world_inverse_inertia_tensor * self.torques;
            self.angular_velocity += angular_acceleration * dt;

            // Step 3: Update position using NEW velocity (symplectic property)
            self.position += self.velocity * dt;

            // Step 4: Update orientation using NEW angular velocity
            let angular_velocity_quat = nalgebra::Quaternion::new(
                0.0,
                self.angular_velocity.x,
                self.angular_velocity.y,
                self.angular_velocity.z,
            );
            let rotation_quat = nalgebra::Quaternion::from_parts(
                self.rotation.scalar(),
                self.rotation.vector().clone(),
            );
            let dq = 0.5 * angular_velocity_quat * rotation_quat;
            let new_rotation = UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(
                rotation_quat.w + dq.w * dt,
                rotation_quat.i + dq.i * dt,
                rotation_quat.j + dq.j * dt,
                rotation_quat.k + dq.k * dt,
            ));
            self.rotation = new_rotation;
            self.rotation.renormalize();

            // Apply damping (after integration for stability)
            self.velocity *= self.linear_damping;
            self.angular_velocity *= self.angular_damping;
        }

        // Update bounding box
        self.bounds = Aabb::from_shape_and_transform(&self.shape, &self.get_world_transform());
    }

    pub fn get_world_transform(&self) -> Isometry3<f32> {
        Isometry3::from_parts(self.position.into(), self.rotation)
    }

    /// Get velocity at a specific point on the rigid body
    pub fn get_velocity_at_point(&self, point: Vector3<f32>) -> Vector3<f32> {
        let r = point - self.position;
        self.velocity + self.angular_velocity.cross(&r)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Aabb {
    pub fn new(min: Vector3<f32>, max: Vector3<f32>) -> Self {
        Self { min, max }
    }

    pub fn from_shape_and_transform(shape: &Shape, transform: &Isometry3<f32>) -> Self {
        match shape {
            Shape::Sphere { radius } => {
                let center = transform.translation.vector;
                let half_size = Vector3::new(*radius, *radius, *radius);

                Aabb {
                    min: center - half_size,
                    max: center + half_size,
                }
            }
            Shape::Box { half_extents } => {
                let center = transform.translation.vector;

                Aabb {
                    min: center - half_extents,
                    max: center + half_extents,
                }
            }
            Shape::Capsule { radius, height } => {
                let center = transform.translation.vector;
                let half_height = height / 2.0;
                // Капсула: цилиндр высотой `height` с полусферами на концах
                // AABB: X и Z = radius, Y = half_height + radius (полная высота включая полусферы)
                let half_size = Vector3::new(*radius, half_height + *radius, *radius);

                Aabb {
                    min: center - half_size,
                    max: center + half_size,
                }
            }
            Shape::Terrain {
                height_map: _,
                scale,
            } => {
                // For terrain, we'll create a large AABB that encompasses the entire terrain
                let center = transform.translation.vector;
                let half_size = Vector3::new(scale.x / 2.0, scale.y / 2.0, scale.z / 2.0);

                Aabb {
                    min: center - half_size,
                    max: center + half_size,
                }
            }
            Shape::Mesh { vertices, .. } => {
                let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
                let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);

                for vertex in vertices {
                    let transformed_vertex = transform.transform_point(&Point3::from(*vertex));
                    let v = transformed_vertex.coords;

                    min.x = min.x.min(v.x);
                    min.y = min.y.min(v.y);
                    min.z = min.z.min(v.z);

                    max.x = max.x.max(v.x);
                    max.y = max.y.max(v.y);
                    max.z = max.z.max(v.z);
                }

                Aabb { min, max }
            }
        }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn contains_point(&self, point: &Vector3<f32>) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    pub fn center(&self) -> Vector3<f32> {
        (self.min + self.max) / 2.0
    }

    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }
}

impl Shape {
    pub fn as_sphere_radius(&self) -> Option<f32> {
        match self {
            Shape::Sphere { radius } => Some(*radius),
            _ => None,
        }
    }

    pub fn as_box_half_extents(&self) -> Option<Vector3<f32>> {
        match self {
            Shape::Box { half_extents } => Some(*half_extents),
            _ => None,
        }
    }

    pub fn as_capsule_dimensions(&self) -> Option<(f32, f32)> {
        match self {
            Shape::Capsule { radius, height } => Some((*radius, *height)),
            _ => None,
        }
    }

    pub fn as_terrain_properties(&self) -> Option<(&Vec<Vec<f32>>, &Vector3<f32>)> {
        match self {
            Shape::Terrain { height_map, scale } => Some((height_map, scale)),
            _ => None,
        }
    }

    pub fn as_mesh_data(&self) -> Option<(&Vec<Vector3<f32>>, &Vec<u32>)> {
        match self {
            Shape::Mesh { vertices, indices } => Some((vertices, indices)),
            _ => None,
        }
    }
}
