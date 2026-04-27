# Архитектурные решения RTGC-1.0

## 📜 Закон имён файлов

**Правило**: В проекте **ЗАПРЕЩЕНО** использование `mod.rs` для основных модулей. 
Каждый файл должен иметь уникальное, описательное имя.

### Почему?
1. **Читаемость**: В стеке ошибки сразу видно, какой модуль задействован
2. **Навигация**: Легко найти файл по имени в IDE
3. **Уникальность**: Никаких конфликтов при рефакторинге

### Исключения
`mod.rs` допускается **только** для объявления подмодулей внутри папки (core/mod.rs, graphics/mod.rs и т.д.)

## 🏗️ Структура ядра

```
main.rs → EngineHub → {RenderPipeline, PhysicsThread}
                      ↓
              GlRhiBackend (OpenGL)
```

### Поток данных

1. **main.rs** создаёт `EngineHub` и запускает event loop
2. **EngineHub** (engine_hub.rs):
   - Создаёт окно через winit
   - Инициализирует `RenderPipeline`
   - Запускает `PhysicsThread`
   - Обрабатывает события окна
   - Синхронизирует рендер и физику через channels

3. **RenderPipeline** (render_pipeline.rs):
   - Получает окно от EngineHub
   - Создаёт `GlRhiBackend` для OpenGL контекста
   - Управляет render passes (terrain, vehicles, UI)

4. **GlRhiBackend** (gl_rhi_backend.rs):
   - Низкоуровневая OpenGL абстракция
   - Создание контекста (glutin)
   - Управление буферами, шейдерами, текстурами

5. **PhysicsThread** (physics_thread.rs):
   - Отдельный поток для физики
   - Fixed timestep (60 Hz)
   - Коммуникация с EngineHub через crossbeam-channel

## 🔧 Технические решения

### OpenGL вместо Vulkan/DX11/DX12
- **Причина**: Рабочий код сейчас только через glutin/glow
- **Преимущество**: Меньше boilerplate, кроссплатформенность
- **Недостаток**: Нет доступа к современным фичам Vulkan

### Физика в отдельном потоке
- **Причина**: Изоляция тяжёлых вычислений от рендера
- **Механизм**: crossbeam-channel для lock-free коммуникации
- **Риск**: Сложность синхронизации состояния

### Fixed Timestep для физики
- **Значение**: 60 Hz (0.01667s)
- **Причина**: Стабильность симуляции, детерминизм
- **Реализация**: Time accumulator pattern

## 📋 Будущие расширения

### Фаза 1: Terrain
- Добавить heightmap generation в world_manager.rs
- Chunk mesh building в chunk_mesh.rs
- Splatmap rendering в terrain_pass.rs

### Фаза 2: Vehicles
- Rigid body physics в rigid_body.rs
- Wheel vehicle controller в vehicle_controller.rs
- UAZ Patriot model в uaz_patriot.rs

### Фаза 3: Procedural World
- Road network generation в road_generator.rs
- Settlement placement в settlement_spawner.rs
- Russian names generator в russian_names.rs

## ⚠️ Антипаттерны (избегать!)

1. **God File**: Файлы >1000 строк — делить на модули
2. **Circular Dependencies**: Модули не должны зависеть друг от друга циклично
3. **Global State**: Избегать static mut, использовать Arc<Mutex<T>>
4. **Magic Numbers**: Константы выносить в отдельные файлы
5. **Silent Failures**: Все ошибки логировать через tracing

---
*Документ обновляется при добавлении новых архитектурных решений*
