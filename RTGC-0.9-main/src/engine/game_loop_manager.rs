//! Менеджер игрового цикла - инкапсуляция основного цикла обновлений
//! 
//! Этот модуль управляет обновлением всех игровых систем,
//! предоставляя централизованную логику обновления.

use crate::engine::state::EngineState;
use crate::game::interaction::InteractionSystem;
use crate::game::debug_menu::DebugMenu;
use crate::physics::PhysicsWorld;
use crate::ui::HudManager;
use crate::graphics::particles::ParticleSystem;
use crate::graphics::renderer::DebugRenderer;
use nalgebra::Vector3;
use tracing::{info, warn};

/// Менеджер игрового цикла
pub struct GameLoopManager {
    /// Система взаимодействия
    interaction_system: InteractionSystem,
    /// Отладочное меню
    debug_menu: DebugMenu,
    /// HUD менеджер
    hud_manager: HudManager,
    /// Система частиц
    particle_system: ParticleSystem,
    /// Отладочный рендерер
    debug_renderer: DebugRenderer,
    /// Таймер сохранения
    save_timer: f32,
    /// Режим отладки
    debug_mode: bool,
}

impl GameLoopManager {
    /// Создаёт новый менеджер игрового цикла
    pub fn new(
        interaction_system: InteractionSystem,
        debug_menu: DebugMenu,
        hud_manager: HudManager,
        particle_system: ParticleSystem,
        debug_renderer: DebugRenderer,
    ) -> Self {
        Self {
            interaction_system,
            debug_menu,
            hud_manager,
            particle_system,
            debug_renderer,
            save_timer: 0.0,
            debug_mode: false,
        }
    }
    
    /// Обновляет все системы игрового цикла
    pub fn update(
        &mut self,
        dt: f32,
        game_state: &EngineState,
        player_position: Option<Vector3<f32>>,
        player_forward: Vector3<f32>,
        physics_world: &PhysicsWorld,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Проверка на NaN/Inf
        if !dt.is_finite() || dt <= 0.0 {
            warn!(target: "gameloop", "Invalid dt value: {}, skipping update", dt);
            return Ok(());
        }
        
        // Обновление системы взаимодействия
        if game_state.is_playing() {
            let player_pos = player_position.unwrap_or_else(Vector3::zeros);
            self.interaction_system.update(dt, player_pos, player_forward, 4.0, physics_world);
        }
        
        // Обновление отладочного меню
        if self.debug_mode {
            self.debug_menu.update_fps(1.0 / dt.max(0.0001), dt * 1000.0);
            self.debug_menu.update_ram_usage(0.0);
        }
        
        // Обновление HUD с данными автомобиля (если есть)
        // Данные передаются извне
        
        // Обновление системы частиц
        self.particle_system.update(dt);
        
        // Обновление debug renderer в режиме отладки (очистка кадровых данных)
        if self.debug_mode {
            self.debug_renderer.clear();
        }
        
        // Таймер автосохранения
        self.save_timer += dt;
        if self.save_timer >= 60.0 {
            info!(target: "gameloop", "Auto-save triggered");
            self.save_timer = 0.0;
            // Сигнал о необходимости сохранения
        }
        
        Ok(())
    }
    
    /// Обрабатывает взаимодействие по клавише F
    pub fn try_interact(&mut self, player_state: &mut crate::game::player::PlayerState) -> bool {
        let result = self.interaction_system.try_interact(player_state);
        if result.success {
            info!(target: "interaction", "Interaction: {}", result.message);
        }
        result.success
    }
    
    /// Получает ссылку на HUD менеджер
    pub fn get_hud_manager(&self) -> &HudManager {
        &self.hud_manager
    }
    
    /// Получает мутабельную ссылку на HUD менеджер
    pub fn get_hud_manager_mut(&mut self) -> &mut HudManager {
        &mut self.hud_manager
    }
    
    /// Устанавливает режим отладки
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }
    
    /// Получает режим отладки
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }
    
    /// Получает ссылку на отладочное меню
    pub fn get_debug_menu(&self) -> &DebugMenu {
        &self.debug_menu
    }
    
    /// Получает мутабельную ссылку на отладочное меню
    pub fn get_debug_menu_mut(&mut self) -> &mut DebugMenu {
        &mut self.debug_menu
    }
    
    /// Проверяет, пора ли делать автосохранение
    pub fn should_autosave(&self) -> bool {
        self.save_timer >= 60.0
    }
    
    /// Сбрасывает таймер сохранения
    pub fn reset_save_timer(&mut self) {
        self.save_timer = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::player::PlayerState;
    
    #[test]
    fn test_game_loop_manager_creation() {
        let manager = GameLoopManager::new(
            InteractionSystem::new(),
            DebugMenu::new(),
            HudManager::new(),
            ParticleSystem::new(100),
            DebugRenderer::new(),
        );
        
        assert!(!manager.is_debug_mode());
        assert!(!manager.should_autosave());
    }
    
    #[test]
    fn test_debug_mode_toggle() {
        let mut manager = GameLoopManager::new(
            InteractionSystem::new(),
            DebugMenu::new(),
            HudManager::new(),
            ParticleSystem::new(100),
            DebugRenderer::new(),
        );
        
        assert!(!manager.is_debug_mode());
        manager.set_debug_mode(true);
        assert!(manager.is_debug_mode());
    }
    
    #[test]
    fn test_update_with_invalid_dt() {
        let mut manager = GameLoopManager::new(
            InteractionSystem::new(),
            DebugMenu::new(),
            HudManager::new(),
            ParticleSystem::new(100),
            DebugRenderer::new(),
        );
        
        let game_state = EngineState::playing(42);
        
        // Обновление с NaN должно быть пропущено
        let result = manager.update(f32::NAN, &game_state, None, Vector3::z());
        assert!(result.is_ok());
        
        // Обновление с отрицательным dt должно быть пропущено
        let result = manager.update(-1.0, &game_state, None, Vector3::z());
        assert!(result.is_ok());
    }
}
