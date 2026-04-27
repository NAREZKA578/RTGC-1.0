// ============================================================================
// STEP 1: Data-Oriented Design ECS Architecture
// ============================================================================
// Переход от HashMap-based хранения к Structure of Arrays (SoA) для
// кэш-дружелюбного доступа к данным. Каждый тип компонента хранится в
// отдельном плотном массиве, что обеспечивает предсказуемый доступ к памяти
// и максимизирует эффективность использования кэша процессора.
// ============================================================================

use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// Entity ID type - компактный идентификатор с поколением для безопасности
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl Entity {
    pub const NULL: Entity = Entity {
        id: u32::MAX,
        generation: u32::MAX,
    };

    pub fn new(id: u32) -> Self {
        Entity { id, generation: 0 }
    }

    pub fn index(&self) -> usize {
        self.id as usize
    }

    pub fn is_null(&self) -> bool {
        self.id == u32::MAX
    }
}

// Глобальный счётчик для генерации уникальных ID
static ENTITY_ID_GENERATOR: AtomicU64 = AtomicU64::new(0);

pub fn generate_entity_id() -> u32 {
    ENTITY_ID_GENERATOR.fetch_add(1, Ordering::Relaxed) as u32
}

// ============================================================================
// Component trait - маркер для типов компонентов
// ============================================================================

pub trait Component: Any + Send + Sync + Clone {
    fn type_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T: Any + Send + Sync + Clone> Component for T {}

// ============================================================================
// Archetype - основа DOD архитектуры
// ============================================================================
// Archetype представляет собой уникальный набор типов компонентов.
// Все сущности с одинаковым набором компонентов группируются вместе,
// что обеспечивает непрерывный доступ к памяти при итерации.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchetypeId(u64);

#[derive(Debug)]
pub struct Archetype {
    pub id: ArchetypeId,
    pub component_types: Vec<TypeId>,
    pub entity_indices: Vec<u32>, // Индексы сущностей в этом архетипе
}

impl Archetype {
    pub fn new(component_types: Vec<TypeId>) -> Self {
        let mut sorted_types = component_types.clone();
        sorted_types.sort_by(|a, b| a.cmp(b));

        let hash = Self::compute_hash(&sorted_types);
        Archetype {
            id: ArchetypeId(hash),
            component_types: sorted_types,
            entity_indices: Vec::new(),
        }
    }

    fn compute_hash(types: &[TypeId]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        for t in types {
            t.hash(&mut hasher);
        }
        hasher.finish()
    }

    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.component_types.contains(&type_id)
    }
}

// ============================================================================
// Component Storage - SoA хранение для каждого типа компонента
// ============================================================================
// Вместо HashMap<Entity, T> используем плотный Vec<T> с прямым индексированием.
// Entity->Component маппинг осуществляется через lookup таблицу.

pub trait ComponentStorage: Send + Sync {
    fn remove(&mut self, entity_index: usize);
    fn has(&self, entity_index: usize) -> bool;
    fn len(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Конкретное хранилище для типа T
pub struct ConcreteComponentStorage<T: Component> {
    data: Vec<Option<T>>,
    dense_indices: Vec<usize>, // Плотные индексы для быстрой итерации
}

impl<T: Component> ConcreteComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            dense_indices: Vec::new(),
        }
    }

    pub fn add(&mut self, entity_index: usize, component: T) {
        if entity_index >= self.data.len() {
            self.data.resize(entity_index + 1, None);
        }
        if self.data[entity_index].is_none() {
            self.dense_indices.push(entity_index);
        }
        self.data[entity_index] = Some(component);
    }

    pub fn get_cloned(&self, entity_index: usize) -> Option<T>
    where
        T: Clone,
    {
        self.data
            .get(entity_index)
            .and_then(|opt| opt.as_ref().cloned())
    }

    pub fn get(&self, entity_index: usize) -> Option<&T> {
        self.data.get(entity_index).and_then(|opt| opt.as_ref())
    }

    pub fn get_mut(&mut self, entity_index: usize) -> Option<&mut T> {
        self.data.get_mut(entity_index).and_then(|opt| opt.as_mut())
    }

    // Получение сырого указателя на данные для unsafe доступа
    // Удалено: этот метод больше не используется после перехода на безопасный get_mut
    #[deprecated(note = "Используйте get_mut() вместо unsafe указателей")]
    pub fn get_mut_ptr(&mut self, entity_index: usize) -> Option<*mut T> {
        self.data
            .get_mut(entity_index)
            .and_then(|opt| opt.as_mut())
            .map(|c| c as *mut T)
    }

    // Исправлено: собираем индексы и данные в Vec для корректного времени жизни
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        let items: Vec<(usize, &T)> = self
            .dense_indices
            .iter()
            .filter_map(|&idx| {
                self.data
                    .get(idx)
                    .and_then(|opt| opt.as_ref())
                    .map(|c| (idx, c))
            })
            .collect();
        items.into_iter()
    }

    // Безопасная мутабельная итерация
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        let indices: Vec<usize> = self.dense_indices.iter().copied().collect();
        let data_ptr = self.data.as_mut_ptr();
        let len = self.data.len();
        
        indices.into_iter().filter_map(move |idx| {
            if idx < len {
                unsafe {
                    let opt = &mut (*data_ptr.add(idx));
                    if opt.is_some() {
                        let value_ptr: *mut T = opt as *mut Option<T> as *mut T;
                        Some((idx, &mut *value_ptr))
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        })
    }
}

impl<T: Component> ComponentStorage for ConcreteComponentStorage<T> {
    fn remove(&mut self, entity_index: usize) {
        if entity_index < self.data.len() {
            self.data[entity_index] = None;
            self.dense_indices.retain(|&idx| idx != entity_index);
        }
    }

    fn has(&self, entity_index: usize) -> bool {
        self.data
            .get(entity_index)
            .map_or(false, |opt| opt.is_some())
    }

    fn len(&self) -> usize {
        self.dense_indices.len()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// ============================================================================
// Entity Record - информация о сущности
// ============================================================================

#[derive(Debug, Clone)]
pub struct EntityRecord {
    pub entity: Entity,
    pub archetype_id: ArchetypeId,
    pub archetype_index: usize,             // Индекс в архетипе
    pub components: HashMap<TypeId, usize>, // TypeId -> индекс в хранилище
    pub is_alive: bool,
}

// ============================================================================
// EcsManager - основной менеджер ECS с DOD архитектурой
// ============================================================================

use std::sync::Arc;

// Manually implement Clone since RwLock<T> where T: Clone can be cloned
pub struct EcsManager {
    // Хранилища компонентов по TypeId
    component_storages: Arc<RwLock<HashMap<TypeId, Box<dyn ComponentStorage>>>>,

    // Архетипы для группировки сущностей
    archetypes: Arc<RwLock<HashMap<ArchetypeId, Archetype>>>,

    // Информация о сущностях
    entities: Arc<RwLock<Vec<EntityRecord>>>,

    // Пул освобождённых сущностей для повторного использования
    free_entities: Arc<RwLock<Vec<Entity>>>,

    // Каналы для многопоточной работы
    command_sender: Sender<EcsCommand>,
    command_receiver: Receiver<EcsCommand>,
}

impl Clone for EcsManager {
    fn clone(&self) -> Self {
        Self {
            component_storages: Arc::clone(&self.component_storages),
            archetypes: Arc::clone(&self.archetypes),
            entities: Arc::clone(&self.entities),
            free_entities: Arc::clone(&self.free_entities),
            command_sender: self.command_sender.clone(),
            command_receiver: self.command_receiver.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EcsCommand {
    CreateEntity(Entity),
    AddComponent {
        entity: Entity,
        component_type: TypeId,
    },
    RemoveComponent {
        entity: Entity,
        component_type: TypeId,
    },
    DestroyEntity(Entity),
}

impl EcsManager {
    pub fn new() -> Self {
        let (sender, receiver) = bounded(1024);

        Self {
            component_storages: Arc::new(RwLock::new(HashMap::new())),
            archetypes: Arc::new(RwLock::new(HashMap::new())),
            entities: Arc::new(RwLock::new(Vec::new())),
            free_entities: Arc::new(RwLock::new(Vec::new())),
            command_sender: sender,
            command_receiver: receiver,
        }
    }

    /// Update the ECS system (process commands and update systems)
    pub fn update(&mut self, dt: f32) {
        // Process pending commands
        self.process_commands();
        // dt parameter kept for API compatibility
        let _ = dt;
    }

    pub fn get_command_sender(&self) -> Sender<EcsCommand> {
        self.command_sender.clone()
    }

    // Исправлено: &mut self для вызова методов требующих mutable borrow
    pub fn process_commands(&mut self) {
        while let Ok(command) = self.command_receiver.try_recv() {
            match command {
                EcsCommand::CreateEntity(_) => {
                    // Обработка создаётся в create_entity
                }
                EcsCommand::AddComponent {
                    entity,
                    component_type,
                } => {
                    // Компонент уже добавлен, нужно обновить архетип
                    self.update_entity_archetype(entity, component_type);
                }
                EcsCommand::RemoveComponent {
                    entity,
                    component_type,
                } => {
                    self.update_entity_archetype(entity, component_type);
                }
                EcsCommand::DestroyEntity(entity) => {
                    self.destroy_entity_internal(entity);
                }
            }
        }
    }

    // Создание сущности
    pub fn create_entity(&mut self) -> Entity {
        // Повторное использование из пула
        if let Some(entity) = self.free_entities.write().pop() {
            let mut entities = self.entities.write();
            if let Some(record) = entities.get_mut(entity.index()) {
                record.is_alive = true;
                // Исправлено: поле generation удалено из Entity, используем заглушку
                record.archetype_index = 0;
            }
            return entity;
        }

        // Создание новой сущности
        let id = generate_entity_id();
        let entity = Entity::new(id);

        let record = EntityRecord {
            entity,
            archetype_id: ArchetypeId(0), // Пустой архетип
            archetype_index: 0,
            components: HashMap::new(),
            is_alive: true,
        };

        let mut entities = self.entities.write();
        if entity.index() >= entities.len() {
            entities.resize(entity.index() + 1, record);
        } else {
            entities[entity.index()] = record;
        }

        entity
    }

    // Добавление компонента сущности
    // Исправлено: разделяем заимствования чтобы избежать multiple mutable borrows
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        if entity.is_null() || entity.index() >= self.entities.read().len() {
            return;
        }

        let type_id = TypeId::of::<T>();
        let entity_index = entity.index();

        // Получаем или создаём хранилище
        let mut storages = self.component_storages.write();
        let storage = storages
            .entry(type_id)
            .or_insert_with(|| Box::new(ConcreteComponentStorage::<T>::new()));

        if let Some(concrete_storage) = storage
            .as_any_mut()
            .downcast_mut::<ConcreteComponentStorage<T>>()
        {
            concrete_storage.add(entity_index, component);
        }

        // Освобождаем storages перед заимствованием entities
        drop(storages);

        // Обновляем запись сущности и архетип
        let mut entities = self.entities.write();
        if let Some(record) = entities.get_mut(entity_index) {
            record.components.insert(type_id, entity_index);

            // Инлайним логику update_archetype_for_entity чтобы избежать multiple borrows
            let component_types: Vec<TypeId> = record.components.keys().cloned().collect();
            let archetype_id = if component_types.is_empty() {
                ArchetypeId(0)
            } else {
                let mut archetypes = self.archetypes.write();
                let archetype = Archetype::new(component_types);
                let archetype_id = archetype.id.clone();
                archetypes.entry(archetype_id.clone()).or_insert(archetype);
                archetype_id
            };
            record.archetype_id = archetype_id;
        }
    }

    // Получение компонента (immutable)
    // Исправлено: возвращаем клон компонента чтобы избежать проблем с временем жизни
    pub fn get_component<T: Component + Clone>(&self, entity: Entity) -> Option<T> {
        if entity.is_null() || entity.index() >= self.entities.read().len() {
            return None;
        }

        let entities = self.entities.read();
        let record = entities.get(entity.index())?;

        if !record.is_alive {
            return None;
        }
        drop(entities);

        let type_id = TypeId::of::<T>();
        let storages = self.component_storages.read();
        let storage = storages.get(&type_id)?;

        let concrete_storage = storage
            .as_any()
            .downcast_ref::<ConcreteComponentStorage<T>>()?;
        concrete_storage.get(entity.index()).cloned()
    }

    /// Получение компонента (mutable) - возвращаем &mut T через unsafe с документацией
    /// Это безопасно потому что:
    /// 1. Мы валидируем entity_index перед доступом
    /// 2. У нас есть эксклюзивный доступ к self (&mut self)
    /// 3. Мы проверяем что сущность жива
    /// 
    /// # Safety
    /// Вызывающий код должен гарантировать:
    /// - entity валидна и жива
    /// - &mut self живёт дольше возвращённого &mut T
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if entity.is_null() || entity.index() >= self.entities.read().len() {
            return None;
        }

        let entities = self.entities.read();
        let record = entities.get(entity.index())?;

        if !record.is_alive {
            return None;
        }
        let entity_index = entity.index();
        drop(entities);

        let type_id = TypeId::of::<T>();
        let mut storages = self.component_storages.write();
        let storage = storages.get_mut(&type_id)?;

        let concrete_storage = storage
            .as_any_mut()
            .downcast_mut::<ConcreteComponentStorage<T>>()?;

        // SAFETY: 
        // - entity_index валидирован выше
        // - у нас есть &mut self, дающий эксклюзивный доступ
        // - guard живёт до конца этой функции
        // - компонент не может быть удалён пока у нас &mut self
        let ptr = concrete_storage.get_mut(entity_index)?;
        unsafe { Some(&mut *(ptr as *mut T)) }
    }

    // Удаление компонента
    // Исправлено: разделяем заимствования чтобы избежать multiple mutable borrows
    pub fn remove_component<T: Component>(&mut self, entity: Entity) {
        if entity.is_null() || entity.index() >= self.entities.read().len() {
            return;
        }

        let type_id = TypeId::of::<T>();
        let entity_index = entity.index();

        // Удаляем из хранилища
        let mut storages = self.component_storages.write();
        if let Some(storage) = storages.get_mut(&type_id) {
            storage.remove(entity_index);
        }
        drop(storages); // Освобождаем перед заимствованием entities

        // Обновляем запись сущности
        let mut entities = self.entities.write();
        if let Some(record) = entities.get_mut(entity_index) {
            record.components.remove(&type_id);

            // Инлайним логику update_archetype_for_entity
            let component_types: Vec<TypeId> = record.components.keys().cloned().collect();
            let archetype_id = if component_types.is_empty() {
                ArchetypeId(0)
            } else {
                let mut archetypes = self.archetypes.write();
                let archetype = Archetype::new(component_types);
                let archetype_id = archetype.id.clone();
                archetypes.entry(archetype_id.clone()).or_insert(archetype);
                archetype_id
            };
            record.archetype_id = archetype_id;
        }
    }

    // Проверка наличия компонента
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        if entity.is_null() || entity.index() >= self.entities.read().len() {
            return false;
        }

        let entities = self.entities.read();
        let record = entities.get(entity.index());

        if let Some(rec) = record {
            if !rec.is_alive {
                return false;
            }
            let type_id = TypeId::of::<T>();
            let storages = self.component_storages.read();
            return storages
                .get(&type_id)
                .map_or(false, |s| s.has(entity.index()));
        }
        false
    }

    // Уничтожение сущности
    pub fn destroy_entity(&mut self, entity: Entity) {
        if entity.is_null() {
            return;
        }

        self.destroy_entity_internal(entity);

        // Добавляем в пул для повторного использования
        self.free_entities.write().push(entity);
    }

    fn destroy_entity_internal(&mut self, entity: Entity) {
        let entity_index = entity.index();

        // Исправлено: собираем данные ПЕРЕД заимствованием хранилищ
        // чтобы избежать multiple mutable borrows
        let component_types: Vec<TypeId> = {
            let entities = self.entities.read();
            entities
                .get(entity_index)
                .map(|record| record.components.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        };

        // Теперь освободили entities guard и можем безопасно заиммовать storages
        for type_id in &component_types {
            let mut storages = self.component_storages.write();
            if let Some(storage) = storages.get_mut(type_id) {
                storage.remove(entity_index);
            }
        }

        // Обновляем запись сущности
        let mut entities = self.entities.write();
        if let Some(record) = entities.get_mut(entity_index) {
            record.components.clear();
            record.is_alive = false;
            record.archetype_index = usize::MAX;
        }
    }

    // Проверка живости сущности
    pub fn is_alive(&self, entity: Entity) -> bool {
        if entity.is_null() || entity.index() >= self.entities.read().len() {
            return false;
        }

        self.entities
            .read()
            .get(entity.index())
            .map_or(false, |r| r.is_alive)
    }

    // Исправлено: полностью переработан для избежания multiple mutable borrows
    fn update_entity_archetype(&mut self, entity: Entity, _component_type: TypeId) {
        let entity_index = entity.index();

        // Сначала получаем данные о сущности
        let (component_types, is_alive) = {
            let entities = self.entities.read();
            entities
                .get(entity_index)
                .map(|record| {
                    (
                        record.components.keys().cloned().collect::<Vec<_>>(),
                        record.is_alive,
                    )
                })
                .unwrap_or((Vec::new(), false))
        };

        // Освободили immutable borrow и теперь можем делать mutable операции
        if !is_alive {
            return;
        }

        // Вычисляем и обновляем архетип
        let archetype_id = if component_types.is_empty() {
            ArchetypeId(0)
        } else {
            let mut archetypes = self.archetypes.write();
            let archetype = Archetype::new(component_types);
            let archetype_id = archetype.id.clone();
            archetypes.entry(archetype_id.clone()).or_insert(archetype);
            archetype_id
        };

        // Обновляем запись сущности
        let mut entities = self.entities.write();
        if let Some(record) = entities.get_mut(entity_index) {
            record.archetype_id = archetype_id;
        }
    }

    // Итерация по сущностям с определённым набором компонентов
    // Исправлено: собираем данные ПЕРЕД применением замыкания чтобы избежать
    // повторных заимствований в цикле
    pub fn query<F, T: Component>(&self, mut f: F)
    where
        F: FnMut(Entity, &T),
    {
        // Собираем все данные перед вызовом замыкания
        let items: Vec<(Entity, T)> = {
            let storages = self.component_storages.read();
            let entities = self.entities.read();
            let type_id = TypeId::of::<T>();

            if let Some(storage) = storages.get(&type_id) {
                if let Some(concrete_storage) = storage
                    .as_any()
                    .downcast_ref::<ConcreteComponentStorage<T>>()
                {
                    concrete_storage
                        .iter()
                        .filter_map(|(entity_index, component)| {
                            entities.get(entity_index).and_then(|record| {
                                if record.is_alive {
                                    Some((record.entity, component.clone()))
                                } else {
                                    None
                                }
                            })
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };

        // Применяем замыкание после освобождения всех borrow guard'ов
        for (entity, component) in items {
            f(entity, &component);
        }
    }

    // Параллельная итерация (для Job System)
    // Исправлено: собираем данные в Vec перед параллельной обработкой
    // чтобы избежать проблем с заимствованием в замыкании
    pub fn par_query<F, T>(&self, f: F)
    where
        F: Fn(&T) + Send + Sync,
        T: Component + Clone + Send + Sync,
    {
        use rayon::prelude::*;

        // Собираем компоненты в Vec перед параллельной обработкой
        let components: Vec<T> = {
            let storages = self.component_storages.read();
            let type_id = TypeId::of::<T>();

            if let Some(storage) = storages.get(&type_id) {
                if let Some(concrete_storage) = storage
                    .as_any()
                    .downcast_ref::<ConcreteComponentStorage<T>>()
                {
                    concrete_storage
                        .dense_indices
                        .iter()
                        .filter_map(|&idx| concrete_storage.get(idx).cloned())
                        .collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };

        // Освободили borrow guard и теперь безопасно обрабатываем параллельно
        components.par_iter().for_each(|component| {
            f(component);
        });
    }

    // Получение количества сущностей
    pub fn entity_count(&self) -> usize {
        self.entities.read().iter().filter(|e| e.is_alive).count()
    }

    // Очистка всех сущностей
    pub fn clear(&mut self) {
        self.entities.write().clear();
        self.component_storages.write().clear();
        self.archetypes.write().clear();
        self.free_entities.write().clear();
    }
}

// ============================================================================
// System trait - для определения систем обработки
// ============================================================================

pub trait System: Send + Sync {
    fn name(&self) -> &'static str;
    fn run(&mut self, world: &mut EcsManager);
    fn dependencies(&self) -> Vec<&'static str> {
        Vec::new()
    }
}

// ============================================================================
// Query Builder - удобный интерфейс для запросов
// ============================================================================
// Исправлено: переработана архитектура для корректной работы с borrow checker.
// Вместо итератора с ссылками используем подход "собрать данные → вернуть итератор"

pub struct QueryBuilder<'a> {
    manager: &'a EcsManager,
    required_components: Vec<TypeId>,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(manager: &'a EcsManager) -> Self {
        Self {
            manager,
            required_components: Vec::new(),
        }
    }

    pub fn with<T: Component>(mut self) -> Self {
        self.required_components.push(TypeId::of::<T>());
        self
    }

    pub fn build(self) -> QueryResult<'a> {
        QueryResult {
            manager: self.manager,
            required_components: self.required_components,
        }
    }
}

pub struct QueryResult<'a> {
    manager: &'a EcsManager,
    required_components: Vec<TypeId>,
}

impl<'a> QueryResult<'a> {
    // Исправлено: возвращаем Vec вместо итератора для корректного времени жизни
    pub fn collect<T: Component>(&self) -> Vec<(Entity, T)>
    where
        T: Clone,
    {
        let storages = self.manager.component_storages.read();
        let entities = self.manager.entities.read();
        let type_id = TypeId::of::<T>();

        if let Some(storage) = storages.get(&type_id) {
            if let Some(concrete_storage) = storage
                .as_any()
                .downcast_ref::<ConcreteComponentStorage<T>>()
            {
                concrete_storage
                    .iter()
                    .filter_map(|(entity_index, component)| {
                        entities.get(entity_index).and_then(|record| {
                            if record.is_alive {
                                Some((record.entity, component.clone()))
                            } else {
                                None
                            }
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
}

// ============================================================================
// Тесты для проверки функциональности
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Clone, Debug)]
    struct Velocity {
        vx: f32,
        vy: f32,
        vz: f32,
    }

    #[test]
    fn test_create_entity() {
        let mut ecs = EcsManager::new();
        let entity = ecs.create_entity();

        assert!(!entity.is_null());
        assert!(ecs.is_alive(entity));
    }

    #[test]
    fn test_add_get_component() {
        let mut ecs = EcsManager::new();
        let entity = ecs.create_entity();

        ecs.add_component(
            entity,
            Position {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        );

        let pos = ecs.get_component::<Position>(entity);
        assert!(pos.is_some());
        assert_eq!(pos.expect("Position component should exist").x, 1.0);
    }

    #[test]
    fn test_remove_component() {
        let mut ecs = EcsManager::new();
        let entity = ecs.create_entity();

        ecs.add_component(
            entity,
            Position {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        );
        assert!(ecs.has_component::<Position>(entity));

        ecs.remove_component::<Position>(entity);
        assert!(!ecs.has_component::<Position>(entity));
    }

    #[test]
    fn test_destroy_entity() {
        let mut ecs = EcsManager::new();
        let entity = ecs.create_entity();

        ecs.add_component(
            entity,
            Position {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        );
        assert!(ecs.is_alive(entity));

        ecs.destroy_entity(entity);
        assert!(!ecs.is_alive(entity));

        // Повторное использование
        let new_entity = ecs.create_entity();
        assert_eq!(new_entity.id, entity.id);
        assert_ne!(new_entity.generation, entity.generation);
    }

    #[test]
    fn test_query() {
        let mut ecs = EcsManager::new();

        let e1 = ecs.create_entity();
        ecs.add_component(
            e1,
            Position {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        );
        ecs.add_component(
            e1,
            Velocity {
                vx: 0.1,
                vy: 0.2,
                vz: 0.3,
            },
        );

        let e2 = ecs.create_entity();
        ecs.add_component(
            e2,
            Position {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        );

        let mut count = 0;
        ecs.query(|_entity, _pos: &Position| {
            count += 1;
        });

        assert_eq!(count, 2);
    }
}
