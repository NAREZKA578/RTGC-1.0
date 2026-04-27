//! Vehicle Parts System - Damage, wear, and integrity tracking
//! Implements detailed component tree for vehicles with damage propagation

use serde::{Deserialize, Serialize};
use crate::graphics::renderer::commands::RenderCommand;

/// Maximum integrity value (100%)
pub const MAX_INTEGRITY: f32 = 100.0;

/// Minimum integrity for functional part
pub const MIN_FUNCTIONAL_INTEGRITY: f32 = 20.0;

/// Part category for grouping components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartCategory {
    Engine,
    Transmission,
    Drivetrain,
    Frame,
    Suspension,
    Wheels,
    Brakes,
    Body,
    Electrical,
    Fuel,
    Steering,
}

impl std::fmt::Display for PartCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartCategory::Engine => write!(f, "Двигатель"),
            PartCategory::Transmission => write!(f, "КПП"),
            PartCategory::Drivetrain => write!(f, "Раздатка/привод"),
            PartCategory::Frame => write!(f, "Рама/кузов"),
            PartCategory::Suspension => write!(f, "Подвеска"),
            PartCategory::Wheels => write!(f, "Колёса"),
            PartCategory::Brakes => write!(f, "Тормоза"),
            PartCategory::Body => write!(f, "Кузов"),
            PartCategory::Electrical => write!(f, "Электрика"),
            PartCategory::Fuel => write!(f, "Топливная система"),
            PartCategory::Steering => write!(f, "Рулевое"),
        }
    }
}

/// Individual vehicle part with integrity tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehiclePart {
    /// Unique part identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Category for grouping
    pub category: PartCategory,
    /// Current integrity (0-100)
    pub integrity: f32,
    /// Maximum integrity (can be reduced by frame damage)
    pub max_integrity: f32,
    /// Wear rate multiplier (1.0 = normal, 2.0 = double wear)
    pub wear_multiplier: f32,
    /// Whether this part is critical for vehicle operation
    pub is_critical: bool,
    /// Replacement cost in rubles
    pub replacement_cost: f32,
    /// Expected lifetime in hours (for wear calculation)
    pub expected_lifetime_hours: f32,
}

impl VehiclePart {
    /// Create a new vehicle part
    pub fn new(
        id: &str,
        name: &str,
        category: PartCategory,
        integrity: f32,
        is_critical: bool,
        replacement_cost: f32,
        expected_lifetime_hours: f32,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category,
            integrity: integrity.clamp(0.0, MAX_INTEGRITY),
            max_integrity: MAX_INTEGRITY,
            wear_multiplier: 1.0,
            is_critical,
            replacement_cost,
            expected_lifetime_hours,
        }
    }

    /// Check if part is functional
    pub fn is_functional(&self) -> bool {
        self.integrity >= MIN_FUNCTIONAL_INTEGRITY
    }

    /// Get integrity as percentage
    pub fn integrity_percent(&self) -> f32 {
        (self.integrity / self.max_integrity) * 100.0
    }

    /// Apply damage to part
    pub fn apply_damage(&mut self, damage: f32) {
        self.integrity = (self.integrity - damage).clamp(0.0, self.max_integrity);
    }

    /// Apply wear over time
    pub fn apply_wear(&mut self, dt_hours: f32, severity: f32) {
        let wear_rate = (1.0 / self.expected_lifetime_hours) * self.wear_multiplier * severity;
        let wear_amount = wear_rate * dt_hours * MAX_INTEGRITY;
        self.apply_damage(wear_amount);
    }

    /// Repair part (returns actual repair amount)
    pub fn repair(&mut self, amount: f32) -> f32 {
        let old_integrity = self.integrity;
        self.integrity = (self.integrity + amount).clamp(0.0, self.max_integrity);
        self.integrity - old_integrity
    }

    /// Reduce max integrity (for frame damage)
    pub fn reduce_max_integrity(&mut self, reduction: f32) {
        self.max_integrity = (self.max_integrity - reduction).clamp(0.0, MAX_INTEGRITY);
        if self.integrity > self.max_integrity {
            self.integrity = self.max_integrity;
        }
    }

    /// Get diagnostic info based on mechanic skill rank
    pub fn get_diagnostic_info(&self, mechanic_rank: u8) -> PartDiagnostic {
        let accuracy = (mechanic_rank as f32 / 12.0).clamp(0.1, 1.0);

        PartDiagnostic {
            name: self.name.clone(),
            integrity_display: if accuracy >= 0.9 {
                format!("{:.1}%", self.integrity_percent())
            } else if accuracy >= 0.7 {
                format!("{:.0}%", self.integrity_percent())
            } else if accuracy >= 0.5 {
                self.get_integrity_range()
            } else {
                self.get_integrity_category()
            },
            condition: self.get_condition(accuracy),
            needs_replacement: self.integrity < MIN_FUNCTIONAL_INTEGRITY,
            estimated_cost: if accuracy >= 0.8 {
                Some(self.replacement_cost)
            } else {
                None
            },
        }
    }

    fn get_integrity_range(&self) -> String {
        let percent = self.integrity_percent();
        if percent >= 80.0 {
            "80-100%".to_string()
        } else if percent >= 60.0 {
            "60-79%".to_string()
        } else if percent >= 40.0 {
            "40-59%".to_string()
        } else if percent >= 20.0 {
            "20-39%".to_string()
        } else {
            "<20%".to_string()
        }
    }

    fn get_integrity_category(&self) -> String {
        let percent = self.integrity_percent();
        if percent >= 70.0 {
            "Отличное".to_string()
        } else if percent >= 50.0 {
            "Хорошее".to_string()
        } else if percent >= 30.0 {
            "Среднее".to_string()
        } else if percent >= 15.0 {
            "Плохое".to_string()
        } else {
            "Критическое".to_string()
        }
    }

    fn get_condition(&self, accuracy: f32) -> String {
        if accuracy < 0.5 {
            return "Неизвестно".to_string();
        }

        let percent = self.integrity_percent();
        if percent >= 90.0 {
            "Новое".to_string()
        } else if percent >= 75.0 {
            "Отличное".to_string()
        } else if percent >= 60.0 {
            "Хорошее".to_string()
        } else if percent >= 45.0 {
            "Нормальное".to_string()
        } else if percent >= 30.0 {
            "Изношенное".to_string()
        } else if percent >= 20.0 {
            "Сильно изношенное".to_string()
        } else {
            "Требует замены".to_string()
        }
    }
}

/// Diagnostic information for a part
#[derive(Debug, Clone)]
pub struct PartDiagnostic {
    pub name: String,
    pub integrity_display: String,
    pub condition: String,
    pub needs_replacement: bool,
    pub estimated_cost: Option<f32>,
}

/// Complete vehicle parts system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehiclePartsSystem {
    /// All parts indexed by ID
    pub parts: std::collections::HashMap<String, VehiclePart>,
    /// Parts grouped by category
    pub by_category: std::collections::HashMap<PartCategory, Vec<String>>,
    /// Critical parts (engine, brakes, steering, etc.)
    pub critical_parts: Vec<String>,
    /// Frame integrity (affects max_integrity of attached parts)
    pub frame_integrity: f32,
    pub frame_max_integrity: f32,
}

impl VehiclePartsSystem {
    /// Create a new parts system for a vehicle
    pub fn new() -> Self {
        Self {
            parts: std::collections::HashMap::new(),
            by_category: std::collections::HashMap::new(),
            critical_parts: Vec::new(),
            frame_integrity: MAX_INTEGRITY,
            frame_max_integrity: MAX_INTEGRITY,
        }
    }

    /// Add a part to the system
    pub fn add_part(&mut self, part: VehiclePart) {
        if part.is_critical {
            self.critical_parts.push(part.id.clone());
        }

        let category = part.category;
        let part_id = part.id.clone();

        self.parts.insert(part_id.clone(), part);

        self.by_category
            .entry(category)
            .or_insert_with(Vec::new)
            .push(part_id);
    }

    /// Get part by ID
    pub fn get_part(&self, part_id: &str) -> Option<&VehiclePart> {
        self.parts.get(part_id)
    }

    /// Get mutable part by ID
    pub fn get_part_mut(&mut self, part_id: &str) -> Option<&mut VehiclePart> {
        self.parts.get_mut(part_id)
    }

    /// Get all parts in a category
    pub fn get_parts_by_category(&self, category: PartCategory) -> Vec<&VehiclePart> {
        self.by_category
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.parts.get(id)).collect())
            .unwrap_or_default()
    }

    /// Apply collision damage to nearest parts
    pub fn apply_collision_damage(
        &mut self,
        force: f32,
        impact_point: &str,
        damage_vector: nalgebra::Vector3<f32>,
    ) {
        let base_damage = (force / 1000.0).min(50.0); // Cap at 50% damage

        // Damage distribution based on impact location
        let affected_categories = self.get_affected_categories(impact_point);

        for category in affected_categories {
            if let Some(part_ids) = self.by_category.get(&category) {
                for part_id in part_ids {
                    // Get damage multiplier first to avoid borrow conflicts
                    let multiplier = if let Some(part) = self.parts.get(part_id) {
                        self.get_damage_multiplier(part, impact_point, &damage_vector)
                    } else {
                        1.0
                    };

                    if let Some(part) = self.parts.get_mut(part_id) {
                        let damage = base_damage * multiplier;
                        part.apply_damage(damage);

                        // Frame damage reduces max_integrity permanently
                        if category == PartCategory::Frame && force > 5000.0 {
                            let max_reduction = base_damage * 0.3;
                            part.reduce_max_integrity(max_reduction);
                            self.frame_integrity = (self.frame_integrity - max_reduction).max(0.0);
                        }
                    }
                }
            }
        }
    }

    fn get_affected_categories(&self, impact_point: &str) -> Vec<PartCategory> {
        match impact_point {
            "front" => vec![
                PartCategory::Body,
                PartCategory::Engine,
                PartCategory::Suspension,
                PartCategory::Wheels,
            ],
            "rear" => vec![
                PartCategory::Body,
                PartCategory::Fuel,
                PartCategory::Suspension,
                PartCategory::Wheels,
            ],
            "left" => vec![
                PartCategory::Body,
                PartCategory::Suspension,
                PartCategory::Wheels,
                PartCategory::Steering,
            ],
            "right" => vec![
                PartCategory::Body,
                PartCategory::Suspension,
                PartCategory::Wheels,
                PartCategory::Steering,
            ],
            "top" => vec![PartCategory::Body],
            "bottom" => vec![
                PartCategory::Frame,
                PartCategory::Suspension,
                PartCategory::Drivetrain,
            ],
            _ => vec![PartCategory::Body],
        }
    }

    fn get_damage_multiplier(
        &self,
        part: &VehiclePart,
        impact_point: &str,
        _damage_vector: &nalgebra::Vector3<f32>,
    ) -> f32 {
        // Simplified damage multiplier based on part location and type
        let base_multiplier = match part.category {
            PartCategory::Body => 1.0,
            PartCategory::Engine | PartCategory::Transmission => 0.7,
            PartCategory::Suspension | PartCategory::Wheels => 1.2,
            PartCategory::Frame => 0.5,
            _ => 0.8,
        };

        // Direct hits do more damage
        let direct_hit = match (part.category, impact_point) {
            (PartCategory::Engine, "front") => true,
            (PartCategory::Fuel, "rear") => true,
            (PartCategory::Wheels, "left") | (PartCategory::Wheels, "right") => true,
            _ => false,
        };

        if direct_hit {
            base_multiplier * 1.5
        } else {
            base_multiplier
        }
    }

    /// Apply wear to all parts based on usage
    pub fn apply_wear(&mut self, dt_hours: f32, surface_severity: f32, rpm_load: f32) {
        let engine_load = (rpm_load / 6000.0).clamp(0.5, 2.0);

        for part in self.parts.values_mut() {
            let severity = match part.category {
                PartCategory::Engine | PartCategory::Transmission => surface_severity * engine_load,
                PartCategory::Wheels | PartCategory::Brakes | PartCategory::Suspension => {
                    surface_severity
                }
                PartCategory::Frame => surface_severity * 0.3,
                _ => surface_severity * 0.5,
            };

            part.apply_wear(dt_hours, severity);
        }
    }

    /// Diagnose all parts based on mechanic skill
    pub fn diagnose(&self, mechanic_rank: u8) -> Vec<PartDiagnostic> {
        self.parts
            .values()
            .map(|part| part.get_diagnostic_info(mechanic_rank))
            .collect()
    }

    /// Calculate engine power modifier based on engine integrity
    pub fn get_engine_power_modifier(&self) -> f32 {
        let engine_parts = self.get_parts_by_category(PartCategory::Engine);
        if engine_parts.is_empty() {
            return 1.0;
        }

        let avg_integrity = engine_parts
            .iter()
            .map(|p| p.integrity_percent() / 100.0)
            .sum::<f32>()
            / engine_parts.len() as f32;

        avg_integrity.clamp(0.0, 1.0)
    }

    /// Calculate tire grip modifier
    pub fn get_tire_grip_modifier(&self) -> f32 {
        let wheel_parts = self.get_parts_by_category(PartCategory::Wheels);
        if wheel_parts.is_empty() {
            return 1.0;
        }

        let min_tire_integrity = wheel_parts
            .iter()
            .map(|p| p.integrity_percent())
            .fold(100.0_f32, |min: f32, val| min.min(val));

        if min_tire_integrity < 20.0 {
            0.4 // Severely reduced grip
        } else if min_tire_integrity < 40.0 {
            0.7
        } else if min_tire_integrity < 60.0 {
            0.85
        } else {
            1.0
        }
    }

    /// Calculate braking efficiency modifier
    pub fn get_braking_modifier(&self) -> f32 {
        let brake_parts = self.get_parts_by_category(PartCategory::Brakes);
        if brake_parts.is_empty() {
            return 1.0;
        }

        let avg_integrity = brake_parts
            .iter()
            .map(|p| p.integrity_percent() / 100.0)
            .sum::<f32>()
            / brake_parts.len() as f32;

        if avg_integrity < 0.3 {
            0.5 // Doubled braking distance
        } else if avg_integrity < 0.5 {
            0.75
        } else {
            avg_integrity.clamp(0.5, 1.0)
        }
    }

    /// Calculate handling modifier based on suspension integrity
    pub fn get_handling_modifier(&self) -> f32 {
        let suspension_parts = self.get_parts_by_category(PartCategory::Suspension);
        if suspension_parts.is_empty() {
            return 1.0;
        }

        let avg_integrity = suspension_parts
            .iter()
            .map(|p| p.integrity_percent() / 100.0)
            .sum::<f32>()
            / suspension_parts.len() as f32;

        if avg_integrity < 0.4 {
            0.6 // Much harder to control
        } else if avg_integrity < 0.6 {
            0.8
        } else {
            avg_integrity.clamp(0.6, 1.0)
        }
    }

    /// Check if vehicle is operational
    pub fn is_operational(&self) -> bool {
        // Check all critical parts
        for part_id in &self.critical_parts {
            if let Some(part) = self.parts.get(part_id) {
                if !part.is_functional() {
                    return false;
                }
            }
        }

        // Check frame integrity
        if self.frame_integrity < 10.0 {
            return false;
        }

        true
    }

    /// Проблема 15: Deform mesh based on part damage
    /// Возвращает смещения вершин для визуальной деформации
    pub fn deform_mesh(
        &self,
        impact_point: &str,
        damage_amount: f32,
    ) -> Vec<(usize, nalgebra::Vector3<f32>)> {
        let mut deformations = Vec::new();

        // Определяем зону повреждения
        let affected_vertices = self.get_affected_vertices(impact_point);

        // Применяем деформацию к вершинам
        for (vertex_idx, base_offset) in &affected_vertices {
            // Сила деформации зависит от повреждения
            let deform_strength = damage_amount / 100.0;

            // Направление деформации (внутрь меша)
            let deform_direction = match impact_point {
                "front" => nalgebra::Vector3::new(0.0, 0.0, -1.0),
                "rear" => nalgebra::Vector3::new(0.0, 0.0, 1.0),
                "left" => nalgebra::Vector3::new(-1.0, 0.0, 0.0),
                "right" => nalgebra::Vector3::new(1.0, 0.0, 0.0),
                "top" => nalgebra::Vector3::new(0.0, -1.0, 0.0),
                "bottom" => nalgebra::Vector3::new(0.0, 1.0, 0.0),
                _ => nalgebra::Vector3::new(0.0, 0.0, 0.0),
            };

            // Добавляем случайность для реалистичности
            let random_factor = ((*vertex_idx % 10) as f32 / 10.0).sin().abs();
            let offset = deform_direction * deform_strength * (0.5 + random_factor * 0.5);

            deformations.push((*vertex_idx, offset));
        }

        deformations
    }

    /// Get vertices affected by damage at specific point
    fn get_affected_vertices(&self, impact_point: &str) -> Vec<(usize, nalgebra::Vector3<f32>)> {
        // Упрощённая реализация - в реальном проекте использовать UV координаты
        match impact_point {
            "front" => vec![
                (0, nalgebra::Vector3::new(0.0, 0.0, 0.0)),
                (1, nalgebra::Vector3::new(0.1, 0.0, 0.0)),
                (2, nalgebra::Vector3::new(0.0, 0.1, 0.0)),
                (3, nalgebra::Vector3::new(0.0, 0.0, 0.1)),
            ],
            "rear" => vec![
                (4, nalgebra::Vector3::new(0.0, 0.0, 0.0)),
                (5, nalgebra::Vector3::new(-0.1, 0.0, 0.0)),
            ],
            "left" => vec![
                (6, nalgebra::Vector3::new(0.0, 0.0, 0.0)),
                (7, nalgebra::Vector3::new(0.0, 0.1, 0.0)),
            ],
            "right" => vec![
                (8, nalgebra::Vector3::new(0.0, 0.0, 0.0)),
                (9, nalgebra::Vector3::new(0.0, 0.0, 0.1)),
            ],
            _ => vec![],
        }
    }

    /// Apply mesh deformation to render command
    pub fn apply_to_render_command(
        &self,
        command: &mut RenderCommand,
        impact_point: &str,
        damage: f32,
    ) {
        let deformations = self.deform_mesh(impact_point, damage);

        // Применяем деформации к RenderCommand для передачи в рендерер
        if !deformations.is_empty() {
            // Добавляем информацию о деформациях в команду рендеринга
            // Рендерер использует эти данные для модификации вершин меша
            for (vertex_idx, offset) in deformations.iter() {
                command.add_vertex_displacement(*vertex_idx, *offset);
            }
            tracing::debug!(
                target: "vehicle",
                "Applied {} deformations to mesh at impact point '{}'",
                deformations.len(),
                impact_point
            );
        } else {
            tracing::trace!(
                target: "vehicle",
                "No deformations generated for impact point '{}' with damage {}",
                impact_point,
                damage
            );
        }
    }

    /// Get total repair cost for all damaged parts
    pub fn get_total_repair_cost(&self) -> f32 {
        self.parts
            .values()
            .filter(|p| p.integrity < p.max_integrity)
            .map(|p| {
                let damage_ratio = 1.0 - (p.integrity / p.max_integrity);
                p.replacement_cost * damage_ratio
            })
            .sum()
    }
}

impl Default for VehiclePartsSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_creation() {
        let part = VehiclePart::new(
            "engine_block",
            "Блок цилиндров",
            PartCategory::Engine,
            85.0,
            true,
            50000.0,
            10000.0,
        );

        assert_eq!(part.integrity, 85.0);
        assert!(part.is_functional());
        assert_eq!(part.integrity_percent(), 85.0);
    }

    #[test]
    fn test_damage_application() {
        let mut part = VehiclePart::new(
            "tire_fl",
            "Шина передняя левая",
            PartCategory::Wheels,
            100.0,
            false,
            8000.0,
            50000.0,
        );

        part.apply_damage(25.0);
        assert_eq!(part.integrity, 75.0);
        assert!(part.is_functional());

        part.apply_damage(60.0);
        assert_eq!(part.integrity, 15.0);
        assert!(!part.is_functional());
    }

    #[test]
    fn test_wear_application() {
        let mut part = VehiclePart::new(
            "brake_pads_front",
            "Тормозные колодки передние",
            PartCategory::Brakes,
            100.0,
            false,
            3000.0,
            20000.0, // 20k hours lifetime
        );

        // Apply 100 hours of wear with normal severity
        part.apply_wear(100.0, 1.0);

        // Should have worn down proportionally
        assert!(part.integrity < 100.0);
        assert!(part.integrity > 90.0); // Should still be mostly good
    }
}
