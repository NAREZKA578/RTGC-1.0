//! Placeholder for fracture/destruction component
use crate::physics::{RigidBody, Shape};
use nalgebra::{Vector3, Matrix3};
use rand::Rng;

/// Abstract component for destructible objects
/// This follows the approach described in the README where "zаглушек и пометок" (placeholders and markers)
/// are created for future implementation of destruction systems
#[derive(Debug, Clone)]
pub struct FractureComponent {
    /// Whether this object can be fractured
    pub can_fracture: bool,

    /// Strength threshold for fracturing (in Newtons or Joules)
    pub strength_threshold: f32,

    /// Fragment geometry information
    pub fragments: Vec<Fragment>,

    /// Material properties affecting fracture behavior
    pub material_properties: MaterialProperties,

    /// Current structural integrity (0.0 = completely destroyed, 1.0 = intact)
    pub structural_integrity: f32,
}

impl FractureComponent {
    /// Validate that all fracture state is finite and safe
    pub fn validate_state(&self) -> bool {
        // Check strength threshold is valid
        if !self.strength_threshold.is_finite() || self.strength_threshold <= 0.0 {
            return false;
        }

        // Check structural integrity is valid
        if !self.structural_integrity.is_finite()
            || self.structural_integrity < 0.0
            || self.structural_integrity > 1.0
        {
            return false;
        }

        // Check material properties
        if !self.material_properties.density.is_finite()
            || !self.material_properties.elastic_modulus.is_finite()
        {
            return false;
        }

        true
    }

    /// Reset fracture component to a safe state
    pub fn reset_to_safe_state(&mut self) {
        tracing::warn!(target: "physics", "Resetting fracture component to safe state");
        self.structural_integrity = 1.0;
        self.fragments.clear();
    }

    pub fn new(strength_threshold: f32) -> Self {
        Self {
            can_fracture: true,
            strength_threshold,
            fragments: Vec::new(),
            material_properties: MaterialProperties::default(),
            structural_integrity: 1.0,
        }
    }

    /// Check if the object should fracture based on applied force
    pub fn should_fracture(&self, impact_force: f32) -> bool {
        // Validate impact force to prevent NaN/Inf
        if !impact_force.is_finite() {
            tracing::warn!(target: "physics", "Invalid impact force in fracture check: {}, returning false", impact_force);
            return false;
        }
        impact_force > self.strength_threshold * self.structural_integrity
    }

    /// Apply damage to the component
    pub fn apply_damage(&mut self, damage: f32) {
        // Validate damage to prevent NaN/Inf propagation
        if !damage.is_finite() || damage < 0.0 {
            tracing::warn!(target: "physics", "Invalid damage value: {}, skipping damage application", damage);
            return;
        }
        // Reduce structural integrity based on damage
        let damage_factor = damage / self.strength_threshold;
        self.structural_integrity = (self.structural_integrity - damage_factor).max(0.0);
    }

    /// Generate fragments when fracturing occurs
    pub fn generate_fragments(&self, original_body: &RigidBody) -> Vec<RigidBody> {
        // Validate structural integrity before generating fragments
        if !self.structural_integrity.is_finite() {
            tracing::warn!(target: "physics", "Invalid structural integrity: {}, cannot generate fragments", self.structural_integrity);
            return vec![];
        }

        if !self.can_fracture || self.fragments.is_empty() {
            return vec![];
        }

        let mut fragments = Vec::new();
        let mut rng = rand::thread_rng();

        // Create fragment rigid bodies based on stored fragment data
        for fragment_info in &self.fragments {
            let mut fragment_body = RigidBody {
                position: original_body.position + fragment_info.offset,
                rotation: original_body.rotation,
                velocity: original_body.velocity + fragment_info.velocity_offset,
                angular_velocity: original_body.angular_velocity,
                mass: fragment_info.mass,
                inverse_mass: if fragment_info.mass > 0.0 {
                    1.0 / fragment_info.mass
                } else {
                    0.0
                },
                inertia_tensor: Matrix3::identity() * fragment_info.mass * 0.1,
                inverse_inertia_tensor: Matrix3::identity() / (fragment_info.mass * 0.1),
                restitution: original_body.restitution,
                friction: original_body.friction,
                linear_damping: original_body.linear_damping,
                angular_damping: original_body.angular_damping,
                shape: fragment_info.shape.clone(),
                is_static: false,
                bounds: crate::physics::Aabb::new(
                    Vector3::new(-0.1, -0.1, -0.1),
                    Vector3::new(0.1, 0.1, 0.1),
                ),
                forces: Vector3::zeros(),
                torques: Vector3::zeros(),
                center_of_mass: Vector3::zeros(),
                drag_coefficient: original_body.drag_coefficient,
                lift_coefficient: original_body.lift_coefficient,
                reference_area: original_body.reference_area,
                idle_timer: 0.0,
                is_sleeping: false,
                collision_layer: original_body.collision_layer,
                collision_mask: original_body.collision_mask,
                is_trigger: false,
                enable_ccd: false,
            };

            // Add some initial velocity to make fragments fly apart
            fragment_body.velocity += Vector3::new(
                (rng.r#gen::<f32>() - 0.5) * 2.0,
                rng.r#gen::<f32>(),
                (rng.r#gen::<f32>() - 0.5) * 2.0,
            ) * 2.0; // Scale factor for initial velocity

            fragments.push(fragment_body);
        }

        fragments
    }
}

#[derive(Debug, Clone)]
pub struct Fragment {
    /// Offset from original object center
    pub offset: Vector3<f32>,

    /// Additional velocity when separated
    pub velocity_offset: Vector3<f32>,

    /// Mass of this fragment
    pub mass: f32,

    /// Shape of the fragment
    pub shape: Shape,

    /// Optional reference to child fragments (for recursive fracturing)
    pub children: Vec<Fragment>,
}

#[derive(Debug, Clone)]
pub struct MaterialProperties {
    /// Density in kg/m³
    pub density: f32,

    /// Elastic modulus (Young's modulus) in Pa
    pub elastic_modulus: f32,

    /// Poisson's ratio (dimensionless)
    pub poissons_ratio: f32,

    /// Ultimate tensile strength in Pa
    pub ultimate_tensile_strength: f32,

    /// Compressive strength in Pa
    pub compressive_strength: f32,

    /// Shear strength in Pa
    pub shear_strength: f32,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            density: 2700.0,       // Aluminum-like density
            elastic_modulus: 70e9, // 70 GPa for aluminum
            poissons_ratio: 0.33,
            ultimate_tensile_strength: 310e6, // 310 MPa
            compressive_strength: 310e6,
            shear_strength: 200e6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_fracture_component_creation() {
        let fracture_comp = FractureComponent::new(1000.0);

        assert_eq!(fracture_comp.strength_threshold, 1000.0);
        assert!(fracture_comp.can_fracture);
    }

    #[test]
    fn test_should_fracture() {
        let fracture_comp = FractureComponent::new(1000.0);

        assert!(fracture_comp.should_fracture(1500.0));
        assert!(!fracture_comp.should_fracture(500.0));
    }

    #[test]
    fn test_fragment_generation() {
        let mut fracture_comp = FractureComponent::new(1000.0);

        // Add a simple fragment
        fracture_comp.fragments.push(Fragment {
            offset: Vector3::new(1.0, 0.0, 0.0),
            velocity_offset: Vector3::new(0.0, 1.0, 0.0),
            mass: 10.0,
            shape: Shape::Sphere { radius: 0.5 },
            children: vec![],
        });

        let original_body = RigidBody::new_sphere(Vector3::new(0.0, 5.0, 0.0), 100.0, 2.0);
        let fragments = fracture_comp.generate_fragments(&original_body);

        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].mass, 10.0);
    }
}
