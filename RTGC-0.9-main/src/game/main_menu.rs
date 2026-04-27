//! Главное меню игры
//! 
//! Реализует экраны: главное меню, настройки, загрузка игры

use crate::graphics::UiCommand;
use crate::input::InputManager;
use crate::graphics::font::FontAtlas;
use std::sync::Arc;

/// Состояния меню
#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    /// Главное меню
    Main,
    /// Настройки
    Settings,
    /// Загрузка игры
    Loading,
    /// Пауза
    Pause,
}

/// Кнопка меню
#[derive(Debug, Clone)]
pub struct MenuButton {
    pub id: usize,
    pub text: String,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub action: ButtonAction,
    pub is_hovered: bool,
    pub is_pressed: bool,
}

/// Действие кнопки
#[derive(Debug, Clone)]
pub enum ButtonAction {
    /// Начать новую игру
    NewGame,
    /// Загрузить игру
    LoadGame,
    /// Настройки
    Settings,
    /// Выход из игры
    Quit,
    /// Назад в предыдущее меню
    Back,
    /// Применить настройку
    ApplySetting(String, String),
}

/// Главное меню
pub struct MainMenu {
    state: MenuState,
    buttons: Vec<MenuButton>,
    font: Option<Arc<FontAtlas>>,
    mouse_position: [f32; 2],
    title_text: String,
    version_text: String,
}

impl MainMenu {
    pub fn new() -> Self {
        let mut menu = Self {
            state: MenuState::Main,
            buttons: Vec::new(),
            font: None,
            mouse_position: [0.0, 0.0],
            title_text: "RTGC-0.9".to_string(),
            version_text: "v0.9.0".to_string(),
        };
        
        // Создаём кнопки главного меню
        menu.create_main_menu_buttons();
        
        menu
    }
    
    /// Установить шрифт
    pub fn set_font(&mut self, font: Arc<FontAtlas>) {
        self.font = Some(font);
    }
    
    /// Создать кнопки главного меню
    fn create_main_menu_buttons(&mut self) {
        self.buttons.clear();
        
        let button_width = 200.0;
        let button_height = 50.0;
        let start_x = 100.0;
        let start_y = 300.0;
        let spacing = 70.0;
        
        self.buttons.push(MenuButton {
            id: 0,
            text: "Новая игра".to_string(),
            position: [start_x, start_y],
            size: [button_width, button_height],
            action: ButtonAction::NewGame,
            is_hovered: false,
            is_pressed: false,
        });
        
        self.buttons.push(MenuButton {
            id: 1,
            text: "Загрузить игру".to_string(),
            position: [start_x, start_y + spacing],
            size: [button_width, button_height],
            action: ButtonAction::LoadGame,
            is_hovered: false,
            is_pressed: false,
        });
        
        self.buttons.push(MenuButton {
            id: 2,
            text: "Настройки".to_string(),
            position: [start_x, start_y + spacing * 2.0],
            size: [button_width, button_height],
            action: ButtonAction::Settings,
            is_hovered: false,
            is_pressed: false,
        });
        
        self.buttons.push(MenuButton {
            id: 3,
            text: "Выход".to_string(),
            position: [start_x, start_y + spacing * 3.0],
            size: [button_width, button_height],
            action: ButtonAction::Quit,
            is_hovered: false,
            is_pressed: false,
        });
    }
    
    /// Создать кнопки настроек
    fn create_settings_buttons(&mut self) {
        self.buttons.clear();
        
        let button_width = 200.0;
        let button_height = 50.0;
        let start_x = 100.0;
        let start_y = 400.0;
        
        self.buttons.push(MenuButton {
            id: 10,
            text: "Назад".to_string(),
            position: [start_x, start_y],
            size: [button_width, button_height],
            action: ButtonAction::Back,
            is_hovered: false,
            is_pressed: false,
        });
    }
    
    /// Обновить состояние меню (обработка ввода, анимации)
    pub fn update(&mut self, dt: f32, input: &InputManager) -> Option<MenuAction> {
        use crate::input::input_module::MouseButton;
        let state = input.state();
        let mouse_pos = state.mouse_position();
        self.mouse_position = [mouse_pos.0 as f32, mouse_pos.1 as f32];
        
        // Сначала собираем данные о кнопках (индексы и действия)
        let button_actions: Vec<_> = self.buttons.iter().enumerate().map(|(i, b)| {
            let hovered = self.is_point_in_rect(self.mouse_position, b.position, b.size);
            let action = if hovered && state.is_mouse_button_just_pressed(MouseButton::Left) {
                Some((i, b.action.clone()))
            } else {
                None
            };
            (hovered, action)
        }).collect();
        
        // Теперь обновляем состояние кнопок
        let mut action_to_perform: Option<MenuAction> = None;
        for (i, button) in self.buttons.iter_mut().enumerate() {
            let (hovered, click_action) = &button_actions[i];
            button.is_hovered = *hovered;
            
            if click_action.is_some() {
                button.is_pressed = true;
            } else if button.is_pressed && !state.is_mouse_button_down(MouseButton::Left) {
                button.is_pressed = false;
            }
        }
        // Проверяем действия после изменения состояния кнопок
        for button in &self.buttons {
            if button.is_pressed && button.is_hovered {
                action_to_perform = Some(self.handle_button_action(&button.action.clone()));
                break;
            }
        }
        
        action_to_perform
    }
    
    /// Отрисовка меню
    pub fn render(&self, ui_commands: &mut Vec<UiCommand>, window_size: [f32; 2]) {
        // Цвета
        let bg_color = [0.1, 0.1, 0.15, 1.0];
        let button_normal = [0.3, 0.3, 0.35, 1.0];
        let button_hover = [0.4, 0.4, 0.5, 1.0];
        let button_pressed = [0.5, 0.5, 0.6, 1.0];
        let text_color = [1.0, 1.0, 1.0, 1.0];
        let title_color = [0.9, 0.7, 0.2, 1.0];
        
        // Фон на весь экран
        ui_commands.push(UiCommand::Rect {
            position: [0.0, 0.0],
            size: window_size,
            color: bg_color,
        });
        
        // Заголовок
        match self.state {
            MenuState::Main => {
                // Рисуем заголовок
                ui_commands.push(UiCommand::Text {
                    text: self.title_text.clone(),
                    position: [100.0, 100.0],
                    font_size: 72.0,
                    color: title_color,
                });
                
                ui_commands.push(UiCommand::Text {
                    text: self.version_text.clone(),
                    position: [100.0, 180.0],
                    font_size: 24.0,
                    color: [0.7, 0.7, 0.7, 1.0],
                });
                
                // Кнопки
                for button in &self.buttons {
                    let color = if button.is_pressed {
                        button_pressed
                    } else if button.is_hovered {
                        button_hover
                    } else {
                        button_normal
                    };
                    
                    ui_commands.push(UiCommand::Rect {
                        position: button.position,
                        size: button.size,
                        color,
                    });
                    
                    // Текст кнопки (центрированный)
                    let text_x = button.position[0] + (button.size[0] - self.measure_text_width(&button.text, 28.0)) / 2.0;
                    let text_y = button.position[1] + (button.size[1] - 28.0) / 2.0;
                    
                    ui_commands.push(UiCommand::Text {
                        text: button.text.clone(),
                        position: [text_x, text_y],
                        font_size: 28.0,
                        color: text_color,
                    });
                }
            }
            MenuState::Settings => {
                ui_commands.push(UiCommand::Text {
                    text: "Настройки".to_string(),
                    position: [100.0, 100.0],
                    font_size: 48.0,
                    color: title_color,
                });
                
                // Здесь будут элементы управления настройками
                
                for button in &self.buttons {
                    let color = if button.is_pressed {
                        button_pressed
                    } else if button.is_hovered {
                        button_hover
                    } else {
                        button_normal
                    };
                    
                    ui_commands.push(UiCommand::Rect {
                        position: button.position,
                        size: button.size,
                        color,
                    });
                    
                    let text_x = button.position[0] + (button.size[0] - self.measure_text_width(&button.text, 28.0)) / 2.0;
                    let text_y = button.position[1] + (button.size[1] - 28.0) / 2.0;
                    
                    ui_commands.push(UiCommand::Text {
                        text: button.text.clone(),
                        position: [text_x, text_y],
                        font_size: 28.0,
                        color: text_color,
                    });
                }
            }
            MenuState::Loading => {
                ui_commands.push(UiCommand::Text {
                    text: "Загрузка...".to_string(),
                    position: [100.0, 100.0],
                    font_size: 48.0,
                    color: title_color,
                });
            }
            MenuState::Pause => {
                ui_commands.push(UiCommand::Text {
                    text: "Пауза".to_string(),
                    position: [100.0, 100.0],
                    font_size: 48.0,
                    color: title_color,
                });
            }
        }
    }
    
    /// Обработать действие кнопки
    fn handle_button_action(&self, action: &ButtonAction) -> MenuAction {
        match action {
            ButtonAction::NewGame => MenuAction::StartNewGame,
            ButtonAction::LoadGame => MenuAction::LoadGame,
            ButtonAction::Settings => MenuAction::ShowSettings,
            ButtonAction::Quit => MenuAction::Quit,
            ButtonAction::Back => MenuAction::BackToMain,
            ButtonAction::ApplySetting(key, value) => {
                MenuAction::ApplySetting(key.clone(), value.clone())
            }
        }
    }
    
    /// Проверить точку в прямоугольнике
    fn is_point_in_rect(&self, point: [f32; 2], rect_pos: [f32; 2], rect_size: [f32; 2]) -> bool {
        point[0] >= rect_pos[0]
            && point[0] <= rect_pos[0] + rect_size[0]
            && point[1] >= rect_pos[1]
            && point[1] <= rect_pos[1] + rect_size[1]
    }
    
    /// Измерить ширину текста с использованием реального шрифта
    fn measure_text_width(&self, text: &str, font_size: f32) -> f32 {
        if let Some(ref font) = self.font {
            // Используем реальные метрики шрифта
            let (width, _) = font.measure_text(text);
            // Масштабируем относительно размера шрифта
            let scale = font_size / font.pixel_height;
            width * scale
        } else {
            // Заглушка, если шрифт не установлен
            text.len() as f32 * font_size * 0.6
        }
    }
    
    /// Переключить состояние меню
    pub fn set_state(&mut self, state: MenuState) {
        self.state = state.clone();
        
        match state {
            MenuState::Main => self.create_main_menu_buttons(),
            MenuState::Settings => self.create_settings_buttons(),
            _ => {}
        }
    }
    
    /// Получить текущее состояние
    pub fn get_state(&self) -> &MenuState {
        &self.state
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

/// Действия меню (результат обработки ввода)
#[derive(Debug, Clone)]
pub enum MenuAction {
    /// Начать новую игру
    StartNewGame,
    /// Загрузить игру
    LoadGame,
    /// Показать настройки
    ShowSettings,
    /// Выйти из игры
    Quit,
    /// Вернуться в главное меню
    BackToMain,
    /// Применить настройку
    ApplySetting(String, String),
}
