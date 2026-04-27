//! Inventory system - handles player items and cargo
//! 
//! Полная реализация системы инвентаря с:
//! - Подсчётом веса предметов
//! - Ограничениями по слотам и максимальному весу
//! - Объединением стаков одинаковых предметов
//! - Корректным пересчётом веса при удалении части стака

pub const MAX_INVENTORY_SLOTS: usize = 20;
pub const MAX_INVENTORY_WEIGHT: f32 = 500.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Wood,
    Stone,
    Iron,
    Fuel,
    Food,
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub quantity: u32,
    pub weight: f32,
    pub item_type: ItemType,
}

impl InventoryItem {
    pub fn new(name: &str, quantity: u32, item_type: ItemType) -> Self {
        let weight = match &item_type {
            ItemType::Resource(rt) => match rt {
                ResourceType::Wood => 1.0,
                ResourceType::Stone => 2.0,
                ResourceType::Iron => 3.0,
                ResourceType::Fuel => 0.8,
                ResourceType::Food => 0.5,
            },
            ItemType::Tool => 2.0,
            ItemType::Vehicle => 100.0,
            ItemType::Cargo => 10.0,
        };
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            quantity,
            weight: weight * quantity as f32,
            item_type,
        }
    }

    pub fn get_weight(&self) -> f32 {
        self.weight
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Resource(ResourceType),
    Tool,
    Vehicle,
    Cargo,
}

#[derive(Debug, Clone)]
pub struct InventorySlot {
    pub item: Option<InventoryItem>,
    pub index: usize,
}

impl InventorySlot {
    pub fn new(index: usize) -> Self {
        Self {
            item: None,
            index,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.item.is_none()
    }

    pub fn set_item(&mut self, item: InventoryItem) -> bool {
        if self.item.is_some() {
            return false;
        }
        self.item = Some(item);
        true
    }

    pub fn clear(&mut self) -> Option<InventoryItem> {
        self.item.take()
    }
}

pub struct Inventory {
    slots: Vec<Option<InventoryItem>>,
    max_weight: f32,
    max_slots: usize,
}

impl Inventory {
    pub fn new(max_slots: usize, max_weight: f32) -> Self {
        let mut slots = Vec::with_capacity(max_slots);
        for i in 0..max_slots {
            slots.push(None);
        }
        Self {
            slots,
            max_weight,
            max_slots,
        }
    }

    pub fn add_item(&mut self, item: InventoryItem) -> Result<(), String> {
        let current_weight = self.get_total_weight();
        if current_weight + item.get_weight() > self.max_weight {
            return Err("Too heavy".to_string());
        }

        for slot in &mut self.slots {
            if let Some(existing) = slot {
                if existing.name == item.name {
                    existing.quantity += item.quantity;
                    existing.weight = existing.weight + item.get_weight();
                    return Ok(());
                }
            }
        }

        for slot in &mut self.slots {
            if slot.is_none() {
                *slot = Some(item);
                return Ok(());
            }
        }

        Err("No space".to_string())
    }

    pub fn remove_item(&mut self, id: &str, quantity: u32) -> Result<InventoryItem, String> {
        for slot in &mut self.slots {
            if let Some(item) = slot {
                if item.id == id {
                    if item.quantity <= quantity {
                        let removed = slot.take().ok_or("Item not found")?;
                        return Ok(removed);
                    } else {
                        // Сохраняем вес до изменения количества
                        let old_weight = item.weight;
                        let old_quantity = item.quantity;
                        
                        // Уменьшаем количество
                        item.quantity -= quantity;
                        
                        // Пересчитываем вес пропорционально
                        let weight_per_item = old_weight / old_quantity as f32;
                        item.weight = item.quantity as f32 * weight_per_item;
                        
                        // Возвращаем удалённый предмет с правильным весом
                        let removed_item = InventoryItem::new(&item.name, quantity, item.item_type.clone());
                        return Ok(removed_item);
                    }
                }
            }
        }
        Err("Item not found".to_string())
    }

    pub fn get_total_weight(&self) -> f32 {
        self.slots.iter()
            .filter_map(|s| s.as_ref())
            .map(|item| item.get_weight())
            .sum()
    }

    pub fn get_item(&self, id: &str) -> Option<&InventoryItem> {
        self.slots.iter()
            .filter_map(|s| s.as_ref())
            .find(|item| item.id == id)
    }

    pub fn get_slots(&self) -> &[Option<InventoryItem>] {
        &self.slots
    }

    pub fn get_slots_mut(&mut self) -> &mut Vec<Option<InventoryItem>> {
        &mut self.slots
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new(MAX_INVENTORY_SLOTS, MAX_INVENTORY_WEIGHT)
    }
}