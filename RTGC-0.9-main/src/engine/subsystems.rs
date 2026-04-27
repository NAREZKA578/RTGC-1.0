//! Подсистемы движка - инкапсулированные модули для разделения ответственности
//!
//! Этот модуль предоставляет структуру для хранения всех подсистем движка,
//! обеспечивая слабую связанность между компонентов.

use crate::audio::AudioSystem;
use crate::ecs::EcsManager;
use crate::game::debug_menu::DebugMenu;
use crate::game::loading_manager::LoadingManager;
use crate::game::save::SaveSystem;
use crate::game::ui::UIManager;
use crate::graphics::renderer::DebugRenderer;
use crate::graphics::material::MaterialManager;
use crate::graphics::particles::ParticleSystem;
use crate::graphics::renderer::Renderer;
use crate::input::InputManager;
use crate::physics;
use crate::ui::HudManager;
use crate::world::DayNightCycle;

/// Контейнер для всех подсистем движка
///
/// Эта структура инкапсулирует все подсистемы, предоставляя контролируемый доступ
/// к ним через методы-геттеры. Это уменьшает связанность и упрощает тестирование.
#[derive(Clone)]
pub struct EngineSubsystems {
    /// Графическая подсистема (рендеринг, материалы, частицы)
    pub graphics: GraphicsSubsystem,

    /// Физическая подсистема
    pub physics: PhysicsSubsystem,

    /// Подсистема ввода
    pub input: InputManager,

    /// Аудио подсистема
    pub audio: AudioSystem,

    /// ECS менеджер
    pub ecs: EcsManager,

    /// UI подсистема
    pub ui: UISubsystem,

    /// Подсистема игрового мира
    pub world: WorldSubsystem,

    /// Подсистема загрузки ресурсов
    pub loading: LoadingManager,

    /// Подсистема сохранения
    pub save: SaveSystem,
}

impl EngineSubsystems {
    /// Создаёт новый контейнер подсистем
    pub fn new(
        graphics: GraphicsSubsystem,
        physics: PhysicsSubsystem,
        input: InputManager,
        audio: AudioSystem,
        ecs: EcsManager,
        ui: UISubsystem,
        world: WorldSubsystem,
        loading: LoadingManager,
        save: SaveSystem,
    ) -> Self {
        Self {
            graphics,
            physics,
            input,
            audio,
            ecs,
            ui,
            world,
            loading,
            save,
        }
    }

    /// Обновляет все подсистемы
    pub fn update(&mut self, dt: f32) {
        self.graphics.update(dt);
        self.physics.update(dt);
        self.ui.ui_manager.update(dt);
        self.world.update(dt);
        self.ecs.update(dt);
    }

    /// Обновляет HUD с реальными данными (вызывается отдельно с контекстом)
    pub fn update_hud(&mut self, vehicle_data: &crate::ui::hud::VehicleHudData, layout: &crate::ui::hud::HudLayout, dt: f32) {
        self.ui.hud_manager.update(vehicle_data, layout, dt);
    }
}

/// Графическая подсистема
/// Clone intentionally excludes renderer since it contains non-clonable GPU resources.
/// Use clone_without_renderer() to get a copy for thread-safe sharing.
pub struct GraphicsSubsystem {
    pub renderer: Option<Renderer>,
    pub material_manager: MaterialManager,
    pub particle_system: ParticleSystem,
    pub debug_renderer: DebugRenderer,
}

impl Clone for GraphicsSubsystem {
    fn clone(&self) -> Self {
        Self {
            renderer: None,
            material_manager: self.material_manager.clone(),
            particle_system: self.particle_system.clone(),
            debug_renderer: self.debug_renderer.clone(),
        }
    }
}

impl GraphicsSubsystem {
    /// Creates a clone that excludes the renderer (for thread-safe sharing)
    pub fn clone_without_renderer(&self) -> Self {
        Self {
            renderer: None,
            material_manager: self.material_manager.clone(),
            particle_system: self.particle_system.clone(),
            debug_renderer: self.debug_renderer.clone(),
        }
    }
}

impl GraphicsSubsystem {
    pub fn new(
        renderer: Option<Renderer>,
        material_manager: MaterialManager,
        particle_system: ParticleSystem,
        debug_renderer: DebugRenderer,
    ) -> Self {
        Self {
            renderer,
            material_manager,
            particle_system,
            debug_renderer,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.particle_system.update(dt);
    }

    pub fn render(&mut self) -> Result<(), crate::error::EngineError> {
        if let Some(ref mut renderer) = self.renderer {
            renderer.render().map_err(|e: String| {
                crate::error::EngineError::Graphics(crate::error::GraphicsError::PipelineError(
                    e.to_string(),
                ))
            })?;
        }
        Ok(())
    }
}

/// Физическая подсистема
#[derive(Clone)]
pub struct PhysicsSubsystem {
    // Clone - O(1) для маленьких структур, данные шарингатся через внутренний Arc в PhysicsWorld
    pub physics_world: physics::PhysicsWorld,
}

impl PhysicsSubsystem {
    pub fn new(physics_world: physics::PhysicsWorld) -> Self {
        Self { physics_world }
    }

    pub fn update(&mut self, _dt: f32) {
        // Базовое обновление физического мира
    }

    pub fn step_simulation(&mut self, dt: f32) {
        self.physics_world.step(dt);
    }
}

/// UI подсистема
#[derive(Clone)]
pub struct UISubsystem {
    pub hud_manager: HudManager,
    pub ui_manager: UIManager,
    pub debug_menu: DebugMenu,
}

impl UISubsystem {
    pub fn new(hud_manager: HudManager, ui_manager: UIManager, debug_menu: DebugMenu) -> Self {
        Self {
            hud_manager,
            ui_manager,
            debug_menu,
        }
    }

    pub fn update(&mut self, _dt: f32) {
        self.hud_manager.update(&crate::ui::hud::VehicleHudData::default(), &crate::ui::hud::HudLayout::default(), _dt);
        self.ui_manager.update(_dt);
    }
}

/// Подсистема игрового мира
#[derive(Clone)]
pub struct WorldSubsystem {
    pub day_night_cycle: DayNightCycle,
}

impl WorldSubsystem {
    pub fn new(day_night_cycle: DayNightCycle) -> Self {
        Self { day_night_cycle }
    }

    pub fn update(&mut self, dt: f32) {
        self.day_night_cycle.advance_time(dt);
    }
}
