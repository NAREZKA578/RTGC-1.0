# RTGC-1.0 — МАСТЕР-ПЛАН РАЗРАБОТКИ ИГРЫ И ДВИЖКА
### Полностью собственный движок на Rust | Команда 2-3 человека
### OpenGL сейчас → DX11/DX12/Vulkan позже | Своя физика с нуля
> Версия документа: 2.0 | Апрель 2026 | NAREZKA578

---

## СОДЕРЖАНИЕ

```
ЧАСТЬ I   — ФУНДАМЕНТ
  Раздел 1  — Философия и принципы архитектуры
  Раздел 2  — Команда: роли и распределение
  Раздел 3  — Полные зависимости (Cargo.toml)
  Раздел 4  — Полная структура папок (все файлы)
  Раздел 5  — Архитектурные слои (диаграмма)

ЧАСТЬ II  — ДВИЖОК (ENGINE)
  Раздел 6  — Платформенный слой (Window, Input, Paths)
  Раздел 7  — RHI: абстракция рендеринга (OpenGL → DX12/Vulkan)
  Раздел 8  — OpenGL бэкенд (полная реализация)
  Раздел 9  — ФИЗИЧЕСКИЙ ДВИЖОК (полная архитектура с нуля)
               9.1  Математические примитивы
               9.2  Широкая фаза (BVH, AABB-дерево)
               9.3  Узкая фаза (GJK, EPA, SAT)
               9.4  Формы столкновений
               9.5  Твёрдое тело (RigidBody)
               9.6  Интегратор (Semi-implicit Euler)
               9.7  Решатель ограничений (Sequential Impulse)
               9.8  Суставы и соединения
               9.9  Физика колёсного транспорта
               9.10 Физика гусеничного транспорта
               9.11 Физика вертолёта
               9.12 Физика ландшафта (коллизия с heightmap)
               9.13 Физика жидкого грунта (грязь, снег)
  Раздел 10 — Система ландшафта (Terrain)
  Раздел 11 — Стриминг мира (World Streaming)
  Раздел 12 — Система освещения и теней
  Раздел 13 — Аудио система
  Раздел 14 — UI система (отсылка к doc v1.0)
  Раздел 15 — Система анимации персонажа (скелетная)

ЧАСТЬ III — ИГРОВЫЕ СИСТЕМЫ
  Раздел 16 — Пешеходный персонаж
  Раздел 17 — Система транспорта
  Раздел 18 — Система повреждений и ремонта
  Раздел 19 — Инвентарь и грузы
  Раздел 20 — Экономика и торговля
  Раздел 21 — Система миссий и контрактов
  Раздел 22 — NPC (водители, пешеходы)
  Раздел 23 — Строительство базы
  Раздел 24 — Погода и сезоны
  Раздел 25 — Система репутации
  Раздел 26 — Компания игрока (ИП/ООО)

ЧАСТЬ IV  — МУЛЬТИПЛЕЕР
  Раздел 27 — Сетевая архитектура (UDP, P2P, STUN)
  Раздел 28 — Синхронизация состояний

ЧАСТЬ V   — КОНТЕНТ И ИНСТРУМЕНТЫ
  Раздел 29 — Контент-пайплайн (ассеты, форматы)
  Раздел 30 — Будущие бэкенды (DX11/DX12/Vulkan)
  Раздел 31 — Система модификаций

ЧАСТЬ VI  — ROADMAP
  Раздел 32 — Полный план по фазам (Ф0 → Ф6)
  Раздел 33 — Критерии готовности каждой фазы
  Раздел 34 — Риски и как их избежать
```

---

# ЧАСТЬ I — ФУНДАМЕНТ

---

## Раздел 1 — ФИЛОСОФИЯ И ПРИНЦИПЫ АРХИТЕКТУРЫ

### Главные принципы

```
1. СЛОИ, НЕ МОНОЛИТ
   Каждый слой знает только о слое ниже.
   Игровая логика не знает про OpenGL.
   Физика не знает про рендер.
   Рендер не знает про логику игры.

2. ДАННЫЕ ОТДЕЛЕНЫ ОТ ЛОГИКИ
   Структуры данных (struct) — в отдельных файлах.
   Логика обработки — в impl или отдельных системах.
   Это позволяет легко тестировать и параллелить.

3. СНАЧАЛА РАБОТАЕТ — ПОТОМ КРАСИВО
   Каждая фаза заканчивается рабочим результатом.
   Никаких "сделаем потом" без явного TODO с номером задачи.

4. ОДНА ОТВЕТСТВЕННОСТЬ
   Каждый файл делает ровно одну вещь.
   Файл > 500 строк — сигнал разбить его.

5. RHI-АБСТРАКЦИЯ С ПЕРВОГО ДНЯ
   OpenGL пишется за трейтом RhiDevice.
   Когда придёт время Vulkan/DX12 — меняем бэкенд, не игру.

6. ФИЗИКА — ДЕТЕРМИНИРОВАННАЯ
   Одни и те же входные данные → один и тот же результат.
   Это критично для сетевой синхронизации в кооперативе.
   Никакого f32::sin() через платформенные функции — только nalgebra.

7. ТЕСТИРУЙ ВСЁ ЧТО МОЖНО
   Физика: unit-тесты на каждую формулу.
   Рендер: интеграционные тесты (рендер в texture, сравнение).
   Экономика: тесты на крайние случаи (0 денег, макс. груз).
```

### Архитектурные слои (стек)

```
┌─────────────────────────────────────────────────────────────┐
│                    ИГРОВОЙ КОНТЕНТ                           │
│  Карта Новосибирска, транспорт, миссии, NPC, экономика      │
├─────────────────────────────────────────────────────────────┤
│                   ИГРОВЫЕ СИСТЕМЫ                            │
│  PlayerSystem, VehicleSystem, InventorySystem, MissionSystem │
├─────────────────────────────────────────────────────────────┤
│                   ДВИЖОК (ENGINE)                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │ Physics  │ │ Renderer │ │  Audio   │ │   World      │  │
│  │ Engine   │ │  (RHI)   │ │  (kira)  │ │  Streaming   │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │  Font    │ │    UI    │ │ Animation│ │   SaveSystem │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                  ПЛАТФОРМЕННЫЙ СЛОЙ                          │
│         winit | glutin | glow | dirs | crossbeam            │
├─────────────────────────────────────────────────────────────┤
│                    ОПЕРАЦИОННАЯ СИСТЕМА                       │
│              Windows 10 (Linux/macOS — позже)               │
└─────────────────────────────────────────────────────────────┘
```

---

## Раздел 2 — КОМАНДА: РОЛИ И РАСПРЕДЕЛЕНИЕ

```
Для команды 2-3 человека рекомендуется такое разделение:

┌─────────────────────────────────────────────────────────────────┐
│ РАЗРАБОТЧИК 1 — Engine Lead (ведущий разработчик движка)        │
│   • Физический движок (весь раздел 9)                           │
│   • RHI абстракция (раздел 7)                                   │
│   • Terrain система (раздел 10)                                 │
│   • World Streaming (раздел 11)                                 │
│   • Сетевая архитектура (раздел 27-28)                          │
├─────────────────────────────────────────────────────────────────┤
│ РАЗРАБОТЧИК 2 — Gameplay Lead (ведущий разработчик геймплея)    │
│   • Все игровые системы (разделы 16-26)                         │
│   • UI система (раздел 14)                                      │
│   • Система анимации (раздел 15)                                │
│   • Система миссий (раздел 21)                                  │
│   • Экономика (раздел 20)                                       │
├─────────────────────────────────────────────────────────────────┤
│ РАЗРАБОТЧИК 3 — Content / Tools (контент и инструменты)         │
│   • Аудио система (раздел 13)                                   │
│   • Контент-пайплайн (раздел 29)                                │
│   • TOML данные (ВУЗы, транспорт, регионы)                      │
│   • Тестирование и QA                                           │
│   • Документация                                                │
└─────────────────────────────────────────────────────────────────┘

Если команда 2 человека: Dev1 берёт Engine + Network,
                          Dev2 берёт Gameplay + Content + Audio.
```

### Правила совместной работы

```
Ветки Git:
  main          — всегда компилируется и запускается
  dev           — текущая разработка
  feature/XXX   — отдельная фича

Правило слияния:
  feature/* → dev (через PR с ревью)
  dev → main   (только после прохождения всех тестов)

Commit-формат:
  [engine/physics] Добавить BVH широкую фазу
  [gameplay/vehicle] Исправить подвеску UAZ при > 90°
  [ui/menu] Добавить анимацию появления кнопок
  [fix] Исправить panic при нулевом векторе в GJK

Обязательно перед каждым коммитом:
  cargo test          — все тесты проходят
  cargo clippy        — нет предупреждений
  cargo fmt           — код отформатирован
```

---

## Раздел 3 — ПОЛНЫЕ ЗАВИСИМОСТИ (Cargo.toml)

```toml
[package]
name        = "rtgc"
version     = "1.0.0-dev"
edition     = "2021"
authors     = ["NAREZKA578"]
description = "RTGC-1.0 — Russian Truck & Helicopter Game"
license     = "Apache-2.0"
resolver    = "2"

# ═══════════════════════════════════════════════════════════════
# ПЛАТФОРМА / ОКНО / ВВОД
# ═══════════════════════════════════════════════════════════════
[dependencies]
winit            = "0.30"         # Окно, события: мышь, клавиатура, геймпад
gilrs            = "0.10"         # Геймпад (xinput на Windows, обёртка над winit)

# ═══════════════════════════════════════════════════════════════
# РЕНДЕРИНГ — OpenGL бэкенд
# ═══════════════════════════════════════════════════════════════
glow             = "0.14"         # Безопасные OpenGL биндинги
glutin           = "0.32"         # OpenGL контекст
glutin-winit     = "0.5"          # glutin ↔ winit интеграция

# ═══════════════════════════════════════════════════════════════
# МАТЕМАТИКА
# ═══════════════════════════════════════════════════════════════
nalgebra         = { version = "0.33", features = ["bytemuck"] }
                                  # Физика: Vec3, Mat3, Mat4, Quat, Isometry3
glam             = "0.28"         # UI: Vec2, Vec4 (SIMD, быстрее для 2D)

# ═══════════════════════════════════════════════════════════════
# ШРИФТЫ
# ═══════════════════════════════════════════════════════════════
fontdue          = "0.9"          # TTF/OTF растеризация, чистый Rust

# ═══════════════════════════════════════════════════════════════
# ИЗОБРАЖЕНИЯ
# ═══════════════════════════════════════════════════════════════
image            = { version = "0.25", default-features = false,
                     features = ["png", "jpeg"] }

# ═══════════════════════════════════════════════════════════════
# АУДИО
# ═══════════════════════════════════════════════════════════════
kira             = "0.9"          # Аудио движок: fade, 3D, клоки

# ═══════════════════════════════════════════════════════════════
# СЕРИАЛИЗАЦИЯ / КОНФИГ
# ═══════════════════════════════════════════════════════════════
serde            = { version = "1.0", features = ["derive"] }
toml             = "0.8"
serde_json       = "1.0"          # Для отладочных дампов физики

# ═══════════════════════════════════════════════════════════════
# ПОТОКИ И ПАРАЛЛЕЛИЗМ
# ═══════════════════════════════════════════════════════════════
crossbeam-channel = "0.5"         # MPSC канал загрузчика
rayon             = "1.10"        # Параллельная обработка (загрузка, физика)
parking_lot       = "0.12"        # Быстрые Mutex/RwLock (замена std::sync)

# ═══════════════════════════════════════════════════════════════
# СЕТЬ (для кооператива, Фаза 4)
# ═══════════════════════════════════════════════════════════════
tokio            = { version = "1.38", features = ["full"] }
                                  # Async runtime для сетевого кода
quinn            = "0.11"         # QUIC транспорт (UDP + надёжность)
                                  # Лучше чем сырой UDP для P2P

# ═══════════════════════════════════════════════════════════════
# УТИЛИТЫ
# ═══════════════════════════════════════════════════════════════
bytemuck         = { version = "1.16", features = ["derive"] }
uuid             = { version = "1.8",  features = ["v4"] }
rand             = "0.8"
anyhow           = "1.0"
dirs             = "5.0"
hashbrown        = "0.14"         # Быстрая HashMap (FxHashMap)
smallvec         = "1.13"         # Vec на стеке до N элементов (BVH узлы)
bitflags         = "2.6"          # Флаги слоёв коллизий
ordered-float    = "4.2"          # f32/f64 в HashMap ключах
slotmap          = "1.0"          # Стабильные ID для физических тел

# ═══════════════════════════════════════════════════════════════
# ЛОГИРОВАНИЕ
# ═══════════════════════════════════════════════════════════════
tracing             = "0.1"
tracing-subscriber  = { version = "0.3", features = ["env-filter"] }

# ═══════════════════════════════════════════════════════════════
# ПРОФИЛИ СБОРКИ
# ═══════════════════════════════════════════════════════════════
[profile.dev]
opt-level       = 1               # Физика без оптимизаций — слишком медленно
overflow-checks = true

[profile.release]
lto             = "thin"
codegen-units   = 1
strip           = true
panic           = "abort"         # Меньший .exe, быстрее

[profile.dev.package."*"]
opt-level       = 2               # Зависимости — всегда оптимизированы

# ═══════════════════════════════════════════════════════════════
# ФИЧИ
# ═══════════════════════════════════════════════════════════════
[features]
default       = ["opengl"]
opengl        = []
dx11          = []                # Будущее
vulkan        = []                # Будущее
debug_physics = []                # Отрисовка коллайдеров
debug_ui      = []                # Bounding box'ы виджетов
debug_network = []                # Сетевые пакеты в лог
```

---

## Раздел 4 — ПОЛНАЯ СТРУКТУРА ПАПОК

```
RTGC-1.0/
│
├── Cargo.toml
├── Cargo.lock
├── .gitignore                   ← /saves /target /.vscode /assets/data/cache
├── LICENSE
├── README.md
├── PLEN3.md
│
├── .github/workflows/rust.yml
│
├── docs/                        ◄ ДОКУМЕНТАЦИЯ ПРОЕКТА
│   ├── MENU_LOADING_DESIGN.md   — Документ v1.0 (меню/загрузка)
│   ├── MASTER_PLAN.md           — Этот документ
│   ├── physics/
│   │   ├── gjk_epa.md           — Теория алгоритмов коллизий
│   │   ├── vehicle_physics.md   — Физика подвески, Pacejka
│   │   └── helicopter_physics.md
│   └── rhi/
│       └── abstraction_layer.md — RHI трейты и интерфейсы
│
├── assets/                      ◄ ВСЕ ИГРОВЫЕ АССЕТЫ
│   ├── fonts/
│   │   ├── main_font.ttf
│   │   ├── title_font.ttf
│   │   └── mono_font.ttf
│   │
│   ├── textures/
│   │   ├── menu/                — Фоны, кнопки, лого
│   │   ├── loading/             — Фон загрузки, прогресс-бар
│   │   ├── character/           — Портреты, причёски, UAZ цвета
│   │   ├── icons/               — Иконки навыков, предметов
│   │   ├── terrain/
│   │   │   ├── grass_albedo.png
│   │   │   ├── grass_normal.png
│   │   │   ├── dirt_albedo.png
│   │   │   ├── dirt_normal.png
│   │   │   ├── rock_albedo.png
│   │   │   ├── rock_normal.png
│   │   │   ├── snow_albedo.png
│   │   │   ├── snow_normal.png
│   │   │   ├── asphalt_albedo.png
│   │   │   ├── asphalt_normal.png
│   │   │   └── mud_albedo.png
│   │   ├── vehicles/
│   │   │   ├── uaz_patriot/
│   │   │   │   ├── body_albedo.png
│   │   │   │   ├── body_normal.png
│   │   │   │   ├── wheel_albedo.png
│   │   │   │   └── interior_albedo.png
│   │   │   └── mi2/              — Ми-2 вертолёт (Фаза 3)
│   │   ├── skybox/
│   │   │   ├── day_px.png        — Skybox: 6 граней × 4 варианта погоды
│   │   │   ├── day_nx.png
│   │   │   ├── day_py.png
│   │   │   ├── day_ny.png
│   │   │   ├── day_pz.png
│   │   │   ├── day_nz.png
│   │   │   ├── night_*.png
│   │   │   ├── overcast_*.png
│   │   │   └── sunrise_*.png
│   │   └── effects/
│   │       ├── rain_drop.png
│   │       ├── snow_particle.png
│   │       └── dust_particle.png
│   │
│   ├── meshes/                  ◄ 3D МОДЕЛИ (формат .rtmesh — свой бинарный)
│   │   ├── vehicles/
│   │   │   ├── uaz_patriot_body.rtmesh
│   │   │   ├── uaz_patriot_wheel.rtmesh
│   │   │   ├── uaz_patriot_interior.rtmesh
│   │   │   └── uaz_patriot_collision.rtmesh  ← упрощённая для физики
│   │   ├── buildings/
│   │   │   ├── khrushchevka.rtmesh
│   │   │   ├── warehouse.rtmesh
│   │   │   ├── gas_station.rtmesh
│   │   │   └── workshop.rtmesh
│   │   ├── props/
│   │   │   ├── barrel_200l.rtmesh
│   │   │   ├── brick_pallet.rtmesh
│   │   │   └── fuel_canister.rtmesh
│   │   └── character/
│   │       ├── male_base.rtmesh
│   │       ├── female_base.rtmesh
│   │       └── skeleton.rtanim   ← скелет + анимации
│   │
│   ├── audio/
│   │   ├── music/
│   │   │   ├── menu_theme.ogg
│   │   │   ├── loading_ambient.ogg
│   │   │   ├── char_creation.ogg
│   │   │   └── game_ambient_*.ogg   — Несколько треков для игры
│   │   └── sfx/
│   │       ├── ui/               — Клики, hover, confirm, error
│   │       ├── engine/
│   │       │   ├── uaz_idle.ogg
│   │       │   ├── uaz_low.ogg   — Нарезки RPM (4 диапазона)
│   │       │   ├── uaz_mid.ogg
│   │       │   ├── uaz_high.ogg
│   │       │   └── uaz_start.ogg
│   │       ├── terrain/
│   │       │   ├── gravel_roll.ogg
│   │       │   ├── mud_roll.ogg
│   │       │   ├── snow_roll.ogg
│   │       │   └── asphalt_roll.ogg
│   │       ├── physics/
│   │       │   ├── impact_light.ogg
│   │       │   ├── impact_medium.ogg
│   │       │   ├── impact_heavy.ogg
│   │       │   └── metal_creak.ogg
│   │       └── environment/
│   │           ├── wind.ogg
│   │           ├── rain.ogg
│   │           └── forest_birds.ogg
│   │
│   ├── shaders/
│   │   ├── ui/
│   │   │   ├── rect.vert / rect.frag
│   │   │   ├── image.vert / image.frag
│   │   │   └── text.vert / text.frag
│   │   └── game/
│   │       ├── terrain.vert / terrain.frag
│   │       ├── terrain_shadow.vert      ← depth pass для теней
│   │       ├── vehicle.vert / vehicle.frag
│   │       ├── vehicle_shadow.vert
│   │       ├── skybox.vert / skybox.frag
│   │       ├── shadow_map.vert / shadow_map.frag
│   │       ├── particle.vert / particle.frag
│   │       └── post/
│   │           ├── tonemap.frag         — HDR tonemapping
│   │           ├── fog.frag             — Атмосферный туман
│   │           └── vignette.frag        — Виньетка
│   │
│   └── data/                    ◄ ИГРОВЫЕ ДАННЫЕ (TOML)
│       ├── universities.toml    — ВУЗы России и Китая
│       ├── vehicles.toml        — Все транспортные средства
│       ├── skills.toml          — Описания навыков
│       ├── regions.toml         — Стартовые районы Новосибирска
│       ├── settlements.toml     — Города, посёлки, АЗС, СТО
│       ├── resources.toml       — Типы ресурсов, цены, вес
│       ├── recipes.toml         — Производственные рецепты
│       ├── missions.toml        — Шаблоны миссий (Серёга и др.)
│       ├── tips.toml            — Подсказки загрузочного экрана
│       ├── engine_sounds.toml   — Кривые звука двигателей
│       └── terrain/
│           ├── novosibirsk_heightmap.r16  — 16-bit heightmap
│           ├── novosibirsk_splatmap.png   — Карта поверхностей
│           ├── novosibirsk_roadmap.png    — Маска дорог
│           └── novosibirsk_meta.toml      — Масштаб, координаты
│
├── config/
│   └── settings_default.toml    — Шаблон (копируется в %APPDATA%)
│
├── tools/                       ◄ ИНСТРУМЕНТЫ РАЗРАБОТЧИКА
│   ├── mesh_converter/          — Конвертер OBJ/FBX → .rtmesh
│   │   └── src/main.rs
│   ├── heightmap_editor/        — Редактор heightmap Новосибирска
│   │   └── src/main.rs
│   └── physics_debugger/        — Визуализация физических объектов
│       └── src/main.rs
│
└── src/                         ◄ ВЕСЬ ИСХОДНЫЙ КОД
    │
    ├── main.rs                  — Точка входа
    ├── lib.rs                   — pub mod декларации
    ├── app.rs                   — App: главный цикл, AppState машина
    │
    ├── core/                    ◄ ЯДРО
    │   ├── mod.rs
    │   ├── app_state.rs         — enum AppState
    │   ├── event_bus.rs         — Внутренние события движка
    │   ├── timer.rs             — FrameTimer, DeltaTime
    │   └── error.rs             — RtgcError enum
    │
    ├── platform/                ◄ ПЛАТФОРМА
    │   ├── mod.rs
    │   ├── window.rs            — Создание окна + GL контекста
    │   ├── input.rs             — InputState (клавиши, мышь, геймпад)
    │   └── paths.rs             — AppPaths: %APPDATA%, assets/
    │
    ├── rhi/                     ◄ RENDERING HARDWARE INTERFACE
    │   ├── mod.rs               — pub use + выбор бэкенда
    │   ├── types.rs             — Общие типы: BufferDesc, TextureDesc...
    │   ├── traits.rs            — trait RhiDevice, RhiBuffer, RhiTexture...
    │   ├── command_buffer.rs    — CommandBuffer: список команд рендера
    │   └── opengl/              — OpenGL БЭКЕНД
    │       ├── mod.rs
    │       ├── device.rs        — GlDevice impl RhiDevice
    │       ├── buffer.rs        — GlBuffer impl RhiBuffer
    │       ├── texture.rs       — GlTexture impl RhiTexture
    │       ├── shader.rs        — GlShader: компиляция GLSL
    │       ├── pipeline.rs      — GlPipeline: VAO + шейдер + состояние
    │       └── framebuffer.rs   — GlFramebuffer (для теней, пост-процессинга)
    │
    ├── renderer/                ◄ ВЫСОКОУРОВНЕВЫЙ РЕНДЕРЕР
    │   ├── mod.rs
    │   ├── texture_cache.rs     — Кеш текстур (путь → RhiTexture)
    │   ├── mesh_cache.rs        — Кеш мешей
    │   ├── render_graph.rs      — Граф рендера: порядок пассов
    │   │
    │   ├── ui_renderer/         — (описан в документе v1.0)
    │   │   ├── mod.rs
    │   │   ├── batch.rs
    │   │   ├── rect_renderer.rs
    │   │   ├── image_renderer.rs
    │   │   └── text_renderer.rs
    │   │
    │   └── game_renderer/       — Рендер игровой сцены
    │       ├── mod.rs
    │       ├── terrain_renderer.rs
    │       ├── vehicle_renderer.rs
    │       ├── character_renderer.rs
    │       ├── skybox_renderer.rs
    │       ├── shadow_renderer.rs    — Shadow map
    │       ├── particle_renderer.rs  — Дождь, снег, пыль
    │       └── post_renderer.rs      — Tonemap, fog, vignette
    │
    ├── physics/                 ◄ ФИЗИЧЕСКИЙ ДВИЖОК (с нуля!)
    │   ├── mod.rs
    │   ├── math/                — Математические примитивы физики
    │   │   ├── mod.rs
    │   │   ├── aabb.rs          — AABB (Axis-Aligned Bounding Box)
    │   │   ├── ray.rs           — Ray: origin + direction
    │   │   └── transform.rs     — PhysTransform: pos + rot
    │   │
    │   ├── broad_phase/         — Широкая фаза обнаружения столкновений
    │   │   ├── mod.rs
    │   │   ├── bvh.rs           — Dynamic BVH (Bounding Volume Hierarchy)
    │   │   └── pair_filter.rs   — Фильтрация пар по слоям
    │   │
    │   ├── narrow_phase/        — Узкая фаза: точные столкновения
    │   │   ├── mod.rs
    │   │   ├── gjk.rs           — GJK (Gilbert-Johnson-Keerthi)
    │   │   ├── epa.rs           — EPA (Expanding Polytope Algorithm)
    │   │   ├── sat.rs           — SAT для box-box (быстрее GJK)
    │   │   ├── sphere_sphere.rs — Аналитическое решение
    │   │   ├── capsule_capsule.rs
    │   │   └── contact.rs       — ContactManifold: точки контакта
    │   │
    │   ├── shapes/              — Формы коллайдеров
    │   │   ├── mod.rs
    │   │   ├── sphere.rs
    │   │   ├── capsule.rs       — Для персонажа
    │   │   ├── box_shape.rs     — OBB (Oriented Bounding Box)
    │   │   ├── convex_hull.rs   — Выпуклая оболочка (транспорт)
    │   │   ├── trimesh.rs       — Треугольная сетка (ландшафт, здания)
    │   │   └── heightfield.rs   — Heightmap коллайдер (оптимизация)
    │   │
    │   ├── dynamics/            — Динамика твёрдых тел
    │   │   ├── mod.rs
    │   │   ├── rigid_body.rs    — RigidBody struct
    │   │   ├── integrator.rs    — Semi-implicit Euler
    │   │   ├── mass_props.rs    — Расчёт момента инерции
    │   │   └── force_gen.rs     — Генераторы сил: гравитация, ветер
    │   │
    │   ├── constraints/         — Решатель ограничений
    │   │   ├── mod.rs
    │   │   ├── solver.rs        — Sequential Impulse Solver
    │   │   ├── contact_constraint.rs — Ограничение контакта
    │   │   ├── friction_constraint.rs
    │   │   └── joints/
    │   │       ├── mod.rs
    │   │       ├── revolute.rs  — Шарнирное соединение (колесо)
    │   │       ├── prismatic.rs — Поступательное (амортизатор)
    │   │       └── ball_socket.rs — Шаровой шарнир
    │   │
    │   ├── vehicle/             — Специализированная физика транспорта
    │   │   ├── mod.rs
    │   │   ├── raycast_vehicle.rs   — Основа: лучи подвески
    │   │   ├── wheel.rs             — Колесо: трение, Pacejka
    │   │   ├── suspension.rs        — Пружина-демпфер
    │   │   ├── engine_model.rs      — Крутящий момент, RPM, передачи
    │   │   ├── differential.rs      — Дифференциал (открытый/блок/LSD)
    │   │   ├── winch.rs             — Лебёдка
    │   │   └── tracked_vehicle.rs   — Гусеничная физика
    │   │
    │   ├── helicopter/          — Физика вертолёта
    │   │   ├── mod.rs
    │   │   ├── rotor.rs         — Модель несущего ротора
    │   │   ├── tail_rotor.rs    — Хвостовой ротор (антиторк)
    │   │   ├── aerodynamics.rs  — Подъёмная сила, сопротивление
    │   │   └── ground_effect.rs — Эффект земли
    │   │
    │   ├── terrain_physics.rs   — Коллизия с ландшафтом
    │   ├── soft_ground.rs       — Грязь, снег, болото (деформация)
    │   └── world.rs             — PhysicsWorld: всё вместе
    │
    ├── terrain/                 ◄ СИСТЕМА ЛАНДШАФТА
    │   ├── mod.rs
    │   ├── heightmap.rs         — Загрузка .r16 heightmap
    │   ├── cdlod.rs             — CDLOD алгоритм LOD
    │   ├── chunk.rs             — TerrainChunk: данные и меш
    │   ├── splatmap.rs          — Смешивание текстур поверхностей
    │   ├── road_network.rs      — RoadNetwork: граф дорог
    │   └── vegetation.rs        — Деревья, кусты (инстансинг)
    │
    ├── world/                   ◄ МИР И СТРИМИНГ
    │   ├── mod.rs
    │   ├── world_manager.rs     — Главный менеджер мира
    │   ├── chunk_streamer.rs    — Загрузка/выгрузка чанков
    │   ├── day_night.rs         — DayNightCycle
    │   ├── weather.rs           — WeatherSystem
    │   ├── season.rs            — Season: лето/осень/зима/весна
    │   └── fog.rs               — Атмосферный туман
    │
    ├── font/                    — (описан в документе v1.0)
    ├── audio/                   — (описан в документе v1.0)
    ├── ui/                      — (описан в документе v1.0)
    ├── animation/               — (описан в документе v1.0)
    ├── screens/                 — (описан в документе v1.0)
    │
    ├── animation_system/        ◄ СКЕЛЕТНАЯ АНИМАЦИЯ (3D)
    │   ├── mod.rs
    │   ├── skeleton.rs          — Skeleton: кости, иерархия
    │   ├── clip.rs              — AnimationClip: ключевые кадры
    │   ├── blender.rs           — Смешивание анимаций
    │   └── state_machine.rs     — AnimStateMachine: ходьба/бег/сидит
    │
    ├── gameplay/                ◄ ИГРОВЫЕ СИСТЕМЫ
    │   ├── mod.rs
    │   │
    │   ├── player/
    │   │   ├── mod.rs
    │   │   ├── player.rs        — Player struct
    │   │   ├── controller.rs    — Управление на ногах
    │   │   └── camera.rs        — Камера (1st/3rd person)
    │   │
    │   ├── vehicle/
    │   │   ├── mod.rs
    │   │   ├── vehicle.rs       — Vehicle struct (параметры)
    │   │   ├── vehicle_input.rs — Обработка ввода транспорта
    │   │   ├── parts.rs         — VehicleParts: детали, прочность
    │   │   ├── fuel.rs          — FuelSystem
    │   │   └── enter_exit.rs    — Вход/выход из транспорта (F)
    │   │
    │   ├── inventory/
    │   │   ├── mod.rs
    │   │   ├── inventory.rs     — Grid-based инвентарь
    │   │   ├── item.rs          — Item: тип, вес, размер
    │   │   └── container.rs     — Container (рюкзак, кузов, склад)
    │   │
    │   ├── economy/
    │   │   ├── mod.rs
    │   │   ├── wallet.rs        — PlayerWallet: RUB/CNY/USD
    │   │   ├── market.rs        — MarketPrice, торговля
    │   │   └── company.rs       — ИП/ООО
    │   │
    │   ├── missions/
    │   │   ├── mod.rs
    │   │   ├── mission.rs       — Mission struct
    │   │   ├── objective.rs     — MissionObjective
    │   │   └── dispatcher.rs    — MissionDispatcher: выдача контрактов
    │   │
    │   ├── skills/
    │   │   ├── mod.rs
    │   │   ├── skill.rs         — Skill: rank, mastery, hours
    │   │   └── skill_set.rs     — PlayerSkills (20+ навыков)
    │   │
    │   ├── npc/
    │   │   ├── mod.rs
    │   │   ├── npc.rs           — NPC struct
    │   │   ├── driver_ai.rs     — ИИ водителя (следование по дороге)
    │   │   └── pedestrian_ai.rs — ИИ пешехода (декорация)
    │   │
    │   ├── base_building/
    │   │   ├── mod.rs
    │   │   ├── structure.rs     — Structure: тип, ресурсы, позиция
    │   │   └── placement.rs     — Система размещения (сетка 1×1м)
    │   │
    │   ├── hud/
    │   │   ├── mod.rs
    │   │   ├── hud.rs           — HUD: все элементы интерфейса
    │   │   ├── compass.rs       — Компас 400×24px
    │   │   ├── speedometer.rs   — Скорость, RPM
    │   │   └── minimap.rs       — Мини-карта
    │   │
    │   └── dialog/
    │       ├── mod.rs
    │       └── sms_dialog.rs    — SMS-стиль диалоги (Серёга)
    │
    ├── network/                 ◄ СЕТЬ (Фаза 4)
    │   ├── mod.rs
    │   ├── protocol.rs          — Пакеты, сериализация
    │   ├── host.rs              — Host (P2P сервер)
    │   ├── client.rs            — Client
    │   └── sync.rs              — Синхронизация состояний
    │
    └── save/                    ◄ СОХРАНЕНИЯ
        ├── mod.rs
        ├── player_profile.rs
        ├── character_data.rs
        ├── game_state.rs        — Полное состояние мира для сохранения
        └── save_manager.rs
```

---

# ЧАСТЬ II — ДВИЖОК (ENGINE)

---

## Раздел 6 — ПЛАТФОРМЕННЫЙ СЛОЙ

### src/platform/window.rs

```
Задача: создать окно + OpenGL контекст, скрыть детали winit/glutin.

Инициализация (по шагам):
1. Создать winit::EventLoop
2. Создать winit::Window (скрытое до готовности)
3. Создать glutin::config (OpenGL 3.3 Core, MSAA=0 для UI)
4. Создать glutin::surface (PBuffer для начала)
5. Создать glutin::context + make_current()
6. Создать glow::Context из raw функций glutin
7. Установить иконку окна из assets/textures/menu/icon.png
8. Показать окно

Fallback при ошибке OpenGL 3.3:
→ Попробовать 3.1 Core
→ Попробовать 2.1 Compatibility
→ Если всё не работает: MessageBox с ошибкой + exit(1)

Обработка событий winit:
  WindowEvent::Resized(size)    → обновить viewport, пересчитать UI layout
  WindowEvent::CloseRequested   → AppState::Exit
  WindowEvent::KeyboardInput    → InputState::process_keyboard()
  WindowEvent::MouseInput       → InputState::process_mouse_button()
  WindowEvent::CursorMoved      → InputState::process_mouse_move()
  WindowEvent::MouseWheel       → InputState::process_scroll()
  DeviceEvent::GamepadButton    → InputState::process_gamepad() (через gilrs)
```

### src/platform/input.rs

```rust
// InputState хранит текущий кадр и предыдущий кадр
// чтобы отличить "нажато впервые" от "удерживается"

pub struct InputState {
    // Клавиатура
    keys_down:     HashSet<KeyCode>,   // удерживаются
    keys_pressed:  HashSet<KeyCode>,   // нажаты в этом кадре
    keys_released: HashSet<KeyCode>,   // отпущены в этом кадре

    // Мышь
    mouse_pos:     Vec2,
    mouse_delta:   Vec2,
    mouse_buttons: [MouseButtonState; 5],
    scroll_delta:  f32,

    // Геймпад (через gilrs)
    gamepad_axes:    [f32; 8],
    gamepad_buttons: [bool; 16],
}

// Методы:
//   is_key_down(key)     → bool  (удерживается)
//   is_key_pressed(key)  → bool  (нажато в этом кадре)
//   is_key_released(key) → bool  (отпущено в этом кадре)
//   mouse_screen_pos()   → Vec2  (в пикселях)
//   mouse_ndc_pos()      → Vec2  (-1..1)
//   is_mouse_over(rect)  → bool
```

---

## Раздел 7 — RHI: АБСТРАКЦИЯ РЕНДЕРИНГА

### Философия RHI

```
Цель: написать OpenGL сейчас, не переписывать всё при переходе на Vulkan/DX12.

Принцип: игровой код работает ТОЛЬКО через RHI трейты.
         Никакого glow:: за пределами src/rhi/opengl/.

Аналог: wgpu, но собственный и проще.
```

### src/rhi/traits.rs

```rust
// ─── Основные дескрипторы ────────────────────────────────────

pub struct BufferDesc {
    pub size:   usize,
    pub usage:  BufferUsage,   // Vertex / Index / Uniform / Storage
    pub access: BufferAccess,  // Static / Dynamic / Stream
}

pub struct TextureDesc {
    pub width:  u32,
    pub height: u32,
    pub format: TextureFormat,  // Rgba8 / R16 / Depth24Stencil8 / ...
    pub mips:   bool,
    pub samples: u32,           // 1 = no MSAA
}

pub struct ShaderDesc<'a> {
    pub vert_src: &'a str,   // GLSL для OpenGL; HLSL для DX; SPIR-V для Vulkan
    pub frag_src: &'a str,
    pub label:    &'a str,   // Для отладки
}

pub struct PipelineDesc<'a> {
    pub shader:       &'a dyn RhiShader,
    pub vertex_layout: VertexLayout,
    pub blend_mode:   BlendMode,   // None / Alpha / Additive
    pub depth_test:   bool,
    pub depth_write:  bool,
    pub cull_mode:    CullMode,    // None / Front / Back
}

// ─── Трейты ──────────────────────────────────────────────────

pub trait RhiDevice: Send + Sync {
    // Создание ресурсов
    fn create_buffer(&self, desc: &BufferDesc) -> Box<dyn RhiBuffer>;
    fn create_texture(&self, desc: &TextureDesc) -> Box<dyn RhiTexture>;
    fn create_shader(&self, desc: &ShaderDesc) -> anyhow::Result<Box<dyn RhiShader>>;
    fn create_pipeline(&self, desc: &PipelineDesc) -> Box<dyn RhiPipeline>;
    fn create_framebuffer(&self, attachments: &[&dyn RhiTexture])
        -> Box<dyn RhiFramebuffer>;

    // Загрузка данных
    fn upload_buffer(&self, buf: &dyn RhiBuffer, offset: usize, data: &[u8]);
    fn upload_texture(&self, tex: &dyn RhiTexture, data: &[u8]);
    fn generate_mipmaps(&self, tex: &dyn RhiTexture);

    // Команды рендера (записываются в CommandBuffer)
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn submit(&mut self, cmds: &CommandBuffer);

    // Утилиты
    fn backend_name(&self) -> &str;  // "OpenGL 3.3" / "Vulkan 1.3" / "D3D12"
    fn capabilities(&self) -> DeviceCaps;
}

pub trait RhiBuffer:   Send + Sync { fn id(&self) -> u64; }
pub trait RhiTexture:  Send + Sync { fn id(&self) -> u64; fn size(&self) -> (u32,u32); }
pub trait RhiShader:   Send + Sync { fn id(&self) -> u64; }
pub trait RhiPipeline: Send + Sync { fn id(&self) -> u64; }
pub trait RhiFramebuffer: Send + Sync {
    fn bind(&self);
    fn unbind(&self);
}
```

### src/rhi/command_buffer.rs

```rust
// CommandBuffer — список команд рендера за один кадр.
// Записывается на CPU, выполняется GPU.
// Это позволит в будущем Vulkan записывать реальные command buffers.

pub enum RenderCommand {
    SetPipeline    { pipeline: PipelineId },
    SetVertexBuffer{ slot: u8, buffer: BufferId, offset: usize },
    SetIndexBuffer { buffer: BufferId, format: IndexFormat },
    SetUniform     { name: &'static str, value: UniformValue },
    SetTexture     { slot: u8, texture: TextureId },
    SetFramebuffer { fb: Option<FramebufferId> },  // None = backbuffer
    SetViewport    { x: i32, y: i32, w: i32, h: i32 },
    SetScissor     { x: i32, y: i32, w: i32, h: i32 },
    ClearColor     { r: f32, g: f32, b: f32, a: f32 },
    ClearDepth     { depth: f32 },
    Draw           { vertices: u32, instances: u32 },
    DrawIndexed    { indices: u32, instances: u32 },
}

pub struct CommandBuffer {
    commands: Vec<RenderCommand>,
}
```

---

## Раздел 8 — OpenGL БЭКЕНД

### src/rhi/opengl/device.rs

```
GlDevice реализует RhiDevice через glow::Context.

Кеши (чтобы не пересоздавать OpenGL объекты):
  shader_cache:   HashMap<u64, NativeProgram>
  pipeline_cache: HashMap<u64, GlPipelineState>
  vao_cache:      HashMap<VertexLayout, NativeVertexArray>

GlPipelineState хранит:
  program:    NativeProgram
  blend:      (GLenum, GLenum)
  depth_test: bool
  depth_write: bool
  cull_face:  Option<GLenum>

submit(cmds) выполняет команды по порядку через glow API:
  SetPipeline    → gl.use_program(), gl.enable/disable(BLEND)...
  SetTexture     → gl.active_texture(), gl.bind_texture()
  Draw           → gl.draw_arrays()
  DrawIndexed    → gl.draw_elements()
```

### Формат вершины (стандартизирован для всего движка)

```rust
// Вершина для terrain и vehicle меша:
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],   // XYZ
    pub normal:   [f32; 3],   // Нормаль
    pub tangent:  [f32; 3],   // Тангент (для normal mapping)
    pub uv:       [f32; 2],   // Текстурные координаты
    pub color:    [f32; 4],   // Цвет вершины (для terrain blend)
}  // 15 × 4 = 60 байт на вершину

// Вершина для UI:
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexUi {
    pub pos:   [f32; 2],
    pub uv:    [f32; 2],
    pub color: [f32; 4],
    pub mode:  f32,   // 0=цвет, 1=текстура, 2=текст
}  // 9 × 4 = 36 байт
```

---

## Раздел 9 — ФИЗИЧЕСКИЙ ДВИЖОК (ПОЛНАЯ АРХИТЕКТУРА С НУЛЯ)

> Это самая большая и сложная часть проекта. Реализуется итеративно.

### 9.1 Математические примитивы физики

```
Используем nalgebra, но создаём обёртки-псевдонимы для удобства:

type Vec3   = nalgebra::Vector3<f32>;
type Vec4   = nalgebra::Vector4<f32>;
type Mat3   = nalgebra::Matrix3<f32>;
type Mat4   = nalgebra::Matrix4<f32>;
type Quat   = nalgebra::UnitQuaternion<f32>;
type Iso3   = nalgebra::Isometry3<f32>;  // позиция + ориентация вместе

// AABB (src/physics/math/aabb.rs):
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}
impl Aabb {
    pub fn center(&self) -> Vec3
    pub fn half_extents(&self) -> Vec3
    pub fn contains(&self, point: Vec3) -> bool
    pub fn intersects(&self, other: &Aabb) -> bool
    pub fn merged(&self, other: &Aabb) -> Aabb
    pub fn surface_area(&self) -> f32   // для BVH cost function
}

// Ray (src/physics/math/ray.rs):
pub struct Ray {
    pub origin:    Vec3,
    pub direction: Vec3,   // единичный вектор
}
impl Ray {
    pub fn at(&self, t: f32) -> Vec3 { self.origin + self.direction * t }
    pub fn intersect_aabb(&self, aabb: &Aabb) -> Option<f32>  // t
    pub fn intersect_plane(&self, normal: Vec3, d: f32) -> Option<f32>
    pub fn intersect_sphere(&self, center: Vec3, radius: f32) -> Option<f32>
    pub fn intersect_triangle(&self, a: Vec3, b: Vec3, c: Vec3) -> Option<f32>
}
```

### 9.2 Широкая фаза: Dynamic BVH

```
Алгоритм: Dynamic AABB Tree (как в Bullet Physics, Box2D)

Структура дерева:
  Каждый узел = AABB + (либо два дочерних, либо один объект)
  Листья = физические объекты
  Внутренние узлы = расширенные AABB (с отступом 0.1м для жирных листьев)

Операции:
  insert(handle, aabb)       — O(log N) — добавить объект
  remove(handle)             — O(log N) — удалить
  update(handle, new_aabb)   — O(log N) — обновить позицию
  query_pairs() → Vec<(A,B)> — O(N log N) — все пересекающиеся пары
  raycast(ray)   → Vec<Hit>  — O(log N)

Балансировка:
  При вставке: Surface Area Heuristic (SAH) для выбора узла
  Не перестраивается полностью — инкрементальные вращения

Реализация в src/physics/broad_phase/bvh.rs:
  BvhTree<T> где T: Copy + Eq + Hash
  Внутренний пул узлов через slotmap::SlotMap
```

### 9.3 Узкая фаза: GJK + EPA + SAT

```
GJK (Gilbert-Johnson-Keerthi) — src/physics/narrow_phase/gjk.rs:
  Определяет: пересекаются ли два выпуклых тела?
  Алгоритм:
    1. Строим Минковскую разность двух фигур
    2. Проверяем, содержит ли она начало координат
    3. Итерационно строим симплекс (точка/линия/треугольник/тетраэдр)
  Результат: bool (пересекаются / нет)

EPA (Expanding Polytope Algorithm) — src/physics/narrow_phase/epa.rs:
  Запускается после GJK если пересечение = true
  Определяет: глубину проникновения и нормаль столкновения
  Нужен для расчёта импульсов

SAT (Separating Axis Theorem) — src/physics/narrow_phase/sat.rs:
  Для box-box быстрее GJK
  15 осей для двух OBB (3+3 ребра + 9 крестов рёбер)
  Результат: нормаль столкновения + глубина

Аналитические случаи (без GJK, быстрее):
  sphere-sphere:   |pos_a - pos_b| < r_a + r_b → O(1)
  sphere-capsule:  расстояние до отрезка < r_sphere + r_capsule
  capsule-capsule: расстояние между двумя отрезками
  ray-heightfield: обход треугольников по сетке вдоль луча

ContactManifold — src/physics/narrow_phase/contact.rs:
  До 4 точек контакта (persistent manifold)
  Каждая точка: позиция, нормаль, глубина, accumulated_impulse
  Persistence: точки переносятся между кадрами (стабильность стека)
```

### 9.4 Формы коллайдеров

```
src/physics/shapes/

Sphere { center: Vec3, radius: f32 }
  Support fn: center + radius * direction

Capsule { a: Vec3, b: Vec3, radius: f32 }
  Два полусфера + цилиндр
  Используется для: персонаж, деревья

BoxShape { half_extents: Vec3 }  // OBB через Iso3
  Support fn: max dot product из 8 вершин
  Используется для: кузов транспорта, ящики

ConvexHull { vertices: Vec<Vec3>, faces: Vec<Face> }
  QuickHull алгоритм для построения из набора точек
  Support fn: max dot product по всем вершинам
  Используется для: капоты, детальные корпуса

TriMesh { triangles: Vec<[Vec3; 3]>, bvh: BvhTree }
  Только для статичных объектов (ландшафт, здания)
  НЕ против TriMesh — только ConvexHull/Sphere/Capsule vs TriMesh
  Это стандарт в играх (Bullet, Havok, PhysX)

HeightField { width: u32, height: u32, data: Vec<f32>, scale: Vec3 }
  Оптимизирован для ландшафта — O(1) определение треугольника по XZ
  Не нужен BVH — напрямую индексируем data[x + z * width]
```

### 9.5 Твёрдое тело (RigidBody)

```rust
// src/physics/dynamics/rigid_body.rs

pub struct RigidBody {
    // Позиция и ориентация
    pub position:    Vec3,
    pub orientation: Quat,

    // Скорости
    pub linear_vel:  Vec3,
    pub angular_vel: Vec3,

    // Массовые свойства
    pub mass:               f32,     // кг
    pub inv_mass:           f32,     // 1/mass (0.0 для статичных)
    pub inertia_tensor:     Mat3,    // в local space
    pub inv_inertia_world:  Mat3,    // в world space (обновляется каждый кадр)

    // Накопленные силы (сбрасываются каждый step)
    pub force:   Vec3,
    pub torque:  Vec3,

    // Флаги и параметры
    pub is_static:      bool,   // true → не двигается (ландшафт, здания)
    pub is_kinematic:   bool,   // true → управляется вручную (движущиеся платформы)
    pub linear_damping: f32,    // 0.01 — сопротивление воздуха
    pub angular_damping: f32,   // 0.01
    pub restitution:    f32,    // упругость: 0.0 = пластик, 1.0 = идеальный мяч
    pub friction:       f32,    // коэффициент трения: 0.3 = лёд, 0.8 = резина

    // Идентификатор в SlotMap
    pub handle: RigidBodyHandle,
}

// Все тела хранятся в:
//   SlotMap<RigidBodyHandle, RigidBody>
// Стабильные ID не инвалидируются при удалении других тел
```

### 9.6 Интегратор (Semi-implicit Euler)

```
src/physics/dynamics/integrator.rs

Почему Semi-implicit Euler, а не RK4?
  RK4 точнее, но дороже (4 оценки на шаг) и НЕ стабильнее для ограничений.
  Semi-implicit Euler: v(t+dt) = v(t) + a*dt, x(t+dt) = x(t) + v(t+dt)*dt
  Он сохраняет энергию (не растёт со временем) — идеален для физики игр.

Шаг интегратора за один PhysicsStep (dt = 1/120 секунды):

fn integrate(body: &mut RigidBody, dt: f32) {
    if body.is_static || body.is_kinematic { return; }

    // 1. Применить гравитацию
    body.force += Vec3::new(0.0, -9.81 * body.mass, 0.0);

    // 2. Вычислить ускорения
    let linear_acc  = body.force  * body.inv_mass;
    let angular_acc = body.inv_inertia_world * body.torque;

    // 3. Обновить скорости
    body.linear_vel  += linear_acc  * dt;
    body.angular_vel += angular_acc * dt;

    // 4. Применить демпфирование
    body.linear_vel  *= (1.0 - body.linear_damping  * dt).max(0.0);
    body.angular_vel *= (1.0 - body.angular_damping * dt).max(0.0);

    // 5. Обновить позиции
    body.position += body.linear_vel * dt;
    let angle = body.angular_vel.magnitude() * dt;
    if angle > 1e-6 {
        let axis = body.angular_vel.normalize();
        body.orientation = UnitQuat::from_axis_angle(&axis, angle)
                         * body.orientation;
    }
    body.orientation.renormalize();  // Предотвращает дрейф кватерниона

    // 6. Обновить inv_inertia_world (зависит от ориентации)
    let r = body.orientation.to_rotation_matrix();
    body.inv_inertia_world = r * body.inertia_tensor.try_inverse().unwrap() * r.transpose();

    // 7. Сбросить силы
    body.force  = Vec3::zeros();
    body.torque = Vec3::zeros();
}

Частота шагов физики:
  PHYSICS_HZ = 120   (120 шагов в секунду, dt = 0.00833с)
  SOLVER_ITERATIONS = 10  (10 итераций решателя ограничений)
  
  В main loop:
    while physics_accumulator >= PHYSICS_DT {
        physics_world.step(PHYSICS_DT);
        physics_accumulator -= PHYSICS_DT;
    }
    let alpha = physics_accumulator / PHYSICS_DT;
    // Интерполяция для рендера (не дёргает при 60fps рендере)
```

### 9.7 Решатель ограничений (Sequential Impulse)

```
src/physics/constraints/solver.rs

Алгоритм Эрина Катто (Erin Catto) — используется в Box2D, Bullet.
Один из лучших для игровой физики реального времени.

Ограничение контакта (ContactConstraint):
  Два тела A и B, точка контакта p, нормаль n, глубина d.
  
  Цель: не дать телам проникать друг в друга.
  
  Скорость разделения: Jv = (vB - vA)·n + (wB×rB - wA×rA)·n
  
  Ламбда (импульс): λ = -Jv / (invMassA + invMassB + rA×n·IA⁻¹·(rA×n) + rB×n·IB⁻¹·(rB×n))
  
  Clamp: λ >= 0 (нельзя тянуть тела — только толкать)
  
  Применяем к телам:
    vA -= λ * invMassA * n
    vB += λ * invMassB * n
    wA -= λ * IA⁻¹ * (rA × n)
    wB += λ * IB⁻¹ * (rB × n)

Трение (FrictionConstraint):
  Две оси трения t1, t2 (перпендикулярны нормали)
  λ_friction clamped по [-μ*λ_normal, +μ*λ_normal]
  μ = sqrt(frictionA * frictionB)  — геометрическое среднее

Позиционная коррекция (Baumgarte stabilization):
  bias = (BAUMGARTE / dt) * max(0, d - SLOP)
  BAUMGARTE = 0.2, SLOP = 0.001м
  Устраняет накопленное проникновение

10 итераций решателя за шаг физики:
  Каждая итерация обходит все контакты и применяет импульсы.
  После 10 итераций система сходится к физически корректному состоянию.
  Больше итераций = точнее, но медленнее.
```

### 9.8 Суставы и соединения

```
src/physics/constraints/joints/

RevoluteJoint (шарнир — ось вращения):
  Используется: колесо на оси, дверь, рычаг
  Ограничивает: все DoF кроме вращения вокруг одной оси
  Опционально: пружина (для подвески), лимиты угла

PrismaticJoint (поступательный — движение вдоль оси):
  Используется: амортизатор (вверх-вниз)
  Ограничивает: все DoF кроме движения вдоль одной оси
  Параметры: min_dist, max_dist, stiffness, damping

BallSocketJoint (шаровой шарнир):
  Используется: карданный вал, шаровая опора
  Ограничивает: только трансляцию (3 DoF вращения свободны)
  
WeldJoint (жёсткая связь):
  Используется: прикреплённый груз, прицеп с сцепкой
  Ограничивает: все 6 DoF (полная жёсткость)
```

### 9.9 Физика колёсного транспорта

```
src/physics/vehicle/ — RAYCAST VEHICLE MODEL

Это стандарт в играх (Bullet, Unity WheelCollider).
Каждое колесо = луч вниз из точки крепления подвески.

─── ПОДВЕСКА (suspension.rs) ───

Параметры на колесо:
  rest_length:   f32,   // длина покоя (например 0.35м для UAZ)
  max_travel:    f32,   // макс. ход (например 0.20м)
  stiffness:     f32,   // жёсткость пружины (25 000 Н/м для UAZ)
  damping:       f32,   // демпфирование (2 500 Н·с/м)
  
Каждый шаг:
  1. Луч из hardpoint вниз (длина = rest_length + max_travel)
  2. Если луч попал в поверхность: compression = rest_length - hit_dist
  3. spring_force   = stiffness * compression
  4. damper_force   = -damping * (prev_compression - compression) / dt
  5. suspension_force = spring_force + damper_force
  6. Применить suspension_force вверх к chassis в точке контакта
  7. Применить -suspension_force вниз к поверхности (земля = статик)

─── ТРЕНИЕ КОЛЁСА (wheel.rs) — Pacejka Magic Formula ───

Упрощённая версия (достаточно для игры):
  slip_ratio      = (wheel_speed - vehicle_speed) / max(vehicle_speed, 0.1)
  slip_angle      = atan2(lateral_vel, longitudinal_vel)
  
  traction_force  = peak_traction * sin(C * atan(B * slip_ratio))
  cornering_force = peak_cornering * sin(C * atan(B * slip_angle))
  
  Коэффициенты для разных поверхностей:
    Асфальт:   peak_traction = 1.0, B=10, C=1.9
    Грязь:     peak_traction = 0.5, B= 5, C=1.4
    Снег:      peak_traction = 0.3, B= 4, C=1.2
    Лёд:       peak_traction = 0.1, B= 3, C=1.0
    Болото:    peak_traction = 0.15, B=3, C=1.1

─── МОДЕЛЬ ДВИГАТЕЛЯ (engine_model.rs) ───

Torque curve (UAZ ЗМЗ-409):
  idle:     900 RPM → 50 Нм
  max torque: 2200 RPM → 225 Нм
  max power:  4600 RPM → 128 л.с. (95 кВт → 205 Нм)
  redline:    5200 RPM

Передаточные числа UAZ Patriot:
  1-я: 4.05, 2-я: 2.34, 3-я: 1.43, 4-я: 1.00, 5-я: 0.82
  Задняя: 3.72
  Раздатка: дорожная 1.14, пониженная 2.48

Логика передач:
  Авто: переключение вверх при RPM > 4800, вниз при < 1200
  Ручная: Space = сцепление, Q/E = переключение

─── ДИФФЕРЕНЦИАЛ (differential.rs) ───

Открытый дифференциал:
  Крутящий момент делится поровну между колёсами оси
  При пробуксовке одного — второе теряет тягу (поведение UAZ по умолчанию)

Заблокированный дифференциал:
  Оба колеса принудительно на одной скорости
  Помогает в грязи, но ухудшает управляемость на асфальте

ЛСД (Limited Slip Differential) — для будущих машин:
  Передаёт момент от буксующего к нагруженному колесу

4WD UAZ Patriot:
  По умолчанию: 2WD (задний привод)
  Включение 4WD: передний мост подключается (жёсткое соединение)
  Пониженная передача: раздаточная коробка

─── ЛЕБЁДКА (winch.rs) ───

  max_force: 4500 кг (44 100 Н)
  rope_length: 0.0 - 20.0 м
  
  Работает как жёсткое SpringJoint с max_force ограничением:
  Если расстояние > rope_length: apply pull force к обоим телам
  Если > max_force: канат рвётся (rope_length = f32::MAX)
```

### 9.10 Физика гусеничного транспорта

```
src/physics/vehicle/tracked_vehicle.rs

Принципиальное отличие от колёсного:
  Управление: НЕ поворотом колёс, а разницей скоростей гусениц
  
Модель:
  Гусеница = массив опорных катков + верхняя ветвь
  Каждый каток = raycast вниз (как колесо в raycast vehicle)
  Но: сила тяги = suммарная тяга всех катков * общий момент / количество катков
  
Поворот:
  Левый бортовой фрикцион: снижает/останавливает правую гусеницу
  Разница скоростей гусениц → момент рыскания вокруг вертикальной оси
  
Разворот на месте:
  Левая гусеница: назад на максимальной тяге
  Правая гусеница: вперёд на максимальной тяге
  
Давление на грунт:
  UAZ Patriot: удельное давление ≈ 0.28 кг/см² (застревает)
  ГАЗ-71:     удельное давление ≈ 0.15 кг/см² (проходит везде)
  
Грунт и давление в soft_ground.rs:
  Если vehicle_pressure > ground_bearing_capacity → погружение в грунт
  Скорость погружения = f(excess_pressure, ground_type)
```

### 9.11 Физика вертолёта

```
src/physics/helicopter/

─── НЕСУЩИЙ РОТОР (rotor.rs) ───

Thrust = ρ * A * k * Ω² * pitch
  ρ     = плотность воздуха (1.225 кг/м³ у земли)
  A     = площадь диска ротора (π * R²)
  k     = коэффициент эффективности лопастей
  Ω     = угловая скорость ротора (рад/с)
  pitch = шаг лопасти (collective pitch)

Реактивный момент (torque reaction):
  Main rotor создаёт момент вокруг вертикальной оси
  Без хвостового ротора → вертолёт вращался бы

Управление:
  Collective pitch (W/S): изменение тяги всех лопастей
  Cyclic pitch (стрелки): наклон тарелки автомата перекоса
    → вертолёт наклоняется → горизонтальная тяга

─── ХВОСТОВОЙ РОТОР (tail_rotor.rs) ───

  Thrust = f(pedal_input, rotor_torque)
  Компенсирует реактивный момент главного ротора
  Педаль влево/вправо → вращение вертолёта

─── ЭФФЕКТ ЗЕМЛИ (ground_effect.rs) ───

  На высоте < R (радиус ротора): тяга увеличивается на 20-40%
  Объяснение: ротор работает в своём же потоке, отражённом от земли
  Реализация: thrust_multiplier = 1.0 + k * exp(-h / R)

─── УПРАВЛЕНИЕ Ми-2 ───

  W     → collective up (увеличить тягу → взлёт)
  S     → collective down (снизить тягу → посадка)
  A/D   → педали хвостового ротора (рыскание)
  ↑/↓   → продольный циклик (вперёд/назад)
  ←/→   → поперечный циклик (крен)
  
  Ограничение: навык piloting < ранг 4 → нельзя взлететь
```

### 9.12 Физика ландшафта

```
src/physics/terrain_physics.rs

HeightField коллайдер:
  Не использует TriMesh для всего ландшафта — слишком медленно.
  
  Вместо этого: при raycast/collision на точку (x, z):
    1. Вычислить индекс ячейки: ix = x / cell_size, iz = z / cell_size
    2. Получить 4 высоты: h00, h10, h01, h11
    3. Билинейная интерполяция → высота точки
    4. Нормаль = cross(dx_direction, dz_direction)
  
  Для collision detection с объектом:
    Взять AABB объекта на XZ
    Получить все ячейки ландшафта в этом AABB
    Для каждой ячейки создать два треугольника
    Проверить узкую фазу
  
  Это O(k²) где k = диаметр объекта / cell_size
  Для UAZ (2м ширина, cell_size 1м): 4-9 ячеек — очень быстро
```

### 9.13 Физика грязи, снега, болота

```
src/physics/soft_ground.rs

SoftGroundType:
  Dry   — сухая земля: нет деформации
  Mud   — грязь: деформация, сопротивление
  Mud_Frozen — мёрзлая грязь: как асфальт
  Snow  — снег: лёгкая деформация, след
  Deep_Snow — глубокий снег: большое сопротивление
  Swamp — болото: очень большое сопротивление, риск застревания
  
Расчёт сопротивления движению:
  Если vehicle_pressure > bearing_capacity[ground_type]:
    depth = f(pressure, ground_type)   // глубина погружения
    drag  = viscosity[ground_type] * contact_area * depth * velocity
    Применяем drag как силу противоположную движению
  
  Для гусеничных машин: bearing_capacity эффективно выше
  из-за большей площади контакта → меньше давления

Визуальные следы (deformation):
  Дополнительная карта деформации (float32 текстура 512×512)
  При проезде: stamp wheel_track в позиции колеса
  Передаётся в terrain shader как дополнительный слой
  Fade со временем (умножение на 0.9999 каждый кадр)
```

### src/physics/world.rs — PhysicsWorld (всё вместе)

```rust
pub struct PhysicsWorld {
    // Объекты
    pub bodies:     SlotMap<RigidBodyHandle, RigidBody>,
    pub colliders:  SlotMap<ColliderHandle,  Collider>,

    // Системы
    broad_phase:  BvhTree<ColliderHandle>,
    solver:       ImpulseSolver,
    integrator:   Integrator,
    
    // Специализированные объекты
    pub vehicles:    Vec<RaycastVehicle>,
    pub helicopters: Vec<HelicopterBody>,
    pub tracked:     Vec<TrackedVehicle>,
    
    // Ландшафт
    terrain:      TerrainCollider,
    soft_ground:  SoftGroundMap,
    
    // Параметры
    gravity:      Vec3,    // Vec3(0, -9.81, 0)
    time_step:    f32,     // 1/120
    accumulator:  f32,
}

impl PhysicsWorld {
    pub fn step(&mut self, dt: f32) {
        self.accumulator += dt;
        while self.accumulator >= self.time_step {
            self.step_internal(self.time_step);
            self.accumulator -= self.time_step;
        }
    }
    
    fn step_internal(&mut self, dt: f32) {
        // 1. Применить внешние силы (гравитация, ветер, двигатели)
        self.apply_forces(dt);
        
        // 2. Обновить подвеску транспорта (до integrate!)
        for v in &mut self.vehicles { v.update_suspension(dt, &self.terrain); }
        
        // 3. Интегрировать скорости и позиции
        for body in self.bodies.values_mut() {
            self.integrator.integrate(body, dt);
        }
        
        // 4. Широкая фаза — обновить BVH, найти пары
        self.broad_phase.update_all(&self.bodies, &self.colliders);
        let pairs = self.broad_phase.query_pairs();
        
        // 5. Узкая фаза — точные контакты
        let contacts = narrow_phase_all(&pairs, &self.bodies, &self.colliders);
        
        // 6. Решатель (10 итераций)
        self.solver.solve(&contacts, &mut self.bodies, dt, 10);
        
        // 7. Коллизия с ландшафтом (отдельно, оптимизировано)
        self.terrain.collide_all(&mut self.bodies, &self.colliders, dt);
    }
}
```

---

## Раздел 10 — СИСТЕМА ЛАНДШАФТА (Terrain)

### src/terrain/

```
Алгоритм LOD: CDLOD (Continuous Distance-based LOD)

Почему CDLOD:
  Нет popping (резких переключений) — плавная морфинг-интерполяция
  Один draw call на чанк (нет сшивок)
  Хорошо работает с shadowmap

Структура ландшафта:
  HeightMap: 4096×4096 пикселей = 4096×4096 метров (4×4 км)
  Cell size: 1м (каждый пиксель = 1м²)
  Max height: 200м
  Формат: R16 (16-bit grayscale = 65536 уровней высоты)

LOD уровни:
  LOD0: 1 вершина / 1м   — только у камеры (до 100м)
  LOD1: 1 вершина / 2м   — 100-300м
  LOD2: 1 вершина / 4м   — 300-700м
  LOD3: 1 вершина / 8м   — 700-2000м
  LOD4: 1 вершина / 16м  — 2000м+ (дальний план)

CDLOD Morph:
  В vertex shader: если вершина попадает в зону morphing (граница LOD)
  Плавно переставляется между позицией LOD и LOD+1
  morph_factor = smoothstep(inner_dist, outer_dist, distance_to_camera)

Сплатмэп (splatmap.rs):
  Текстура RGBA 2048×2048
  R = трава, G = земля, B = камень, A = дорога
  Дополнительный канал: снег (осенью/зимой процедурно)
  В terrain.frag: mix(grass, dirt, splatmap.r) etc.

Коллизия ландшафта:
  Не меш! HeightField коллайдер (см. физику раздел 9.12)

Дорожная сеть (road_network.rs):
  Граф: узлы = перекрёстки, рёбра = участки дороги
  Каждый участок = сплайн Безье 3-го порядка
  При рендере: дорога = вытянутая полоска геометрии вдоль сплайна
  Terrain blend: дорога перекрывает splatmap на своей полосе
  Навигация NPC: A* по графу дорог
```

---

## Раздел 11 — СТРИМИНГ МИРА (World Streaming)

```
src/world/

Размер мира: ~100×100 км (Новосибирский регион)
Активная зона: 4×4 км вокруг игрока (64 terrain chunks)
Чанк: 256×256м = 65536м²

ChunkState:
  Unloaded  — нет в памяти
  Loading   — грузится в фоновом потоке
  Loaded    — в памяти, не на GPU
  Rendered  — на GPU, отрисовывается
  Unloading — выгружается

chunk_streamer.rs:
  Каждые 500мс (не каждый кадр — дорого):
  1. Вычислить, какие чанки должны быть Rendered (в радиусе 4км)
  2. Какие Loaded (в радиусе 6км — буфер)
  3. Запустить фоновую загрузку нужных чанков
  4. Выгрузить слишком далёкие чанки

Фоновая загрузка чанка:
  Thread pool (rayon): читать heightmap секцию
  Создать terrain mesh
  Отправить в главный поток через crossbeam-channel
  Главный поток: загрузить меш на GPU

Дальность прорисовки настраивается (settings.video.render_distance):
  Minimum: 2000м  — для слабых GPU
  Low:     3000м
  Medium:  5000м
  High:    7000м
  Ultra:   10000м
```

---

## Раздел 12 — СИСТЕМА ОСВЕЩЕНИЯ И ТЕНЕЙ

```
src/renderer/game_renderer/

Модель освещения: Blinn-Phong (в OpenGL бэкенде)
При переходе на Vulkan/DX12: PBR (metallic-roughness)

Источники света:
  Солнце/луна: направленный свет (из DayNightCycle)
  АЗС, здания: точечные источники (до 32 одновременно)
  Фары UAZ: прожекторы (spot light)

Shadow Map:
  Один каскад для солнца (пока)
  Разрешение: 2048×2048 (shadow_resolution в настройках)
  Алгоритм: PCF (Percentage Closer Filtering) — мягкие тени
  Обновление: каждый кадр (слежение за камерой)

CASCADED Shadow Map (позже, Фаза 5):
  3 каскада: 0-50м, 50-300м, 300-2000м
  Высокое разрешение вблизи, низкое вдали

Атмосферный туман (fog.rs):
  Exponential fog: c = exp(-density * distance²)
  Цвет тумана = sky_color с учётом времени суток
  Density изменяется от погоды (туман, дождь → выше)

HDR и tonemapping (post/tonemap.frag):
  Рендер в HDR framebuffer (RGBA16F)
  Tonemap: Reinhard или ACES (настройка в settings)
  Exposure: автоматическая или ручная
```

---

## Раздел 13 — АУДИО СИСТЕМА

```
src/audio/  — kira 0.9

AudioManager:
  kira::AudioManager с DefaultBackend (WASAPI на Windows)
  Инициализация в отдельном потоке (kira сам управляет аудио потоком)

3D Пространственный звук:
  Для каждого источника звука: позиция, дистанция затухания
  Формула: volume = max_vol / (1 + dist² / ref_dist²)
  Панорамирование: (pos - listener_pos).normalize() → L/R pan

Слои громкости:
  Master (0.0-1.0) × Music (0.0-1.0) → музыкальный трек
  Master × SFX    → звуковые эффекты
  Master × Ambient → окружение (ветер, птицы, дождь)
  Master × Engine  → звуки двигателя транспорта

Звук двигателя (engine_sounds.toml):
  4 диапазона RPM: idle, low, mid, high
  Каждый — зацикленный .ogg
  Громкость: blend между соседними диапазонами
  Pitch: pitch_shift = rpm / reference_rpm
  Нагрузка: если throttle > 0.5 → pitch += 0.1 (нагруженный двигатель)

Звук поверхности:
  При движении колеса: play(surface_sound[surface_type])
  Громкость = velocity / 30.0 (нормирование до 30 км/ч)

Ревербация (позже):
  В зданиях, тоннелях: добавлять reverb effect через kira
```

---

## Раздел 15 — СКЕЛЕТНАЯ АНИМАЦИЯ

```
src/animation_system/

Формат .rtanim (собственный бинарный):
  Header: имя, количество костей, количество анимаций, FPS
  Skeleton: иерархия костей (parent_index, bind_pose_matrix)
  Clips: список AnimationClip (имя, длительность, ключевые кадры)
  KeyFrame: time, bone_index, translation, rotation, scale

Инструмент конвертации: tools/mesh_converter/
  Читает FBX или GLTF (через библиотеку)
  Экспортирует .rtmesh + .rtanim

Blending:
  Idle → Walk → Run: плавное смешивание по speed
  Animation blend tree: дерево состояний
  CrossFade: плавный переход между клипами за N секунд

В игре:
  OnFoot: Idle / Walk / Run / Jump / Sit (в машине)
  InVehicle: только руки на руле (часть тела)
  Нет full body vehicle animation в альфе (позже)
```

---

# ЧАСТЬ III — ИГРОВЫЕ СИСТЕМЫ

---

## Раздел 16 — ПЕШЕХОДНЫЙ ПЕРСОНАЖ

```
src/gameplay/player/

Player struct:
  rigid_body:   RigidBodyHandle   // капсула радиус 0.35м, высота 1.93м
  state:        PlayerState       // OnFoot / InVehicle(handle)
  height_m:     f32               // из создания персонажа
  skills:       Arc<RwLock<PlayerSkills>>
  wallet:       PlayerWallet
  inventory:    Inventory         // рюкзак 60кг
  
PlayerState::OnFoot:
  Управление:
    WASD / стрелки → apply_force на капсулу (горизонталь)
    Space → прыжок: apply_impulse(Vec3(0, jump_force, 0))
    Shift → бег: speed_multiplier = 2.0
    F → взаимодействие / войти в машину
    Tab → инвентарь
    M → карта
    V → переключение камеры (1st/3rd)
    Escape → пауза
  
  Движение (controller.rs):
    Направление = (камера_forward * W) + (камера_right * D)
    normalize() → * speed * sprint_multiplier
    apply_force() каждый кадр
    На земле: friction damping (резкое торможение)
    В воздухе: малый air_control (0.3x от нормального)
  
  Камера (camera.rs):
    3rd person: offset = (0, 2, -4) в local space игрока
    Вращение ПКМ: добавить к pitch/yaw
    Zoom колесо: изменить distance (1м - 8м)
    Авто-препятствия: SpringArm (камера не проходит сквозь стены)
    
    1st person: позиция = глаза (0, height - 0.15, 0)
    Mouse delta → pitch/yaw напрямую
```

---

## Раздел 17 — СИСТЕМА ТРАНСПОРТА

```
src/gameplay/vehicle/

Vehicle struct (vehicle.rs):
  id:             VehicleId
  vehicle_type:   VehicleType      // Wheeled / Tracked / Helicopter
  physics_handle: RaycastVehicleHandle
  
  // Параметры (из vehicles.toml)
  params:         VehicleParams {
    mass: 2100.0,                  // UAZ кг
    engine_torque_curve: [...],
    max_speed_kmh: 150.0,
    fuel_capacity_l: 68.0,
    wheel_radius_m: 0.41,
    // ...
  }
  
  // Текущее состояние
  fuel_l:         f32
  odometer_km:    f32
  parts:          VehicleParts
  driver:         Option<EntityId>  // None = пустая

VehicleInput (vehicle_input.rs):
  throttle:   f32   // 0..1
  brake:      f32   // 0..1
  steer:      f32   // -1..1
  handbrake:  bool
  gear:       i32   // -1=R, 0=N, 1..5
  
  Из клавиатуры:
    W → throttle = 1.0
    S → brake = 1.0 (если скорость > 0) или throttle = -1.0 (если стоим)
    A/D → steer = ±1.0 (с плавным нарастанием)
    Space → handbrake = true
    
  Из геймпада:
    RT → throttle, LT → brake, LS → steer

Транспортные средства в игре:
  Фаза 0: UAZ Patriot 2017 (основной)
  Фаза 2: КрАЗ-255 (грузовик), КамАЗ-5320 (грузовик)
  Фаза 3: ГАЗ-71 (гусеничный), ДТ-30П Витязь (гусеничный)
           Ми-2 (вертолёт), Ан-2 (самолёт)
  Фаза 4: Кран КС-55, Экскаватор ЭО-4121
```

---

## Раздел 18 — СИСТЕМА ПОВРЕЖДЕНИЙ И РЕМОНТА

```
src/gameplay/vehicle/parts.rs

VehicleParts:
  engine:     ComponentHealth  // 0.0 - 100.0
  gearbox:    ComponentHealth
  transfer:   ComponentHealth  // раздаточная коробка
  frame:      FrameHealth      // особая — permanent damage
  suspension: [ComponentHealth; 4]   // по колесу
  wheels:     [WheelHealth; 4]
  brakes:     [ComponentHealth; 4]
  body:       BodyHealth       // кузов (косметика + аэродинамика)
  electrics:  ComponentHealth  // АКБ, генератор, стартер
  
apply_collision_damage(force: f32, contact_point: Vec3):
  Найти ближайшие к contact_point компоненты
  damage = force * damage_multiplier[component]
  Если frame_damage: frame.max_integrity -= damage * 0.1 (permanent!)

apply_wear(dt: f32, surface: SurfaceType, rpm: f32):
  engine.health   -= dt * wear_rate * f(rpm, engine_temperature)
  tire.tread      -= dt * tire_wear_rate * f(load, surface)
  brakes.health   -= dt * braking_force * brake_wear_rate

Влияние на физику:
  engine.health < 60%  → max_torque *= engine.health / 100.0
  tire.health < 20%    → peak_traction *= 0.4  (почти лысая)
  brakes.health < 30%  → max_brake_force *= 0.4
  suspension < 40%     → damping *= 0.5 (мягкое, раскачка)
  
Ремонт (repair system):
  На СТО: cost = f(damage_level, part_type, skill_mechanics)
  Самостоятельно (в поле):
    Нужно: запчасть в инвентаре + skill_mechanics >= min_rank
    Качество ремонта = f(skill.mastery)
    skill_mechanics >= ранг 5: ремонт до 95%
    < ранг 2: только временная починка (до 60%)
  
  Frame damage: нельзя починить в поле — только полная замена секции на СТО
```

---

## Раздел 19 — ИНВЕНТАРЬ И ГРУЗЫ

```
src/gameplay/inventory/

Grid-based инвентарь (как в Escape from Tarkov, но проще):

Inventory { grid: Vec<Option<ItemId>>, width: u8, height: u8 }

Контейнеры:
  Рюкзак:   10×6 слотов, max 60кг
  Кузов UAZ: 16×8 слотов, max 400кг
  Бардачок:  4×3 слотов, max 5кг
  Склад:     бесконечно (по весу), не переносной
  Сейф:      8×8 слотов, закрывается на ключ
  Ящик:      6×4 слотов, можно погрузить в кузов

Item { type: ItemType, width: u8, height: u8, weight_kg: f32, quantity: u32 }

Предмет размеры (ширина × высота):
  MEDKIT:          1×2
  FUEL_CANISTER:   2×3  (20л = 17кг)
  TIRE_WHEEL:      3×3  (вес: 25кг)
  WRENCH:          1×3
  BRICK_PALLET:    4×3  (200 кг — только в кузов!)
  LUMBER:          2×6  (тяжёлое — только в кузов!)
  DOCUMENTS:       1×1

Rotation: R → повернуть предмет на 90°
Auto-placement: найти первое подходящее место в сетке

Ограничение по весу ОТДЕЛЬНО от сетки:
  Если сетка позволяет, но weight > capacity → запретить размещение
```

---

## Раздел 20 — ЭКОНОМИКА И ТОРГОВЛЯ

```
src/gameplay/economy/

PlayerWallet { rub: f64, cny: f64, usd: f64 }

MarketPrice:
  Каждый населённый пункт имеет свои цены
  base_price + location_modifier + supply_modifier + reputation_modifier
  
  Топливо: 55-65 руб/л (зависит от удалённости от НПЗ)
  Запчасти: рыночные цены ± 20% (репутация влияет)
  Ресурсы: спрос/предложение меняется с игровым временем
  
Зарплата по навыку (wage = base × rank_multiplier):
  Водитель ранг 2: 45 000 руб/мес
  Водитель ранг 6: 120 000 руб/мес
  Пилот ранг 4:   80 000 руб/мес
  Пилот ранг 8:   250 000 руб/мес
  Механик ранг 4: 65 000 руб/мес
  
Налоги (company.rs):
  ИП (УСН 6%): налог с каждой оплаченной работы
  Раз в 30 игровых дней — уведомление об уплате
  Неуплата 3+ раза: штраф × 3, блокировка контрактов
```

---

## Раздел 21 — СИСТЕМА МИССИЙ И КОНТРАКТОВ

```
src/gameplay/missions/

Mission:
  id:         MissionId
  title:      String
  giver:      ContactId      // Серёга, Профессор, etc.
  objectives: Vec<Objective>
  reward:     Reward { rub, xp: HashMap<SkillType, f32>, reputation }
  deadline:   Option<f32>    // игровые часы (None = без дедлайна)
  
Objective типы:
  DeliverCargo { from, to, cargo_type, weight_kg }
  RepairVehicle { vehicle_id, min_health }
  Explore { location, radius }
  Build { structure_type, location }
  Transport { entity_id, to }  // перевезти NPC
  
MissionDispatcher:
  Телефон (как в известной игре, только мы не упоминаем её):
  Через 30 сек после старта: SMS от Серёги
  После выполнения первого → открываются новые контакты
  
Прогресс:
  DeliverCargo: отслеживается по расстоянию до destination
  При въезде в trigger zone (50м от точки) → complete objective
  
Первая миссия (Серёга):
  "Серёга: Привет! Можешь помочь?"
  "Надо перевезти кирпич в Бердск. 800 кг, срочно"
  "Заплачу 18 000 рублей. Договорились?"
  → Маркер на карте: точка загрузки + точка доставки
  → Ограничение скорости: груз хрупкий, > 5g удар = штраф
```

---

## Раздел 22 — NPC

```
src/gameplay/npc/

NPC водители (Фаза 2):
  Типы: КамАЗ с грузом, Газель, Лада (легковая)
  Путь: A* по RoadNetwork от точки А до точки Б
  Поведение: следование пути, остановка на светофорах (в будущем)
  В альфе: не реагируют на игрока (украшение трафика)
  
  Spawn/despawn:
    Спавн: за горизонтом (>800м от игрока)
    Деспавн: за горизонтом (>1200м от игрока) + дорога вне активной зоны
    Всего активных NPC: до 20 машин одновременно

NPC пешеходы (Фаза 2):
  Только в городских зонах (settlement triggers)
  Анимации: ходьба/стояние
  Нет столкновений с игроком в альфе (ghost mode)
  Нет диалогов

NPC работники (Фаза 4):
  Нанимается через интерфейс компании
  Получает задание → едет сам (driver_ai.rs)
  Может поломаться в дороге → SMS игроку "сломался, жди"
  Зарплата списывается раз в игровую неделю
```

---

## Раздел 23 — СТРОИТЕЛЬСТВО БАЗЫ

```
src/gameplay/base_building/

Structure типы:
  FuelBarrel200L  — бочка 200л, стоит сразу
  CarShelter      — навес 3×6м, нужно 20 досок + 5 листов
  StorageContainer — вагончик-склад, нужно привезти
  HeliPad         — вертолётная площадка (Фаза 3)
  
Placement system:
  Нажать B (build mode) → выбор структуры
  Полупрозрачный ghost меш следует за курсором
  Сетка 1×1м для выравнивания
  Зелёный ghost = можно поставить, красный = нельзя (препятствие)
  Нажать ЛКМ → разместить мгновенно (моментальное строительство)
  
  Проверки перед размещением:
    1. Ровная поверхность (max наклон 15°)
    2. Нет пересечения с другими объектами
    3. В инвентаре/кузове есть нужные ресурсы
    4. Списать ресурсы → Structure.placed = true
```

---

## Раздел 24 — ПОГОДА И СЕЗОНЫ

```
src/world/

DayNightCycle:
  4 часа реального времени = 1 игровые сутки
  sun_angle = f(game_time, latitude)  // Новосибирск ≈ 55°N
  Цвет неба: lerp по time_of_day (sunrise_color, day_color, sunset_color, night_color)
  Интенсивность солнца: 0.05 (ночь) → 1.0 (полдень)
  Луна: фаза меняется каждые 29.5 игровых дней

Season:
  Summer (июнь-август): длинный день, сухие дороги
  Autumn (сент-ноябрь): дождь, начало морозов, опадание листьев
  Winter (дек-февраль): снег, короткий день, зимники открыты
  Spring (март-май):   распутица, таяние

WeatherSystem:
  Состояния: Clear, Partly_Cloudy, Overcast, Rain, Heavy_Rain,
             Light_Snow, Heavy_Snow, Blizzard, Fog
  Переходы: марковская цепь с весами по сезону
  Зимой в Новосибирске: Clear/Snow 60%, Overcast 30%, Blizzard 10%
  
Влияние на геймплей:
  Дождь:     mud_wetness += 0.01/с → дороги ухудшаются
  Мороз:     запуск двигателя сложнее (нужен прогрев)
  Метель:    visibility_range снижается до 50м
  Туман:     visibility_range снижается до 200м
  Жара:      engine_temperature растёт быстрее
```

---

## Раздел 25 — СИСТЕМА РЕПУТАЦИИ

```
src/gameplay/

Репутация с каждым Settlement отдельно (-100 до +100):
  Начало: 0 (нейтрал) для всех кроме стартового района (+10)
  
События:
  +5..+20  Выполнить контракт вовремя
  +3       Доставить без повреждений
  +10      Срочная доставка
  -10      Провал контракта
  -1       Нарушение ПДД (ГИБДД поймал)
  -20      Авария с NPC транспортом
  
Влияние:
  rep > 50:  скидка 10% в магазинах, доступны элитные контракты
  rep > 75:  скидка 20%, кредит в местном банке
  rep < -25: некоторые магазины не обслуживают
  rep < -50: ГИБДД останавливает чаще
```

---

# ЧАСТЬ IV — МУЛЬТИПЛЕЕР

---

## Раздел 27 — СЕТЕВАЯ АРХИТЕКТУРА

```
src/network/ — Фаза 4

Транспорт: QUIC через quinn 0.11
  QUIC = UDP + надёжность + шифрование + multiplexing
  Лучше чем сырой UDP (не нужно самому делать reliability)
  Лучше чем TCP (нет head-of-line blocking для физики)

Топология: P2P с выделенным хостом
  Один игрок = Host (сервер физики, авторитет)
  Остальные = Clients
  P2P (без дедик сервера): NAT traversal через STUN

Частота пакетов:
  Критичное (позиции): 20 пакетов/сек (каждые 50мс)
  Менее критичное (инвентарь): 5 пакетов/сек (каждые 200мс)
  Редкие события (миссии): по событию

Пакеты (protocol.rs):
  PlayerInput      { tick, throttle, steer, brake, buttons }
  EntityState      { id, position, velocity, orientation }
  VehicleState     { id, rpm, gear, wheel_states }
  GameEvent        { type: Delivery/Repair/Build/Chat, data }
  WeatherSync      { state, wind_dir, time_of_day }

Что синхронизируется:
  ✓ Позиции и скорости всех игроков и транспорта
  ✓ Физические объекты (груз, пропсы)
  ✓ Trigger-события (заправка, доставка)
  ✓ Погода и время суток
  ✗ NPC (каждый клиент считает локально, хост авторитет)
  ✗ Детали повреждений (экономим трафик)
```

---

# ЧАСТЬ V — КОНТЕНТ И ИНСТРУМЕНТЫ

---

## Раздел 29 — КОНТЕНТ-ПАЙПЛАЙН

```
tools/mesh_converter/ — конвертер моделей

Поддерживаемые форматы входа:
  .obj (Wavefront OBJ) — простой, без анимации
  .gltf / .glb         — современный, с анимацией

Формат .rtmesh (собственный бинарный):
  Magic: "RTMS" (4 байта)
  Version: u32
  Vertex count: u32
  Index count: u32
  Vertices: [Vertex3D; count]
  Indices:  [u32; count]
  Submeshes: список (material_id, index_start, index_count)
  BoundingBox: Aabb
  CollisionMesh: опционально (упрощённый меш)

Конвертация текстур:
  PNG → хранится как PNG (в разработке)
  PNG → DDS (BC7 сжатие) — для релиза (текстуры меньше в 4-8 раз)
  Инструмент: texconv от Microsoft (запускается из пайплайна)

tools/heightmap_editor/:
  Загрузить реальные данные высот SRTM для Новосибирска
  Источник: https://srtm.csi.cgiar.org/  (открытые данные, 90м разрешение)
  Upscale с помощью билинейной интерполяции до 1м/пиксель
  Ручное редактирование: поднять дороги, сгладить въезды в город
  Экспорт в .r16
```

---

## Раздел 30 — БУДУЩИЕ БЭКЕНДЫ (DX11/DX12/Vulkan)

```
Благодаря RHI абстракции (раздел 7):
  Весь игровой код не меняется.
  Меняется только src/rhi/opengl/ → src/rhi/dx12/ или src/rhi/vulkan/

DX11 бэкенд (проще Vulkan, хорошая поддержка Windows):
  d3d11 crate или через windows-rs
  Реализует те же трейты: RhiDevice, RhiBuffer, etc.
  CommandBuffer → ID3D11DeviceContext calls
  
DX12 бэкенд (максимальная производительность на Windows):
  d3d12 crate
  Существенно сложнее: явное управление памятью, barriery, descriptor heaps
  Многопоточная запись command buffers

Vulkan бэкенд (кроссплатформенный):
  ash crate (Rust биндинги)
  Самый сложный: explicit sync, renderpass, pipeline cache
  Но: лучший контроль над GPU

Когда переходить:
  Перейти к DX12/Vulkan только после того как:
  ✓ Вся игровая логика работает
  ✓ OpenGL бэкенд стабилен
  ✓ Шейдеры написаны и отлажены
  Тогда: написать транслятор GLSL → SPIRV → HLSL через spirv-cross
```

---

## Раздел 31 — СИСТЕМА МОДИФИКАЦИЙ (Фаза 6)

```
Мод = папка в %APPDATA%\RTGC\mods\<mod_name>\

Структура мода:
  mod.toml       — метаданные (name, version, author, rtgc_version)
  assets/        — заменяют или добавляют к основным assets
  data/          — дополнительные .toml файлы
  scripts/       — будущее (Lua или Rhai скрипты)

Типы модов:
  Texture pack:  assets/textures/ → перекрывают оригинальные
  Vehicle mod:   data/vehicles.toml → новый транспорт
  Map mod:       data/terrain/ → новая карта
  Mission mod:   data/missions/ → новые контракты

Загрузка модов:
  При старте: сканировать mods/ папку
  Порядок загрузки: алфавитный (переопределяют друг друга)
  mod.toml проверяется на совместимость с версией игры
```

---

# ЧАСТЬ VI — ROADMAP

---

## Раздел 32 — ПОЛНЫЙ ПЛАН ПО ФАЗАМ

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ФАЗА 0 — ДВИЖОК: ФУНДАМЕНТ (≈ 2-3 месяца)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ф0-RHI:   RHI абстракция + OpenGL бэкенд
  [ ] RhiDevice, RhiBuffer, RhiTexture трейты
  [ ] GlDevice реализация (glow)
  [ ] CommandBuffer
  [ ] Компиляция шейдеров из файлов
  [ ] Вертекс буфер + draw call
  ТЕСТ: нарисовать цветной треугольник

Ф0-UI:    UI рендерер
  [ ] DrawBatch (накопление команд)
  [ ] rect.vert/frag с SDF скруглёнными углами
  [ ] Загрузка PNG текстур
  [ ] fontdue интеграция: TTF → GPU атлас
  [ ] Text рендеринг (кириллица)
  ТЕСТ: кнопка с текстом, hover работает

Ф0-WIN:   Платформенный слой
  [ ] winit окно + glutin контекст
  [ ] InputState (клавиши, мышь)
  [ ] AppPaths (%APPDATA%\RTGC)
  [ ] Settings загрузка/сохранение (toml)
  [ ] FrameTimer + delta time
  ТЕСТ: окно открылось, мышь отслеживается

Ф0-MENU:  Меню и загрузочный экран
  [ ] AppState машина
  [ ] SplashScreen (2 секунды)
  [ ] MainMenuScreen (кнопки с анимацией)
  [ ] SettingsScreen (видео/аудио/управление)
  [ ] CharacterCreationScreen (10 шагов)
  [ ] LoadingScreen (11 стадий, поток)
  [ ] Аудио (kira): музыка + SFX кнопок
  [ ] Tweening (анимации)
  ТЕСТ: полный путь от запуска до начала игры

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ФАЗА 1 — ДВИЖОК: ФИЗИКА И РЕНДЕР (≈ 3-4 месяца)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ф1-PHYS:  Базовая физика
  [ ] AABB, Ray математика
  [ ] Dynamic BVH (широкая фаза)
  [ ] GJK + EPA (узкая фаза)
  [ ] SAT для box-box
  [ ] Sphere, Capsule, Box формы
  [ ] RigidBody struct
  [ ] Semi-implicit Euler интегратор
  [ ] Sequential Impulse решатель (10 итераций)
  [ ] Трение и упругость
  ТЕСТ: кубы падают, стакиваются без проникновения

Ф1-TERR:  Ландшафт
  [ ] Загрузка .r16 heightmap
  [ ] Terrain mesh генерация (flat LOD для начала)
  [ ] Splatmap текстурирование (5 текстур)
  [ ] terrain.vert/frag шейдеры
  [ ] HeightField коллайдер
  ТЕСТ: ландшафт виден, объект на нём стоит физически

Ф1-RENDER: Базовый рендер сцены
  [ ] Vertex3D формат
  [ ] RenderGraph: shadow pass → opaque pass → UI pass
  [ ] Skybox рендер
  [ ] Направленный свет (солнце)
  [ ] Shadow map (2048×2048, PCF)
  [ ] Blinn-Phong освещение
  [ ] DayNightCycle (цвет неба + sun_angle)
  ТЕСТ: ландшафт с тенями, день/ночь меняются

Ф1-STREAM: Стриминг мира
  [ ] Chunk система (256×256м)
  [ ] Фоновая загрузка чанков
  [ ] LOD переключение (5 уровней)
  [ ] CDLOD морфинг вершин
  ТЕСТ: езда по ландшафту без фризов

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ФАЗА 2 — ПЕРСОНАЖ И UAZ (≈ 2-3 месяца)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ф2-CHAR:  Пешеходный персонаж
  [ ] Capsule RigidBody для игрока
  [ ] WASD движение с apply_force
  [ ] Прыжок (Space)
  [ ] Бег (Shift)
  [ ] 3rd person камера + SpringArm
  [ ] 1st person переключение (V)
  [ ] Скелетная анимация: Idle/Walk/Run
  ТЕСТ: персонаж ходит по ландшафту

Ф2-UAZ:   UAZ Patriot физика
  [ ] Raycast vehicle (4 луча подвески)
  [ ] Suspension (spring-damper)
  [ ] Pacejka трение (упрощённое)
  [ ] Engine model (кривая момента ЗМЗ-409)
  [ ] Differential (открытый)
  [ ] 4WD переключение
  [ ] Вход/выход по F
  [ ] Переключение камеры в/из транспорта
  [ ] Звук двигателя (RPM → pitch + volume)
  ТЕСТ: UAZ едет, поворачивает, тормозит

Ф2-DAMAGE: Система повреждений
  [ ] VehicleParts (все компоненты)
  [ ] apply_collision_damage
  [ ] apply_wear (wear rate)
  [ ] Влияние на физику (тяга, тормоза, подвеска)
  [ ] Диагностика через навык mechanics
  ТЕСТ: после аварии тяга падает

Ф2-MAP:   Новосибирск
  [ ] Загрузка SRTM данных Новосибирска
  [ ] Road network (основные дороги: Бердское, Томское, etc.)
  [ ] Settlements (Новосибирск, Бердск, посёлки)
  [ ] Buildings (box-placeholder с текстурами)
  [ ] АЗС: trigger zone → заправка
  [ ] СТО: trigger zone → ремонт
  ТЕСТ: проехать Новосибирск → Бердск (32 км)

Ф2-HUD:   HUD
  [ ] Спидометр + RPM
  [ ] Компас (полоска 400×24)
  [ ] Топливо
  [ ] Состояние деталей
  [ ] Время суток
  [ ] Мини-карта (базовая)
  ТЕСТ: все элементы HUD видны и обновляются

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ФАЗА 3 — ПЕРВАЯ ИГРАБЕЛЬНАЯ СЕССИЯ (≈ 2 месяца)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ф3-INV:   Инвентарь
  [ ] Grid-based система
  [ ] 7 типов контейнеров
  [ ] UI инвентаря (Tab)
  [ ] Drag-and-drop / rotation предметов
  [ ] Ограничение по весу
  [ ] Погрузка/выгрузка из кузова
  ТЕСТ: взять предмет, положить в кузов, вес ограничен

Ф3-ECON:  Экономика
  [ ] PlayerWallet
  [ ] Магазины (топливо, запчасти)
  [ ] Биржа контрактов (телефон)
  [ ] Первый контракт Серёги
  ТЕСТ: получить контракт, выполнить, получить деньги

Ф3-MISS:  Миссии
  [ ] MissionDispatcher
  [ ] SMS диалог (Серёга)
  [ ] DeliverCargo objective
  [ ] Маркер на карте
  [ ] Reward (RUB + XP навыков)
  ТЕСТ: Новосибирск → Бердск → 18 000 руб

Ф3-SKILL: Навыки
  [ ] PlayerSkills struct (20+ навыков)
  [ ] gain_xp(hours, difficulty)
  [ ] Влияние навыков на геймплей
  [ ] Отображение в UI
  ТЕСТ: проехать 100 км → driving.hours += 100

Ф3-SAVE:  Сохранения
  [ ] Автосохранение каждые 15 мин
  [ ] Ручное сохранение
  [ ] Загрузка сохранения из меню
  ТЕСТ: сохранить, выйти, загрузить → всё на месте

Ф3-WEATHER: Погода и сезоны
  [ ] WeatherSystem (Clear/Rain/Snow/Fog)
  [ ] Визуальные эффекты (частицы дождя/снега)
  [ ] Влияние на тягу (mud_wetness)
  [ ] Season (Summer/Autumn/Winter/Spring)
  [ ] Зимники (декабрь-февраль)
  [ ] Распутица (апрель-май)
  ТЕСТ: пришла зима — зимник открылся

Ф3-BASE:  База
  [ ] Бочка 200л
  [ ] Навес для техники
  [ ] Вагончик-склад
  [ ] Placement system (сетка 1×1м)
  ТЕСТ: поставить бочку, заправиться из неё

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ФАЗА 4 — РАСШИРЕНИЕ ТЕХНИКИ (≈ 3-4 месяца)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ф4-TRACK: Гусеничная техника
  [ ] TrackedVehicle физика
  [ ] ГАЗ-71 (лёгкий болотоход)
  [ ] ДТ-30П Витязь (шарнирный, 2 секции)
  [ ] Удельное давление → проходимость болот
  ТЕСТ: ГАЗ-71 проходит там, где UAZ застревает

Ф4-HELI:  Вертолёт
  [ ] Rotor model (thrust, torque reaction)
  [ ] Tail rotor (anti-torque)
  [ ] Collective + cyclic управление
  [ ] Автопилот (удержание высоты) — опционально
  [ ] Ground effect
  [ ] Ми-2 меш + текстуры
  [ ] HUD вертолёта (altimeter, VSI, rotor RPM)
  ТЕСТ: взлёт, полёт, посадка Ми-2

Ф4-NPC:   NPC транспорт
  [ ] A* по RoadNetwork
  [ ] Спавн/деспавн по дальности
  [ ] 3 типа NPC машин (КамАЗ, Газель, Лада)
  [ ] NPC пешеходы (декорация в городе)
  ТЕСТ: трафик на Бердском шоссе

Ф4-COMPANY: Компания
  [ ] ИП регистрация (ранг business ≥ 3)
  [ ] ООО регистрация (ранг business ≥ 5)
  [ ] Нанять NPC водителя
  [ ] Нанять NPC механика
  [ ] Налоги (6% УСН)
  ТЕСТ: нанять водителя, выдать контракт, он едет сам

Ф4-COOP:  Мультиплеер (2-32 игрока)
  [ ] QUIC транспорт (quinn)
  [ ] P2P хост модель
  [ ] Синхронизация позиций
  [ ] Синхронизация физики транспорта
  [ ] Синхронизация погоды/времени
  [ ] Пассажир в транспорте другого игрока
  ТЕСТ: два игрока видят друг друга

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ФАЗА 5 — БЕТА: ГРАФИКА И КАЧЕСТВО (≈ 2-3 месяца)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ф5-TEX:   Текстуры
  [ ] DDS сжатие (BC7) для всех текстур
  [ ] Normal maps для terrain
  [ ] PBR материалы транспорта (albedo + normal + roughness + metallic)
  [ ] TextureQuality настройки

Ф5-LIGHT:  Улучшенное освещение
  [ ] Cascaded Shadow Maps (3 каскада)
  [ ] Soft shadows (PCF 4×4)
  [ ] Point lights (АЗС, здания)
  [ ] Spot lights (фары UAZ)
  [ ] HDR tonemapping (ACES)
  [ ] Bloom (фары в темноте)
  [ ] SSAO (ambient occlusion)

Ф5-PARTICLES: Частицы
  [ ] Дождь (instanced quads)
  [ ] Снег
  [ ] Пыль из-под колёс (в сухую погоду)
  [ ] Брызги грязи (mud_wetness > 0.5)
  [ ] Дым из трубы (exhaust)
  [ ] Пар от горячего двигателя зимой

Ф5-ANIM:  Анимация
  [ ] .rtanim конвертер (из GLTF)
  [ ] Blend tree (Idle/Walk/Run/Jump)
  [ ] Full body персонаж (не капсула)
  [ ] Анимации транспорта (дворники, открытие дверей)

Ф5-BACKEND: DX11 бэкенд (или Vulkan)
  [ ] Реализовать RhiDevice для DX11
  [ ] Тест: та же сцена через DX11
  [ ] Сравнение производительности OpenGL vs DX11

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ФАЗА 6 — МОДЫ И РАСШИРЕНИЯ (≈ 1-2 месяца)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ф6-MOD:
  [ ] Система загрузки модов (mods/ папка)
  [ ] Документация для мододелов
  [ ] Пример мода (новый транспорт)
  [ ] Texture pack поддержка
  [ ] Скриптинг (Lua или Rhai — исследовать)
```

---

## Раздел 33 — КРИТЕРИИ ГОТОВНОСТИ КАЖДОЙ ФАЗЫ

```
ФАЗА 0 ГОТОВА когда:
  ✓ cargo build --release → нет ошибок, нет предупреждений
  ✓ Запуск → заставка → главное меню за < 2 секунды
  ✓ Создать персонажа → все 10 шагов работают
  ✓ Загрузочный экран проходит все 11 стадий
  ✓ Клик по кнопке → звук воспроизводится
  ✓ Изменить разрешение в настройках → применилось после ОК
  ✓ Нет memory leak (проверить через heaptrack или valgrind в WSL)

ФАЗА 1 ГОТОВА когда:
  ✓ Ландшафт Новосибирского региона отображается с тенями
  ✓ 100 кубов падают и стакиваются без проникновения
  ✓ Raycast против heightmap работает корректно
  ✓ Езда по ландшафту 10 км без фризов (frame time < 16мс)
  ✓ День/ночь меняется плавно
  ✓ CDLOD: нет pop-in при движении

ФАЗА 2 ГОТОВА когда:
  ✓ Персонаж ходит, прыгает, бегает без артефактов физики
  ✓ UAZ заводится, едет, тормозит (физически корректно)
  ✓ Аварию (удар > 5g) → повреждения деталей
  ✓ Проехать Новосибирск → Бердск (32 км)
  ✓ HUD показывает скорость, RPM, топливо
  ✓ 3rd/1st person переключение работает

ФАЗА 3 ГОТОВА когда:
  ✓ Первый контракт Серёги: выполнить полностью
  ✓ Получить деньги, потратить на топливо
  ✓ Построить бочку и навес у стартовой точки
  ✓ Наступила зима → открылся зимник (проверить)
  ✓ Сохранить игру → выйти → загрузить → персонаж на том же месте
  ✓ Это и есть АЛЬФА-ВЕРСИЯ

ФАЗА 4 ГОТОВА когда:
  ✓ ГАЗ-71 проезжает болото, UAZ застревает
  ✓ Ми-2 взлетает, летит, садится
  ✓ Два игрока видят друг друга и едут в колонне
  ✓ NPC транспорт ездит по дорогам
  ✓ Нанять водителя → он выполняет контракт без участия игрока
  ✓ Это и есть PRE-BETA

ФАЗА 5 ГОТОВА когда:
  ✓ Скриншот выглядит как игра, а не прототип
  ✓ DX11 бэкенд показывает ту же сцену
  ✓ 60 FPS в среднем в Новосибирске (GTX 1070 или выше)
  ✓ Normal maps работают на ландшафте и транспорте
  ✓ Это и есть БЕТА
```

---

## Раздел 34 — РИСКИ И КАК ИХ ИЗБЕЖАТЬ

```
РИСК 1: Физика нестабильна (объекты улетают, проникают)
  Причина: неправильно реализован солвер или интегратор
  Решение: начать с unit-тестов каждой формулы
           Тест: один куб на плоскости — должен остановиться за 3 секунды
           Тест: стек из 10 кубов — не должен разваливаться без касания
           Тест: fast-moving объект — нет туннелирования (swept collision)

РИСК 2: Низкая производительность ландшафта
  Причина: слишком много вершин, много draw call'ов
  Решение: CDLOD обязателен с первого дня
           Frustum culling для всех чанков
           Instanced rendering для деревьев
           Профилирование с первого рабочего ландшафта

РИСК 3: Раздувание Cargo.toml (feature creep зависимостей)
  Причина: добавляем крейты "на будущее"
  Решение: добавлять зависимость только когда она НУЖНА СЕЙЧАС
           Раз в месяц: ревью зависимостей

РИСК 4: Рассинхрон физики в мультиплеере
  Причина: f32 недетерминирован (разные CPU, разный порядок операций)
  Решение: Физика только на хосте, клиенты получают готовые позиции
           Это называется "авторитарный сервер" и это стандарт

РИСК 5: Утечки памяти GPU (OpenGL ресурсы)
  Причина: создаём текстуры/буферы, не удаляем
  Решение: RAII обёртки (Drop trait)
           struct GlBuffer { id: NativeBuffer, gl: Arc<glow::Context> }
           impl Drop for GlBuffer { fn drop → gl.delete_buffer(self.id) }

РИСК 6: Burn-out соло/малой команды
  Причина: слишком большой scope
  Решение: Фазы 0-3 = АЛЬФА = минимальная играбельная версия
           Выпустить в ранний доступ после Фазы 3
           Фазы 4-6 разрабатывать с фидбэком от игроков

РИСК 7: Сложность перехода OpenGL → DX12/Vulkan
  Причина: написали код напрямую через glow, без RHI
  Решение: Строго соблюдать RHI архитектуру с первого дня
           Никакого glow:: за пределами src/rhi/opengl/
           Проверять это в code review
```

---

## ИТОГОВЫЙ ТАЙМЛАЙН

```
Команда 2-3 человека, полная занятость:

Фаза 0 (Фундамент): ─────────────── 2-3 месяца
Фаза 1 (Физика + Рендер): ───────── 3-4 месяца
Фаза 2 (Персонаж + UAZ): ────────── 2-3 месяца
Фаза 3 (Первая сессия = АЛЬФА): ─── 2-3 месяца
─────────────────────────────────── ИТОГО ДО АЛЬФЫ: 9-13 месяцев

Фаза 4 (Техника + Коп): ─────────── 3-4 месяца
Фаза 5 (Бета = графика): ────────── 2-3 месяца
Фаза 6 (Моды): ──────────────────── 1-2 месяца
─────────────────────────────────── ИТОГО ДО РЕЛИЗА: 15-22 месяца

Оптимистичный сценарий: 15 месяцев
Реалистичный сценарий:  18-20 месяцев
Пессимистичный:         24+ месяца (всегда закладывать +30%)

ПЕРВЫЙ ПРИОРИТЕТ: Фаза 3 (Альфа).
Всё что после — улучшение уже работающей игры.
```

---

*Конец документа. Версия 2.0 — Апрель 2026.*
*Следующий документ: Детальная реализация физического движка (GJK/EPA код)*
