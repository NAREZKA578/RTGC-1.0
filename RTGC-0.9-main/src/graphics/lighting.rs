//! Lighting System for RTGC-0.8
//! PBR освещение, Directional/Point/Spot lights, ACES Tone Mapping

use nalgebra::Vector3;

/// Типы источников света
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

/// Базовая структура источника света
#[derive(Debug, Clone)]
pub struct Light {
    pub light_type: LightType,
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub range: f32,
    
    // Для spot light
    pub spot_angle: f32,
    pub spot_falloff: f32,
}

impl Light {
    pub fn new_directional(direction: Vector3<f32>, color: Vector3<f32>, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional,
            position: Vector3::zeros(),
            direction: direction.normalize(),
            color,
            intensity,
            range: f32::INFINITY,
            spot_angle: 0.0,
            spot_falloff: 0.0,
        }
    }
    
    pub fn new_point(position: Vector3<f32>, color: Vector3<f32>, intensity: f32, range: f32) -> Self {
        Self {
            light_type: LightType::Point,
            position,
            direction: Vector3::y(),
            color,
            intensity,
            range,
            spot_angle: 0.0,
            spot_falloff: 0.0,
        }
    }
    
    pub fn new_spot(
        position: Vector3<f32>,
        direction: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
        range: f32,
        spot_angle: f32,
        spot_falloff: f32,
    ) -> Self {
        Self {
            light_type: LightType::Spot,
            position,
            direction: direction.normalize(),
            color,
            intensity,
            range,
            spot_angle,
            spot_falloff,
        }
    }
}

/// PBR материалы
#[derive(Debug, Clone)]
pub struct PbrMaterial {
    pub albedo: Vector3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub ao: f32,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            albedo: Vector3::new(1.0, 1.0, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            ao: 1.0,
        }
    }
}

/// ACES Tone Mapping
pub fn aces_tonemap(color: Vector3<f32>) -> Vector3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    
    Vector3::new(
        ((color.x * (a * color.x + b)) / (color.x * (c * color.x + d) + e)).clamp(0.0, 1.0),
        ((color.y * (a * color.y + b)) / (color.y * (c * color.y + d) + e)).clamp(0.0, 1.0),
        ((color.z * (a * color.z + b)) / (color.z * (c * color.z + d) + e)).clamp(0.0, 1.0),
    )
}

/// Расчёт освещения по модели PBR
pub fn calculate_pbr_lighting(
    material: &PbrMaterial,
    lights: &[Light],
    normal: Vector3<f32>,
    view_dir: Vector3<f32>,
    world_pos: Vector3<f32>,
) -> Vector3<f32> {
    let mut final_color = Vector3::zeros();
    let n = normal.normalize();
    let v = view_dir.normalize();
    
    for light in lights {
        let l = match light.light_type {
            LightType::Directional => -light.direction,
            LightType::Point | LightType::Spot => {
                (light.position - world_pos).normalize()
            }
        };
        
        let h = (l + v).normalize();
        
        // Расстояние затухания
        let distance = match light.light_type {
            LightType::Directional => 1.0,
            LightType::Point | LightType::Spot => {
                let dist = (light.position - world_pos).norm();
                if dist > light.range {
                    continue;
                }
                1.0 / (dist * dist)
            }
        };
        
        // Spot light attenuation
        let spot_attenuation = if light.light_type == LightType::Spot {
            let theta = l.dot(&light.direction);
            let cos_inner = light.spot_angle.cos();
            let cos_outer = (light.spot_angle - light.spot_falloff).cos();
            let intensity = (theta - cos_outer) / (cos_inner - cos_outer);
            intensity.clamp(0.0, 1.0)
        } else {
            1.0
        };

        // Cook-Torrance BRDF
        let diff = cook_torrance_diffuse(&n, &l, &v, material);
        let spec = cook_torrance_specular(&n, &l, &v, &h, material);

        let radiance = light.color * light.intensity * distance * spot_attenuation;
        final_color += (diff + spec).component_mul(&radiance);
    }

    // Ambient term
    final_color += material.albedo.component_mul(&Vector3::new(0.03, 0.03, 0.03)) * material.ao;
    
    // Tone mapping
    aces_tonemap(final_color)
}

fn cook_torrance_diffuse(
    n: &Vector3<f32>,
    l: &Vector3<f32>,
    _v: &Vector3<f32>,
    material: &PbrMaterial,
) -> Vector3<f32> {
    let n_dot_l = n.dot(l).max(0.0);
    material.albedo / std::f32::consts::PI * n_dot_l * (1.0 - material.metallic)
}

fn cook_torrance_specular(
    n: &Vector3<f32>,
    l: &Vector3<f32>,
    v: &Vector3<f32>,
    h: &Vector3<f32>,
    material: &PbrMaterial,
) -> Vector3<f32> {
    let n_dot_l = n.dot(l).max(0.0);
    let n_dot_h = n.dot(h).max(0.0);
    let n_dot_v = n.dot(v).max(0.0);
    let h_dot_v = h.dot(v).max(0.0);
    
    if n_dot_l <= 0.0 || n_dot_v <= 0.0 {
        return Vector3::zeros();
    }
    
    // F0 для диэлектриков и металлов
    let f0 = Vector3::new(0.04, 0.04, 0.04) * (1.0 - material.metallic) + material.albedo * material.metallic;
    
    // Fresnel
    let f = fresnel_schlick(f0, h_dot_v);
    
    // Distribution (GGX)
    let d = distribution_ggx(n_dot_h, material.roughness);
    
    // Geometry
    let g = geometry_smith(n, v, l, material.roughness);
    
    let numerator = d * f * g;
    let denominator = 4.0 * n_dot_v * n_dot_l;
    
    numerator / denominator.max(0.001)
}

fn fresnel_schlick(f0: Vector3<f32>, x: f32) -> Vector3<f32> {
    f0 + (Vector3::new(1.0, 1.0, 1.0) - f0) * (1.0 - x).powi(5)
}

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let n_dot_h2 = n_dot_h * n_dot_h;
    
    let num = a2;
    let denom = n_dot_h2 * (a2 - 1.0) + 1.0;
    num / (std::f32::consts::PI * denom * denom)
}

fn geometry_smith(n: &Vector3<f32>, v: &Vector3<f32>, l: &Vector3<f32>, roughness: f32) -> f32 {
    geometry_schlick_ggx(n.dot(v), roughness) * geometry_schlick_ggx(n.dot(l), roughness)
}

fn geometry_schlick_ggx(n_dot_x: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    n_dot_x / (n_dot_x * (1.0 - k) + k)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aces_tonemap() {
        let color = Vector3::new(10.0, 5.0, 2.0);
        let result = aces_tonemap(color);
        assert!(result.x <= 1.0 && result.y <= 1.0 && result.z <= 1.0);
        assert!(result.x >= 0.0 && result.y >= 0.0 && result.z >= 0.0);
    }
    
    #[test]
    fn test_light_creation() {
        let dir_light = Light::new_directional(Vector3::y(), Vector3::x(), 1.0);
        assert_eq!(dir_light.light_type, LightType::Directional);
        
        let point_light = Light::new_point(Vector3::zeros(), Vector3::ones(), 100.0, 50.0);
        assert_eq!(point_light.light_type, LightType::Point);
    }
}
