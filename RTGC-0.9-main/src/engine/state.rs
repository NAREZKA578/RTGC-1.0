//! Состояния движка - явное управление состоянием приложения
//!
//! Этот модуль предоставляет типизированные состояния для управления жизненным циклом
//! движка, устраняя дублирование состояний и обеспечивая единый источник истины.

/// Основное состояние движка
///
/// Это enum представляет все возможные состояния приложения в любой момент времени.
/// Использование enum вместо разрозненных флагов обеспечивает:
/// - Единственный источник истины для состояния
/// - Типобезопасные переходы между состояниями
/// - Исключение невозможных состояний на уровне типов
#[derive(Debug, Clone, PartialEq)]
pub enum EngineState {
    /// Движок инициализируется (загрузка ресурсов, создание контекста)
    Initializing {
        /// Прогресс инициализации (0.0 - 1.0)
        progress: f32,
        /// Сообщение о текущем этапе загрузки
        message: String,
    },

    /// Главное меню
    MainMenu {
        /// Состояние меню
        menu_state: MenuState,
    },

    /// Создание персонажа
    CharacterCreation {
        /// Прогресс создания персонажа
        progress: f32,
    },

    /// Загрузка мира/уровня
    Loading {
        /// Прогресс загрузки (0.0 - 1.0)
        progress: f32,
        /// Тип загружаемого ресурса
        resource_type: LoadingResourceType,
    },

    /// Игра активна
    Playing {
        /// Текущий мир/уровень
        world_id: u64,
        /// Количество игроков в сессии
        player_count: u32,
    },

    /// Игра на паузе
    Paused {
        /// Причина паузы
        reason: PauseReason,
        /// Наложение паузы (UI)
        overlay_visible: bool,
    },

    /// Ошибка в работе движка
    Error {
        /// Описание ошибки
        reason: String,
        /// Критичность ошибки
        critical: bool,
    },
}

/// Состояние главного меню
#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    /// Главное меню активно
    Active,
    /// Подменю настроек
    Settings,
    /// Подменю выбора миссии
    MissionSelect,
    /// Подменю загрузки сохранения
    LoadGame,
    /// Выход из игры
    Quitting,
}

/// Тип загружаемого ресурса
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingResourceType {
    /// Загрузка мира
    World,
    /// Загрузка транспортного средства
    Vehicle,
    /// Загрузка персонажа
    Character,
    /// Загрузка миссии
    Mission,
    /// Загрузка текстур
    Textures,
    /// Загрузка моделей
    Models,
}

/// Причина паузы
#[derive(Debug, Clone, PartialEq)]
pub enum PauseReason {
    /// Пользователь поставил на паузу
    UserRequested,
    /// Потеря фокуса окна
    WindowLostFocus,
    /// Загрузка фоном
    BackgroundLoading,
    /// Ожидание сети
    NetworkWait,
}

impl EngineState {
    /// Создаёт состояние инициализации
    pub fn initializing(message: impl Into<String>) -> Self {
        Self::Initializing {
            progress: 0.0,
            message: message.into(),
        }
    }

    /// Создаёт состояние главного меню
    pub fn main_menu() -> Self {
        Self::MainMenu {
            menu_state: MenuState::Active,
        }
    }

    /// Создаёт состояние загрузки
    pub fn loading(resource_type: LoadingResourceType) -> Self {
        Self::Loading {
            progress: 0.0,
            resource_type,
        }
    }

    /// Создаёт состояние игры
    pub fn playing(world_id: u64) -> Self {
        Self::Playing {
            world_id,
            player_count: 1,
        }
    }

    /// Создаёт состояние паузы
    pub fn paused(reason: PauseReason) -> Self {
        Self::Paused {
            reason,
            overlay_visible: true,
        }
    }

    /// Создаёт состояние ошибки
    pub fn error(reason: impl Into<String>, critical: bool) -> Self {
        Self::Error {
            reason: reason.into(),
            critical,
        }
    }

    /// Проверяет, находится ли движок в состоянии игры
    pub fn is_playing(&self) -> bool {
        matches!(self, EngineState::Playing { .. })
    }

    /// Проверяет, находится ли движок в состоянии меню
    pub fn is_in_menu(&self) -> bool {
        matches!(self, EngineState::MainMenu { .. })
    }

    /// Проверяет, находится ли движок в состоянии загрузки
    pub fn is_loading(&self) -> bool {
        matches!(
            self,
            EngineState::Loading { .. } | EngineState::Initializing { .. }
        )
    }

    /// Проверяет, находится ли движок в состоянии ошибки
    pub fn is_error(&self) -> bool {
        matches!(self, EngineState::Error { .. })
    }

    /// Проверяет, находится ли движок на паузе
    pub fn is_paused(&self) -> bool {
        matches!(self, EngineState::Paused { .. })
    }

    /// Переключает состояние паузы (Paused <-> Playing)
    pub fn toggle_pause(&mut self) {
        *self = match self {
            EngineState::Playing {
                world_id,
                player_count,
            } => EngineState::Paused {
                reason: PauseReason::UserRequested,
                overlay_visible: true,
            },
            EngineState::Paused {
                reason: _,
                overlay_visible: _,
            } => {
                // Возвращаемся в Playing с дефолтными значениями
                // В реальной игре нужно сохранить world_id и player_count
                EngineState::Playing {
                    world_id: 0,
                    player_count: 1,
                }
            }
            _ => return, // Не переключаем в других состояниях
        };
    }

    /// Возвращает прогресс загрузки, если применимо
    pub fn loading_progress(&self) -> Option<f32> {
        match self {
            EngineState::Initializing { progress, .. } => Some(*progress),
            EngineState::Loading { progress, .. } => Some(*progress),
            _ => None,
        }
    }

    /// Устанавливает прогресс загрузки
    pub fn set_loading_progress(&mut self, progress: f32) {
        let progress = progress.clamp(0.0, 1.0);
        match self {
            EngineState::Initializing { progress: p, .. } => *p = progress,
            EngineState::Loading { progress: p, .. } => *p = progress,
            _ => {}
        }
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::initializing("Starting engine...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_state_initialization() {
        let state = EngineState::initializing("Loading assets...");
        assert!(state.is_loading());
        assert_eq!(state.loading_progress(), Some(0.0));
    }

    #[test]
    fn test_engine_state_transitions() {
        let mut state = EngineState::default();
        assert!(state.is_loading());

        state = EngineState::main_menu();
        assert!(state.is_in_menu());

        state = EngineState::playing(42);
        assert!(state.is_playing());

        state = EngineState::paused(PauseReason::UserRequested);
        assert!(!state.is_playing());
    }

    #[test]
    fn test_loading_progress_clamping() {
        let mut state = EngineState::loading(LoadingResourceType::World);
        state.set_loading_progress(1.5);
        assert_eq!(state.loading_progress(), Some(1.0));

        state.set_loading_progress(-0.5);
        assert_eq!(state.loading_progress(), Some(0.0));
    }
}
