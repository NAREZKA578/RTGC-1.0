#[derive(Debug, Clone, Default)]
pub struct VehicleHudData {
    pub speed_kmh: f32,
    pub speed_max_kmh: f32,
    pub gear: GearState,
    pub engine_rpm: f32,
    pub engine_rpm_max: f32,
    pub engine_running: bool,
    pub fuel_level: f32,
    pub fuel_reserve: bool,
    pub engine_temp: f32,
    pub engine_overheating: bool,
    pub diff_front_locked: bool,
    pub diff_rear_locked: bool,
    pub diff_center_locked: bool,
    pub awd_active: bool,
    pub low_range: bool,
    pub wheel_contact: [bool; 4],
    pub wheel_slip: [f32; 4],
    pub suspension_load: [f32; 4],
    pub cargo_attached: bool,
    pub cargo_weight_kg: f32,
    pub cargo_damage: f32,
    pub winch_active: bool,
    pub winch_length_m: f32,
    pub altitude_m: f32,
    pub terrain_angle_deg: f32,
    pub vehicle_roll_deg: f32,
    pub vehicle_pitch_deg: f32,
    pub is_tipped_over: bool,
    pub vehicle_health: f32,
    pub heading_degrees: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GearState {
    Park,
    Reverse,
    Neutral,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub struct HudLayout {
    pub show_wheel_status: bool,
    pub show_cargo: bool,
    pub show_compass: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HudFlashElement {
    Fuel,
    EngineTemp,
    Rpm,
    None,
}

pub struct HudManager {
    settings_enabled: bool,
}

impl Clone for HudManager {
    fn clone(&self) -> Self {
        Self { settings_enabled: self.settings_enabled }
    }
}

impl Default for HudManager {
    fn default() -> Self {
        Self { settings_enabled: true }
    }
}

impl HudManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, _data: &VehicleHudData, _layout: &HudLayout, _dt: f32) {
    }

    pub fn render(&mut self, _data: &VehicleHudData, _layout: &HudLayout, _screen_width: f32, _screen_height: f32) {
    }
}