# RTGC-1.0 — ПОЛНАЯ АРХИТЕКТУРА МЕНЮ, ЗАГРУЗКИ И ИНИЦИАЛИЗАЦИИ
### Собственный движок на Rust | Windows 10 | OpenGL (glow + glutin + winit)
> Версия документа: 1.0 | Апрель 2026 | Автор: NAREZKA578

---

## СОДЕРЖАНИЕ

1. [Философия архитектуры](#1-философия-архитектуры)
2. [Зависимости — Cargo.toml (полный список)](#2-зависимости--cargotoml-полный-список)
3. [Полная структура папок и файлов](#3-полная-структура-папок-и-файлов)
4. [Глобальная машина состояний (AppState)](#4-глобальная-машина-состояний-appstate)
5. [Точка входа main.rs — полная схема](#5-точка-входа-mainrs--полная-схема)
6. [Подсистема рендеринга UI (ui_renderer)](#6-подсистема-рендеринга-ui-ui_renderer)
7. [Шрифтовая система (font_system)](#7-шрифтовая-система-font_system)
8. [Аудиосистема меню (audio)](#8-аудиосистема-меню-audio)
9. [Анимационная система (tweening)](#9-анимационная-система-tweening)
10. [Главное меню (MainMenu)](#10-главное-меню-mainmenu)
11. [Экран настроек (SettingsScreen)](#11-экран-настроек-settingsscreen)
12. [Экран создания персонажа (CharacterCreation)](#12-экран-создания-персонажа-charactercreation)
13. [Система загрузки — 11 стадий (LoadingScreen)](#13-система-загрузки--11-стадий-loadingscreen)
14. [Переход в игровую сцену (Playing)](#14-переход-в-игровую-сцену-playing)
15. [Файл конфигурации игрока (config/)](#15-файл-конфигурации-игрока-config)
16. [Ассеты — какие файлы, где, в каком формате](#16-ассеты--какие-файлы-где-в-каком-формате)
17. [Порядок реализации (roadmap)](#17-порядок-реализации-roadmap)

---

## 1. ФИЛОСОФИЯ АРХИТЕКТУРЫ

### Принципы
```
Один поток рендеринга    — OpenGL НЕ thread-safe, весь рендер в главном потоке
Один поток загрузки      — crossbeam-channel отправляет прогресс в главный поток
Нет сторонних UI фреймов — собственный immediate-mode рендерер прямоугольников + текста
Состояния — стек          — можно «наложить» паузу/настройки поверх любого экрана
Данные отделены от логики — каждый экран = struct данных + impl логики + fn render()
```

### Диаграмма переходов состояний
```
[Запуск] ──▶ Splash ──▶ MainMenu ─────────────────────────▶ [Выход]
                            │
                    ┌───────┼───────────────┐
                    ▼       ▼               ▼
               Settings  CharacterCreation  LoadGame
                    │       │               │
                    └───────┴───────────────┘
                                │
                                ▼
                         LoadingScreen (11 стадий)
                                │
                                ▼
                            Playing ◀──▶ PauseMenu
                                │
                                ▼
                            [Выход из сессии → MainMenu]
```

---

## 2. ЗАВИСИМОСТИ — Cargo.toml (ПОЛНЫЙ СПИСОК)

```toml
[package]
name    = "rtgc"
version = "1.0.0-dev"
edition = "2021"
authors = ["NAREZKA578"]
description = "RTGC-1.0 — Russian Truck & Helicopter Game"
license = "Apache-2.0"

# ─── ОКНО И ВВОД ──────────────────────────────────────────────────────────────
[dependencies]
winit           = "0.30"          # Окно, события мыши/клавиатуры/геймпада

# ─── OPENGL РЕНДЕРИНГ ─────────────────────────────────────────────────────────
glow            = "0.14"          # Типизированные OpenGL биндинги (без unsafe-ада)
glutin          = "0.32"          # OpenGL контекст поверх winit
glutin-winit    = "0.5"           # Интеграция glutin ↔ winit

# ─── МАТЕМАТИКА ───────────────────────────────────────────────────────────────
nalgebra        = "0.33"          # Vec2, Vec3, Mat4, Quat — вся математика движка
glam            = "0.28"          # БОЛЕЕ БЫСТРЫЕ Vec2f/Vec4f для UI (SIMD)
                                  # nalgebra — физика/3D; glam — UI/2D

# ─── ШРИФТЫ ───────────────────────────────────────────────────────────────────
fontdue         = "0.9"           # Чистый Rust, растеризация TTF/OTF в bitmap
                                  # Без системных зависимостей — работает везде

# ─── ИЗОБРАЖЕНИЯ ──────────────────────────────────────────────────────────────
image           = { version = "0.25", default-features = false,
                    features = ["png", "jpeg"] }
                                  # Загрузка PNG/JPG для текстур меню, иконок

# ─── АУДИО ────────────────────────────────────────────────────────────────────
kira            = "0.9"           # Современный аудио движок: пространственный звук,
                                  # fade-in/out, clock-sync. Лучший выбор для игр.
                                  # Альтернатива: rodio (проще, но без fade-контроля)

# ─── СЕРИАЛИЗАЦИЯ / КОНФИГ ────────────────────────────────────────────────────
serde           = { version = "1.0", features = ["derive"] }
toml            = "0.8"           # Парсинг .toml: настройки, ВУЗы, транспорт

# ─── ПОТОКИ И КАНАЛЫ ─────────────────────────────────────────────────────────
crossbeam-channel = "0.5"         # Канал загрузочного потока → UI поток
rayon             = "1.10"        # Параллельная загрузка ассетов

# ─── ЛОГИРОВАНИЕ ──────────────────────────────────────────────────────────────
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# ─── УТИЛИТЫ ──────────────────────────────────────────────────────────────────
bytemuck     = { version = "1.16", features = ["derive"] }
                                  # Безопасное &[T] → &[u8] для OpenGL буферов
uuid         = { version = "1.8",  features = ["v4"] }
                                  # ID для сохранений персонажа
rand         = "0.8"              # RNG для стартовых параметров персонажа
anyhow       = "1.0"              # Эргономичная обработка ошибок (Result<T, anyhow::Error>)
dirs         = "5.0"              # %APPDATA%\RTGC — путь к сохранениям на Windows

# ─── ПРОФИЛИ СБОРКИ ───────────────────────────────────────────────────────────
[profile.dev]
opt-level = 1

[profile.release]
lto           = "thin"
codegen-units = 1
strip         = true              # Убирает отладочные символы — меньше .exe

# ─── ФИЧИ ─────────────────────────────────────────────────────────────────────
[features]
default     = ["opengl"]
opengl      = []                  # Текущий бэкенд
vulkan      = []                  # Будущий бэкенд (Фаза 5)
debug_ui    = []                  # Отрисовка bounding box'ов UI-элементов
```

### Почему именно эти зависимости?

| Задача | Выбрано | Отклонено | Причина выбора |
|---|---|---|---|
| Шрифты | `fontdue` | `freetype-sys`, `rusttype` | Чистый Rust, без C-зависимостей, активно развивается |
| Аудио | `kira` | `rodio`, `cpal` | Fade-in/out, аудио-клоки, пространственный звук из коробки |
| 2D математика | `glam` | только `nalgebra` | glam быстрее в UI (SIMD Vec2/Vec4), nalgebra для физики |
| Конфиг | `toml` + `serde` | `config`, `figment` | Соответствует уже выбранному в проекте формату .toml |
| Ошибки | `anyhow` | `thiserror`, panic! | Быстро прототипировать, потом заменить на thiserror |

---

## 3. ПОЛНАЯ СТРУКТУРА ПАПОК И ФАЙЛОВ

```
RTGC-1.0/
│
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── LICENSE
├── README.md
├── PLEN3.md
│
├── .github/
│   └── workflows/
│       └── rust.yml
│
├── assets/                          ◄ ВСЕ ИГРОВЫЕ АССЕТЫ
│   │
│   ├── fonts/                       ◄ ШРИФТЫ
│   │   ├── main_font.ttf            — Основной UI шрифт (меню, текст)
│   │   ├── title_font.ttf           — Жирный заголовочный шрифт (RTGC)
│   │   └── mono_font.ttf            — Моноширинный (загрузочные логи)
│   │
│   ├── textures/                    ◄ ТЕКСТУРЫ
│   │   ├── menu/
│   │   │   ├── background.png       — Фон главного меню (1920×1080 JPG/PNG)
│   │   │   ├── logo.png             — Логотип RTGC (прозрачный PNG)
│   │   │   ├── button_normal.png    — Состояние кнопки: обычное
│   │   │   ├── button_hover.png     — Состояние кнопки: наведение
│   │   │   ├── button_pressed.png   — Состояние кнопки: нажато
│   │   │   └── panel_bg.png         — Фон панелей (полупрозрачный)
│   │   │
│   │   ├── loading/
│   │   │   ├── loading_bg.png       — Фон экрана загрузки
│   │   │   ├── progress_bar_bg.png  — Фон прогресс-бара
│   │   │   └── progress_bar_fill.png — Заполнение прогресс-бара
│   │   │
│   │   ├── character/
│   │   │   ├── portrait_male_1.png  — Превью лиц мужских (6 штук)
│   │   │   ├── portrait_male_2.png
│   │   │   ├── portrait_male_3.png
│   │   │   ├── portrait_male_4.png
│   │   │   ├── portrait_male_5.png
│   │   │   ├── portrait_male_6.png
│   │   │   ├── portrait_female_1.png — Превью лиц женских (6 штук)
│   │   │   ├── portrait_female_2.png
│   │   │   ├── portrait_female_3.png
│   │   │   ├── portrait_female_4.png
│   │   │   ├── portrait_female_5.png
│   │   │   ├── portrait_female_6.png
│   │   │   ├── hair_preview_*.png    — 8 вариантов причёсок (16 файлов: м+ж)
│   │   │   └── uaz_colors/
│   │   │       ├── uaz_white.png    — 12 превью цветов UAZ Patriot
│   │   │       ├── uaz_black.png
│   │   │       └── ...
│   │   │
│   │   └── icons/
│   │       ├── skill_mechanics.png  — Иконки навыков для экрана итогов
│   │       ├── skill_driving.png
│   │       ├── skill_piloting.png
│   │       └── ...
│   │
│   ├── audio/                       ◄ ЗВУКИ
│   │   ├── music/
│   │   │   ├── menu_theme.ogg       — Фоновая музыка главного меню (loop)
│   │   │   ├── loading_ambient.ogg  — Окружение во время загрузки
│   │   │   └── char_creation.ogg    — Музыка экрана создания персонажа
│   │   │
│   │   └── sfx/
│   │       ├── button_click.ogg     — Клик по кнопке
│   │       ├── button_hover.ogg     — Наведение на кнопку
│   │       ├── menu_open.ogg        — Открытие меню/панели
│   │       ├── menu_close.ogg       — Закрытие
│   │       ├── confirm.ogg          — Подтверждение выбора
│   │       ├── error.ogg            — Ошибка/недоступное действие
│   │       └── loading_complete.ogg — Загрузка завершена
│   │
│   ├── shaders/                     ◄ GLSL ШЕЙДЕРЫ
│   │   ├── ui/
│   │   │   ├── rect.vert            — Вертексный шейдер: прямоугольник
│   │   │   ├── rect.frag            — Фрагментный: цвет + скруглённые углы
│   │   │   ├── image.vert           — Вертексный: текстурированный rect
│   │   │   ├── image.frag           — Фрагментный: текстура + alpha
│   │   │   ├── text.vert            — Вертексный: символы шрифта
│   │   │   └── text.frag            — Фрагментный: SDF/bitmap шрифт
│   │   │
│   │   └── game/                    — (для будущего рендера игровой сцены)
│   │       ├── terrain.vert
│   │       ├── terrain.frag
│   │       ├── vehicle.vert
│   │       └── vehicle.frag
│   │
│   └── data/                        ◄ ДАННЫЕ (TOML/JSON)
│       ├── universities.toml        — ВУЗы (уже в плане проекта)
│       ├── vehicles.toml            — Параметры транспорта
│       ├── skills.toml              — Описания навыков, формулы
│       ├── regions.toml             — Стартовые районы Новосибирска
│       └── skin_presets.toml        — Цвета кожи, варианты персонажей
│
├── config/                          ◄ ПОЛЬЗОВАТЕЛЬСКИЕ НАСТРОЙКИ
│   │                                  (не в assets — не в git, в %APPDATA%)
│   └── settings_default.toml        — Шаблон настроек (копируется при первом запуске)
│
├── src/                             ◄ ИСХОДНЫЙ КОД
│   │
│   ├── main.rs                      — Точка входа, инициализация OS-уровня
│   ├── lib.rs                       — pub mod декларации
│   ├── app.rs                       — App struct: главный цикл, state machine
│   │
│   ├── core/                        ◄ ЯДРО ДВИЖКА
│   │   ├── mod.rs
│   │   ├── app_state.rs             — enum AppState + переходы
│   │   ├── event_bus.rs             — Внутренние события движка
│   │   └── timer.rs                 — DeltaTime, FrameTimer
│   │
│   ├── platform/                    ◄ ПЛАТФОРМО-ЗАВИСИМЫЙ КОД
│   │   ├── mod.rs
│   │   ├── window.rs                — Создание winit окна, OpenGL контекста
│   │   ├── input.rs                 — InputState: клавиши, мышь, геймпад
│   │   └── paths.rs                 — Пути: %APPDATA%\RTGC\, assets/, saves/
│   │
│   ├── renderer/                    ◄ РЕНДЕРИНГ (вся графика)
│   │   ├── mod.rs
│   │   ├── context.rs               — GlContext wrapper (glow::Context)
│   │   ├── shader.rs                — Компиляция/кеш шейдеров
│   │   ├── texture.rs               — Загрузка и кеш текстур (TextureCache)
│   │   ├── buffer.rs                — VBO/VAO обёртки
│   │   │
│   │   ├── ui_renderer/             ◄ 2D UI РЕНДЕРЕР
│   │   │   ├── mod.rs
│   │   │   ├── batch.rs             — DrawBatch: накопление команд → один draw call
│   │   │   ├── rect_renderer.rs     — Рендер прямоугольников (цвет, скругление)
│   │   │   ├── image_renderer.rs    — Рендер текстурированных прямоугольников
│   │   │   └── text_renderer.rs     — Рендер текста через fontdue атлас
│   │   │
│   │   └── game_renderer/           — (для игровой сцены, позже)
│   │       ├── mod.rs
│   │       └── terrain_renderer.rs
│   │
│   ├── font/                        ◄ ШРИФТОВАЯ СИСТЕМА
│   │   ├── mod.rs
│   │   ├── font_atlas.rs            — FontAtlas: растеризация → GPU текстура
│   │   ├── font_cache.rs            — Кеш уже растеризованных глифов
│   │   └── text_layout.rs           — Разбивка текста на строки, выравнивание
│   │
│   ├── audio/                       ◄ АУДИО СИСТЕМА
│   │   ├── mod.rs
│   │   ├── audio_manager.rs         — Обёртка над kira::AudioManager
│   │   └── sound_player.rs          — Управление каналами: музыка, SFX
│   │
│   ├── ui/                          ◄ UI КОМПОНЕНТЫ
│   │   ├── mod.rs
│   │   ├── widget.rs                — Trait Widget (update, render)
│   │   ├── button.rs                — Button: состояния, колбэки
│   │   ├── slider.rs                — Slider: ползунок с min/max
│   │   ├── label.rs                 — Label: текст с форматированием
│   │   ├── image_widget.rs          — Виджет отображения изображения
│   │   ├── progress_bar.rs          — ProgressBar для экрана загрузки
│   │   ├── panel.rs                 — Panel: контейнер с фоном
│   │   ├── selector.rs              — Selector: ◀ значение ▶ (выбор из списка)
│   │   └── color_picker.rs          — ColorPicker: сетка цветных кнопок
│   │
│   ├── screens/                     ◄ ЭКРАНЫ (каждый — отдельный state)
│   │   ├── mod.rs
│   │   │
│   │   ├── splash/                  — ЗАСТАВКА при запуске
│   │   │   ├── mod.rs
│   │   │   └── splash_screen.rs
│   │   │
│   │   ├── main_menu/               — ГЛАВНОЕ МЕНЮ
│   │   │   ├── mod.rs
│   │   │   ├── main_menu_screen.rs
│   │   │   └── menu_background.rs   — Анимированный фон
│   │   │
│   │   ├── settings/                — НАСТРОЙКИ
│   │   │   ├── mod.rs
│   │   │   ├── settings_screen.rs
│   │   │   ├── video_tab.rs         — Вкладка: Видео
│   │   │   ├── audio_tab.rs         — Вкладка: Аудио
│   │   │   └── controls_tab.rs      — Вкладка: Управление
│   │   │
│   │   ├── character_creation/      — СОЗДАНИЕ ПЕРСОНАЖА
│   │   │   ├── mod.rs
│   │   │   ├── character_creation_screen.rs — Координатор шагов
│   │   │   ├── step_gender.rs       — Шаг 1: Пол
│   │   │   ├── step_height.rs       — Шаг 2: Рост
│   │   │   ├── step_skin.rs         — Шаг 3: Цвет кожи
│   │   │   ├── step_face.rs         — Шаг 4: Лицо
│   │   │   ├── step_hair.rs         — Шаг 5: Причёска
│   │   │   ├── step_hair_color.rs   — Шаг 6: Цвет волос
│   │   │   ├── step_education.rs    — Шаг 7: Образование / ВУЗ
│   │   │   ├── step_uaz_color.rs    — Шаг 8: Цвет UAZ
│   │   │   ├── step_start_region.rs — Шаг 9: Стартовый район
│   │   │   └── step_summary.rs      — Шаг 10: Итог → НАЧАТЬ
│   │   │
│   │   └── loading/                 — ЭКРАН ЗАГРУЗКИ
│   │       ├── mod.rs
│   │       ├── loading_screen.rs    — UI загрузочного экрана
│   │       └── load_stages.rs       — 11 стадий загрузки (логика)
│   │
│   ├── save/                        ◄ СОХРАНЕНИЯ
│   │   ├── mod.rs
│   │   ├── player_profile.rs        — Struct PlayerProfile (serde)
│   │   ├── save_manager.rs          — Чтение/запись .toml сохранений
│   │   └── character_data.rs        — CharacterData (результат создания персонажа)
│   │
│   ├── animation/                   ◄ АНИМАЦИЯ/ТВИНИНГ
│   │   ├── mod.rs
│   │   ├── tween.rs                 — Tween<T>: интерполяция значений
│   │   ├── easing.rs                — Функции ease: linear, ease_in_out, bounce...
│   │   └── animator.rs              — Animator: набор тween'ов с таймлайном
│   │
│   ├── physics/                     — (существующее, без изменений пока)
│   │   └── thread_f.rs
│   │
│   └── graphics/                    — (существующее, без изменений пока)
│       └── rhi/
│           └── thread_rhi.rs
│
└── saves/                           ◄ СОХРАНЕНИЯ ИГРОКА (gitignored)
    └── player_*.toml                — Каждый персонаж = отдельный файл
```

---

## 4. ГЛОБАЛЬНАЯ МАШИНА СОСТОЯНИЙ (AppState)

### src/core/app_state.rs

```rust
// ============================================================
// AppState — перечисление всех состояний приложения
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Заставка при первом запуске: логотип студии/движка
    /// Длительность: ~2 секунды, пропускается по любому нажатию
    Splash,

    /// Главное меню: НОВАЯ ИГРА / ЗАГРУЗИТЬ / НАСТРОЙКИ / ВЫХОД
    MainMenu,

    /// Настройки (открываются поверх MainMenu или Playing)
    Settings { return_to: Box<AppState> },

    /// 10-шаговый экран создания персонажа (только для новой игры)
    CharacterCreation,

    /// Экран выбора сохранения (для "Загрузить игру")
    SaveSelect,

    /// Загрузочный экран с прогресс-баром (11 стадий)
    Loading {
        /// Данные персонажа, переданные из CharacterCreation или SaveSelect
        character_data: Box<CharacterData>,
    },

    /// Основная игровая сцена — всё готово, игра запущена
    Playing,

    /// Меню паузы (рендерит Playing под собой)
    PauseMenu,
}

// ─── Переходы ───────────────────────────────────────────────

impl AppState {
    /// Возможные переходы ИЗ данного состояния
    pub fn valid_transitions(&self) -> &[&str] {
        match self {
            AppState::Splash            => &["MainMenu"],
            AppState::MainMenu          => &["CharacterCreation", "SaveSelect",
                                              "Settings", "Exit"],
            AppState::CharacterCreation => &["Loading", "MainMenu"],
            AppState::SaveSelect        => &["Loading", "MainMenu"],
            AppState::Loading { .. }    => &["Playing"],
            AppState::Playing           => &["PauseMenu", "MainMenu"],
            AppState::PauseMenu         => &["Playing", "Settings", "MainMenu"],
            AppState::Settings { .. }   => &["return_to"],
        }
    }
}
```

---

## 5. ТОЧКА ВХОДА main.rs — ПОЛНАЯ СХЕМА

### src/main.rs

```rust
// ============================================================
// Последовательность инициализации при запуске
// ============================================================
//
// 1. Инициализация логирования (tracing_subscriber)
// 2. Разрешение путей (dirs::data_dir → %APPDATA%\RTGC\)
// 3. Загрузка настроек (settings.toml или default)
// 4. Создание winit EventLoop
// 5. Создание окна (из настроек: разрешение, полный экран)
// 6. Создание OpenGL контекста (glutin)
// 7. Создание glow::Context
// 8. Инициализация AudioManager (kira)
// 9. Создание App { ... }
// 10. Запуск event_loop.run(app)

fn main() -> anyhow::Result<()> {
    // 1. Логирование
    tracing_subscriber::fmt()
        .with_env_filter("rtgc=debug,warn")
        .init();

    tracing::info!("RTGC-1.0 запускается...");

    // 2. Пути
    let paths = AppPaths::resolve()?;
    // %APPDATA%\RTGC\settings.toml
    // %APPDATA%\RTGC\saves\
    // рядом с .exe\assets\

    // 3. Настройки
    let settings = Settings::load_or_default(&paths.settings_file)?;

    // 4-7. Окно + OpenGL
    let event_loop = EventLoop::new()?;
    let (window, gl_context, gl) = platform::window::create(
        &event_loop,
        &settings.video,
    )?;

    // 8. Аудио
    let audio = AudioManager::new()?;

    // 9. Приложение
    let mut app = App::new(gl, audio, paths, settings)?;

    // 10. Главный цикл
    event_loop.run(move |event, target| {
        app.handle_event(event, target);
    })?;

    Ok(())
}
```

### src/platform/window.rs — Создание окна

```rust
// Параметры окна из settings.video:
// - width, height           (по умолчанию 1280×720)
// - fullscreen              (false)
// - vsync                   (true)
// - msaa_samples            (0 — без MSAA для UI, включается для игры)
// - title                   ("RTGC 1.0")
// - icon                    (assets/textures/menu/icon.png)

// Версия OpenGL: 3.3 Core Profile
// Минимальная поддерживаемая: Windows 10 + любая дискретная GPU 2012+

pub fn create(
    event_loop: &EventLoop<()>,
    video: &VideoSettings,
) -> anyhow::Result<(Window, PossiblyCurrentContext, Arc<glow::Context>)> {
    let window_attrs = Window::default_attributes()
        .with_title("RTGC 1.0")
        .with_inner_size(LogicalSize::new(video.width, video.height))
        .with_resizable(true)
        .with_visible(false);  // ← скрыто до появления splash

    // glutin: создание контекста 3.3 Core
    // При неудаче → fallback 2.1 Compatibility (старые Intel GPU)

    // После создания: window.set_visible(true)
    // До этого показываем заставку без мерцания
}
```

---

## 6. ПОДСИСТЕМА РЕНДЕРИНГА UI (ui_renderer)

### Архитектура: Batched Immediate Mode

Все UI-элементы за один кадр накапливаются в `DrawBatch`,
затем один draw call отправляется в GPU.

```
UI код вызывает:             DrawBatch накапливает:        GPU получает:
button.render(&mut batch) →  [RectCmd, TextCmd, ...]    →  1 draw call
label.render(&mut batch)  →  [TextCmd, ...]
panel.render(&mut batch)  →  [RectCmd, ...]

                          → batch.flush(gl)  → glDrawArrays
```

### src/renderer/ui_renderer/batch.rs

```rust
// Буфер вершин для UI: позиция + UV + цвет
// Формат вершины:
//   [f32; 2] pos     — NDC координаты (-1..1)
//   [f32; 2] uv      — текстурные координаты (0..1)
//   [f32; 4] color   — RGBA (0..1)
//   f32      mode    — 0.0=цвет, 1.0=текстура, 2.0=текст (SDF/bitmap)

// Максимум 65536 вершин за кадр (должно хватить на любое меню)

pub struct DrawBatch {
    vertices:     Vec<UiVertex>,   // CPU буфер
    vbo:          NativeBuffer,    // GPU VBO
    vao:          NativeVertexArray,
    current_tex:  Option<NativeTexture>,
}

impl DrawBatch {
    // Нарисовать закрашенный прямоугольник
    pub fn push_rect(&mut self, rect: Rect, color: Color, corner_radius: f32);

    // Нарисовать текстурированный прямоугольник
    pub fn push_image(&mut self, rect: Rect, tex: NativeTexture, tint: Color);

    // Нарисовать строку текста
    pub fn push_text(&mut self, text: &str, pos: Vec2, size: f32,
                     color: Color, font: &FontAtlas);

    // Отправить всё в GPU
    pub fn flush(&mut self, gl: &glow::Context, screen_size: Vec2);
}
```

### src/renderer/ui_renderer/rect_renderer.rs — Шейдер прямоугольника

```glsl
// assets/shaders/ui/rect.frag
// Поддерживает скруглённые углы через SDF

uniform vec4  u_color;
uniform float u_corner_radius;
uniform vec2  u_rect_size;   // размер прямоугольника в пикселях

in vec2 v_local_pos;  // позиция внутри прямоугольника (0..rect_size)

float roundedBoxSDF(vec2 pos, vec2 half_size, float radius) {
    vec2 q = abs(pos) - half_size + radius;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}

void main() {
    vec2 half = u_rect_size * 0.5;
    vec2 centered = v_local_pos - half;
    float dist = roundedBoxSDF(centered, half, u_corner_radius);
    float alpha = 1.0 - smoothstep(-1.0, 1.0, dist);
    out_color = vec4(u_color.rgb, u_color.a * alpha);
}
```

---

## 7. ШРИФТОВАЯ СИСТЕМА (font_system)

### src/font/font_atlas.rs

```
Как работает fontdue в RTGC:

1. При инициализации рендерера:
   fontdue::Font::from_bytes(ttf_bytes, settings)

2. Растеризация глифа:
   font.rasterize('А', 32.0)  →  (metrics, bitmap: Vec<u8>)

3. Упаковка в атлас (TextureAtlas, 1024×1024 R8):
   Алгоритм: shelf packing (строки глифов)

4. Загрузка атласа в GPU:
   gl.tex_image_2d(..., R8, bitmap_data)

5. При рендере текста:
   Для каждого символа → поиск UV в атласе → push_text() в DrawBatch

Шрифты и их размеры (пре-растеризуются при старте):
┌──────────────────┬─────────────────────────────┬───────────────────┐
│ Файл             │ Используется                 │ Размеры           │
├──────────────────┼─────────────────────────────┼───────────────────┤
│ main_font.ttf    │ Кнопки, описания, диалоги    │ 16, 20, 24, 28    │
│ title_font.ttf   │ Заголовки, логотип RTGC      │ 36, 48, 72        │
│ mono_font.ttf    │ Загрузочные сообщения, лог   │ 14, 16            │
└──────────────────┴─────────────────────────────┴───────────────────┘

Кириллица обязательна! fontdue поддерживает Unicode нативно.
Растеризовать заранее: А-Я а-я + A-Z a-z + 0-9 + пунктуация
```

---

## 8. АУДИОСИСТЕМА МЕНЮ (audio)

### src/audio/audio_manager.rs

```rust
// Kira AudioManager — инициализация
// Два слоя громкости (из настроек):
//   master_volume: f64   (0.0 - 1.0)
//   music_volume:  f64
//   sfx_volume:    f64

// Музыкальные треки:
//   Splash          → тишина
//   MainMenu        → menu_theme.ogg (loop, fade-in 1.5с)
//   CharacterCreation → char_creation.ogg (loop, crossfade 1.0с)
//   Loading         → loading_ambient.ogg (loop, fade-out перед концом)
//   Playing         → начинается игровой саундтрек

// Переход между треками: crossfade через kira::tween
//   старый трек: .set_volume(0, Tween { duration: 1.0s })
//   новый трек:  .play() с volume = 0, потом set_volume(target, Tween)

// SFX вызываются из UI виджетов:
//   button.on_hover → audio.play_sfx("button_hover")
//   button.on_click → audio.play_sfx("button_click")
```

---

## 9. АНИМАЦИОННАЯ СИСТЕМА (tweening)

### src/animation/tween.rs

```rust
// Используется для:
//   - Появление кнопок меню (fade-in + slide)
//   - Переходы между экранами (fade to black)
//   - Прогресс-бар загрузки (smooth interpolation)
//   - Пульсация highlight-элементов

pub struct Tween<T: Lerp> {
    start:    T,
    end:      T,
    duration: f32,     // секунды
    elapsed:  f32,
    easing:   EasingFn,
    pub done: bool,
}

impl<T: Lerp> Tween<T> {
    pub fn update(&mut self, dt: f32) -> T {
        self.elapsed = (self.elapsed + dt).min(self.duration);
        self.done    = self.elapsed >= self.duration;
        let t = self.easing.apply(self.elapsed / self.duration);
        T::lerp(self.start, self.end, t)
    }
}

// Функции ease (src/animation/easing.rs):
//   Linear
//   EaseInQuad / EaseOutQuad / EaseInOutQuad
//   EaseInCubic / EaseOutCubic / EaseInOutCubic
//   EaseOutBounce   ← для подтверждения выбора
//   EaseOutElastic  ← для появления панелей
```

---

## 10. ГЛАВНОЕ МЕНЮ (MainMenu)

### Визуальный дизайн

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│         [Фоновое изображение: зимняя дорога Новосибирска]           │
│                  + виньетка по краям (тёмный градиент)              │
│                                                                      │
│                                                                      │
│              ██████  ████████  ██████  ██████                       │
│              ██   ██    ██    ██       ██                            │
│              ██████     ██    ██   ███ ██                            │
│              ██   ██    ██    ██    ██ ██                            │
│              ██   ██    ██     ██████  ██████  v1.0-dev              │
│                                                                      │
│                                                                      │
│                    ┌──────────────────────┐                         │
│                    │     НОВАЯ ИГРА       │  ← hover: подсветка     │
│                    └──────────────────────┘                         │
│                    ┌──────────────────────┐                         │
│                    │   ЗАГРУЗИТЬ ИГРУ     │                         │
│                    └──────────────────────┘                         │
│                    ┌──────────────────────┐                         │
│                    │     НАСТРОЙКИ        │                         │
│                    └──────────────────────┘                         │
│                    ┌──────────────────────┐                         │
│                    │       ВЫХОД          │                         │
│                    └──────────────────────┘                         │
│                                                                      │
│  v1.0.0-dev                          © 2026 NAREZKA578              │
└─────────────────────────────────────────────────────────────────────┘

Анимации при появлении:
  - Логотип: fade-in сверху (0.8с, EaseOutCubic)
  - Кнопки: появляются по очереди снизу вверх (delay +0.1с каждая)
  - Фон: медленный Ken Burns (zoom 100%→105% за 20с)
```

### src/screens/main_menu/main_menu_screen.rs

```rust
pub struct MainMenuScreen {
    // Анимации появления
    logo_alpha:    Tween<f32>,
    buttons_alpha: [Tween<f32>; 4],
    bg_zoom:       Tween<f32>,

    // Кнопки
    btn_new_game:  Button,
    btn_load_game: Button,
    btn_settings:  Button,
    btn_exit:      Button,

    // Фоновая текстура
    bg_texture:    NativeTexture,
    logo_texture:  NativeTexture,

    // Результат (что выбрал игрок)
    pub result: Option<MenuResult>,
}

pub enum MenuResult {
    NewGame,
    LoadGame,
    Settings,
    Exit,
}

impl MainMenuScreen {
    pub fn update(&mut self, input: &InputState, dt: f32) {
        // 1. Обновить анимации появления
        // 2. Обновить hover/click состояния кнопок
        // 3. При клике → self.result = Some(...)
        // 4. При Escape → self.result = Some(Exit)
    }

    pub fn render(&self, batch: &mut DrawBatch, screen: Vec2) {
        // 1. Фоновое изображение с Ken Burns zoom
        // 2. Тёмная виньетка (полупрозрачный rect поверх)
        // 3. Логотип
        // 4. Кнопки с их текущим состоянием (normal/hover/pressed)
        // 5. Версия внизу слева
        // 6. Копирайт внизу справа
    }
}
```

---

## 11. ЭКРАН НАСТРОЕК (SettingsScreen)

### Структура вкладок

```
┌────────────────────────────────────────────────┐
│                  НАСТРОЙКИ                      │
│  [ВИДЕО]  [АУДИО]  [УПРАВЛЕНИЕ]                │
├────────────────────────────────────────────────┤
│ ВИДЕО                                           │
│                                                 │
│  Разрешение:    ◀  1920×1080  ▶                │
│  Режим экрана:  ◀  Оконный    ▶                │
│  Качество:      ◀  Высокое    ▶                │
│  VSync:         ◀  Вкл        ▶                │
│  Дальность:     ████████░░   8000м              │
│  Тени:          ◀  Средние   ▶                 │
│                                                 │
│               [ПРИМЕНИТЬ]  [ОТМЕНА]             │
└────────────────────────────────────────────────┘

Вкладка АУДИО:
  Общая громкость:   ████████░░  80%
  Музыка:            ██████░░░░  60%
  Звуковые эффекты:  █████████░  90%
  Окружение:         ████████░░  80%

Вкладка УПРАВЛЕНИЕ:
  Список привязок клавиш + кнопка сброса
```

### src/screens/settings/settings_screen.rs

```rust
pub struct SettingsScreen {
    active_tab:   SettingsTab,   // Video / Audio / Controls
    video_tab:    VideoTab,
    audio_tab:    AudioTab,
    controls_tab: ControlsTab,

    // Состояние ДО изменений (для "Отмена")
    original_settings: Settings,

    // Кнопки
    btn_apply:  Button,
    btn_cancel: Button,

    pub result: Option<SettingsResult>,
}

// settings сохраняются в:
//   %APPDATA%\RTGC\settings.toml
// (src/platform/paths.rs возвращает этот путь)
```

---

## 12. ЭКРАН СОЗДАНИЯ ПЕРСОНАЖА (CharacterCreation)

### Навигация по шагам

```
Шаг 1/10 ●●○○○○○○○○   [◀ НАЗАД]              [ДАЛЕЕ ▶]
```

### Шаг 7 (Образование) — самый сложный

```
┌────────────────────────────────────────────────────────┐
│  Выберите образование                   Шаг 7 / 10     │
│  ─────────────────────────────────────────────────     │
│  Специальность:  ◀  Автоинженер  ▶                     │
│                                                         │
│  ВУЗ:            ◀  НГТУ Новосибирск  ▶               │
│                                                         │
│  ──────── Начальные навыки ────────                    │
│    mechanics:    ████░░░░  ранг 3                      │
│    driving:      ████░░░░  ранг 3                      │
│    electrics:    ███░░░░░  ранг 2                      │
│                                                         │
│  ──────── Стартовый капитал ────────                   │
│    85 000 ₽                                            │
│                                                         │
│  ──────── Начальные контакты ───────                   │
│    • Профессор Иванов (механика)                       │
│    • Серёга (транспортные заказы)                      │
│                                                         │
│  Данные загружаются из:  assets/data/universities.toml │
└────────────────────────────────────────────────────────┘
```

### src/screens/character_creation/character_creation_screen.rs

```rust
pub struct CharacterCreationScreen {
    current_step: u8,   // 1..=10

    // Данные со всех шагов (накапливаются по мере прохождения)
    data: CharacterData,

    // Шаги
    step_gender:    StepGender,
    step_height:    StepHeight,
    step_skin:      StepSkin,
    step_face:      StepFace,
    step_hair:      StepHair,
    step_hair_color: StepHairColor,
    step_education: StepEducation,
    step_uaz_color: StepUazColor,
    step_start:     StepStartRegion,
    step_summary:   StepSummary,

    btn_back: Button,
    btn_next: Button,

    pub result: Option<CharCreationResult>,
}

pub enum CharCreationResult {
    Confirmed(CharacterData),  // → переход в LoadingScreen
    Cancelled,                 // → назад в MainMenu
}
```

### src/save/character_data.rs

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharacterData {
    pub id:           uuid::Uuid,
    pub created_at:   String,       // ISO 8601

    // Шаг 1
    pub gender:       Gender,       // Male / Female

    // Шаг 2
    pub height_m:     f32,          // 1.50 - 2.10

    // Шаг 3
    pub skin_color:   u8,           // 0..7

    // Шаг 4
    pub face_index:   u8,           // 0..5

    // Шаг 5
    pub hair_index:   u8,           // 0..7

    // Шаг 6
    pub hair_color:   [f32; 3],     // RGB

    // Шаг 7
    pub university_id:   String,    // ID из universities.toml
    pub specialty:       String,
    pub skills:          SkillSet,  // рассчитывается из образования
    pub start_capital:   f64,       // RUB

    // Шаг 8
    pub uaz_color:    [f32; 3],     // RGB

    // Шаг 9
    pub start_region: StartRegion,  // Центр / Академгородок / Левый берег / ...
    pub start_pos:    [f64; 3],     // XYZ в мировых координатах

    // Шаг 10 — только для отображения, данные уже выше
}
```

---

## 13. СИСТЕМА ЗАГРУЗКИ — 11 СТАДИЙ (LoadingScreen)

### Визуальный дизайн загрузочного экрана

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│    [Фоновое изображение: вид с дороги на Новосибирск, туман]        │
│                                                                      │
│                                                                      │
│                        ЗАГРУЗКА...                                  │
│                                                                      │
│         Стадия 4 / 11: Генерация дорожной сети                      │
│                                                                      │
│         ┌────────────────────────────────────────────┐              │
│         │████████████████████░░░░░░░░░░░░░░░░░░░░░░│  38%          │
│         └────────────────────────────────────────────┘              │
│                                                                      │
│    [Текущее действие]: Генерация узлов (14 832 / 38 000)            │
│                                                                      │
│    Подсказка: UAZ Patriot 2017 оснащён двигателем ЗМЗ-409           │
│              мощностью 128 л.с. при 4 600 об/мин                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘

Подсказки меняются каждые 8 секунд (случайно из tips.toml)
```

### src/screens/loading/load_stages.rs — 11 стадий

```rust
// ============================================================
// 11 СТАДИЙ ЗАГРУЗКИ — детальная разбивка
// ============================================================

pub const LOAD_STAGES: &[LoadStage] = &[

  LoadStage {
    id:      1,
    name:    "Инициализация движка",
    weight:  3.0,   // % от общего времени
    steps: &[
      "Инициализация OpenGL контекста",
      "Загрузка UI шейдеров",
      "Создание прогресс-бара",     // чтобы сразу что-то показать
    ],
  },

  LoadStage {
    id:      2,
    name:    "Загрузка шрифтов",
    weight:  4.0,
    steps: &[
      "Загрузка main_font.ttf",
      "Растеризация кириллицы 16pt",
      "Растеризация кириллицы 24pt",
      "Загрузка title_font.ttf",
      "Растеризация заголовков 48pt",
      "Загрузка в GPU (атлас 1024×1024)",
    ],
  },

  LoadStage {
    id:      3,
    name:    "Загрузка текстур мира",
    weight:  12.0,
    steps: &[
      "Terrain: трава (grass_albedo.png)",
      "Terrain: земля (dirt_albedo.png)",
      "Terrain: камень (rock_albedo.png)",
      "Terrain: снег (snow_albedo.png)",
      "Terrain: дорога (asphalt_albedo.png)",
      "Skybox: загрузка 6 граней",
      "Загрузка splatmap региона",
    ],
  },

  LoadStage {
    id:      4,
    name:    "Генерация дорожной сети",
    weight:  15.0,
    steps: &[
      "Загрузка road_network.toml",
      "Построение графа дорог",
      "Генерация узлов пересечений",
      "Расчёт кривых Безье для поворотов",
      "Построение навигационной сетки",
      "Генерация collision mesh дорог",
    ],
  },

  LoadStage {
    id:      5,
    name:    "Генерация ландшафта",
    weight:  18.0,
    steps: &[
      "Загрузка heightmap Новосибирского региона",
      "Создание LOD-пирамиды (5 уровней)",
      "Генерация terrain mesh LOD0",
      "Генерация terrain mesh LOD1",
      "Генерация terrain mesh LOD2",
      "Загрузка terrain буферов в GPU",
      "Расчёт нормалей и тангентов",
    ],
  },

  LoadStage {
    id:      6,
    name:    "Загрузка транспорта",
    weight:  8.0,
    steps: &[
      "Загрузка vehicles.toml",
      "Разбор параметров UAZ Patriot 2017",
      "Создание vehicle mesh",
      "Инициализация физики шасси",
      "Загрузка текстур транспорта",
      "Инициализация системы износа деталей",
    ],
  },

  LoadStage {
    id:      7,
    name:    "Загрузка данных мира",
    weight:  6.0,
    steps: &[
      "Загрузка universities.toml",
      "Загрузка settlements (города и посёлки)",
      "Загрузка trade routes",
      "Загрузка resource deposits",
      "Разбор данных персонажа",
      "Расчёт стартового инвентаря",
    ],
  },

  LoadStage {
    id:      8,
    name:    "Инициализация физики",
    weight:  10.0,
    steps: &[
      "Создание физического мира",
      "Загрузка collision mesh ландшафта",
      "Расчёт BVH (Bounding Volume Hierarchy)",
      "Инициализация физики транспорта",
      "Размещение транспорта в стартовой точке",
      "Тест коллизии старт-позиции",
    ],
  },

  LoadStage {
    id:      9,
    name:    "Генерация мира",
    weight:  10.0,
    steps: &[
      "Размещение зданий по settlements",
      "Генерация деревьев и растительности",
      "Размещение ресурсных точек",
      "Генерация NPC транспорта (routes)",
      "Инициализация системы погоды",
      "Инициализация системы времени суток",
    ],
  },

  LoadStage {
    id:      10,
    name:    "Инициализация аудио",
    weight:  4.0,
    steps: &[
      "Загрузка звуков двигателя",
      "Загрузка звуков окружения",
      "Загрузка звуков физики (удары, скрип)",
      "Загрузка музыкального трека",
      "Fade-out музыки меню",
    ],
  },

  LoadStage {
    id:      11,
    name:    "Финализация",
    weight:  10.0,
    steps: &[
      "Прогрев шейдеров (shader warmup)",
      "Инициализация HUD",
      "Инициализация системы миссий",
      "Первый рендер игровой сцены (culling warm)",
      "Очистка загрузочных буферов",
      "Готово!",
    ],
  },
];
```

### Архитектура потоков загрузки

```
Главный поток (рендер):              Загрузочный поток:
                                     (crossbeam-channel sender)
  LoadingScreen::update()     ◀───  LoadProgress { stage, step, pct }
  └─ читает прогресс из rx          │
  └─ обновляет progress_bar         └─ Stage 1: init_engine()
  └─ рендерит экран                 └─ Stage 2: load_fonts()
                                    └─ Stage 3: load_textures() (rayon!)
  main_loop продолжает рендерить    └─ Stage 4: gen_road_network()
  со скоростью 60 FPS               └─ ...
  пока загрузка идёт в фоне         └─ Stage 11: finalize()
                                       sends: LoadComplete(GameWorld)

  При получении LoadComplete:
  AppState → Playing
```

```rust
// src/screens/loading/loading_screen.rs

pub struct LoadingScreen {
    // UI
    progress_bar:  ProgressBar,
    stage_label:   Label,
    step_label:    Label,
    tip_label:     Label,

    // Анимации
    bar_tween:     Tween<f32>,   // smooth прогресс-бар
    tip_timer:     f32,          // смена подсказок каждые 8с
    tip_fade:      Tween<f32>,   // fade подсказки при смене

    // Связь с потоком загрузки
    progress_rx:   crossbeam_channel::Receiver<LoadProgress>,
    current_progress: LoadProgress,

    // Результат
    pub world: Option<Box<GameWorld>>,  // готов после Stage 11
}

pub struct LoadProgress {
    pub stage:       u8,         // 1..=11
    pub stage_name:  String,
    pub step:        String,
    pub percent:     f32,        // 0.0..1.0
}

// Запуск загрузочного потока
pub fn start_loading_thread(
    character_data: CharacterData,
    paths: Arc<AppPaths>,
    tx: crossbeam_channel::Sender<LoadProgress>,
) -> std::thread::JoinHandle<anyhow::Result<GameWorld>> {
    std::thread::spawn(move || {
        let world = load_game_world(&character_data, &paths, &tx)?;
        Ok(world)
    })
}
```

---

## 14. ПЕРЕХОД В ИГРОВУЮ СЦЕНУ (Playing)

### Механизм плавного перехода

```
LoadingScreen (Stage 11, 100%) →
  fade_out_tween: Tween<f32> { 0.0 → 1.0, 0.8с } →
    Чёрный экран →
      AppState = Playing →
        GameWorld рендерится →
          fade_in_tween: Tween<f32> { 1.0 → 0.0, 1.0с } →
            Игра видна →
              Диалог Серёги появляется через 30с
```

### src/app.rs — Главный App и смена состояний

```rust
pub struct App {
    // Ядро
    gl:       Arc<glow::Context>,
    window:   Arc<Window>,
    paths:    Arc<AppPaths>,
    settings: Settings,

    // Системы
    audio:    AudioManager,
    batch:    DrawBatch,
    input:    InputState,
    timer:    FrameTimer,

    // Текущее состояние
    state:    AppState,

    // Экраны (только активные инициализированы)
    splash:       Option<SplashScreen>,
    main_menu:    Option<MainMenuScreen>,
    settings_scr: Option<SettingsScreen>,
    char_create:  Option<CharacterCreationScreen>,
    loading:      Option<LoadingScreen>,

    // Игровой мир (Some когда Playing)
    game_world:   Option<Box<GameWorld>>,

    // Переходный fade
    fade_overlay: FadeOverlay,
}

impl App {
    pub fn handle_event(&mut self, event: Event<()>, target: &EventLoopWindowTarget<()>) {
        match event {
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                self.update();
                self.render();
                self.window.request_redraw();
            }
            // ... обработка ввода, resize, close
        }
    }

    fn update(&mut self) {
        let dt = self.timer.tick();
        self.input.update();

        match &self.state {
            AppState::Splash         => self.update_splash(dt),
            AppState::MainMenu       => self.update_main_menu(dt),
            AppState::Settings{..}   => self.update_settings(dt),
            AppState::CharacterCreation => self.update_char_create(dt),
            AppState::Loading{..}    => self.update_loading(dt),
            AppState::Playing        => self.update_playing(dt),
            AppState::PauseMenu      => self.update_pause(dt),
        }
    }

    fn transition_to(&mut self, new_state: AppState) {
        // 1. Деинициализировать старый экран (освободить ресурсы)
        // 2. Сменить музыку
        // 3. Запустить fade
        // 4. Инициализировать новый экран
        self.state = new_state;
    }
}
```

---

## 15. ФАЙЛ КОНФИГУРАЦИИ ИГРОКА (config/)

### %APPDATA%\RTGC\settings.toml

```toml
[video]
width         = 1280
height        = 720
fullscreen    = false
vsync         = true
render_distance = 5000.0
shadow_quality = "medium"    # "off" | "low" | "medium" | "high"
texture_quality = "high"     # "minimum" | "low" | "medium" | "high" | "ultra"
fov           = 75.0

[audio]
master_volume = 0.8
music_volume  = 0.6
sfx_volume    = 0.9
ambient_volume = 0.8

[controls]
# Клавиши — строки из winit::keyboard::KeyCode
move_forward  = "KeyW"
move_back     = "KeyS"
move_left     = "KeyA"
move_right    = "KeyD"
handbrake     = "Space"
interact      = "KeyF"
inventory     = "Tab"
map           = "KeyM"
camera_toggle = "KeyV"

[game]
language      = "ru"
show_tutorial = true
autosave_interval_min = 15

[debug]
show_fps      = false
show_collision = false
show_chunks    = false
```

---

## 16. АССЕТЫ — КАКИЕ ФАЙЛЫ, ГДЕ, В КАКОМ ФОРМАТЕ

| Тип | Формат | Где хранится | Описание |
|---|---|---|---|
| Шрифты | `.ttf` | `assets/fonts/` | TTF без кодировки платформы |
| Текстуры меню | `.png` | `assets/textures/menu/` | PNG с прозрачностью |
| Фон меню | `.jpg` | `assets/textures/menu/` | JPG (нет прозрачности = меньше размер) |
| Звуки SFX | `.ogg` | `assets/audio/sfx/` | OGG Vorbis, моно, 44100 Гц |
| Музыка | `.ogg` | `assets/audio/music/` | OGG Vorbis, стерео, 44100 Гц |
| UI шейдеры | `.vert/.frag` | `assets/shaders/ui/` | GLSL 330 core |
| Игровые шейдеры | `.vert/.frag` | `assets/shaders/game/` | GLSL 330 core |
| ВУЗы | `.toml` | `assets/data/` | UTF-8, TOML 1.0 |
| Транспорт | `.toml` | `assets/data/` | UTF-8, TOML 1.0 |
| Подсказки загрузки | `.toml` | `assets/data/tips.toml` | Список строк |
| Heightmap | `.png` (R16) | `assets/data/terrain/` | 16-bit grayscale |
| Сохранения | `.toml` | `%APPDATA%\RTGC\saves\` | НЕ в assets! |
| Настройки | `.toml` | `%APPDATA%\RTGC\` | НЕ в assets! |

### assets/data/tips.toml (примеры подсказок)

```toml
[[tips]]
text = "UAZ Patriot 2017 оснащён двигателем ЗМЗ-409 мощностью 128 л.с."

[[tips]]
text = "В распутицу (апрель-май) грунтовые дороги становятся практически непроходимыми для колёсной техники."

[[tips]]
text = "Зимой болота замерзают — открываются зимники, которые летом недоступны."

[[tips]]
text = "Лебёдка UAZ выдерживает нагрузку до 4 500 кг."

[[tips]]
text = "Навык mechanics ниже ранга 2 — самостоятельный ремонт двигателя невозможен."

[[tips]]
text = "Для управления вертолётом необходим навык piloting не ниже ранга 4."

[[tips]]
text = "Расстояние от Новосибирска до Бердска по Бердскому шоссе — около 32 км."
```

---

## 17. ПОРЯДОК РЕАЛИЗАЦИИ (roadmap)

### Фаза 0: Фундамент рендеринга UI (1-2 недели)

```
[ ] 1. src/platform/window.rs — создание окна glutin + winit
[ ] 2. src/renderer/context.rs — GlContext (glow)
[ ] 3. src/renderer/shader.rs — компиляция GLSL из файлов
[ ] 4. assets/shaders/ui/rect.vert + rect.frag — базовый шейдер
[ ] 5. src/renderer/ui_renderer/batch.rs — DrawBatch
[ ] 6. src/renderer/ui_renderer/rect_renderer.rs — рисуем прямоугольник
[ ] 7. src/renderer/ui_renderer/image_renderer.rs — текстура PNG
[ ] 8. Тест: красный прямоугольник на экране → РАБОТАЕТ
```

### Фаза 1: Шрифты и текст (3-5 дней)

```
[ ] 9.  src/font/font_atlas.rs — fontdue → GPU текстура
[ ] 10. assets/shaders/ui/text.vert + text.frag
[ ] 11. src/renderer/ui_renderer/text_renderer.rs
[ ] 12. Тест: "RTGC 1.0" на экране кириллицей → РАБОТАЕТ
```

### Фаза 2: UI виджеты (3-5 дней)

```
[ ] 13. src/ui/button.rs — Button с 3 состояниями
[ ] 14. src/ui/label.rs
[ ] 15. src/ui/panel.rs
[ ] 16. src/platform/input.rs — InputState (мышь + клавиатура)
[ ] 17. Тест: кнопка с hover + click → РАБОТАЕТ
```

### Фаза 3: Аудио (2-3 дня)

```
[ ] 18. src/audio/audio_manager.rs — kira init
[ ] 19. src/audio/sound_player.rs — play_music, play_sfx
[ ] 20. Тест: click звук при нажатии кнопки → РАБОТАЕТ
```

### Фаза 4: Заставка + Главное меню (3-4 дня)

```
[ ] 21. src/animation/tween.rs + easing.rs
[ ] 22. src/screens/splash/splash_screen.rs
[ ] 23. src/screens/main_menu/main_menu_screen.rs
[ ] 24. src/app.rs — App + AppState машина
[ ] 25. src/main.rs — полная точка входа
[ ] 26. Тест: запуск → заставка → меню с кнопками → РАБОТАЕТ
```

### Фаза 5: Настройки + Конфиг (2-3 дня)

```
[ ] 27. src/platform/paths.rs — AppPaths
[ ] 28. src/screens/settings/ — все вкладки
[ ] 29. Тест: изменить разрешение → сохранить → перезапустить → применилось
```

### Фаза 6: Создание персонажа (1 неделя)

```
[ ] 30. src/save/character_data.rs
[ ] 31. src/ui/selector.rs + color_picker.rs + slider.rs + progress_bar.rs
[ ] 32. src/screens/character_creation/ — все 10 шагов
[ ] 33. Тест: пройти все 10 шагов → CharacterData сохранена
```

### Фаза 7: Загрузочный экран (4-5 дней)

```
[ ] 34. src/screens/loading/load_stages.rs — все 11 стадий
[ ] 35. src/screens/loading/loading_screen.rs — UI
[ ] 36. Многопоточная загрузка (crossbeam-channel)
[ ] 37. Тест: загрузка проходит все 11 стадий, прогресс-бар двигается
```

### Фаза 8: Переход в Playing (2-3 дня)

```
[ ] 38. Fade transition (FadeOverlay)
[ ] 39. Подключение GameWorld к AppState::Playing
[ ] 40. Финальный тест: полный путь Запуск → Меню →
        Создание персонажа → Загрузка → Игра → Пауза → Меню
```

---

## ИТОГОВАЯ СВОДКА ЗАВИСИМОСТЕЙ

| Крейт | Версия | Зачем |
|---|---|---|
| `winit` | 0.30 | Окно + ввод |
| `glow` | 0.14 | OpenGL биндинги |
| `glutin` | 0.32 | GL контекст |
| `glutin-winit` | 0.5 | Интеграция |
| `nalgebra` | 0.33 | 3D математика (физика) |
| `glam` | 0.28 | **NEW** 2D математика UI (SIMD) |
| `fontdue` | 0.9 | **NEW** Шрифты TTF → bitmap |
| `image` | 0.25 | **NEW** Загрузка PNG/JPG |
| `kira` | 0.9 | **NEW** Аудио (музыка + SFX) |
| `serde` | 1.0 | **NEW** Сериализация |
| `toml` | 0.8 | **NEW** Парсинг .toml |
| `crossbeam-channel` | 0.5 | Канал загрузки |
| `rayon` | 1.10 | Параллельная загрузка |
| `tracing` | 0.1 | Логи |
| `tracing-subscriber` | 0.3 | Логи |
| `bytemuck` | 1.16 | Буферы GPU |
| `uuid` | 1.8 | **NEW** ID сохранений |
| `rand` | 0.8 | **NEW** Генерация параметров |
| `anyhow` | 1.0 | **NEW** Обработка ошибок |
| `dirs` | 5.0 | **NEW** %APPDATA% путь |

**Жирным** — новые зависимости, которых нет в текущем Cargo.toml.

---

*Конец документа. Версия 1.0 — Апрель 2026.*
