//! Math utilities for RTGC-0.8
//! Provides common mathematical functions: lerp, slerp, clamping, etc.

use nalgebra::{Vector3, UnitQuaternion};

/// Linear interpolation between two values
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Linear interpolation for Vector3
#[inline]
pub fn lerp_vec3(a: Vector3<f32>, b: Vector3<f32>, t: f32) -> Vector3<f32> {
    a.lerp(&b, t.clamp(0.0, 1.0))
}

/// Spherical linear interpolation for quaternions
#[inline]
pub fn slerp(a: UnitQuaternion<f32>, b: UnitQuaternion<f32>, t: f32) -> UnitQuaternion<f32> {
    a.slerp(&b, t.clamp(0.0, 1.0))
}

/// Clamp a value between min and max
#[inline]
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Clamp f32 specifically with NaN handling
#[inline]
pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    if value.is_nan() {
        return min;
    }
    value.clamp(min, max)
}

/// Smooth step interpolation (Hermite)
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smoother step interpolation (Ken Perlin)
#[inline]
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Convert degrees to radians
#[inline]
pub fn deg_to_rad(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

/// Convert radians to degrees
#[inline]
pub fn rad_to_deg(radians: f32) -> f32 {
    radians * 180.0 / std::f32::consts::PI
}

/// Calculate dot product sign (returns -1, 0, or 1)
#[inline]
pub fn signum(value: f32) -> f32 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// Normalize angle to [-PI, PI] range
#[inline]
pub fn normalize_angle(angle: f32) -> f32 {
    let mut result = angle % (2.0 * std::f32::consts::PI);
    if result > std::f32::consts::PI {
        result -= 2.0 * std::f32::consts::PI;
    } else if result < -std::f32::consts::PI {
        result += 2.0 * std::f32::consts::PI;
    }
    result
}

/// Calculate distance between two points
#[inline]
pub fn distance(a: Vector3<f32>, b: Vector3<f32>) -> f32 {
    (a - b).norm()
}

/// Calculate squared distance (faster, avoids sqrt)
#[inline]
pub fn distance_squared(a: Vector3<f32>, b: Vector3<f32>) -> f32 {
    (a - b).norm_squared()
}

/// Check if two floats are approximately equal
#[inline]
pub fn approx_equal(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_deg_to_rad() {
        assert!(approx_equal(deg_to_rad(180.0), std::f32::consts::PI, 0.0001));
        assert!(approx_equal(deg_to_rad(90.0), std::f32::consts::PI / 2.0, 0.0001));
    }

    #[test]
    fn test_normalize_angle() {
        assert!(approx_equal(normalize_angle(0.0), 0.0, 0.0001));
        assert!(approx_equal(normalize_angle(std::f32::consts::PI * 2.0), 0.0, 0.0001));
        assert!(approx_equal(normalize_angle(-std::f32::consts::PI * 1.5), std::f32::consts::PI / 2.0, 0.0001));
    }
}
