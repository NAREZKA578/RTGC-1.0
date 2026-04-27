//! Dynamic Weather System
//!
//! Implements realistic weather simulation including:
//! - Dynamic cloud cover and sky coloring
//! - Precipitation (rain, snow, hail)
//! - Wind simulation affecting vegetation and particles
//! - Temperature and humidity cycles
//! - Weather transitions and forecasting

use nalgebra::Vector2 as V2;
use nalgebra::Vector3 as V3;
use nalgebra::{Vector2, Vector3};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::f32::consts::PI;
use tracing::info;

/// Weather types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherType {
    Clear,
    PartlyCloudy,
    Cloudy,
    Overcast,
    LightRain,
    HeavyRain,
    Thunderstorm,
    LightSnow,
    HeavySnow,
    Blizzard,
    Fog,
    Mist,
}

impl WeatherType {
    /// Get precipitation intensity (0.0 = none, 1.0 = maximum)
    pub fn precipitation_intensity(&self) -> f32 {
        match self {
            WeatherType::Clear
            | WeatherType::PartlyCloudy
            | WeatherType::Cloudy
            | WeatherType::Overcast
            | WeatherType::Fog
            | WeatherType::Mist => 0.0,
            WeatherType::LightRain | WeatherType::LightSnow => 0.3,
            WeatherType::HeavyRain | WeatherType::HeavySnow => 0.7,
            WeatherType::Thunderstorm => 0.9,
            WeatherType::Blizzard => 0.8,
        }
    }

    /// Get cloud coverage (0.0 = clear, 1.0 = fully overcast)
    pub fn cloud_coverage(&self) -> f32 {
        match self {
            WeatherType::Clear => 0.0,
            WeatherType::PartlyCloudy => 0.3,
            WeatherType::Cloudy => 0.6,
            WeatherType::Overcast => 0.9,
            WeatherType::LightRain | WeatherType::LightSnow => 0.7,
            WeatherType::HeavyRain | WeatherType::HeavySnow | WeatherType::Thunderstorm => 0.95,
            WeatherType::Blizzard => 1.0,
            WeatherType::Fog | WeatherType::Mist => 0.8,
        }
    }

    /// Check if this weather has lightning
    pub fn has_lightning(&self) -> bool {
        matches!(self, WeatherType::Thunderstorm | WeatherType::Blizzard)
    }
}

/// Cloud layer properties
#[derive(Debug, Clone)]
pub struct CloudLayer {
    /// Altitude of cloud layer (meters)
    pub altitude: f32,
    /// Thickness of cloud layer (meters)
    pub thickness: f32,
    /// Coverage density (0.0-1.0)
    pub density: f32,
    /// Cloud type (cumulus, stratus, cirrus, etc.)
    pub cloud_type: CloudType,
    /// Movement velocity (m/s)
    pub velocity: V2<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum CloudType {
    Cumulus,      // Fluffy, fair weather clouds
    Stratus,      // Layered, overcast clouds
    Cirrus,       // High, wispy clouds
    Cumulonimbus, // Storm clouds
    Nimbostratus, // Rain clouds
}

/// Wind simulation
#[derive(Debug, Clone)]
pub struct Wind {
    /// Direction (normalized 3D vector)
    pub direction: V3<f32>,
    /// Speed at ground level (m/s)
    pub ground_speed: f32,
    /// Speed at altitude (m/s)
    pub altitude_speed: f32,
    /// Gustiness (0.0 = steady, 1.0 = very gusty)
    pub gustiness: f32,
    /// Current gust factor
    pub current_gust: f32,
    /// Turbulence seed for noise
    pub turbulence_seed: u64,
}

impl Wind {
    pub fn new(direction: V3<f32>, speed: f32) -> Self {
        let mut rng = ChaCha8Rng::from_entropy();
        Self {
            direction: direction.normalize(),
            ground_speed: speed,
            altitude_speed: speed * 1.5, // Wind is stronger at altitude
            gustiness: 0.3,
            current_gust: 1.0,
            turbulence_seed: rng.r#gen(),
        }
    }

    /// Get wind speed at given altitude with gusts
    pub fn get_wind_at_altitude(&self, altitude: f32, time: f32) -> V3<f32> {
        // Wind shear: wind speed increases with altitude
        let shear_factor = 1.0 + (altitude / 1000.0).min(1.0) * 0.5;

        // Add gusts using simple noise
        let gust = self.gust_factor(time);

        let speed = ((self.ground_speed
            + (self.altitude_speed - self.ground_speed) * (altitude / 1000.0).min(1.0))
            * shear_factor
            * gust)
            .max(0.0);

        self.direction * speed
    }

    /// Calculate current gust factor using pseudo-noise
    fn gust_factor(&self, time: f32) -> f32 {
        let mut rng = ChaCha8Rng::seed_from_u64(self.turbulence_seed);

        // Simple periodic gust pattern with randomness
        let base_gust = 1.0 + self.gustiness * 0.5 * (time * 0.5).sin();
        let random_variation = rng.gen_range(-0.2..0.2) * self.gustiness;

        (base_gust + random_variation).clamp(0.5, 2.0)
    }

    /// Update wind with time-varying gusts
    pub fn update(&mut self, dt: f32, time: f32) {
        self.current_gust = self.gust_factor(time);
    }
}

/// Atmospheric conditions
#[derive(Debug, Clone)]
pub struct Atmosphere {
    /// Air temperature (Celsius)
    pub temperature: f32,
    /// Relative humidity (0.0-1.0)
    pub humidity: f32,
    /// Air pressure (hPa)
    pub pressure: f32,
    /// Visibility distance (meters)
    pub visibility: f32,
    /// UV index (0-11+)
    pub uv_index: f32,
}

impl Default for Atmosphere {
    fn default() -> Self {
        Self {
            temperature: 20.0,
            humidity: 0.5,
            pressure: 1013.25,   // Standard sea level pressure
            visibility: 10000.0, // 10 km
            uv_index: 5.0,
        }
    }
}

impl Atmosphere {
    /// Calculate dew point from temperature and humidity
    pub fn dew_point(&self) -> f32 {
        let a = 17.27;
        let b = 237.7;
        let alpha = ((a * self.temperature) / (b + self.temperature)) + self.humidity.ln();
        (b * alpha) / (a - alpha)
    }

    /// Check if precipitation should be snow based on temperature
    pub fn is_snowing(&self) -> bool {
        self.temperature < 0.0
    }

    /// Update visibility based on weather
    pub fn update_visibility(&mut self, weather: WeatherType) {
        self.visibility = match weather {
            WeatherType::Clear => 20000.0,
            WeatherType::PartlyCloudy => 15000.0,
            WeatherType::Cloudy => 12000.0,
            WeatherType::Overcast => 10000.0,
            WeatherType::LightRain | WeatherType::LightSnow => 5000.0,
            WeatherType::HeavyRain | WeatherType::HeavySnow => 2000.0,
            WeatherType::Thunderstorm => 1500.0,
            WeatherType::Blizzard => 500.0,
            WeatherType::Fog => 200.0,
            WeatherType::Mist => 1000.0,
        };
    }
}

/// Precipitation particle
#[derive(Debug, Clone, Copy)]
pub struct PrecipitationParticle {
    pub position: V3<f32>,
    pub velocity: V3<f32>,
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

impl PrecipitationParticle {
    pub fn new(position: V3<f32>, velocity: V3<f32>, size: f32, lifetime: f32) -> Self {
        Self {
            position,
            velocity,
            size,
            lifetime,
            max_lifetime: lifetime,
        }
    }

    pub fn update(&mut self, dt: f32, wind: V3<f32>) {
        self.position += (self.velocity + wind) * dt;
        self.lifetime -= dt;
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0 && self.position.y >= 0.0
    }
}

/// Main weather system
pub struct WeatherSystem {
    /// Current weather type
    pub current_weather: WeatherType,
    /// Target weather for transition
    pub target_weather: WeatherType,
    /// Weather transition progress (0.0-1.0)
    pub transition_progress: f32,
    /// Time in current weather (seconds)
    pub weather_timer: f32,
    /// Minimum time in current weather before transition
    pub min_weather_duration: f32,
    /// Maximum time in current weather before forced transition
    pub max_weather_duration: f32,
    /// Cloud layers
    pub cloud_layers: Vec<CloudLayer>,
    /// Wind simulation
    pub wind: Wind,
    /// Atmospheric conditions
    pub atmosphere: Atmosphere,
    /// Active precipitation particles
    pub precipitation_particles: Vec<PrecipitationParticle>,
    /// Maximum number of precipitation particles
    pub max_precipitation_particles: usize,
    /// Lightning flash intensity (0.0-1.0)
    pub lightning_flash: f32,
    /// Lightning timer
    pub lightning_timer: f32,
    /// RNG seed for deterministic weather
    pub seed: u64,
}

impl WeatherSystem {
    pub fn new(seed: u64) -> Self {
        let initial_weather = WeatherType::Clear;

        Self {
            current_weather: initial_weather,
            target_weather: initial_weather,
            transition_progress: 1.0,
            weather_timer: 0.0,
            min_weather_duration: 300.0,  // 5 minutes minimum
            max_weather_duration: 1800.0, // 30 minutes maximum
            cloud_layers: Vec::new(),
            wind: Wind::new(Vector3::new(1.0, 0.0, 0.0), 5.0),
            atmosphere: Atmosphere::default(),
            precipitation_particles: Vec::new(),
            max_precipitation_particles: 10000,
            lightning_flash: 0.0,
            lightning_timer: 0.0,
            seed,
        }
    }

    /// Initialize cloud layers based on weather
    fn initialize_clouds(&mut self, weather: WeatherType) {
        self.cloud_layers.clear();

        let coverage = weather.cloud_coverage();
        if coverage < 0.1 {
            return; // No clouds needed
        }

        // Add appropriate cloud layers based on weather
        match weather {
            WeatherType::PartlyCloudy => {
                self.cloud_layers.push(CloudLayer {
                    altitude: 2000.0,
                    thickness: 500.0,
                    density: coverage,
                    cloud_type: CloudType::Cumulus,
                    velocity: Vector2::new(5.0, 0.0),
                });
            }
            WeatherType::Cloudy | WeatherType::Overcast => {
                self.cloud_layers.push(CloudLayer {
                    altitude: 1500.0,
                    thickness: 800.0,
                    density: coverage,
                    cloud_type: CloudType::Stratus,
                    velocity: Vector2::new(3.0, 1.0),
                });
            }
            WeatherType::LightRain | WeatherType::HeavyRain => {
                self.cloud_layers.push(CloudLayer {
                    altitude: 1000.0,
                    thickness: 1000.0,
                    density: 0.9,
                    cloud_type: CloudType::Nimbostratus,
                    velocity: Vector2::new(4.0, 0.5),
                });
            }
            WeatherType::Thunderstorm => {
                self.cloud_layers.push(CloudLayer {
                    altitude: 800.0,
                    thickness: 2000.0,
                    density: 1.0,
                    cloud_type: CloudType::Cumulonimbus,
                    velocity: Vector2::new(6.0, 1.0),
                });
            }
            WeatherType::LightSnow | WeatherType::HeavySnow | WeatherType::Blizzard => {
                self.cloud_layers.push(CloudLayer {
                    altitude: 1200.0,
                    thickness: 600.0,
                    density: coverage,
                    cloud_type: CloudType::Nimbostratus,
                    velocity: Vector2::new(2.0, 0.0),
                });
            }
            WeatherType::Fog | WeatherType::Mist => {
                self.cloud_layers.push(CloudLayer {
                    altitude: 50.0,
                    thickness: 200.0,
                    density: 0.8,
                    cloud_type: CloudType::Stratus,
                    velocity: Vector2::new(0.5, 0.0),
                });
            }
            _ => {}
        }
    }

    /// Transition to new weather
    pub fn transition_to(&mut self, new_weather: WeatherType) {
        if self.current_weather == new_weather {
            return;
        }

        self.target_weather = new_weather;
        self.transition_progress = 0.0;
        self.weather_timer = 0.0;

        info!(
            "Weather transitioning from {:?} to {:?}",
            self.current_weather, new_weather
        );
    }

    /// Random weather transition based on probabilities
    pub fn update_weather_transition(&mut self, dt: f32, rng: &mut ChaCha8Rng) {
        self.weather_timer += dt;

        // Only consider transition after minimum duration
        if self.weather_timer < self.min_weather_duration {
            return;
        }

        // Force transition after maximum duration
        let should_transition = self.weather_timer >= self.max_weather_duration
            || (self.weather_timer >= self.min_weather_duration && rng.gen_bool(0.001));

        if should_transition {
            // Choose new weather based on current weather (weighted probabilities)
            let new_weather = self.choose_next_weather(rng);
            self.transition_to(new_weather);

            // Reset timer with random duration
            self.min_weather_duration = rng.gen_range(300.0..600.0);
            self.max_weather_duration = rng.gen_range(1200.0..2400.0);
        }
    }

    /// Choose next weather type based on weighted probabilities
    fn choose_next_weather(&self, rng: &mut ChaCha8Rng) -> WeatherType {
        use WeatherType::*;

        let weights: Vec<(WeatherType, f32)> = match self.current_weather {
            Clear => vec![(Clear, 0.6), (PartlyCloudy, 0.3), (Cloudy, 0.1)],
            PartlyCloudy => vec![
                (Clear, 0.3),
                (PartlyCloudy, 0.4),
                (Cloudy, 0.2),
                (LightRain, 0.1),
            ],
            Cloudy => vec![
                (PartlyCloudy, 0.3),
                (Cloudy, 0.3),
                (Overcast, 0.3),
                (LightRain, 0.1),
            ],
            Overcast => vec![
                (Cloudy, 0.3),
                (Overcast, 0.3),
                (LightRain, 0.3),
                (HeavyRain, 0.1),
            ],
            LightRain => vec![
                (Cloudy, 0.3),
                (LightRain, 0.4),
                (HeavyRain, 0.2),
                (Clear, 0.1),
            ],
            HeavyRain => vec![
                (LightRain, 0.4),
                (HeavyRain, 0.3),
                (Thunderstorm, 0.2),
                (Overcast, 0.1),
            ],
            Thunderstorm => vec![(HeavyRain, 0.5), (Thunderstorm, 0.3), (LightRain, 0.2)],
            LightSnow => vec![
                (Cloudy, 0.3),
                (LightSnow, 0.4),
                (HeavySnow, 0.2),
                (Overcast, 0.1),
            ],
            HeavySnow => vec![
                (LightSnow, 0.4),
                (HeavySnow, 0.3),
                (Blizzard, 0.2),
                (Overcast, 0.1),
            ],
            Blizzard => vec![(HeavySnow, 0.6), (Blizzard, 0.3), (LightSnow, 0.1)],
            Fog => vec![(Mist, 0.4), (Fog, 0.4), (Overcast, 0.2)],
            Mist => vec![(Fog, 0.3), (Mist, 0.4), (Cloudy, 0.3)],
        };

        let total_weight: f32 = weights.iter().map(|(_, w)| w).sum();
        let mut random = rng.gen_range(0.0..total_weight);

        for (weather, weight) in weights {
            if random < weight {
                return weather;
            }
            random -= weight;
        }

        self.current_weather
    }

    /// Update weather system
    pub fn update(&mut self, dt: f32, total_time: f32) {
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed.wrapping_add(total_time as u64));

        // Update weather transition
        if self.transition_progress < 1.0 {
            self.transition_progress = (self.transition_progress + dt / 60.0).min(1.0);

            if self.transition_progress >= 1.0 {
                self.current_weather = self.target_weather;
                self.initialize_clouds(self.current_weather);
                self.atmosphere.update_visibility(self.current_weather);
            }
        } else {
            self.update_weather_transition(dt, &mut rng);
        }

        // Update wind
        self.wind.update(dt, total_time);

        // Update atmospheric conditions
        self.update_atmosphere(dt, total_time, &mut rng);

        // Update precipitation particles
        self.update_precipitation(dt, &mut rng);

        // Update lightning
        self.update_lightning(dt, &mut rng);
    }

    /// Update atmospheric conditions
    fn update_atmosphere(&mut self, dt: f32, total_time: f32, rng: &mut ChaCha8Rng) {
        // Diurnal temperature cycle
        let hour = (total_time / 3600.0) % 24.0;
        let base_temp = 20.0; // Average temperature
        let temp_amplitude = 8.0; // Daily variation
        let diurnal_factor = (-(hour - 14.0) / 24.0 * 2.0 * PI).cos(); // Peak at 2 PM
        let target_temp = base_temp + temp_amplitude * diurnal_factor;

        // Slowly approach target temperature
        self.atmosphere.temperature += (target_temp - self.atmosphere.temperature) * dt * 0.001;

        // Humidity based on weather
        let target_humidity = match self.current_weather {
            WeatherType::Clear => 0.4,
            WeatherType::PartlyCloudy => 0.5,
            WeatherType::Cloudy | WeatherType::Overcast => 0.6,
            WeatherType::LightRain | WeatherType::HeavyRain | WeatherType::Thunderstorm => 0.85,
            WeatherType::LightSnow | WeatherType::HeavySnow | WeatherType::Blizzard => 0.7,
            WeatherType::Fog | WeatherType::Mist => 0.95,
        };
        self.atmosphere.humidity += (target_humidity - self.atmosphere.humidity) * dt * 0.01;

        // UV index based on time and clouds
        let day_factor = (-(hour - 12.0) / 12.0 * PI).sin().max(0.0);
        let cloud_factor = 1.0 - self.current_weather.cloud_coverage() * 0.7;
        self.atmosphere.uv_index = day_factor * cloud_factor * 10.0;

        // Pressure variations
        let pressure_variation = rng.gen_range(-5.0..5.0);
        self.atmosphere.pressure = 1013.25 + pressure_variation;
    }

    /// Update precipitation particles
    fn update_precipitation(&mut self, dt: f32, rng: &mut ChaCha8Rng) {
        let intensity = self.current_weather.precipitation_intensity();

        // Remove dead particles
        self.precipitation_particles.retain(|p| p.is_alive());

        // Spawn new particles
        let spawn_rate = (intensity * 100.0) as usize;
        let current_count = self.precipitation_particles.len();

        for i in 0..spawn_rate {
            if current_count + i >= self.max_precipitation_particles {
                break;
            }

            let is_snow = self.atmosphere.is_snowing();
            let x = rng.gen_range(-100.0..100.0);
            let z = rng.gen_range(-100.0..100.0);
            let y = rng.gen_range(50.0..200.0);

            let fall_speed = if is_snow { 2.0 } else { 15.0 };
            let size = if is_snow { 0.05 } else { 0.02 };
            let lifetime = y / fall_speed;

            self.precipitation_particles
                .push(PrecipitationParticle::new(
                    Vector3::new(x, y, z),
                    Vector3::new(0.0, -fall_speed, 0.0),
                    size,
                    lifetime,
                ));
        }

        // Update existing particles
        let wind = self.wind.get_wind_at_altitude(10.0, 0.0);
        for particle in &mut self.precipitation_particles {
            particle.update(dt, wind);
        }
    }

    /// Update lightning effects
    fn update_lightning(&mut self, dt: f32, rng: &mut ChaCha8Rng) {
        if !self.current_weather.has_lightning() {
            self.lightning_flash = 0.0;
            return;
        }

        // Decay flash
        self.lightning_flash = (self.lightning_flash - dt * 5.0).max(0.0);

        // Random lightning strikes
        self.lightning_timer -= dt;
        if self.lightning_timer <= 0.0 && rng.gen_bool(0.02) {
            self.lightning_flash = 1.0;
            self.lightning_timer = rng.gen_range(2.0..10.0);
        }
    }

    /// Get interpolated weather properties during transition
    pub fn get_interpolated_cloud_coverage(&self) -> f32 {
        let current = self.current_weather.cloud_coverage();
        let target = self.target_weather.cloud_coverage();
        current + (target - current) * self.transition_progress
    }

    /// Get sky color based on weather and time
    pub fn get_sky_color(&self, sun_height: f32) -> V3<f32> {
        let base_color = match self.current_weather {
            WeatherType::Clear => Vector3::new(0.4, 0.6, 0.9),
            WeatherType::PartlyCloudy => Vector3::new(0.5, 0.65, 0.85),
            WeatherType::Cloudy => Vector3::new(0.6, 0.6, 0.7),
            WeatherType::Overcast => Vector3::new(0.65, 0.65, 0.65),
            WeatherType::LightRain | WeatherType::HeavyRain => Vector3::new(0.4, 0.4, 0.5),
            WeatherType::Thunderstorm => Vector3::new(0.25, 0.25, 0.35),
            WeatherType::LightSnow | WeatherType::HeavySnow => Vector3::new(0.7, 0.75, 0.8),
            WeatherType::Blizzard => Vector3::new(0.8, 0.8, 0.85),
            WeatherType::Fog | WeatherType::Mist => Vector3::new(0.7, 0.7, 0.7),
        };

        // Apply interpolation during transition
        let target_color = match self.target_weather {
            WeatherType::Clear => Vector3::new(0.4, 0.6, 0.9),
            WeatherType::PartlyCloudy => Vector3::new(0.5, 0.65, 0.85),
            WeatherType::Cloudy => Vector3::new(0.6, 0.6, 0.7),
            WeatherType::Overcast => Vector3::new(0.65, 0.65, 0.65),
            WeatherType::LightRain | WeatherType::HeavyRain => Vector3::new(0.4, 0.4, 0.5),
            WeatherType::Thunderstorm => Vector3::new(0.25, 0.25, 0.35),
            WeatherType::LightSnow | WeatherType::HeavySnow => Vector3::new(0.7, 0.75, 0.8),
            WeatherType::Blizzard => Vector3::new(0.8, 0.8, 0.85),
            WeatherType::Fog | WeatherType::Mist => Vector3::new(0.7, 0.7, 0.7),
        };

        let mut color = base_color + (target_color - base_color) * self.transition_progress;

        // Adjust for sun height (sunrise/sunset colors)
        if sun_height < 0.0 {
            // Night
            color *= 0.1;
        } else if sun_height < 0.1 {
            // Sunrise/sunset
            let t = sun_height / 0.1;
            let sunset_color = Vector3::new(0.9, 0.5, 0.3);
            color = color * (1.0 - t) + sunset_color * t;
        }

        color
    }

    /// Get fog density based on weather
    pub fn get_fog_density(&self) -> f32 {
        match self.current_weather {
            WeatherType::Fog => 0.003,
            WeatherType::Mist => 0.001,
            WeatherType::HeavyRain | WeatherType::Thunderstorm => 0.0005,
            WeatherType::Blizzard => 0.004,
            _ => 0.0001,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_precipitation_intensity() {
        assert_eq!(WeatherType::Clear.precipitation_intensity(), 0.0);
        assert!(WeatherType::LightRain.precipitation_intensity() > 0.0);
        assert!(
            WeatherType::HeavyRain.precipitation_intensity()
                > WeatherType::LightRain.precipitation_intensity()
        );
    }

    #[test]
    fn test_weather_cloud_coverage() {
        assert_eq!(WeatherType::Clear.cloud_coverage(), 0.0);
        assert_eq!(WeatherType::Overcast.cloud_coverage(), 0.9);
    }

    #[test]
    fn test_dew_point_calculation() {
        let atmosphere = Atmosphere {
            temperature: 25.0,
            humidity: 0.6,
            ..Default::default()
        };

        let dew_point = atmosphere.dew_point();
        assert!(dew_point < atmosphere.temperature);
        assert!(dew_point > 0.0);
    }

    #[test]
    fn test_weather_system_initialization() {
        let weather = WeatherSystem::new(12345);
        assert_eq!(weather.current_weather, WeatherType::Clear);
        assert_eq!(weather.transition_progress, 1.0);
    }

    #[test]
    fn test_wind_at_altitude() {
        let wind = Wind::new(Vector3::new(1.0, 0.0, 0.0), 10.0);

        let ground_wind = wind.get_wind_at_altitude(0.0, 0.0);
        let high_wind = wind.get_wind_at_altitude(1000.0, 0.0);

        assert!(high_wind.norm() >= ground_wind.norm());
    }
}
