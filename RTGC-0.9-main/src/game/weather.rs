#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrecipitationType {
    None,
    Rain,
    Snow,
}

#[derive(Debug, Clone)]
pub struct WeatherState {
    pub precipitation_type: PrecipitationType,
    pub cloud_coverage: f32,
    pub temperature: f32,
}

impl WeatherState {
    pub fn new() -> Self {
        Self {
            precipitation_type: PrecipitationType::None,
            cloud_coverage: 0.0,
            temperature: 20.0,
        }
    }
    
    pub fn update(&mut self, _dt: f32) {
        // Weather update
    }
    
    pub fn description(&self) -> &str {
        if self.precipitation_type == PrecipitationType::None {
            if self.cloud_coverage < 0.3 {
                "Clear"
            } else {
                "Cloudy"
            }
        } else if self.precipitation_type == PrecipitationType::Rain {
            "Rain"
        } else {
            "Snow"
        }
    }
}

impl Default for WeatherState {
    fn default() -> Self {
        Self::new()
    }
}