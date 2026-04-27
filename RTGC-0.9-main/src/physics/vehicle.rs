//! Physics - Vehicle physics simulation

use crate::physics::physics_module::RigidBody;
use crate::world::SurfaceType;
use nalgebra::{UnitQuaternion, Vector3};

pub const DEFAULT_CHASSIS_WIDTH: f32 = 0.9;
pub const DEFAULT_CHASSIS_HEIGHT: f32 = 0.3;
pub const DEFAULT_CHASSIS_LENGTH: f32 = 2.25;

pub const DEFAULT_WHEEL_FRONT_LEFT: Vector3<f32> = Vector3::new(1.0, -0.5, 0.8);
pub const DEFAULT_WHEEL_FRONT_RIGHT: Vector3<f32> = Vector3::new(1.0, -0.5, -0.8);
pub const DEFAULT_WHEEL_REAR_LEFT: Vector3<f32> = Vector3::new(-1.0, -0.5, 0.8);
pub const DEFAULT_WHEEL_REAR_RIGHT: Vector3<f32> = Vector3::new(-1.0, -0.5, -0.8);

/// Vehicle configuration
#[derive(Debug, Clone)]
pub struct VehicleConfig {
    pub mass: f32,
    pub wheel_count: u8,
    pub wheel_radius: f32,
    pub suspension_stiffness: f32,
    pub suspension_damping: f32,
    pub suspension_rest_length: f32,
    pub max_suspension_travel: f32,
    pub engine_force: f32,
    pub brake_force: f32,
    pub max_steering_angle: f32,
    pub lateral_friction: f32,
    pub longitudinal_friction: f32,
    pub drag_coefficient: f32,
    pub downforce_coefficient: f32,
    /// Differential locks (front/rear)
    pub diff_front_locked: bool,
    pub diff_rear_locked: bool,
    /// Low range transfer case
    pub low_range_enabled: bool,
    pub low_range_ratio: f32,
}

impl Default for VehicleConfig {
    fn default() -> Self {
        Self {
            mass: 1500.0,
            wheel_count: 4,
            wheel_radius: 0.35,
            suspension_stiffness: 35000.0,
            suspension_damping: 4500.0,
            suspension_rest_length: 0.4,
            max_suspension_travel: 0.2,
            engine_force: 5000.0,
            brake_force: 10000.0,
            max_steering_angle: 0.6, // ~35 degrees
            lateral_friction: 1.0,
            longitudinal_friction: 1.0,
            drag_coefficient: 0.3,
            downforce_coefficient: 0.0,
            diff_front_locked: false,
            diff_rear_locked: false,
            low_range_enabled: false,
            low_range_ratio: 2.15, // Typical off-road low range
        }
    }
}

/// Wheel state
#[derive(Debug, Clone)]
pub struct WheelState {
    /// Position relative to vehicle center
    pub local_position: Vector3<f32>,
    /// Current steering angle (radians)
    pub steering_angle: f32,
    /// Suspension compression (0 = rest, positive = compressed)
    pub suspension_compression: f32,
    /// Suspension velocity
    pub suspension_velocity: f32,
    /// Wheel rotation angle
    pub rotation_angle: f32,
    /// Wheel angular velocity
    pub angular_velocity: f32,
    /// Is wheel in contact with ground
    pub is_in_contact: bool,
    /// Contact point world position
    pub contact_point: Option<Vector3<f32>>,
    /// Contact normal
    pub contact_normal: Option<Vector3<f32>>,
}

impl WheelState {
    pub fn new(local_position: Vector3<f32>) -> Self {
        Self {
            local_position,
            steering_angle: 0.0,
            suspension_compression: 0.0,
            suspension_velocity: 0.0,
            rotation_angle: 0.0,
            angular_velocity: 0.0,
            is_in_contact: false,
            contact_point: None,
            contact_normal: None,
        }
    }

    pub fn front_left(_config: &VehicleConfig) -> Self {
        Self::new(DEFAULT_WHEEL_FRONT_LEFT)
    }

    pub fn front_right(_config: &VehicleConfig) -> Self {
        Self::new(DEFAULT_WHEEL_FRONT_RIGHT)
    }

    pub fn rear_left(_config: &VehicleConfig) -> Self {
        Self::new(DEFAULT_WHEEL_REAR_LEFT)
    }

    pub fn rear_right(_config: &VehicleConfig) -> Self {
        Self::new(DEFAULT_WHEEL_REAR_RIGHT)
    }
}

/// Vehicle control inputs
#[derive(Debug, Clone, Copy, Default)]
pub struct VehicleControls {
    /// Throttle input (-1.0 to 1.0, negative for reverse)
    pub throttle: f32,
    /// Brake input (0.0 to 1.0)
    pub brake: f32,
    /// Steering input (-1.0 to 1.0, left to right)
    pub steering: f32,
    /// Handbrake (0.0 to 1.0)
    pub handbrake: f32,
    /// Lock front differential (50/50 torque split)
    pub diff_front_lock: bool,
    /// Lock rear differential (50/50 torque split)
    pub diff_rear_lock: bool,
    /// Enable low range transfer case
    pub low_range: bool,
}

impl VehicleControls {
    pub fn new(throttle: f32, brake: f32, steering: f32, handbrake: f32) -> Self {
        Self {
            throttle: throttle.clamp(-1.0, 1.0),
            brake: brake.clamp(0.0, 1.0),
            steering: steering.clamp(-1.0, 1.0),
            handbrake: handbrake.clamp(0.0, 1.0),
            diff_front_lock: false,
            diff_rear_lock: false,
            low_range: false,
        }
    }

    pub fn new_full(
        throttle: f32,
        brake: f32,
        steering: f32,
        handbrake: f32,
        diff_front: bool,
        diff_rear: bool,
        low_range: bool,
    ) -> Self {
        Self {
            throttle: throttle.clamp(-1.0, 1.0),
            brake: brake.clamp(0.0, 1.0),
            steering: steering.clamp(-1.0, 1.0),
            handbrake: handbrake.clamp(0.0, 1.0),
            diff_front_lock: diff_front,
            diff_rear_lock: diff_rear,
            low_range,
        }
    }
}

/// Simple vehicle physics model
#[derive(Clone, Debug)]
pub struct Vehicle {
    config: VehicleConfig,
    body: RigidBody,
    wheels: Vec<WheelState>,
    controls: VehicleControls,
    /// Center of gravity offset from body origin
    pub cog_offset: Vector3<f32>,
    /// ID тела шасси в PhysicsWorld
    chassis_body_id: Option<usize>,
}

impl Vehicle {
    /// Creates a new vehicle with the given configuration
    pub fn new(config: VehicleConfig) -> Self {
        let chassis_dims = Vector3::new(DEFAULT_CHASSIS_WIDTH, DEFAULT_CHASSIS_HEIGHT, DEFAULT_CHASSIS_LENGTH);
        let body = RigidBody::new_box(Vector3::zeros(), config.mass, chassis_dims);

        let mut wheels = Vec::with_capacity(config.wheel_count as usize);

        // Set up default 4-wheel configuration
        if config.wheel_count >= 4 {
            wheels.push(WheelState::front_left(&config));
            wheels.push(WheelState::front_right(&config));
            wheels.push(WheelState::rear_left(&config));
            wheels.push(WheelState::rear_right(&config));
        }

        Self {
            config,
            body,
            wheels,
            controls: VehicleControls::default(),
            cog_offset: Vector3::new(0.0, 0.0, 0.0),
            chassis_body_id: None,
        }
    }

    /// Sets the vehicle controls
    pub fn set_controls(&mut self, controls: VehicleControls) {
        self.controls = controls;
    }

    /// Gets the current controls
    pub fn get_controls(&self) -> &VehicleControls {
        &self.controls
    }

    /// Set throttle input
    pub fn set_throttle(&mut self, throttle: f32) {
        self.controls.throttle = throttle.clamp(-1.0, 1.0);
    }

    /// Set steering input
    pub fn set_steering(&mut self, steering: f32) {
        self.controls.steering = steering.clamp(-1.0, 1.0);
    }

    /// Set brake input
    pub fn set_brake(&mut self, brake: f32) {
        self.controls.brake = brake.clamp(0.0, 1.0);
    }

    /// Updates the vehicle physics with surface type information and NaN/Inf protection
    pub fn update(
        &mut self,
        dt: f32,
        ground_height: impl Fn(f32, f32) -> f32,
        surface_getter: impl Fn(f32, f32) -> SurfaceType,
    ) {
        // Validate dt
        if !dt.is_finite() || dt <= 0.0 {
            tracing::warn!(target: "physics", "Invalid dt in vehicle physics: {}, skipping update", dt);
            return;
        }

        // Validate current state
        if !self.validate_state() {
            tracing::warn!(target: "physics", "Invalid state detected in vehicle, resetting to safe state");
            self.reset_to_safe_state();
            return;
        }

        // Apply steering to front wheels
        let target_steering = self.controls.steering * self.config.max_steering_angle;

        if self.wheels.len() >= 2 {
            // Front wheel drive or 4WD
            self.wheels[0].steering_angle = target_steering;
            self.wheels[1].steering_angle = target_steering;
        }

        // Update suspension and wheel forces using index-based loop to avoid borrow issues
        let wheel_count = self.wheels.len();
        for i in 0..wheel_count {
            let wheel_local_pos = self.wheels[i].local_position;
            let wheel_steering = self.wheels[i].steering_angle;

            // Calculate wheel world position
            let wheel_world_pos = self.body.position + self.body.rotation * wheel_local_pos;

            // Sample ground height and surface type
            let ground_y = ground_height(wheel_world_pos.x, wheel_world_pos.z);
            let surface_type = surface_getter(wheel_world_pos.x, wheel_world_pos.z);

            // Update wheel suspension and forces
            self.update_wheel_simple(i, dt, ground_y, surface_type, wheel_world_pos);
        }

        // Apply aerodynamic drag
        self.apply_aerodynamics(dt);

        // Integrate rigid body motion
        self.body.update(dt);

        // Final validation
        if !self.validate_state() {
            tracing::error!(target: "physics", "Vehicle state became invalid after update, resetting");
            self.reset_to_safe_state();
        }
    }

    /// Validate that all vehicle physics state values are finite
    pub fn validate_state(&self) -> bool {
        self.body.position.x.is_finite()
            && self.body.position.y.is_finite()
            && self.body.position.z.is_finite()
            && self.body.velocity.x.is_finite()
            && self.body.velocity.y.is_finite()
            && self.body.velocity.z.is_finite()
            && self.body.angular_velocity.x.is_finite()
            && self.body.angular_velocity.y.is_finite()
            && self.body.angular_velocity.z.is_finite()
            && self.body.rotation.i.is_finite()
            && self.body.rotation.j.is_finite()
            && self.body.rotation.k.is_finite()
            && self.body.rotation.w.is_finite()
    }

    /// Reset vehicle to a safe state when invalid values are detected
    pub fn reset_to_safe_state(&mut self) {
        tracing::info!(target: "physics", "Resetting vehicle to safe state");
        self.body.velocity = nalgebra::Vector3::zeros();
        self.body.angular_velocity = nalgebra::Vector3::zeros();
        self.body.position.y = self.body.position.y.max(0.5); // Ensure we're above ground
        for wheel in &mut self.wheels {
            wheel.angular_velocity = 0.0;
            wheel.suspension_velocity = 0.0;
        }
        self.controls.throttle = 0.0;
        self.controls.brake = 0.0;
        self.controls.steering = 0.0;
    }

    /// Updates a single wheel's physics (simplified version to avoid borrow issues)
    fn update_wheel_simple(
        &mut self,
        wheel_index: usize,
        dt: f32,
        ground_y: f32,
        surface_type: SurfaceType,
        wheel_world_pos: Vector3<f32>,
    ) {
        // Calculate suspension compression
        let wheel_bottom_y = wheel_world_pos.y - self.config.wheel_radius;
        let suspension_deflection = ground_y - wheel_bottom_y;

        // Исправление: ограничиваем scope заимствования wheel чтобы избежать multiple mutable borrow
        // Сначала собираем данные о колесе, затем освобождаем borrow и применяем силы
        let (wheel_contact, wheel_angular_vel, needs_tire_forces) = {
            let wheel = &mut self.wheels[wheel_index];

            wheel.is_in_contact = suspension_deflection > 0.0;

            if wheel.is_in_contact {
                wheel.contact_normal = Some(Vector3::new(0.0, 1.0, 0.0));
                wheel.contact_point =
                    Some(Vector3::new(wheel_world_pos.x, ground_y, wheel_world_pos.z));
                wheel.suspension_compression =
                    suspension_deflection.clamp(0.0, self.config.max_suspension_travel);

                // Calculate suspension force
                let spring_force = wheel.suspension_compression * self.config.suspension_stiffness;
                let damping_force = wheel.suspension_velocity * self.config.suspension_damping;
                // Don't clamp to zero - damping can be negative (rebound)
                let suspension_force = spring_force + damping_force;

                // Apply suspension force to vehicle body
                let force_dir = self.body.rotation * Vector3::new(0.0, 1.0, 0.0);
                let force = force_dir * suspension_force;

                self.body.apply_force_at_point(force, wheel_world_pos);

                // Собираем данные для tire forces
                let wheel_contact = wheel.is_in_contact;
                let wheel_angular_vel = wheel.angular_velocity;

                // Update wheel rotation based on vehicle speed with slip consideration
                let linear_speed = self.body.velocity.norm();
                // Calculate wheel slip: difference between linear speed and rotational speed
                let expected_angular_vel = linear_speed / self.config.wheel_radius;
                // Apply some slip based on acceleration/braking (simplified)
                let drive_force = self.controls.throttle * self.config.engine_force;
                let slip_factor = 1.0 + (drive_force * 0.01).clamp(-0.3, 0.3);
                wheel.angular_velocity = expected_angular_vel * slip_factor;

                (wheel_contact, wheel_angular_vel, wheel_contact)
            } else {
                wheel.suspension_compression = 0.0;
                wheel.suspension_velocity = 0.0;
                (false, 0.0, false)
            }
        }; // <- borrow wheel освобождается здесь

        // Применяем силы шины после освобождения borrow wheel
        // Исправление: вызов apply_tire_forces_simple вынесен за пределы borrow wheel
        if needs_tire_forces {
            self.apply_tire_forces_simple(
                wheel_index,
                dt,
                ground_y,
                surface_type,
                wheel_world_pos,
                wheel_angular_vel,
            );
        }

        // Update wheel rotation - снова заиммуем wheel
        let wheel = &mut self.wheels[wheel_index];
        wheel.rotation_angle += wheel.angular_velocity * dt;
    }

    /// Updates a single wheel's physics
    fn update_wheel(
        &mut self,
        wheel_index: usize,
        wheel: &mut WheelState,
        dt: f32,
        ground_height: &impl Fn(f32, f32) -> f32,
        surface_getter: &impl Fn(f32, f32) -> SurfaceType,
    ) {
        // Get wheel world position
        let wheel_world_pos = self.body.position + self.body.rotation * wheel.local_position;

        // Raycast вниз для определения контакта с землёй
        let ray_origin = wheel_world_pos;
        let ray_direction = Vector3::new(0.0, -1.0, 0.0);
        let ray_length = self.config.wheel_radius
            + self.config.suspension_rest_length
            + self.config.max_suspension_travel;

        // Sample ground height at wheel position
        let ground_y = ground_height(wheel_world_pos.x, wheel_world_pos.z);

        // Get surface type at wheel position
        let surface_type = surface_getter(wheel_world_pos.x, wheel_world_pos.z);

        // Calculate suspension compression
        let wheel_bottom_y = wheel_world_pos.y - self.config.wheel_radius;
        let suspension_deflection = ground_y - wheel_bottom_y;

        wheel.is_in_contact = suspension_deflection > 0.0;

        if wheel.is_in_contact {
            // Устанавливаем нормаль контакта (вверх, так как земля горизонтальная)
            wheel.contact_normal = Some(Vector3::new(0.0, 1.0, 0.0));
            wheel.contact_point =
                Some(Vector3::new(wheel_world_pos.x, ground_y, wheel_world_pos.z));

            wheel.suspension_compression =
                suspension_deflection.clamp(0.0, self.config.max_suspension_travel);

            // Calculate suspension force
            let spring_force = wheel.suspension_compression * self.config.suspension_stiffness;
            let damping_force = wheel.suspension_velocity * self.config.suspension_damping;
            // Don't clamp to zero - damping can be negative (rebound)
            let suspension_force = spring_force + damping_force;

            // Apply suspension force to vehicle body
            let force_dir = self.body.rotation * Vector3::new(0.0, 1.0, 0.0);
            let force = force_dir * suspension_force;

            self.body.apply_force_at_point(force, wheel_world_pos);

            // Calculate tire forces based on slip and surface type
            self.apply_tire_forces(wheel, wheel_index, dt, ground_y, surface_type);

            // Update wheel rotation based on vehicle speed with slip consideration
            let linear_speed = self.body.velocity.norm();
            let drive_force = self.controls.throttle * self.config.engine_force;
            let expected_angular_vel = linear_speed / self.config.wheel_radius;
            let slip_factor = 1.0 + (drive_force * 0.01).clamp(-0.3, 0.3);
            wheel.angular_velocity = expected_angular_vel * slip_factor;
        } else {
            wheel.suspension_compression = 0.0;
            wheel.suspension_velocity = 0.0;
        }

        // Update wheel rotation
        wheel.rotation_angle += wheel.angular_velocity * dt;
    }

    /// Applies tire forces based on slip angles and surface type (simplified version)
    fn apply_tire_forces_simple(
        &mut self,
        wheel_index: usize,
        dt: f32,
        ground_y: f32,
        surface_type: SurfaceType,
        wheel_world_pos: Vector3<f32>,
        _wheel_angular_vel: f32,
    ) {
        let wheel_vel = self.body.get_velocity_at_point(wheel_world_pos);

        // Get friction coefficients from surface type
        let surface_friction = surface_type.friction_coefficient();
        let rolling_resistance = surface_type.rolling_resistance();

        // Calculate slip angle (simplified)
        let forward = self.body.rotation * Vector3::new(0.0, 0.0, 1.0);
        let lateral = self.body.rotation * Vector3::new(1.0, 0.0, 0.0);

        let forward_vel = wheel_vel.dot(&forward);
        let lateral_vel = wheel_vel.dot(&lateral);

        // Apply surface friction to forces
        let friction_multiplier = surface_friction;

        // Calculate rolling resistance force (opposes motion)
        let speed = wheel_vel.norm();
        let rolling_resistance_force = if speed > 0.01 {
            -rolling_resistance * self.config.mass * 9.81 * (wheel_vel.normalize())
        } else {
            Vector3::zeros()
        };

        // Apply driving/braking force with low range multiplier
        let torque_multiplier = if self.controls.low_range && self.config.low_range_enabled {
            self.config.low_range_ratio
        } else {
            1.0
        };

        // Determine drive force based on wheel index and differential locks
        let is_front_wheel = wheel_index < 2;
        let is_rear_wheel = wheel_index >= 2;

        let throttle_force = self.controls.throttle * self.config.engine_force * torque_multiplier;

        // 4WD by default, but respect differential locks for torque distribution
        let drive_force = if is_front_wheel && is_rear_wheel {
            // All wheels driven (4WD)
            throttle_force / 4.0
        } else if is_front_wheel {
            // Front wheels
            throttle_force / 2.0
        } else {
            // Rear wheels
            throttle_force / 2.0
        };

        let braking_force = -self.controls.brake * self.config.brake_force
            - self.controls.handbrake * self.config.brake_force * 0.5;

        let longitudinal_force = drive_force + braking_force;

        // Apply forces
        let drive_dir = forward * longitudinal_force;
        self.body.apply_force(drive_dir);
    }

    /// Applies tire forces based on slip angles and surface type
    fn apply_tire_forces(
        &mut self,
        wheel: &WheelState,
        wheel_index: usize,
        dt: f32,
        ground_y: f32,
        surface_type: SurfaceType,
    ) {
        if !wheel.is_in_contact {
            return;
        }

        let wheel_world_pos = self.body.position + self.body.rotation * wheel.local_position;
        let wheel_vel = self.body.get_velocity_at_point(wheel_world_pos);

        // Get friction coefficients from surface type
        let surface_friction = surface_type.friction_coefficient();
        let rolling_resistance = surface_type.rolling_resistance();

        // Calculate slip angle (simplified)
        let forward = self.body.rotation * Vector3::new(0.0, 0.0, 1.0);
        let lateral = self.body.rotation * Vector3::new(1.0, 0.0, 0.0);

        let forward_vel = wheel_vel.dot(&forward);
        let lateral_vel = wheel_vel.dot(&lateral);

        // Apply surface friction to tire forces
        let friction_multiplier = surface_friction;

        // Calculate rolling resistance force (opposes motion)
        let speed = wheel_vel.norm();
        let rolling_resistance_force = if speed > 0.01 {
            -rolling_resistance * self.config.mass * 9.81 * (wheel_vel.normalize())
        } else {
            Vector3::zeros()
        };

        // Apply driving/braking force with low range multiplier
        let torque_multiplier = if self.controls.low_range && self.config.low_range_enabled {
            self.config.low_range_ratio
        } else {
            1.0
        };

        // Determine drive force based on wheel index and differential locks
        let is_front_wheel = wheel_index < 2;
        let is_rear_wheel = wheel_index >= 2;

        let throttle_force = self.controls.throttle * self.config.engine_force * torque_multiplier;

        // 4WD by default, but respect differential locks for torque distribution
        let drive_force = if is_front_wheel && is_rear_wheel {
            // All wheels driven (4WD)
            throttle_force / 4.0
        } else if is_front_wheel {
            // Front wheels
            if self.controls.diff_front_lock || self.config.diff_front_locked {
                // Locked diff: equal torque to both front wheels
                throttle_force * 0.5
            } else {
                // Open diff: torque follows path of least resistance
                throttle_force * 0.5
            }
        } else {
            // Rear wheels
            if self.controls.diff_rear_lock || self.config.diff_rear_locked {
                // Locked diff: equal torque to both rear wheels
                throttle_force * 0.5
            } else {
                // Open diff: torque follows path of least resistance
                throttle_force * 0.5
            }
        };

        let braking_force = -self.controls.brake * self.config.brake_force
            - self.controls.handbrake * self.config.brake_force * 0.5;

        let longitudinal_force = drive_force + braking_force;

        // Apply forces
        let drive_dir = forward * longitudinal_force;
        self.body.apply_force(drive_dir);
    }

    /// Applies aerodynamic forces
    fn apply_aerodynamics(&mut self, dt: f32) {
        let speed_sq = self.body.velocity.norm_squared();
        let speed = self.body.velocity.norm();

        if speed < 0.01 {
            return;
        }

        // Air drag
        let drag_direction = -self.body.velocity.normalize();
        let drag_magnitude = 0.5 * 1.225 * self.config.drag_coefficient * 2.0 * speed_sq;
        let drag_force = drag_direction * drag_magnitude;

        self.body.apply_force(drag_force);

        // Downforce (if configured)
        if self.config.downforce_coefficient > 0.0 {
            let downforce = self.body.rotation
                * Vector3::new(0.0, -1.0, 0.0)
                * self.config.downforce_coefficient
                * speed_sq;
            self.body.apply_force(downforce);
        }
    }

    /// Gets the vehicle's rigid body
    pub fn body(&self) -> &RigidBody {
        &self.body
    }

    /// Gets the vehicle's rigid body (mutable)
    pub fn body_mut(&mut self) -> &mut RigidBody {
        &mut self.body
    }

    /// Gets all wheels
    pub fn wheels(&self) -> &[WheelState] {
        &self.wheels
    }

    /// Gets the vehicle speed
    pub fn speed(&self) -> f32 {
        self.body.velocity.norm()
    }

    /// Gets the vehicle position
    pub fn position(&self) -> Vector3<f32> {
        self.body.position
    }

    /// Sets the vehicle position
    pub fn set_position(&mut self, pos: Vector3<f32>) {
        self.body.position = pos;
    }

    /// Gets the vehicle rotation
    pub fn rotation(&self) -> UnitQuaternion<f32> {
        self.body.rotation
    }

    /// Resets the vehicle state
    pub fn reset(&mut self) {
        let chassis_dims = Vector3::new(DEFAULT_CHASSIS_WIDTH, DEFAULT_CHASSIS_HEIGHT, DEFAULT_CHASSIS_LENGTH);
        self.body = RigidBody::new_box(
            Vector3::zeros(),
            self.config.mass,
            chassis_dims,
        );

        for wheel in &mut self.wheels {
            wheel.suspension_compression = 0.0;
            wheel.suspension_velocity = 0.0;
            wheel.rotation_angle = 0.0;
            wheel.angular_velocity = 0.0;
            wheel.is_in_contact = false;
            wheel.contact_point = None;
            wheel.contact_normal = None;
        }

        self.controls = VehicleControls::default();
    }

    /// Интеграция с PhysicsWorld - добавляет тело шасси в мир и возвращает его ID
    pub fn add_to_physics_world(
        &mut self,
        physics_world: &mut crate::physics::PhysicsWorld,
    ) -> usize {
        let body = std::mem::replace(
            &mut self.body,
            RigidBody::new_box(Vector3::zeros(), 1.0, Vector3::new(0.1, 0.1, 0.1)),
        );
        let body_id = physics_world.add_body(body);
        body_id
    }

    /// Обновление физики транспорта через PhysicsWorld
    pub fn physics_update(
        &mut self,
        dt: f32,
        physics_world: &mut crate::physics::PhysicsWorld,
        terrain_getter: &dyn Fn(f32, f32) -> f32,
        surface_getter: &dyn Fn(f32, f32) -> crate::world::SurfaceType,
        deformable_terrain: Option<&mut crate::physics::DeformableTerrainComponent>,
    ) {
        // Сначала выполняем основной update с terrain и surface
        self.update(dt, terrain_getter, surface_getter);

        // Деформация ландшафта колёсами
        if let Some(terrain) = deformable_terrain {
            self.deform_terrain(terrain);
        }

        // Синхронизируем состояние тела с physics_world
        if let Some(chassis_id) = self.chassis_body_id {
            if let Some(body) = physics_world.get_body_mut(chassis_id) {
                // Применяем силы от колёс к шасси через physics_world
                self.apply_wheel_forces_to_chassis(body);

                // Синхронизируем позицию и ориентацию
                body.position = self.body.position;
                body.rotation = self.body.rotation;
                body.velocity = self.body.velocity;
                body.angular_velocity = self.body.angular_velocity;
            }
        }
    }

    /// Деформация ландшафта под колёсами
    fn deform_terrain(&self, terrain: &mut crate::physics::DeformableTerrainComponent) {
        use crate::physics::deformable_terrain::DeformationType;

        for wheel in &self.wheels {
            if wheel.is_in_contact {
                let wheel_pos = self.body.position + self.body.rotation * wheel.local_position;
                // Давление колеса создаёт небольшую вмятину
                let pressure = self.config.mass / self.wheels.len() as f32 * 9.81 * 0.001;
                terrain.apply_deformation(wheel_pos, DeformationType::Press(pressure));
            }
        }
    }

    /// Возвращает ID тела шасси
    pub fn chassis_body_id(&self) -> Option<usize> {
        self.chassis_body_id
    }

    /// Устанавливает ID тела шасси
    pub fn set_chassis_body_id(&mut self, id: usize) {
        self.chassis_body_id = Some(id);
    }

    /// Применяет силы от колёс к шасси
    fn apply_wheel_forces_to_chassis(&mut self, chassis_body: &mut RigidBody) {
        // Силы уже были применены в update(), здесь просто синхронизируем состояние
        // Можно добавить дополнительную логику если нужно
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_creation() {
        let config = VehicleConfig::default();
        let vehicle = Vehicle::new(config.clone());

        assert_eq!(vehicle.body.mass, config.mass);
        assert_eq!(vehicle.wheels.len(), 4);
    }

    #[test]
    fn test_vehicle_controls() {
        let mut vehicle = Vehicle::new(VehicleConfig::default());

        let controls = VehicleControls::new(1.0, 0.5, 0.3, 0.0);
        vehicle.set_controls(controls.clone());

        assert_eq!(vehicle.get_controls().throttle, 1.0);
        assert_eq!(vehicle.get_controls().brake, 0.5);
        assert_eq!(vehicle.get_controls().steering, 0.3);
    }
}
