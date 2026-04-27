//! Day/Night Cycle and Celestial Bodies
//!
//! Implements realistic day/night cycle with:
//! - Sun and moon positioning based on time
//! - Sunrise/sunset transitions
//! - Seasonal variations
//! - Lunar phases
//! - Star field visibility

use nalgebra::{Matrix4, Vector3};
use std::f32::consts::PI;

/// Celestial body type
#[derive(Debug, Clone, Copy)]
pub enum CelestialBody {
    Sun,
    Moon,
}

/// Celestial body state
#[derive(Debug, Clone)]
pub struct CelestialState {
    /// Position in sky (spherical coordinates converted to Cartesian)
    pub position: Vector3<f32>,
    /// Azimuth angle (0-360 degrees, radians)
    pub azimuth: f32,
    /// Altitude angle (-90 to 90 degrees, radians)
    pub altitude: f32,
    /// Intensity (0.0-1.0)
    pub intensity: f32,
    /// Color temperature (Kelvin)
    pub color_temperature: f32,
}

impl CelestialState {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 1.0, 0.0),
            azimuth: 0.0,
            altitude: PI / 2.0,
            intensity: 1.0,
            color_temperature: 6500.0, // Noon sunlight
        }
    }

    /// Convert spherical to Cartesian coordinates
    pub fn update_position(&mut self, azimuth: f32, altitude: f32, distance: f32) {
        self.azimuth = azimuth;
        self.altitude = altitude;

        self.position = Vector3::new(
            distance * altitude.cos() * azimuth.sin(),
            distance * altitude.sin(),
            distance * altitude.cos() * azimuth.cos(),
        );
    }
}

/// Moon phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonPhase {
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    FullMoon,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl MoonPhase {
    /// Get illumination factor (0.0 = new moon, 1.0 = full moon)
    pub fn illumination(&self) -> f32 {
        match self {
            MoonPhase::NewMoon => 0.0,
            MoonPhase::WaxingCrescent | MoonPhase::WaningCrescent => 0.25,
            MoonPhase::FirstQuarter | MoonPhase::LastQuarter => 0.5,
            MoonPhase::WaxingGibbous | MoonPhase::WaningGibbous => 0.75,
            MoonPhase::FullMoon => 1.0,
        }
    }
}

/// Time of day classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    Night,
    Dawn,
    Sunrise,
    Morning,
    Noon,
    Afternoon,
    Sunset,
    Dusk,
}

/// Main day/night cycle manager
#[derive(Clone)]
pub struct DayNightCycle {
    /// Current time of day in seconds (0-86400)
    pub current_time: f32,
    /// Day length in seconds (default 24 hours)
    pub day_duration: f32,
    /// Year day (0-365) for seasonal calculations
    pub year_day: u32,
    /// Latitude for sun angle calculations (-90 to 90 degrees)
    pub latitude: f32,
    /// Longitude for timezone offset (-180 to 180 degrees)
    pub longitude: f32,
    /// Sun state
    pub sun: CelestialState,
    /// Moon state
    pub moon: CelestialState,
    /// Current moon phase
    pub moon_phase: MoonPhase,
    /// Lunar cycle day (0-29.5)
    pub lunar_day: f32,
    /// Ambient light multiplier
    pub ambient_multiplier: f32,
    /// Sky color gradient
    pub sky_color_top: Vector3<f32>,
    pub sky_color_bottom: Vector3<f32>,
    /// Stars visibility (0.0-1.0)
    pub stars_visibility: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self::new(45.0, 0.0) // Default latitude 45°N
    }
}

impl DayNightCycle {
    /// Create new day/night cycle with given latitude and longitude
    pub fn new(latitude: f32, longitude: f32) -> Self {
        let mut cycle = Self {
            current_time: 12.0 * 3600.0, // Start at noon
            day_duration: 14400.0,       // 4 real hours = 1 game day (per GDD)
            year_day: 172,               // Summer solstice (approx)
            latitude,
            longitude,
            sun: CelestialState::new(),
            moon: CelestialState::new(),
            moon_phase: MoonPhase::FullMoon,
            lunar_day: 14.75,
            ambient_multiplier: 1.0,
            sky_color_top: Vector3::new(0.4, 0.6, 0.9),
            sky_color_bottom: Vector3::new(0.6, 0.7, 0.9),
            stars_visibility: 0.0,
        };

        cycle.update_celestial_bodies();
        cycle
    }

    /// Get current hour (0-24)
    pub fn get_hour(&self) -> f32 {
        (self.current_time / 3600.0) % 24.0
    }

    /// Get time of day classification
    pub fn get_time_of_day(&self) -> TimeOfDay {
        let hour = self.get_hour();

        if hour >= 21.0 || hour < 5.0 {
            TimeOfDay::Night
        } else if hour >= 5.0 && hour < 6.0 {
            TimeOfDay::Dawn
        } else if hour >= 6.0 && hour < 7.0 {
            TimeOfDay::Sunrise
        } else if hour >= 7.0 && hour < 11.0 {
            TimeOfDay::Morning
        } else if hour >= 11.0 && hour < 13.0 {
            TimeOfDay::Noon
        } else if hour >= 13.0 && hour < 17.0 {
            TimeOfDay::Afternoon
        } else if hour >= 17.0 && hour < 18.0 {
            TimeOfDay::Sunset
        } else {
            TimeOfDay::Dusk
        }
    }

    /// Check if sun is above horizon
    pub fn is_daytime(&self) -> bool {
        self.sun.altitude > 0.0
    }

    /// Calculate sun position based on time and location
    fn calculate_sun_position(&self) -> (f32, f32) {
        let hour_angle = (self.current_time / self.day_duration) * 2.0 * PI - PI / 2.0;

        // Simplified declination based on year day
        let declination = 0.409 * ((2.0 * PI / 365.0) * (self.year_day as f32 - 81.0)).sin();

        let lat_rad = self.latitude.to_radians();

        // Solar altitude
        let sin_altitude = lat_rad.sin() * declination.sin()
            + lat_rad.cos() * declination.cos() * hour_angle.cos();
        let altitude = sin_altitude.asin().clamp(-PI / 2.0, PI / 2.0);

        // Solar azimuth
        let cos_azimuth =
            (declination.sin() - lat_rad.sin() * sin_altitude) / (lat_rad.cos() * altitude.cos());
        let mut azimuth = cos_azimuth.acos().clamp(0.0, PI);

        if hour_angle.sin() > 0.0 {
            azimuth = 2.0 * PI - azimuth;
        }

        (azimuth, altitude)
    }

    /// Calculate moon position and phase
    fn calculate_moon_position(&self) -> (f32, f32, MoonPhase, f32) {
        // Moon rises about 50 minutes later each day
        let moon_offset = self.lunar_day * 50.0 * 60.0; // seconds
        let moon_time = (self.current_time + moon_offset) % self.day_duration;

        let hour_angle = (moon_time / self.day_duration) * 2.0 * PI - PI / 2.0;

        // Moon's declination varies more than sun's
        let moon_declination = 0.45 * ((2.0 * PI / 27.3) * self.lunar_day).sin();

        let lat_rad = self.latitude.to_radians();

        let sin_altitude = lat_rad.sin() * moon_declination.sin()
            + lat_rad.cos() * moon_declination.cos() * hour_angle.cos();
        let altitude = sin_altitude.asin().clamp(-PI / 2.0, PI / 2.0);

        let cos_azimuth = (moon_declination.sin() - lat_rad.sin() * sin_altitude)
            / (lat_rad.cos() * altitude.cos());
        let mut azimuth = cos_azimuth.acos().clamp(0.0, PI);

        if hour_angle.sin() > 0.0 {
            azimuth = 2.0 * PI - azimuth;
        }

        // Determine moon phase
        let phase = if self.lunar_day < 1.84 {
            MoonPhase::NewMoon
        } else if self.lunar_day < 5.53 {
            MoonPhase::WaxingCrescent
        } else if self.lunar_day < 9.22 {
            MoonPhase::FirstQuarter
        } else if self.lunar_day < 12.91 {
            MoonPhase::WaxingGibbous
        } else if self.lunar_day < 16.59 {
            MoonPhase::FullMoon
        } else if self.lunar_day < 20.28 {
            MoonPhase::WaningGibbous
        } else if self.lunar_day < 23.97 {
            MoonPhase::LastQuarter
        } else {
            MoonPhase::WaningCrescent
        };

        (azimuth, altitude, phase, self.lunar_day)
    }

    /// Update celestial bodies positions
    pub fn update_celestial_bodies(&mut self) {
        // Update sun
        let (sun_azimuth, sun_altitude) = self.calculate_sun_position();
        let sun_intensity = sun_altitude.max(0.0).sin().clamp(0.0, 1.0);
        let sun_color_temp = self.calculate_sun_color_temperature(sun_altitude);

        self.sun.update_position(sun_azimuth, sun_altitude, 1000.0);
        self.sun.intensity = sun_intensity;
        self.sun.color_temperature = sun_color_temp;

        // Update moon
        let (moon_azimuth, moon_altitude, moon_phase, _) = self.calculate_moon_position();
        let moon_intensity = if moon_altitude > 0.0 {
            moon_phase.illumination() * 0.3 // Moon is much dimmer than sun
        } else {
            0.0
        };

        self.moon
            .update_position(moon_azimuth, moon_altitude, 1000.0);
        self.moon.intensity = moon_intensity;
        self.moon.color_temperature = 4100.0; // Moonlight color temp
        self.moon_phase = moon_phase;

        // Update sky colors and stars
        self.update_sky_appearance(sun_altitude);
    }

    /// Calculate sun color temperature based on altitude
    fn calculate_sun_color_temperature(&self, altitude: f32) -> f32 {
        if altitude > 0.5 {
            6500.0 // Noon
        } else if altitude > 0.0 {
            // Sunrise/sunset - warmer
            3500.0 + 3000.0 * (altitude / 0.5)
        } else if altitude > -0.1 {
            // Civil twilight
            2500.0 + 1000.0 * ((altitude + 0.1) / 0.1)
        } else {
            2000.0 // Below horizon
        }
    }

    /// Update sky appearance based on sun position
    fn update_sky_appearance(&mut self, sun_altitude: f32) {
        if sun_altitude > 0.5 {
            // Day
            self.sky_color_top = Vector3::new(0.4, 0.6, 0.9);
            self.sky_color_bottom = Vector3::new(0.6, 0.7, 0.9);
            self.stars_visibility = 0.0;
            self.ambient_multiplier = 1.0;
        } else if sun_altitude > 0.0 {
            // Sunrise/sunset
            let t = sun_altitude / 0.5;
            self.sky_color_top =
                Vector3::new(0.4, 0.6, 0.9) * t + Vector3::new(0.9, 0.5, 0.3) * (1.0 - t);
            self.sky_color_bottom =
                Vector3::new(0.6, 0.7, 0.9) * t + Vector3::new(0.9, 0.6, 0.4) * (1.0 - t);
            self.stars_visibility = 0.0;
            self.ambient_multiplier = 0.5 + 0.5 * t;
        } else if sun_altitude > -0.1 {
            // Civil twilight
            let t = (sun_altitude + 0.1) / 0.1;
            self.sky_color_top =
                Vector3::new(0.9, 0.5, 0.3) * t + Vector3::new(0.1, 0.1, 0.2) * (1.0 - t);
            self.sky_color_bottom =
                Vector3::new(0.9, 0.6, 0.4) * t + Vector3::new(0.15, 0.15, 0.25) * (1.0 - t);
            self.stars_visibility = (1.0 - t) * 0.5;
            self.ambient_multiplier = 0.2 + 0.3 * t;
        } else if sun_altitude > -0.3 {
            // Nautical twilight
            let t = (sun_altitude + 0.3) / 0.2;
            self.sky_color_top = Vector3::new(0.1, 0.1, 0.2);
            self.sky_color_bottom =
                Vector3::new(0.15, 0.15, 0.25) * t + Vector3::new(0.05, 0.05, 0.1) * (1.0 - t);
            self.stars_visibility = 0.5 + 0.5 * (1.0 - t);
            self.ambient_multiplier = 0.1 + 0.1 * t;
        } else {
            // Night
            self.sky_color_top = Vector3::new(0.02, 0.02, 0.05);
            self.sky_color_bottom = Vector3::new(0.05, 0.05, 0.1);
            self.stars_visibility = 1.0;
            self.ambient_multiplier = 0.05;
        }
    }

    /// Advance time by delta
    pub fn advance_time(&mut self, dt: f32) {
        self.current_time = (self.current_time + dt) % self.day_duration;

        // Advance lunar cycle (29.5 days)
        self.lunar_day = (self.lunar_day + dt / self.day_duration) % 29.5;

        // Advance year day (optional, for seasonal changes)
        // self.year_day = (self.year_day + 1) % 366;

        self.update_celestial_bodies();
    }

    /// Set specific time of day
    pub fn set_time(&mut self, hour: f32, minute: f32) {
        self.current_time = (hour * 3600.0 + minute * 60.0) % self.day_duration;
        self.update_celestial_bodies();
    }

    /// Get sun direction vector (normalized)
    pub fn get_sun_direction(&self) -> Vector3<f32> {
        self.sun.position.normalize()
    }

    /// Get moon direction vector (normalized)
    pub fn get_moon_direction(&self) -> Vector3<f32> {
        self.moon.position.normalize()
    }

    /// Get combined celestial lighting intensity
    pub fn get_total_light_intensity(&self) -> f32 {
        self.sun.intensity + self.moon.intensity
    }

    /// Граф-4: Get sky color top for renderer
    pub fn get_sky_color_top(&self) -> Vector3<f32> {
        self.sky_color_top
    }

    /// Граф-4: Get sky color horizon (bottom) for renderer
    pub fn get_sky_color_horizon(&self) -> Vector3<f32> {
        self.sky_color_bottom
    }

    /// Граф-4: Get ambient intensity for renderer
    pub fn get_ambient_intensity(&self) -> f32 {
        self.get_total_light_intensity()
    }

    /// Calculate view matrix looking at celestial body
    pub fn get_celestial_view_matrix(&self, body: CelestialBody, distance: f32) -> Matrix4<f32> {
        // Заглушка - упрощено для компиляции
        Matrix4::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_night_cycle_initialization() {
        let cycle = DayNightCycle::new(45.0, 0.0);
        assert_eq!(cycle.get_hour(), 12.0);
        assert!(cycle.is_daytime());
    }

    #[test]
    fn test_time_of_day_classification() {
        let mut cycle = DayNightCycle::new(45.0, 0.0);

        cycle.set_time(3.0, 0.0);
        assert_eq!(cycle.get_time_of_day(), TimeOfDay::Night);

        cycle.set_time(6.0, 0.0);
        assert_eq!(cycle.get_time_of_day(), TimeOfDay::Sunrise);

        cycle.set_time(12.0, 0.0);
        assert_eq!(cycle.get_time_of_day(), TimeOfDay::Noon);

        cycle.set_time(18.0, 0.0);
        assert_eq!(cycle.get_time_of_day(), TimeOfDay::Sunset);
    }

    #[test]
    fn test_moon_phases() {
        assert_eq!(MoonPhase::NewMoon.illumination(), 0.0);
        assert_eq!(MoonPhase::FullMoon.illumination(), 1.0);
        assert_eq!(MoonPhase::FirstQuarter.illumination(), 0.5);
    }

    #[test]
    fn test_advance_time() {
        let mut cycle = DayNightCycle::new(45.0, 0.0);
        let initial_hour = cycle.get_hour();

        cycle.advance_time(3600.0); // Advance 1 hour

        let new_hour = cycle.get_hour();
        assert!((new_hour - initial_hour - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_sky_colors() {
        let mut cycle = DayNightCycle::new(45.0, 0.0);

        // Noon - bright blue sky
        cycle.set_time(12.0, 0.0);
        assert!(cycle.sky_color_top.y > 0.5); // Green component high
        assert!(cycle.stars_visibility == 0.0);

        // Midnight - dark sky with stars
        cycle.set_time(0.0, 0.0);
        assert!(cycle.sky_color_top.x < 0.1);
        assert!(cycle.stars_visibility == 1.0);
    }
}
