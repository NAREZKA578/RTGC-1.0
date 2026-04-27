//! Novosibirsk Region Map Configuration
//! Реалистичная карта Новосибирской области масштаба 1:1
//! Включает: Новосибирск, Бердск, Обь, Кольцово, аэропорт Толмачёво

use serde::{Deserialize, Serialize};

/// Основная конфигурация карты
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovosibirskMap {
    /// Размер карты (км)
    pub map_size_km: f32,
    /// Центр карты (координаты)
    pub center: MapCoordinates,
    /// Основные города
    pub cities: Vec<City>,
    /// Дороги
    pub highways: Vec<Highway>,
    /// Реки
    pub rivers: Vec<River>,
    /// Особые объекты
    pub landmarks: Vec<Landmark>,
    /// Промышленные зоны
    pub industrial_zones: Vec<IndustrialZone>,
    /// Сельскохозяйственные угодья
    pub farmlands: Vec<Farmland>,
    /// Лесные массивы
    pub forests: Vec<Forest>,
}

/// Координаты на карте (широта, долгота, высота)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapCoordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: f32,
}

/// Город
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct City {
    pub name: String,
    pub position: MapCoordinates,
    pub population: u32,
    pub area_km2: f32,
    pub city_type: CityType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CityType {
    /// Город-миллионник
    MillionPlus,
    /// Крупный город
    Large,
    /// Средний город
    Medium,
    /// Малый город
    Small,
    /// Посёлок городского типа
    UrbanSettlement,
    /// Село
    Village,
}

/// Автодорога
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highway {
    pub name: String,
    pub route_type: RouteType,
    pub waypoints: Vec<MapCoordinates>,
    pub lanes: u8,
    pub surface: SurfaceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteType {
    /// Федеральная трасса (Р-256 "Чуйский тракт")
    Federal,
    /// Региональная дорога
    Regional,
    /// Местная дорога
    Local,
    /// Грунтовая дорога
    Dirt,
    /// Технологический проезд
    Service,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceType {
    AsphaltGood,
    AsphaltBad,
    Concrete,
    Gravel,
    Dirt,
    Mud,
}

/// Река
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct River {
    pub name: String,
    pub waypoints: Vec<MapCoordinates>,
    pub width_m: f32,
    pub depth_m: f32,
    pub flow_speed_ms: f32,
}

/// Ориентир/ landmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Landmark {
    pub name: String,
    pub position: MapCoordinates,
    pub landmark_type: LandmarkType,
    pub height_m: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LandmarkType {
    /// Аэропорт
    Airport,
    /// Ж/д вокзал
    TrainStation,
    /// Речной порт
    RiverPort,
    /// Мост
    Bridge,
    /// Телебашня
    TVTower,
    /// Памятник
    Monument,
    /// Стадион
    Stadium,
    /// ТЦ
    ShoppingMall,
    /// ВУЗ
    University,
    /// Больница
    Hospital,
}

/// Промышленная зона
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialZone {
    pub name: String,
    pub position: MapCoordinates,
    pub zone_type: ZoneType,
    pub area_km2: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneType {
    /// Завод
    Factory,
    /// Складской комплекс
    Warehouse,
    /// Карьер
    Quarry,
    /// Нефтебаза
    OilDepot,
    /// Элеватор
    GrainElevator,
    /// Лесопилка
    Sawmill,
    /// Буровая площадка
    DrillingSite,
}

/// Сельхозугодья
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Farmland {
    pub position: MapCoordinates,
    pub crop_type: CropType,
    pub area_km2: f32,
    pub productivity: f32, // тонн/га
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropType {
    Wheat,
    Barley,
    Oats,
    Sunflower,
    Rapeseed,
    Corn,
    Potato,
}

/// Лесной массив
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forest {
    pub name: String,
    pub bounds: ForestBounds,
    pub tree_density: f32, // деревьев/га
    pub dominant_species: TreeSpecies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForestBounds {
    pub center: MapCoordinates,
    pub radius_km: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeSpecies {
    Pine,
    Spruce,
    Birch,
    Aspen,
    Mixed,
}

/// Создать конфигурацию карты Новосибирска 1:1
pub fn create_novosibirsk_map() -> NovosibirskMap {
    NovosibirskMap {
        map_size_km: 150.0, // 150x150 км
        
        center: MapCoordinates {
            latitude: 55.0084,
            longitude: 82.9357,
            altitude_m: 150.0,
        },
        
        cities: vec![
            // Новосибирск - основной город
            City {
                name: "Новосибирск".to_string(),
                position: MapCoordinates {
                    latitude: 55.0084,
                    longitude: 82.9357,
                    altitude_m: 150.0,
                },
                population: 1_625_000,
                area_km2: 505.0,
                city_type: CityType::MillionPlus,
            },
            
            // Бердск
            City {
                name: "Бердск".to_string(),
                position: MapCoordinates {
                    latitude: 54.7581,
                    longitude: 83.1153,
                    altitude_m: 170.0,
                },
                population: 102_000,
                area_km2: 67.0,
                city_type: CityType::Large,
            },
            
            // Обь
            City {
                name: "Обь".to_string(),
                position: MapCoordinates {
                    latitude: 54.9933,
                    longitude: 82.7167,
                    altitude_m: 140.0,
                },
                population: 29_000,
                area_km2: 25.0,
                city_type: CityType::Medium,
            },
            
            // Кольцово
            City {
                name: "Кольцово".to_string(),
                position: MapCoordinates {
                    latitude: 55.0333,
                    longitude: 83.0167,
                    altitude_m: 160.0,
                },
                population: 6_500,
                area_km2: 8.0,
                city_type: CityType::Small,
            },
            
            // Мошково
            City {
                name: "Мошково".to_string(),
                position: MapCoordinates {
                    latitude: 55.3167,
                    longitude: 83.3833,
                    altitude_m: 130.0,
                },
                population: 8_000,
                area_km2: 10.0,
                city_type: CityType::Small,
            },
        ],
        
        highways: vec![
            // Р-256 "Чуйский тракт" - федеральная трасса
            Highway {
                name: "Р-256 Чуйский тракт".to_string(),
                route_type: RouteType::Federal,
                lanes: 2,
                surface: SurfaceType::AsphaltGood,
                waypoints: vec![
                    MapCoordinates { latitude: 55.05, longitude: 82.95, altitude_m: 150.0 },
                    MapCoordinates { latitude: 54.95, longitude: 82.98, altitude_m: 155.0 },
                    MapCoordinates { latitude: 54.85, longitude: 83.02, altitude_m: 160.0 },
                    MapCoordinates { latitude: 54.76, longitude: 83.12, altitude_m: 170.0 }, // Бердск
                ],
            },
            
            // Новосибирск - Обь - аэропорт
            Highway {
                name: "Трасса М-51 Байкал".to_string(),
                route_type: RouteType::Federal,
                lanes: 2,
                surface: SurfaceType::AsphaltGood,
                waypoints: vec![
                    MapCoordinates { latitude: 55.01, longitude: 82.94, altitude_m: 150.0 },
                    MapCoordinates { latitude: 55.00, longitude: 82.85, altitude_m: 145.0 },
                    MapCoordinates { latitude: 54.99, longitude: 82.72, altitude_m: 140.0 }, // Обь
                ],
            },
            
            // Подъезд к аэропорту Толмачёво
            Highway {
                name: "Подъезд к аэропорту Толмачёво".to_string(),
                route_type: RouteType::Regional,
                lanes: 2,
                surface: SurfaceType::AsphaltGood,
                waypoints: vec![
                    MapCoordinates { latitude: 54.99, longitude: 82.75, altitude_m: 142.0 },
                    MapCoordinates { latitude: 54.98, longitude: 82.70, altitude_m: 138.0 },
                ],
            },
            
            // Новосибирск - Мошково
            Highway {
                name: "Новосибирск - Мошково".to_string(),
                route_type: RouteType::Regional,
                lanes: 1,
                surface: SurfaceType::AsphaltBad,
                waypoints: vec![
                    MapCoordinates { latitude: 55.01, longitude: 82.94, altitude_m: 150.0 },
                    MapCoordinates { latitude: 55.10, longitude: 83.10, altitude_m: 145.0 },
                    MapCoordinates { latitude: 55.32, longitude: 83.38, altitude_m: 130.0 },
                ],
            },
        ],
        
        rivers: vec![
            // Река Обь
            River {
                name: "Обь".to_string(),
                width_m: 800.0,
                depth_m: 8.0,
                flow_speed_ms: 1.2,
                waypoints: vec![
                    MapCoordinates { latitude: 55.15, longitude: 82.80, altitude_m: 140.0 },
                    MapCoordinates { latitude: 55.05, longitude: 82.85, altitude_m: 142.0 },
                    MapCoordinates { latitude: 54.95, longitude: 82.90, altitude_m: 145.0 },
                    MapCoordinates { latitude: 54.80, longitude: 82.95, altitude_m: 150.0 },
                ],
            },
            
            // Река Иня
            River {
                name: "Иня".to_string(),
                width_m: 150.0,
                depth_m: 3.0,
                flow_speed_ms: 0.8,
                waypoints: vec![
                    MapCoordinates { latitude: 55.10, longitude: 83.30, altitude_m: 135.0 },
                    MapCoordinates { latitude: 55.05, longitude: 83.15, altitude_m: 140.0 },
                    MapCoordinates { latitude: 55.00, longitude: 83.00, altitude_m: 145.0 },
                ],
            },
            
            // Река Каменка
            River {
                name: "Каменка".to_string(),
                width_m: 40.0,
                depth_m: 1.5,
                flow_speed_ms: 0.5,
                waypoints: vec![
                    MapCoordinates { latitude: 55.05, longitude: 82.98, altitude_m: 155.0 },
                    MapCoordinates { latitude: 55.02, longitude: 82.95, altitude_m: 152.0 },
                    MapCoordinates { latitude: 55.00, longitude: 82.92, altitude_m: 150.0 },
                ],
            },
        ],
        
        landmarks: vec![
            // Аэропорт Толмачёво
            Landmark {
                name: "Аэропорт Толмачёво".to_string(),
                position: MapCoordinates {
                    latitude: 55.0127,
                    longitude: 82.6507,
                    altitude_m: 111.0,
                },
                landmark_type: LandmarkType::Airport,
                height_m: 15.0,
            },
            
            // Ж/д вокзал Новосибирск-Главный
            Landmark {
                name: "Вокзал Новосибирск-Главный".to_string(),
                position: MapCoordinates {
                    latitude: 55.0453,
                    longitude: 82.9132,
                    altitude_m: 155.0,
                },
                landmark_type: LandmarkType::TrainStation,
                height_m: 25.0,
            },
            
            // Речной вокзал
            Landmark {
                name: "Речной вокзал".to_string(),
                position: MapCoordinates {
                    latitude: 55.0380,
                    longitude: 82.9250,
                    altitude_m: 148.0,
                },
                landmark_type: LandmarkType::RiverPort,
                height_m: 20.0,
            },
            
            // Бугринский мост
            Landmark {
                name: "Бугринский мост".to_string(),
                position: MapCoordinates {
                    latitude: 55.0150,
                    longitude: 82.9800,
                    altitude_m: 155.0,
                },
                landmark_type: LandmarkType::Bridge,
                height_m: 60.0,
            },
            
            // Телебашня
            Landmark {
                name: "Новосибирская телебашня".to_string(),
                position: MapCoordinates {
                    latitude: 55.0250,
                    longitude: 82.9450,
                    altitude_m: 160.0,
                },
                landmark_type: LandmarkType::TVTower,
                height_m: 192.0,
            },
            
            // Академгородок (НГУ)
            Landmark {
                name: "Новосибирский государственный университет".to_string(),
                position: MapCoordinates {
                    latitude: 54.8450,
                    longitude: 83.1000,
                    altitude_m: 175.0,
                },
                landmark_type: LandmarkType::University,
                height_m: 30.0,
            },
        ],
        
        industrial_zones: vec![
            // Промзона Левобережная
            IndustrialZone {
                name: "Промзона Левобережная".to_string(),
                position: MapCoordinates {
                    latitude: 54.9800,
                    longitude: 82.8800,
                    altitude_m: 145.0,
                },
                zone_type: ZoneType::Factory,
                area_km2: 5.0,
            },
            
            // Складской комплекс Толмачёво
            IndustrialZone {
                name: "Складской комплекс Толмачёво".to_string(),
                position: MapCoordinates {
                    latitude: 54.9950,
                    longitude: 82.6800,
                    altitude_m: 115.0,
                },
                zone_type: ZoneType::Warehouse,
                area_km2: 2.5,
            },
            
            // Нефтебаза Обь
            IndustrialZone {
                name: "Нефтебаза Обь".to_string(),
                position: MapCoordinates {
                    latitude: 54.9900,
                    longitude: 82.7300,
                    altitude_m: 140.0,
                },
                zone_type: ZoneType::OilDepot,
                area_km2: 1.5,
            },
            
            // Элеватор Бердск
            IndustrialZone {
                name: "Бердский элеватор".to_string(),
                position: MapCoordinates {
                    latitude: 54.7500,
                    longitude: 83.1000,
                    altitude_m: 168.0,
                },
                zone_type: ZoneType::GrainElevator,
                area_km2: 0.8,
            },
        ],
        
        farmlands: vec![
            Farmland {
                position: MapCoordinates {
                    latitude: 54.9000,
                    longitude: 83.0500,
                    altitude_m: 180.0,
                },
                crop_type: CropType::Wheat,
                area_km2: 25.0,
                productivity: 3.5,
            },
            Farmland {
                position: MapCoordinates {
                    latitude: 55.1500,
                    longitude: 83.2000,
                    altitude_m: 140.0,
                },
                crop_type: CropType::Barley,
                area_km2: 18.0,
                productivity: 2.8,
            },
            Farmland {
                position: MapCoordinates {
                    latitude: 54.8500,
                    longitude: 82.8000,
                    altitude_m: 160.0,
                },
                crop_type: CropType::Sunflower,
                area_km2: 12.0,
                productivity: 2.2,
            },
        ],
        
        forests: vec![
            // Академгородок лес
            Forest {
                name: "Лес Академгородка".to_string(),
                bounds: ForestBounds {
                    center: MapCoordinates {
                        latitude: 54.8500,
                        longitude: 83.0800,
                        altitude_m: 175.0,
                    },
                    radius_km: 8.0,
                },
                tree_density: 250.0,
                dominant_species: TreeSpecies::Mixed,
            },
            
            // Бор Заельцовский
            Forest {
                name: "Заельцовский бор".to_string(),
                bounds: ForestBounds {
                    center: MapCoordinates {
                        latitude: 55.0800,
                        longitude: 82.9200,
                        altitude_m: 155.0,
                    },
                    radius_km: 5.0,
                },
                tree_density: 300.0,
                dominant_species: TreeSpecies::Pine,
            },
            
            // Лес под Бердском
            Forest {
                name: "Бердский лес".to_string(),
                bounds: ForestBounds {
                    center: MapCoordinates {
                        latitude: 54.7200,
                        longitude: 83.1500,
                        altitude_m: 185.0,
                    },
                    radius_km: 12.0,
                },
                tree_density: 280.0,
                dominant_species: TreeSpecies::Birch,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_creation() {
        let map = create_novosibirsk_map();
        
        assert_eq!(map.map_size_km, 150.0);
        assert!(!map.cities.is_empty());
        assert!(!map.highways.is_empty());
        assert!(!map.rivers.is_empty());
        
        // Проверка Новосибирска
        let novosibirsk = map.cities.iter().find(|c| c.name == "Новосибирск").ok_or("City not found")?;
        assert_eq!(novosibirsk.population, 1_625_000);
        assert_eq!(novosibirsk.city_type, CityType::MillionPlus);
        
        // Проверка аэропорта
        let airport = map.landmarks.iter().find(|l| l.name == "Аэропорт Толмачёво").ok_or("City not found")?;
        assert_eq!(airport.landmark_type, LandmarkType::Airport);
    }

    #[test]
    fn test_highway_surface() {
        let map = create_novosibirsk_map();
        
        let federal = map.highways.iter().find(|h| h.route_type == RouteType::Federal).ok_or("City not found")?;
        assert_eq!(federal.surface, SurfaceType::AsphaltGood);
    }
}
