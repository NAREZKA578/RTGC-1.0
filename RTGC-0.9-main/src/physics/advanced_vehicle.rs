//! Advanced Vehicle Physics for RTGC-0.8
//!
//! Реализация продвинутой физики транспортных средств:
//! - Нелинейная подвеска с раздельным сжатием/отбоем
//! - Симуляция давления и температуры шин
//! - Износ шин и влияние на сцепление
//! - Аэродинамическое сопротивление
//! - Защита от NaN/Inf во всех вычислениях

use nalgebra::Vector3;
use std::f32::consts::PI;

/// Advanced suspension model with non-linear spring and separate compression/rebound damping
#[derive(Debug, Clone)]
pub struct AdvancedSuspension {
    pub spring_stiffness: f32,     // N/m (base stiffness)
    pub compression_damping: f32,  // N*s/m for compression
    pub rebound_damping: f32,      // N*s/m for rebound
    pub rest_length: f32,          // meters
    pub current_length: f32,       // meters
    pub max_compression: f32,      // meters
    pub max_extension: f32,        // meters
    pub tire_radius: f32,          // meters
    pub friction_coefficient: f32, // static friction coefficient
    pub slip_ratio: f32,           // dimensionless
    pub slip_angle: f32,           // radians
    pub camber_angle: f32,         // radians
    // Non-linear spring parameters
    pub progressive_rate: f32, // Stiffness increase per meter of compression
    pub bump_stop_stiffness: f32, // Extra stiffness at max compression
    pub bump_stop_start: f32,  // Compression ratio where bump stop engages (0.0-1.0)
    // Tire pressure and temperature simulation
    pub tire_pressure: f32,       // PSI (pounds per square inch)
    pub tire_temperature: f32,    // Celsius
    pub ambient_temperature: f32, // Ambient air temperature
    pub tire_wear: f32,           // 0.0 = new, 1.0 = worn out
}

impl AdvancedSuspension {
    /// Validates that all physical quantities are finite (not NaN or Inf)
    pub fn validate_state(&self) -> bool {
        self.spring_stiffness.is_finite()
            && self.compression_damping.is_finite()
            && self.rebound_damping.is_finite()
            && self.rest_length.is_finite()
            && self.current_length.is_finite()
            && self.tire_pressure.is_finite()
            && self.tire_temperature.is_finite()
            && self.ambient_temperature.is_finite()
            && self.tire_wear.is_finite()
            && self.friction_coefficient.is_finite()
            && self.slip_ratio.is_finite()
            && self.slip_angle.is_finite()
            && self.camber_angle.is_finite()
    }

    /// Resets suspension to a safe state when invalid values are detected
    pub fn reset_to_safe_state(&mut self) {
        if !self.spring_stiffness.is_finite() || self.spring_stiffness <= 0.0 {
            self.spring_stiffness = 20000.0;
        }
        if !self.compression_damping.is_finite() || self.compression_damping < 0.0 {
            self.compression_damping = 1500.0;
        }
        if !self.rebound_damping.is_finite() || self.rebound_damping < 0.0 {
            self.rebound_damping = 2000.0;
        }
        if !self.rest_length.is_finite() || self.rest_length <= 0.0 {
            self.rest_length = 0.5;
        }
        if !self.current_length.is_finite() {
            self.current_length = self.rest_length;
        }
        if !self.tire_pressure.is_finite() || self.tire_pressure <= 0.0 {
            self.tire_pressure = 32.0;
        }
        if !self.tire_temperature.is_finite() {
            self.tire_temperature = self.ambient_temperature;
        }
        if !self.ambient_temperature.is_finite() {
            self.ambient_temperature = 20.0;
        }
        if !self.tire_wear.is_finite() || self.tire_wear < 0.0 || self.tire_wear > 1.0 {
            self.tire_wear = 0.0;
        }
        if !self.friction_coefficient.is_finite() || self.friction_coefficient <= 0.0 {
            self.friction_coefficient = 0.8;
        }
        self.slip_ratio = 0.0;
        self.slip_angle = 0.0;
        self.camber_angle = 0.0;
    }

    pub fn new(
        spring_stiffness: f32,
        compression_damping: f32,
        rebound_damping: f32,
        rest_length: f32,
        max_compression: f32,
        max_extension: f32,
        tire_radius: f32,
    ) -> Self {
        Self {
            spring_stiffness,
            compression_damping,
            rebound_damping,
            rest_length,
            current_length: rest_length,
            max_compression,
            max_extension,
            tire_radius,
            friction_coefficient: 0.8, // Default grip
            slip_ratio: 0.0,
            slip_angle: 0.0,
            camber_angle: 0.0,
            // Non-linear spring defaults
            progressive_rate: spring_stiffness * 0.5, // 50% increase at full compression
            bump_stop_stiffness: spring_stiffness * 5.0, // 5x stiffness at bump stop
            bump_stop_start: 0.8,                     // Engage at 80% compression
            // Tire pressure and temperature defaults
            tire_pressure: 32.0,       // PSI (typical car tire)
            tire_temperature: 20.0,    // Celsius (ambient)
            ambient_temperature: 20.0, // Celsius
            tire_wear: 0.0,            // New tire
        }
    }

    /// Create suspension with realistic car parameters
    pub fn new_realistic_car_suspension(tire_radius: f32) -> Self {
        // Typical values for a passenger car
        Self {
            spring_stiffness: 25000.0,   // N/m (25 kN/m)
            compression_damping: 1500.0, // N*s/m
            rebound_damping: 2500.0,     // N*s/m (higher than compression for stability)
            rest_length: 0.4,            // 40 cm
            current_length: 0.4,         // 40 cm
            max_compression: 0.15,       // 15 cm
            max_extension: 0.1,          // 10 cm
            tire_radius,
            friction_coefficient: 0.9, // Good asphalt grip
            slip_ratio: 0.0,
            slip_angle: 0.0,
            camber_angle: 0.0,
            progressive_rate: 15000.0,
            bump_stop_stiffness: 150000.0,
            bump_stop_start: 0.7,
            // Tire pressure and temperature
            tire_pressure: 32.0,       // PSI
            tire_temperature: 20.0,    // Celsius
            ambient_temperature: 20.0, // Celsius
            tire_wear: 0.0,
        }
    }

    /// Create suspension with realistic truck parameters
    pub fn new_realistic_truck_suspension(tire_radius: f32) -> Self {
        // Typical values for a heavy truck
        Self {
            spring_stiffness: 80000.0,   // N/m (80 kN/m) - stiffer for heavier loads
            compression_damping: 4000.0, // N*s/m
            rebound_damping: 6000.0,     // N*s/m
            rest_length: 0.5,            // 50 cm
            current_length: 0.5,         // 50 cm
            max_compression: 0.2,        // 20 cm
            max_extension: 0.15,         // 15 cm
            tire_radius,
            friction_coefficient: 0.7, // Lower grip due to larger contact patch
            slip_ratio: 0.0,
            slip_angle: 0.0,
            camber_angle: 0.0,
            progressive_rate: 40000.0,
            bump_stop_stiffness: 400000.0,
            bump_stop_start: 0.75,
            // Tire pressure and temperature
            tire_pressure: 80.0,       // PSI (truck tires have higher pressure)
            tire_temperature: 25.0,    // Celsius
            ambient_temperature: 20.0, // Celsius
            tire_wear: 0.0,
        }
    }

    /// Update tire pressure based on temperature (ideal gas law approximation)
    pub fn update_tire_pressure(&mut self, _dt: f32) {
        // P1/T1 = P2/T2 (Gay-Lussac's law)
        // Pressure increases ~1 PSI per 10°F (5.5°C) temperature rise
        let temp_kelvin = self.tire_temperature + 273.15;
        let ref_temp_kelvin = 20.0 + 273.15; // Reference temperature at which pressure is set
        let ref_pressure = 32.0; // Reference pressure

        // Adjust pressure based on temperature
        self.tire_pressure = ref_pressure * (temp_kelvin / ref_temp_kelvin);

        // Clamp to reasonable range
        self.tire_pressure = self.tire_pressure.clamp(20.0, 50.0);
    }

    /// Update tire temperature based on friction, ambient temp, and cooling
    pub fn update_tire_temperature(
        &mut self,
        slip_ratio: f32,
        slip_angle: f32,
        normal_force: f32,
        dt: f32,
    ) {
        // Heat generation from friction (simplified model)
        let total_slip = slip_ratio.abs() + slip_angle.abs();
        let friction_heat = total_slip * normal_force * 0.001; // Simplified coefficient

        // Heat dissipation to ambient (Newton's law of cooling)
        let cooling_rate = 0.5; // Cooling coefficient
        let temp_diff = self.tire_temperature - self.ambient_temperature;
        let cooling = cooling_rate * temp_diff;

        // Update temperature
        self.tire_temperature += (friction_heat - cooling) * dt;

        // Clamp to reasonable range (-40°C to 120°C)
        self.tire_temperature = self.tire_temperature.clamp(-40.0, 120.0);

        // Update pressure based on new temperature
        self.update_tire_pressure(dt);
    }

    /// Get effective friction coefficient based on tire condition
    pub fn get_effective_friction(&self) -> f32 {
        // Base friction modified by pressure, temperature, and wear
        let mut friction = self.friction_coefficient;

        // Optimal pressure gives best grip (around 32 PSI for cars)
        let optimal_pressure = 32.0;
        let pressure_factor =
            1.0 - ((self.tire_pressure - optimal_pressure).abs() / optimal_pressure) * 0.3;
        friction *= pressure_factor.max(0.5);

        // Optimal temperature range (60-90°C for racing, lower for street)
        let optimal_temp = 70.0;
        let temp_diff = (self.tire_temperature - optimal_temp).abs();
        let temp_factor = 1.0 - (temp_diff / 100.0).min(0.5);
        friction *= temp_factor;

        // Wear reduces grip significantly
        let wear_factor = 1.0 - self.tire_wear * 0.4; // Up to 40% reduction when fully worn
        friction *= wear_factor;

        friction.clamp(0.3, 1.5)
    }

    /// Update tire wear based on usage
    pub fn update_tire_wear(
        &mut self,
        distance: f32,
        slip_ratio: f32,
        slip_angle: f32,
        normal_force: f32,
    ) {
        // Base wear from distance
        let base_wear = distance * 0.000001; // Very small per meter

        // Additional wear from slipping
        let slip_wear = (slip_ratio.abs() + slip_angle.abs()) * normal_force * 0.00001;

        // High temperature accelerates wear
        let temp_wear_factor = if self.tire_temperature > 90.0 {
            1.0 + (self.tire_temperature - 90.0) / 30.0
        } else {
            1.0
        };

        // Low pressure causes uneven wear
        let pressure_wear_factor = if self.tire_pressure < 28.0 {
            1.0 + (28.0 - self.tire_pressure) / 10.0
        } else {
            1.0
        };

        self.tire_wear += (base_wear + slip_wear) * temp_wear_factor * pressure_wear_factor;
        self.tire_wear = self.tire_wear.min(1.0);
    }

    pub fn update_suspension(
        &mut self,
        wheel_velocity: Vector3<f32>,
        contact_normal: Vector3<f32>,
        vehicle_velocity: Vector3<f32>,
        dt: f32,
    ) -> (Vector3<f32>, f32) {
        // Calculate current suspension compression
        let compression = self.rest_length - (self.current_length - self.tire_radius);

        // Calculate spring force (non-linear progressive stiffness could be added here)
        let spring_force = self.calculate_spring_force(compression);

        // Calculate damping force based on suspension velocity
        let damping_force = self.calculate_damping_force(wheel_velocity, contact_normal);

        // Total suspension force
        let suspension_force = spring_force + damping_force;

        // Calculate tire contact patch forces using Pacejka/Magic Formula approach
        let (longitudinal_force, lateral_force) = self.calculate_tire_forces(
            vehicle_velocity,
            wheel_velocity,
            contact_normal,
            suspension_force,
            dt,
        );

        // Combine all forces
        let total_force = longitudinal_force + lateral_force + contact_normal * suspension_force;

        (total_force, suspension_force)
    }

    fn calculate_spring_force(&self, compression: f32) -> f32 {
        // Non-linear progressive spring with bump stop
        let mut spring_force = 0.0;

        // Base linear spring force
        let effective_compression = compression
            .max(-self.max_compression)
            .min(self.max_extension);
        spring_force += self.spring_stiffness * effective_compression;

        // Progressive rate: stiffness increases with compression
        if compression > 0.0 {
            let progress = (compression / self.max_compression).min(1.0);
            spring_force += self.progressive_rate * compression * progress;
        }

        // Bump stop: extra stiffness when approaching max compression
        let compression_ratio = if self.max_compression > 0.0 {
            compression / self.max_compression
        } else {
            0.0
        };

        if compression_ratio > self.bump_stop_start {
            let bump_compression = compression_ratio - self.bump_stop_start;
            let bump_factor = bump_compression / (1.0 - self.bump_stop_start);
            spring_force += self.bump_stop_stiffness * bump_factor.powi(2) * self.max_compression;
        }

        spring_force
    }

    fn calculate_damping_force(
        &self,
        wheel_velocity: Vector3<f32>,
        contact_normal: Vector3<f32>,
    ) -> f32 {
        let velocity_along_normal = wheel_velocity.dot(&contact_normal);

        // Different damping coefficients for compression vs rebound
        let damping_coefficient = if velocity_along_normal > 0.0 {
            // Compression (wheel moving upward)
            self.compression_damping
        } else {
            // Rebound (wheel moving downward)
            self.rebound_damping
        };

        damping_coefficient * velocity_along_normal
    }

    /// Advanced tire model using Pacejka Magic Formula for realistic force calculation
    fn calculate_tire_forces(
        &mut self,
        vehicle_velocity: Vector3<f32>,
        wheel_velocity: Vector3<f32>,
        _contact_normal: Vector3<f32>,
        normal_force: f32,
        _dt: f32,
    ) -> (Vector3<f32>, Vector3<f32>) {
        // Calculate wheel's forward and lateral directions relative to vehicle
        let forward = Vector3::new(0.0, 0.0, 1.0); // Assuming wheel's forward direction
        let lateral = Vector3::new(1.0, 0.0, 0.0); // Assuming wheel's lateral direction

        // Calculate slip ratio: longitudinal slip during acceleration/braking
        let wheel_forward_vel = wheel_velocity.dot(&forward);
        let vehicle_forward_vel = vehicle_velocity.dot(&forward);

        // Improved slip ratio calculation with proper sign handling
        if vehicle_forward_vel.abs() > 0.1 {
            self.slip_ratio = (wheel_forward_vel - vehicle_forward_vel) / vehicle_forward_vel.abs();
        } else if wheel_forward_vel.abs() > 0.1 {
            // Vehicle is stationary or very slow, use wheel speed as reference
            self.slip_ratio = if wheel_forward_vel > 0.0 { 1.0 } else { -1.0 };
        } else {
            self.slip_ratio = 0.0;
        }

        // Calculate slip angle: lateral slip during cornering
        let side_slip_vel = vehicle_velocity.dot(&lateral);
        if vehicle_forward_vel.abs() > 0.1 {
            self.slip_angle = (side_slip_vel / vehicle_forward_vel.abs()).atan();
        } else {
            self.slip_angle = 0.0;
        }

        // Apply Pacejka Magic Formula for realistic tire forces
        let max_grip = self.friction_coefficient * normal_force;

        // Longitudinal force (acceleration/braking) using Pacejka formula
        let longitudinal_factor = self.calculate_pacejka_magic_formula(
            self.slip_ratio,
            10.0, // B (stiffness factor)
            1.9,  // C (shape factor for longitudinal)
            1.0,  // D (peak value)
            0.0,  // E (curvature factor)
        );

        // Lateral force (cornering) using Pacejka formula
        let lateral_factor = self.calculate_pacejka_magic_formula(
            self.slip_angle.to_degrees(),
            8.0,  // B (stiffness factor for lateral)
            1.6,  // C (shape factor for lateral)
            1.0,  // D (peak value)
            -0.5, // E (curvature factor for lateral)
        );

        // Combine friction ellipse: reduce lateral grip when using longitudinal grip
        let combined_longitudinal = longitudinal_factor;
        let combined_lateral = lateral_factor * (1.0 - longitudinal_factor.abs() * 0.3);

        let longitudinal_force_magnitude = max_grip * combined_longitudinal;
        let lateral_force_magnitude = max_grip * combined_lateral;

        // Apply forces in the correct directions
        let longitudinal_force = forward * longitudinal_force_magnitude;
        let lateral_force = lateral * lateral_force_magnitude;

        (longitudinal_force, lateral_force)
    }

    /// Pacejka Magic Formula: F(x) = D * sin(C * atan(B * x - E * (B * x - atan(B * x))))
    /// This produces realistic tire force curves based on slip
    fn calculate_pacejka_magic_formula(&self, slip: f32, b: f32, c: f32, d: f32, e: f32) -> f32 {
        let bx = b * slip;
        let atan_bx = bx.atan();
        let inner = bx - e * (bx - atan_bx);
        d * (c * inner).sin()
    }
}

#[derive(Debug, Clone)]
pub struct AdvancedWheel {
    pub position: Vector3<f32>, // Position relative to vehicle center
    pub radius: f32,            // Tire radius in meters
    pub suspension: AdvancedSuspension,
    pub rotation_angle: f32,     // Current rotation in radians
    pub steering_angle: f32,     // Steering angle in radians
    pub angular_velocity: f32,   // Angular velocity in rad/s
    pub drive_torque: f32,       // Applied drive/brake torque in N*m
    pub brake_torque: f32,       // Brake torque in N*m
    pub rolling_resistance: f32, // Rolling resistance coefficient
}

impl AdvancedWheel {
    pub fn new(position: Vector3<f32>, radius: f32, suspension: AdvancedSuspension) -> Self {
        Self {
            position,
            radius,
            suspension,
            rotation_angle: 0.0,
            steering_angle: 0.0,
            angular_velocity: 0.0,
            drive_torque: 0.0,
            brake_torque: 0.0,
            rolling_resistance: 0.015, // Typical value for car tires
        }
    }

    pub fn update(
        &mut self,
        vehicle_velocity: Vector3<f32>,
        __vehicle_angular_velocity: Vector3<f32>,
        wheel_linear_velocity: Vector3<f32>,
        contact_normal: Vector3<f32>,
        dt: f32,
    ) -> (Vector3<f32>, f32) {
        // Validate dt to prevent NaN/Inf propagation
        if !dt.is_finite() || dt <= 0.0 {
            tracing::warn!(target: "physics", "Invalid dt in advanced vehicle physics: {}, skipping update", dt);
            return (Vector3::zeros(), 0.0);
        }

        // Validate suspension state before update
        if !self.suspension.validate_state() {
            tracing::warn!(target: "physics", "Invalid suspension state detected, resetting to safe state");
            self.suspension.reset_to_safe_state();
        }

        // Calculate forces from suspension model
        let (force, normal_force) = self.suspension.update_suspension(
            wheel_linear_velocity,
            contact_normal,
            vehicle_velocity,
            dt,
        );

        // Apply drive/brake torques
        let net_torque = self.drive_torque
            - self.brake_torque.signum()
                * self
                    .brake_torque
                    .abs()
                    .min(normal_force * self.suspension.friction_coefficient * self.radius);

        // Update wheel angular velocity based on torques and forces
        self.update_angular_velocity(net_torque, force, normal_force, dt);

        // Update rotation angle
        self.rotation_angle += self.angular_velocity * dt;

        (force, normal_force)
    }

    fn update_angular_velocity(
        &mut self,
        torque: f32,
        _lateral_force: Vector3<f32>,
        normal_force: f32,
        dt: f32,
    ) {
        // Moment of inertia for a wheel (approximated as a solid disk)
        let inertia = 0.5 * 30.0 * self.radius * self.radius; // Assuming 30kg wheel mass

        // Calculate angular acceleration from torques
        let angular_acceleration = torque / inertia;

        // Consider forces affecting rotation (like rolling resistance)
        let rolling_resistance_torque =
            -self.rolling_resistance * normal_force * self.radius * self.angular_velocity.signum();

        // Update angular velocity
        self.angular_velocity +=
            (angular_acceleration + (rolling_resistance_torque / inertia)) * dt;
    }

    pub fn apply_drive_torque(&mut self, torque: f32) {
        self.drive_torque = torque;
    }

    pub fn apply_brake_torque(&mut self, torque: f32) {
        self.brake_torque = torque;
    }

    pub fn set_steering_angle(&mut self, angle: f32) {
        self.steering_angle = angle.clamp(-PI / 3.0, PI / 3.0); // Limit to ~60 degrees
    }
}

#[derive(Debug, Clone)]
pub struct AdvancedVehicle {
    pub chassis_body_index: usize,       // Reference to chassis rigid body
    pub wheels: Vec<AdvancedWheel>,      // Vehicle wheels
    pub mass: f32,                       // Total vehicle mass in kg
    pub engine_torque: f32,              // Current engine torque in N*m
    pub engine_rpm: f32,                 // Current engine RPM
    pub gear_ratio: f32,                 // Current gear ratio
    pub final_drive_ratio: f32,          // Final drive ratio
    pub steering_angle: f32,             // Current steering angle in radians
    pub max_steering_angle: f32,         // Maximum steering angle in radians
    pub max_engine_torque: f32,          // Maximum engine torque in N*m
    pub max_engine_rpm: f32,             // Maximum engine RPM
    pub brake_torque: f32,               // Current brake torque per wheel in N*m
    pub aero_drag_coefficient: f32,      // Aerodynamic drag coefficient
    pub frontal_area: f32,               // Frontal area in m^2
    pub air_density: f32,                // Air density in kg/m^3
    pub center_of_gravity: Vector3<f32>, // CoG offset from chassis center
}

impl AdvancedVehicle {
    pub fn new(chassis_body_index: usize, mass: f32) -> Self {
        Self {
            chassis_body_index,
            wheels: Vec::new(),
            mass,
            engine_torque: 0.0,
            engine_rpm: 0.0,
            gear_ratio: 1.0,
            final_drive_ratio: 3.5,
            steering_angle: 0.0,
            max_steering_angle: PI / 3.0, // ~60 degrees
            max_engine_torque: 400.0,     // 400 N*m typical for a car
            max_engine_rpm: 6000.0,
            brake_torque: 0.0,
            aero_drag_coefficient: 0.35, // Typical for a sedan
            frontal_area: 2.2,           // Typical for a sedan in m^2
            air_density: 1.225,          // At sea level
            center_of_gravity: Vector3::new(0.0, -0.5, 0.0), // Below chassis center
        }
    }

    pub fn add_wheel(&mut self, wheel: AdvancedWheel) {
        self.wheels.push(wheel);
    }

    pub fn update_vehicle_physics(
        &mut self,
        chassis_body: &mut crate::physics::RigidBody,
        dt: f32,
    ) {
        // Validate dt to prevent NaN/Inf propagation
        if !dt.is_finite() || dt <= 0.0 {
            tracing::warn!(target: "physics", "Invalid dt in advanced vehicle physics: {}, skipping update", dt);
            return;
        }

        // Get chassis state
        let chassis_velocity = chassis_body.velocity;
        let chassis_angular_velocity = chassis_body.angular_velocity;
        let chassis_transform = chassis_body.get_world_transform();

        // Calculate aerodynamic drag force
        let velocity_mag_sq = chassis_velocity.magnitude_squared();
        let drag_force_magnitude = 0.5
            * self.air_density
            * self.aero_drag_coefficient
            * self.frontal_area
            * velocity_mag_sq;
        let drag_direction = -chassis_velocity.normalize();
        let drag_force = drag_direction * drag_force_magnitude;

        // Apply drag force at center of gravity
        let cog_world_pos =
            chassis_transform.transform_point(&nalgebra::Point3::from(self.center_of_gravity));
        chassis_body.apply_force(drag_force);

        // Update each wheel and apply forces to chassis
        for (i, wheel) in self.wheels.iter_mut().enumerate() {
            // Calculate wheel world position and velocity
            let wheel_local_pos = wheel.position;
            let wheel_world_pos =
                chassis_transform.transform_point(&nalgebra::Point3::from(wheel_local_pos));
            let wheel_linear_velocity = chassis_velocity
                + chassis_angular_velocity.cross(&(wheel_world_pos.coords - chassis_body.position));

            // For simplicity, assume contact normal is up (would require raycast in real implementation)
            let contact_normal = Vector3::new(0.0, 1.0, 0.0);

            // Apply steering angle to front wheels
            if i == 0 || i == 1 {
                // Assuming first two wheels are front wheels
                wheel.set_steering_angle(self.steering_angle);
            }

            // Update wheel physics
            let (wheel_force, _normal_force) = wheel.update(
                chassis_velocity,
                chassis_angular_velocity,
                wheel_linear_velocity,
                contact_normal,
                dt,
            );

            // Apply forces to chassis body
            chassis_body.apply_force(wheel_force);

            // Apply torque from lateral forces (simplified)
            let moment_arm = wheel_world_pos.coords - cog_world_pos.coords;
            let torque = moment_arm.cross(&wheel_force);
            chassis_body.apply_torque(torque);
        }

        // Update engine RPM based on wheel speeds (simplified)
        self.update_engine_rpm(dt);
    }

    fn update_engine_rpm(&mut self, _dt: f32) {
        // Simplified calculation based on average wheel angular velocity
        if !self.wheels.is_empty() {
            let avg_wheel_angular_velocity: f32 = self
                .wheels
                .iter()
                .map(|w| w.angular_velocity.abs())
                .sum::<f32>()
                / self.wheels.len() as f32;

            // Convert wheel angular velocity to engine RPM considering gear ratios
            self.engine_rpm =
                avg_wheel_angular_velocity * self.final_drive_ratio * self.gear_ratio * 60.0
                    / (2.0 * PI);
            self.engine_rpm = self.engine_rpm.clamp(0.0, self.max_engine_rpm);
        }
    }

    pub fn apply_throttle(&mut self, throttle: f32) {
        // Validate throttle input
        if !throttle.is_finite() {
            tracing::warn!(target: "physics", "Invalid throttle input: {}, clamping to 0.0", throttle);
            self.engine_torque = 0.0;
            return;
        }

        // Calculate engine torque based on throttle input and engine characteristics
        // Simplified engine curve
        let normalized_rpm = self.engine_rpm / self.max_engine_rpm;
        let efficiency_factor = 1.0 - (normalized_rpm - 0.5).powi(2) * 0.2; // Simplified power curve
        self.engine_torque = throttle * self.max_engine_torque * efficiency_factor;
    }

    pub fn apply_brakes(&mut self, brake_intensity: f32) {
        // Validate brake input
        if !brake_intensity.is_finite() {
            tracing::warn!(target: "physics", "Invalid brake intensity: {}, clamping to 0.0", brake_intensity);
            self.brake_torque = 0.0;
            return;
        }
        self.brake_torque = brake_intensity * 2000.0; // Max 2000 N*m per wheel
    }

    pub fn set_steering(&mut self, steering_input: f32) {
        // Validate steering input
        if !steering_input.is_finite() {
            tracing::warn!(target: "physics", "Invalid steering input: {}, clamping to 0.0", steering_input);
            self.steering_angle = 0.0;
            return;
        }
        self.steering_angle = steering_input * self.max_steering_angle;
    }

    pub fn shift_gear(&mut self, gear_ratio: f32) {
        // Validate gear ratio
        if !gear_ratio.is_finite() || gear_ratio <= 0.0 {
            tracing::warn!(target: "physics", "Invalid gear ratio: {}, using default 1.0", gear_ratio);
            self.gear_ratio = 1.0;
            return;
        }
        self.gear_ratio = gear_ratio;
    }

    /// Validates that all physical quantities in the vehicle are finite
    pub fn validate_state(&self) -> bool {
        self.engine_rpm.is_finite()
            && self.gear_ratio.is_finite()
            && self.final_drive_ratio.is_finite()
            && self.steering_angle.is_finite()
            && self.engine_torque.is_finite()
            && self.brake_torque.is_finite()
            && self.air_density.is_finite()
            && self.aero_drag_coefficient.is_finite()
            && self.frontal_area.is_finite()
            && self.center_of_gravity.x.is_finite()
            && self.center_of_gravity.y.is_finite()
            && self.center_of_gravity.z.is_finite()
            && self.wheels.iter().all(|w| {
                w.rotation_angle.is_finite()
                    && w.steering_angle.is_finite()
                    && w.angular_velocity.is_finite()
                    && w.drive_torque.is_finite()
                    && w.brake_torque.is_finite()
                    && w.suspension.validate_state()
            })
    }

    /// Resets the vehicle to a safe state when invalid values are detected
    pub fn reset_to_safe_state(&mut self) {
        if !self.engine_rpm.is_finite() || self.engine_rpm < 0.0 {
            self.engine_rpm = 0.0;
        }
        if !self.gear_ratio.is_finite() || self.gear_ratio <= 0.0 {
            self.gear_ratio = 1.0;
        }
        if !self.final_drive_ratio.is_finite() || self.final_drive_ratio <= 0.0 {
            self.final_drive_ratio = 3.5;
        }
        self.steering_angle = 0.0;
        self.engine_torque = 0.0;
        self.brake_torque = 0.0;

        // Reset all wheels
        for wheel in &mut self.wheels {
            wheel.rotation_angle = 0.0;
            wheel.steering_angle = 0.0;
            wheel.angular_velocity = 0.0;
            wheel.drive_torque = 0.0;
            wheel.brake_torque = 0.0;
            wheel.suspension.reset_to_safe_state();
        }
    }
}
