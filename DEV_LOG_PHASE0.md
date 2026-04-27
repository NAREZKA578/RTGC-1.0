# Dev Log: RTGC-1.0 Phase 0 — Core Creation

## 📅 Дата: Начало разработки RTGC-1.0

## 🎯 Цель
Создать минимальное работающее ядро движка с чистой архитектурой и уникальными именами файлов.

## ✅ Выполненные действия

### 1. Полная очистка репозитория
- Удалены все файлы RTGC-0.9 (кроме LICENSE Apache 2.0)
- Удалены папки `assets/`, `data/` (будут воссозданы позже)
- Сохранена полная структура в `OLD_STRUCTURE_BACKUP.txt` для истории

### 2. Создана новая структура файлов

```
src/
├── main.rs                      # Точка входа
├── lib.rs                       # Экспорт библиотеки
├── core/
│   ├── mod.rs                   # Модуль core
│   └── engine_hub.rs            # Центральный хаб (бывший core_engine.rs)
├── graphics/
│   ├── mod.rs                   # Модуль graphics
│   ├── render_pipeline.rs       # Рендеринг (бывший render.rs)
│   └── rhi/
│       ├── mod.rs               # Модуль rhi
│       └── gl_rhi_backend.rs    # OpenGL RHI (бывший opengl_rhi.rs)
└── physics/
    ├── mod.rs                   # Модуль physics
    └── physics_thread.rs        # Физический поток (бывший thread_f.rs)
```

### 3. Переименование по "Закону имён"
**Правило**: Никаких `mod.rs` для основной логики, только уникальные имена:
- `core_engine.rs` → `engine_hub.rs`
- `render.rs` → `render_pipeline.rs`
- `opengl_rhi.rs` → `gl_rhi_backend.rs`
- `thread_f.rs` → `physics_thread.rs`

### 4. Обновлены все импорты
- `lib.rs`: `pub use core::engine_hub::EngineHub;`
- `main.rs`: `use rtgc::EngineHub;`
- `engine_hub.rs`: Использует `RenderPipeline` и `PhysicsThread`
- `render_pipeline.rs`: Использует `GlRhiBackend`
- `physics_thread.rs`: Использует `PhysicsCommand/PhysicsMessage` из `engine_hub`

### 5. Документация
- `README.md`: Манифест проекта, архитектура, план разработки
- `ARCHITECTURE_DECISIONS.md`: Закон имён, технические решения, антипаттерны
- `OLD_STRUCTURE_BACKUP.txt`: История старой структуры RTGC-0.9

## 🔧 Технические детали

### EngineHub (engine_hub.rs)
- Центральный оркестратор всего движка
- Создаёт окно через winit
- Инициализирует RenderPipeline
- Запускает PhysicsThread в отдельном потоке
- Обрабатывает события окна (CloseRequested, RedrawRequested)
- Коммуникация с физикой через crossbeam-channel

### RenderPipeline (render_pipeline.rs)
- Высокоуровневый рендеринг
- Получает окно от EngineHub
- Создаёт GlRhiBackend для OpenGL контекста
- Метод `render(delta_time)` очищает экран и свопит буферы
- Заготовка для будущих render passes (terrain, vehicles, UI)

### GlRhiBackend (gl_rhi_backend.rs)
- Низкоуровневая OpenGL абстракция
- Создание контекста через glutin + glow
- Методы: `new()`, `swap_buffers()`, `resize()`
- Поддержка OpenGL 4.5+ или GLES 3.0+

### PhysicsThread (physics_thread.rs)
- Отдельный поток для физики
- Fixed timestep 60 Hz (0.01667s)
- Communication channels:
  - `PhysicsCommand::Step(f32)` от EngineHub
  - `PhysicsMessage::StepComplete(f32)` обратно
- Graceful shutdown через AtomicBool

## 📦 Зависимости (Cargo.toml)
```toml
winit = "0.30"           # Window management
glutin = "0.32"          # OpenGL context
glow = "0.14"            # OpenGL bindings
nalgebra = "0.33"        # Math library
crossbeam-channel = "0.5" # Lock-free channels
tracing = "0.1"          # Logging
tracing-subscriber = "0.3" # Logging subscriber
```

## ⚠️ Проблемы и решения

### Проблема 1: mod.rs запрет
**Решение**: Использовать уникальные имена для всех основных файлов, mod.rs только для объявления подмодулей.

### Проблема 2: Циклические зависимости
**Решение**: PhysicsCommand/PhysicsMessage определены в engine_hub.rs, чтобы physics_thread.rs мог их импортировать без циклов.

### Проблема 3: Downcast окна
**Решение**: Использовать `window.as_any().downcast_ref::<winit::window::Window>()` для получения конкретного типа.

## 🎯 Следующие шаги (Фаза 1)

1. **Базовый рендеринг**:
   - Добавить простой fullscreen quad
   - Загрузить шейдеры из файлов
   - Реализовать camera controller

2. **Terrain generation**:
   - Heightmap через шум Перлина
   - Chunk mesh building
   - Splatmap для текстур

3. **Тестирование**:
   - Интеграционный тест: "окно открывается, рендерится кадр"
   - Benchmark: FPS на пустой сцене

## 📊 Метрики
- Файлов кода: 10
- Строк кода: ~600
- Модулей: 3 (core, graphics, physics)
- Уникальных имён: 100% compliance

---
*Статус: Фаза 0 завершена, готово к компиляции и тестированию*
