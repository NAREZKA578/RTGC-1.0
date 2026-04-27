# RTGC-1.0 — Russian Open World Vehicle Simulator (Core)

## 🎯 Манифест проекта

RTGC-1.0 — это **полная переработка** RTGC-0.9 с фокусом на:
- **Минимализм**: Только работающий код, никаких заглушек
- **Вертикальные срезы**: Каждая фича доводится до playable состояния
- **Чистая архитектура**: Явное разделение слоёв, уникальные имена файлов

## 🏗️ Архитектура (Закон имён)

**ВАЖНО**: В проекте запрещено использование `mod.rs`. Каждый модуль должен иметь уникальное имя:

```
src/
├── main.rs                  # Точка входа
├── lib.rs                   # Экспорт библиотеки
├── core/
│   ├── mod.rs               # Исключение для модулей
│   └── engine_hub.rs        # Центральный хаб (Core)
├── graphics/
│   ├── mod.rs
│   ├── render_pipeline.rs   # Рендеринг
│   └── rhi/
│       ├── mod.rs
│       └── gl_rhi_backend.rs # OpenGL RHI
└── physics/
    ├── mod.rs
    └── physics_thread.rs    # Физический поток
```

## 🔧 Ключевые принципы

1. **engine_hub.rs** — всё соединяется здесь (окно, рендер, физика, input)
2. **render_pipeline.rs** — высокоуровневый рендеринг, RHI подключается сюда
3. **gl_rhi_backend.rs** — низкоуровневая OpenGL абстракция (glow + glutin)
4. **physics_thread.rs** — физика в отдельном потоке, channel коммуникация
5. **main.rs** — чистая точка входа без логики

## 📦 Зависимости

- `winit` — окно и event loop
- `glutin` + `glow` — OpenGL контекст и bindings
- `nalgebra` — математика
- `crossbeam-channel` — межпоточная коммуникация
- `tracing` — логирование

## 🚀 Запуск

```bash
cargo run --release
```

## 📋 План разработки

### Фаза 0 (текущая) — Ядро
- [x] Базовая структура с уникальными именами
- [x] EngineHub как центральный оркестратор
- [x] RenderPipeline + OpenGL RHI
- [x] PhysicsThread с channel коммуникацией
- [ ] Окно открывается, clears экран

### Фаза 1 — Terrain Rendering
- [ ] Heightmap generation
- [ ] Chunk mesh building
- [ ] Splatmap rendering
- [ ] Camera controller

### Фаза 2 — Basic Vehicle
- [ ] Rigid body physics
- [ ] Wheel vehicle controller
- [ ] UAZ Patriot model

### Фаза 3 — Procedural World
- [ ] Road network generation
- [ ] Settlement placement
- [ ] Russian names generator

## 📝 История

Этот репозиторий — результат жёсткого аудита RTGC-0.9. 
Все лишние файлы удалены, архитектура упрощена до минимально работающего ядра.

**Лицензия**: Apache 2.0
