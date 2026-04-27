// Audio Module - 3D Spatial Audio with Occlusion
// Advanced audio system with positional audio and occlusion support

use nalgebra::Vector3;
use parking_lot::Mutex;
use rodio::{OutputStream, Sink};
use std::collections::HashMap;
use std::sync::Arc;
use tracing;

/// Тип для 3D вектора (единый стек с nalgebra)
pub type Vec3 = Vector3<f32>;

/// 3D аудио источник с позиционированием
#[derive(Debug, Clone)]
pub struct AudioSource {
    /// Позиция источника в мире
    pub position: Vec3,
    /// Скорость источника (для Doppler эффекта)
    pub velocity: Vec3,
    /// Громкость (0.0 - 1.0)
    pub volume: f32,
    /// Питч (1.0 = нормальный)
    pub pitch: f32,
    /// Радиус затухания
    pub min_distance: f32,
    /// Максимальная дистанция слышимости
    pub max_distance: f32,
    /// Зацикливание
    pub looping: bool,
    /// Путь к файлу
    pub sound_path: String,
    /// Приоритет (чем выше, тем важнее)
    pub priority: u32,
    /// 3D панорамирование включено
    pub spatial: bool,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            volume: 1.0,
            pitch: 1.0,
            min_distance: 1.0,
            max_distance: 100.0,
            looping: false,
            sound_path: String::new(),
            priority: 1,
            spatial: true,
        }
    }
}

/// Аудио слушатель (обычно камера/игрок)
#[derive(Debug, Clone)]
pub struct AudioListener {
    /// Позиция слушателя
    pub position: Vec3,
    /// Направление взгляда (forward)
    pub forward: Vec3,
    /// Вектор вверх
    pub up: Vec3,
    /// Скорость слушателя (для Doppler)
    pub velocity: Vec3,
    /// Мастер громкость
    pub master_volume: f32,
    /// Doppler фактор
    pub doppler_factor: f32,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            forward: Vector3::new(0.0, 0.0, -1.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            velocity: Vector3::zeros(),
            master_volume: 1.0,
            doppler_factor: 1.0,
        }
    }
}

impl AudioListener {
    pub fn new(
        position: nalgebra::Vector3<f32>,
        forward: nalgebra::Vector3<f32>,
        up: nalgebra::Vector3<f32>,
    ) -> Self {
        Self {
            position,
            forward: forward.normalize(),
            up: up.normalize(),
            ..Default::default()
        }
    }

    /// Вычисляет матрицу 3D звука для HRTF
    pub fn get_audio_matrix(&self) -> [[f32; 4]; 4] {
        let forward = self.forward.normalize();
        let up = self.up.normalize();
        let right = forward.cross(&up).normalize();

        [
            [right.x, right.y, right.z, 0.0],
            [up.x, up.y, up.z, 0.0],
            [-forward.x, -forward.y, -forward.z, 0.0],
            [self.position.x, self.position.y, self.position.z, 1.0],
        ]
    }
}

/// Результат трассировки луча для окклюзии
#[derive(Debug, Clone)]
pub struct OcclusionResult {
    /// Есть ли окклюзия
    pub occluded: bool,
    /// Степень окклюзии (0.0 - 1.0)
    pub occlusion_factor: f32,
    /// Количество препятствий на пути
    pub obstacle_count: u32,
    /// Материал последнего препятствия
    pub material: OcclusionMaterial,
}

/// Типы материалов для окклюзии
#[derive(Debug, Clone, Copy)]
pub enum OcclusionMaterial {
    Air,
    Wood,
    Metal,
    Concrete,
    Glass,
    Water,
    Fabric,
    Earth,
}

impl OcclusionMaterial {
    /// Коэффициент поглощения звука (0.0 = полное прохождение, 1.0 = полная блокировка)
    pub fn absorption_coefficient(&self, _frequency: f32) -> f32 {
        // Упрощенная модель - в реальности зависит от частоты
        match self {
            OcclusionMaterial::Air => 0.0,
            OcclusionMaterial::Wood => 0.3,
            OcclusionMaterial::Metal => 0.7,
            OcclusionMaterial::Concrete => 0.8,
            OcclusionMaterial::Glass => 0.5,
            OcclusionMaterial::Water => 0.6,
            OcclusionMaterial::Fabric => 0.4,
            OcclusionMaterial::Earth => 0.9,
        }
    }
}

/// Параметры среды для распространения звука
#[derive(Debug, Clone)]
pub struct EnvironmentParams {
    /// Температура воздуха (влияет на скорость звука)
    pub temperature: f32,
    /// Влажность (влияет на поглощение)
    pub humidity: f32,
    /// Давление
    pub pressure: f32,
    /// Скорость ветра
    pub wind_velocity: Vec3,
}

impl Default for EnvironmentParams {
    fn default() -> Self {
        Self {
            temperature: 20.0,  // Celsius
            humidity: 50.0,     // %
            pressure: 101325.0, // Pascal
            wind_velocity: Vector3::zeros(),
        }
    }
}

impl EnvironmentParams {
    /// Вычисляет скорость звука в м/с
    pub fn speed_of_sound(&self) -> f32 {
        // Формула: c = 331.3 * sqrt(1 + T/273.15)
        331.3 * (1.0 + self.temperature / 273.15).sqrt()
    }

    /// Вычисляет коэффициент поглощения воздуха
    pub fn air_absorption(&self, frequency: f32, distance: f32) -> f32 {
        // Упрощенная модель поглощения по ISO 9613-1
        let absorption_coef = 0.0001 * frequency * (1.0 - self.humidity / 100.0);
        (-absorption_coef * distance).exp()
    }
}

/// Основной класс аудиосистемы
pub struct AudioSystem {
    /// Активные источники звука
    sources: HashMap<u32, AudioSource>,
    /// Следующий ID источника
    next_source_id: u32,
    /// Аудио слушатель
    pub listener: AudioListener,
    /// Параметры среды
    pub environment: EnvironmentParams,
    /// Кэш декодированных звуков
    sound_cache: Arc<Mutex<HashMap<String, Vec<f32>>>>,
    /// Максимум одновременных источников
    max_sources: u32,
    /// Включена ли окклюзия
    occlusion_enabled: bool,
    /// Аудио устройство вывода (not cloneable - only original owner holds it)
    /// Clones set this to None as audio device cannot be shared
    #[allow(dead_code)]
    audio_device: Option<OutputStream>,
    /// Активный sink для воспроизведения
    sink: Option<Arc<Sink>>,
}

impl Clone for AudioSystem {
    fn clone(&self) -> Self {
        Self {
            sources: self.sources.clone(),
            next_source_id: self.next_source_id,
            listener: self.listener.clone(),
            environment: self.environment.clone(),
            sound_cache: self.sound_cache.clone(),
            max_sources: self.max_sources,
            occlusion_enabled: self.occlusion_enabled,
            // NOTE: _stream is not cloned - only the original owns the audio device.
            // The clone shares the same sink if available, but audio playback will
            // only work if the original AudioSystem is still alive and holding the stream.
            // This is a design trade-off - use AudioSystem::new() for active audio.
            audio_device: None,
            sink: self.sink.clone(),
        }
    }
}

impl AudioSystem {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Инициализация аудиоустройства - не падаем если нет аудио
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("No audio device available: {}, running without audio", e);
                return Ok(Self {
                    sources: HashMap::new(),
                    next_source_id: 1,
                    listener: AudioListener::default(),
                    environment: EnvironmentParams::default(),
                    sound_cache: Arc::new(Mutex::new(HashMap::new())),
                    max_sources: 64,
                    occlusion_enabled: true,
                    audio_device: None,
                    sink: None,
                });
            }
        };

        let sink = match Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Could not create audio sink: {}, running without audio", e);
                return Ok(Self {
                    sources: HashMap::new(),
                    next_source_id: 1,
                    listener: AudioListener::default(),
                    environment: EnvironmentParams::default(),
                    sound_cache: Arc::new(Mutex::new(HashMap::new())),
                    max_sources: 64,
                    occlusion_enabled: true,
                    audio_device: Some(stream),
                    sink: None,
                });
            }
        };

        Ok(Self {
            sources: HashMap::new(),
            next_source_id: 1,
            listener: AudioListener::default(),
            environment: EnvironmentParams::default(),
            sound_cache: Arc::new(Mutex::new(HashMap::new())),
            max_sources: 64,
            occlusion_enabled: true,
            audio_device: Some(stream),
            sink: Some(Arc::new(sink)),
        })
    }

    /// Создает новый источник звука
    pub fn create_source(&mut self, sound_path: &str) -> u32 {
        if self.sources.len() >= self.max_sources as usize {
            // Удаляем самый низкоприоритетный источник
            self.remove_lowest_priority_source();
        }

        let id = self.next_source_id;
        self.next_source_id += 1;

        let source = AudioSource {
            sound_path: sound_path.to_string(),
            ..Default::default()
        };

        self.sources.insert(id, source);
        id
    }

    /// Удаляет источник звука
    pub fn remove_source(&mut self, id: u32) {
        self.sources.remove(&id);
    }

    /// Воспроизводит звук из файла
    pub fn play_sound(&self, sound_path: &str) {
        if let Some(sink) = &self.sink {
            match std::fs::File::open(sound_path) { Ok(file) => {
                match rodio::Decoder::new(file) { Ok(source) => {
                    sink.append(source);
                } _ => {
                    tracing::warn!("Failed to decode sound file: {}", sound_path);
                }}
            } _ => {
                tracing::warn!("Failed to open sound file: {}", sound_path);
            }}
        }
    }

    /// Останавливает все звуки
    pub fn stop_all_sounds(&self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
    }

    /// Обновляет позицию источника
    pub fn set_source_position(&mut self, id: u32, position: nalgebra::Vector3<f32>) {
        if let Some(source) = self.sources.get_mut(&id) {
            source.position = position;
        }
    }

    /// Обновляет позицию слушателя
    pub fn set_listener_position(
        &mut self,
        position: nalgebra::Vector3<f32>,
        forward: nalgebra::Vector3<f32>,
        up: nalgebra::Vector3<f32>,
    ) {
        self.listener.position = position;
        self.listener.forward = forward;
        self.listener.up = up;
    }

    /// Вычисляет 3D параметры для источника (панорамирование, громкость, питч)
    pub fn calculate_3d_params(&self, source_id: u32) -> Option<(f32, f32, f32)> {
        let source = self.sources.get(&source_id)?;

        if !source.spatial {
            return Some((source.volume, 1.0, 0.0));
        }

        // Вектор от слушателя к источнику
        let to_source = source.position - self.listener.position;
        let distance = to_source.norm();

        // Проверка на максимальную дистанцию
        if distance > source.max_distance {
            return Some((0.0, 1.0, 0.0));
        }

        // Затухание по расстоянию (обратное квадратичное)
        let distance_attenuation = if distance < source.min_distance {
            1.0
        } else {
            (source.min_distance / distance).powi(2)
        };

        // Доплер эффект
        let relative_velocity = source.velocity - self.listener.velocity;
        let doppler_shift =
            self.calculate_doppler(relative_velocity, to_source.normalize(), distance);

        // Окклюзия
        let occlusion = if self.occlusion_enabled {
            self.calculate_occlusion(self.listener.position, source.position)
        } else {
            OcclusionResult {
                occluded: false,
                occlusion_factor: 0.0,
                obstacle_count: 0,
                material: OcclusionMaterial::Air,
            }
        };

        // Поглощение средой
        let air_absorption = self.environment.air_absorption(1000.0, distance);

        // Итоговая громкость
        let final_volume = source.volume
            * distance_attenuation
            * (1.0 - occlusion.occlusion_factor * 0.8)
            * air_absorption
            * self.listener.master_volume;

        // Панорамирование (лево/право)
        let right = self.listener.forward.cross(&self.listener.up).normalize();
        let pan = to_source.normalize().dot(&right).clamp(-1.0, 1.0);

        Some((final_volume, doppler_shift, pan))
    }

    /// Вычисляет Doppler сдвиг
    fn calculate_doppler(
        &self,
        relative_velocity: nalgebra::Vector3<f32>,
        direction: nalgebra::Vector3<f32>,
        distance: f32,
    ) -> f32 {
        if distance < 0.001 {
            return 1.0;
        }

        let speed_of_sound = self.environment.speed_of_sound();
        let velocity_toward_listener = -relative_velocity.dot(&direction);

        // Формула Доплера: f' = f * (c / (c - v))
        let doppler = speed_of_sound
            / (speed_of_sound - velocity_toward_listener * self.listener.doppler_factor);
        doppler.clamp(0.5, 2.0) // Ограничиваем эффект
    }

    /// Трассировка луча для проверки окклюзии
    pub fn calculate_occlusion(
        &self,
        from: nalgebra::Vector3<f32>,
        to: nalgebra::Vector3<f32>,
    ) -> OcclusionResult {
        // В реальной реализации здесь была бы трассировка луча через физический движок
        // Для примера возвращаем простую заглушку

        let direction = to - from;
        let distance = direction.norm();

        // Заглушка - проверяем "препятствия" на фиксированных позициях
        let obstacles = [
            (
                nalgebra::Vector3::new(5.0, 0.0, 5.0),
                2.0,
                OcclusionMaterial::Concrete,
            ),
            (
                nalgebra::Vector3::new(-3.0, 0.0, 2.0),
                1.5,
                OcclusionMaterial::Wood,
            ),
        ];

        let mut total_occlusion = 0.0;
        let mut obstacle_count = 0;
        let mut last_material = OcclusionMaterial::Air;

        for (obs_pos, obs_radius, material) in &obstacles {
            // Простейшая проверка пересечения луча со сферой
            let to_obs = *obs_pos - from;
            let projection = to_obs.dot(&direction.normalize());

            if projection > 0.0 && projection < distance {
                let closest_point = from + direction.normalize() * projection;
                let dist_to_line = (closest_point - *obs_pos).norm();

                if dist_to_line < *obs_radius {
                    total_occlusion += material.absorption_coefficient(1000.0);
                    obstacle_count += 1;
                    last_material = *material;
                }
            }
        }

        OcclusionResult {
            occluded: obstacle_count > 0,
            occlusion_factor: total_occlusion.min(1.0),
            obstacle_count,
            material: last_material,
        }
    }

    /// Применяет HRTF (Head-Related Transfer Function) для бинаурального звука
    pub fn apply_hrtf(
        &self,
        samples: &[f32],
        azimuth: f32,
        _elevation: f32,
    ) -> (Vec<f32>, Vec<f32>) {
        // Упрощенная HRTF модель
        // В реальной реализации использовались бы таблицы HRTF (например, CIPIC или MIT databases)

        let left_delay = ((azimuth * 0.0005) as f32).abs();
        let right_delay = ((-azimuth * 0.0005) as f32).abs();

        let left_attenuation = if azimuth > 0.0 { 0.8 } else { 1.0 };
        let right_attenuation = if azimuth < 0.0 { 0.8 } else { 1.0 };

        // Применяем задержку и аттенюацию
        let left_channel: Vec<f32> = samples
            .iter()
            .zip(samples.iter().skip((left_delay * 44100.0) as usize))
            .map(|(&s, &delayed)| s * left_attenuation + delayed * 0.3)
            .collect();

        let right_channel: Vec<f32> = samples
            .iter()
            .zip(samples.iter().skip((right_delay * 44100.0) as usize))
            .map(|(&s, &delayed)| s * right_attenuation + delayed * 0.3)
            .collect();

        (left_channel, right_channel)
    }

    /// Реверберация на основе размера помещения
    pub fn apply_reverb(&self, samples: &[f32], room_size: f32, decay: f32) -> Vec<f32> {
        // Простая реверберация с использованием линии задержки
        let delay_samples = (room_size * 100.0) as usize;
        let mut output = Vec::with_capacity(samples.len() + delay_samples);

        let mut delay_line = vec![0.0f32; delay_samples];
        let mut write_idx = 0;

        for &sample in samples {
            let delayed = delay_line[write_idx];
            let new_sample = sample + delayed * decay;
            delay_line[write_idx] = new_sample;
            output.push(new_sample);

            write_idx = (write_idx + 1) % delay_samples;
        }

        output
    }

    /// Удаляет самый низкоприоритетный источник
    fn remove_lowest_priority_source(&mut self) {
        if let Some((&lowest_id, _)) = self.sources.iter().min_by_key(|(_, s)| s.priority) {
            self.sources.remove(&lowest_id);
        }
    }

    /// Обновляет все источники (вызывать каждый кадр)
    pub fn update(&mut self) {
        // Здесь должна быть логика обновления состояний источников
        // и отправки данных на аудио устройство
        if let Some(sink) = &self.sink {
            sink.set_volume(self.listener.master_volume);
        }
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to initialize audio system: {}, using silent fallback",
                e
            );
            // Create a minimal silent audio system as fallback
            AudioSystem {
                sources: HashMap::new(),
                next_source_id: 1,
                listener: AudioListener::default(),
                environment: EnvironmentParams::default(),
                sound_cache: Arc::new(Mutex::new(HashMap::new())),
                max_sources: 64,
                occlusion_enabled: true,
                audio_device: None,
                sink: None,
            }
        })
    }
}
