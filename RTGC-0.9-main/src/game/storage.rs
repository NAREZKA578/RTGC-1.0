//! Storage System for RTGC-0.8
//! Handles inventory storage with grid-based item placement (Tarkov-style)
//! Note: No game references in public-facing content

use std::collections::HashMap;

/// Maximum storage grid size (width x height in slots)
pub const MAX_STORAGE_WIDTH: usize = 10;
pub const MAX_STORAGE_HEIGHT: usize = 8;
pub const MAX_STORAGE_SLOTS: usize = MAX_STORAGE_WIDTH * MAX_STORAGE_HEIGHT;

/// Item dimensions in grid slots
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemDimensions {
    pub width: u8,  // slots wide
    pub height: u8, // slots tall
}

impl ItemDimensions {
    pub const fn new(width: u8, height: u8) -> Self {
        Self { width, height }
    }

    /// Total slots occupied
    pub fn total_slots(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    /// Check if fits in given space
    pub fn fits_in(&self, width: usize, height: usize) -> bool {
        self.width as usize <= width && self.height as usize <= height
    }
}

/// Common item dimensions presets
impl ItemDimensions {
    pub const EMPTY: Self = Self::new(0, 0);
    pub const AMMO_BOX_SMALL: Self = Self::new(1, 1); // Pistol ammo
    pub const AMMO_BOX_LARGE: Self = Self::new(2, 1); // Rifle ammo
    pub const MEDKIT: Self = Self::new(1, 2);
    pub const TOOL_KIT: Self = Self::new(2, 2);
    pub const FUEL_CANISTER: Self = Self::new(2, 3);
    pub const SPARK_PLUG: Self = Self::new(1, 1);
    pub const BRAKE_PADS: Self = Self::new(2, 1);
    pub const TIRE: Self = Self::new(3, 3);
    pub const ENGINE_PART: Self = Self::new(3, 2);
    pub const DOCUMENTS: Self = Self::new(2, 2);
    pub const FOOD_RATION: Self = Self::new(1, 1);
    pub const WATER_BOTTLE: Self = Self::new(1, 2);
    pub const BACKPACK_SMALL: Self = Self::new(2, 3);
    pub const BACKPACK_LARGE: Self = Self::new(3, 4);
    pub const WEAPON_CASE: Self = Self::new(4, 2);
    pub const CEMENT_BAG: Self = Self::new(2, 2); // 50kg bag
    pub const BRICK_STACK: Self = Self::new(2, 2); // Stack of bricks
    pub const WOOD_PLANK: Self = Self::new(4, 1);
    pub const METAL_SHEET: Self = Self::new(4, 2);
}

/// Storage slot in the grid
#[derive(Debug, Clone)]
pub struct StorageSlot {
    pub x: u8,
    pub y: u8,
    pub occupied: bool,
    pub item_id: Option<u64>,
}

impl Default for StorageSlot {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            occupied: false,
            item_id: None,
        }
    }
}

/// Stored item with position and rotation
#[derive(Debug, Clone)]
pub struct StoredItem {
    pub id: u64,
    pub name: String,
    pub item_type: String,
    pub dimensions: ItemDimensions,
    pub weight_kg: f32,
    pub x: u8,         // Grid position X
    pub y: u8,         // Grid position Y
    pub rotated: bool, // 90° rotation
    pub stack_size: u32,
    pub max_stack: u32,
    pub metadata: HashMap<String, String>,
}

impl StoredItem {
    pub fn new(
        id: u64,
        name: &str,
        item_type: &str,
        dimensions: ItemDimensions,
        weight_kg: f32,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            item_type: item_type.to_string(),
            dimensions,
            weight_kg,
            x: 0,
            y: 0,
            rotated: false,
            stack_size: 1,
            max_stack: 1,
            metadata: HashMap::new(),
        }
    }

    /// Get effective dimensions (considering rotation)
    pub fn effective_dimensions(&self) -> ItemDimensions {
        if self.rotated {
            ItemDimensions::new(self.dimensions.height, self.dimensions.width)
        } else {
            self.dimensions
        }
    }

    /// Check if position collides with another item
    pub fn collides_with(&self, other: &StoredItem) -> bool {
        let my_dims = self.effective_dimensions();
        let other_dims = other.effective_dimensions();

        // Check rectangle overlap
        !(self.x as i32 + my_dims.width as i32 <= other.x as i32
            || other.x as i32 + other_dims.width as i32 <= self.x as i32
            || self.y as i32 + my_dims.height as i32 <= other.y as i32
            || other.y as i32 + other_dims.height as i32 <= self.y as i32)
    }

    /// Get all occupied slots
    pub fn occupied_slots(&self) -> Vec<(u8, u8)> {
        let dims = self.effective_dimensions();
        let mut slots = Vec::new();

        for dx in 0..dims.width {
            for dy in 0..dims.height {
                slots.push((self.x + dx, self.y + dy));
            }
        }

        slots
    }
}

/// Storage container (backpack, stash, vehicle trunk, etc.)
#[derive(Debug, Clone)]
pub struct StorageContainer {
    pub id: u64,
    pub name: String,
    pub container_type: ContainerType,
    pub grid_width: usize,
    pub grid_height: usize,
    pub max_weight_kg: f32,
    pub current_weight_kg: f32,
    pub items: Vec<StoredItem>,
    pub grid: Vec<Vec<bool>>, // true = occupied
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerType {
    Backpack,
    Stash,
    VehicleTrunk,
    VehicleGlovebox,
    BuildingStorage,
    Safe,
    Crate,
}

impl ContainerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContainerType::Backpack => "Рюкзак",
            ContainerType::Stash => "Схрон",
            ContainerType::VehicleTrunk => "Багажник",
            ContainerType::VehicleGlovebox => "Бардачок",
            ContainerType::BuildingStorage => "Склад",
            ContainerType::Safe => "Сейф",
            ContainerType::Crate => "Ящик",
        }
    }
}

impl StorageContainer {
    pub fn new(
        id: u64,
        name: &str,
        container_type: ContainerType,
        grid_width: usize,
        grid_height: usize,
        max_weight_kg: f32,
    ) -> Self {
        let grid = vec![vec![false; grid_height]; grid_width];

        Self {
            id,
            name: name.to_string(),
            container_type,
            grid_width,
            grid_height,
            max_weight_kg,
            current_weight_kg: 0.0,
            items: Vec::new(),
            grid,
        }
    }

    /// Rebuild grid from items
    fn rebuild_grid(&mut self) {
        self.grid = vec![vec![false; self.grid_height]; self.grid_width];

        for item in &self.items {
            let slots = item.occupied_slots();
            for (x, y) in slots {
                if (x as usize) < self.grid_width && (y as usize) < self.grid_height {
                    self.grid[x as usize][y as usize] = true;
                }
            }
        }

        // Recalculate weight
        self.current_weight_kg = self.items.iter().map(|i| i.weight_kg).sum();
    }

    /// Check if item can be placed at position
    pub fn can_place_at(&self, dimensions: ItemDimensions, x: u8, y: u8, rotated: bool) -> bool {
        let (w, h) = if rotated {
            (dimensions.height, dimensions.width)
        } else {
            (dimensions.width, dimensions.height)
        };

        // Check bounds
        if (x as usize) + (w as usize) > self.grid_width
            || (y as usize) + (h as usize) > self.grid_height
        {
            return false;
        }

        // Check collisions
        for dx in 0..w {
            for dy in 0..h {
                if self.grid[(x + dx) as usize][(y + dy) as usize] {
                    return false;
                }
            }
        }

        true
    }

    /// Find valid position for item (auto-placement)
    pub fn find_position_for(&self, dimensions: ItemDimensions) -> Option<(u8, u8, bool)> {
        // Try without rotation first
        for y in 0..self.grid_height as u8 {
            for x in 0..self.grid_width as u8 {
                if self.can_place_at(dimensions, x, y, false) {
                    return Some((x, y, false));
                }
            }
        }

        // Try with rotation
        for y in 0..self.grid_height as u8 {
            for x in 0..self.grid_width as u8 {
                if self.can_place_at(dimensions, x, y, true) {
                    return Some((x, y, true));
                }
            }
        }

        None
    }

    /// Add item to storage (auto-position)
    pub fn add_item(&mut self, mut item: StoredItem) -> Result<(), &'static str> {
        // Check weight
        if self.current_weight_kg + item.weight_kg > self.max_weight_kg {
            return Err("Превышен лимит веса");
        }

        // Find position
        if let Some((x, y, rotated)) = self.find_position_for(item.dimensions) {
            item.x = x;
            item.y = y;
            item.rotated = rotated;
            self.items.push(item);
            self.rebuild_grid();
            Ok(())
        } else {
            Err("Нет места для предмета")
        }
    }

    /// Add item at specific position
    pub fn add_item_at(
        &mut self,
        mut item: StoredItem,
        x: u8,
        y: u8,
        rotated: bool,
    ) -> Result<(), &'static str> {
        if !self.can_place_at(item.dimensions, x, y, rotated) {
            return Err("Невозможно разместить предмет");
        }

        if self.current_weight_kg + item.weight_kg > self.max_weight_kg {
            return Err("Превышен лимит веса");
        }

        item.x = x;
        item.y = y;
        item.rotated = rotated;
        self.items.push(item);
        self.rebuild_grid();
        Ok(())
    }

    /// Remove item by ID
    pub fn remove_item(&mut self, item_id: u64) -> Option<StoredItem> {
        if let Some(pos) = self.items.iter().position(|i| i.id == item_id) {
            let item = self.items.remove(pos);
            self.rebuild_grid();
            Some(item)
        } else {
            None
        }
    }

    /// Get item by ID
    pub fn get_item(&self, item_id: u64) -> Option<&StoredItem> {
        self.items.iter().find(|i| i.id == item_id)
    }

    /// Get mutable item by ID
    pub fn get_item_mut(&mut self, item_id: u64) -> Option<&mut StoredItem> {
        self.items.iter_mut().find(|i| i.id == item_id)
    }

    /// Move item to new position
    pub fn move_item(
        &mut self,
        item_id: u64,
        new_x: u8,
        new_y: u8,
        rotated: bool,
    ) -> Result<(), &'static str> {
        // Get item dimensions and current position first
        let (item_dims, _item_rotated, old_slots) = if let Some(item) = self.get_item(item_id) {
            (item.dimensions, item.rotated, item.occupied_slots())
        } else {
            return Err("Предмет не найден");
        };

        let grid_width = self.grid_width;
        let grid_height = self.grid_height;

        // Temporarily remove item from grid
        for (x, y) in &old_slots {
            if (*x as usize) < grid_width && (*y as usize) < grid_height {
                self.grid[*x as usize][*y as usize] = false;
            }
        }

        // Check new position (excluding self)
        let dims = if rotated {
            ItemDimensions::new(item_dims.height, item_dims.width)
        } else {
            item_dims
        };

        if !self.can_place_at(dims, new_x, new_y, false) {
            // Restore old position
            for (x, y) in &old_slots {
                if (*x as usize) < grid_width && (*y as usize) < grid_height {
                    self.grid[*x as usize][*y as usize] = true;
                }
            }
            return Err("Невозможно переместить предмет");
        }

        // Update item position
        if let Some(item) = self.get_item_mut(item_id) {
            item.x = new_x;
            item.y = new_y;
            item.rotated = rotated;
        }

        self.rebuild_grid();
        Ok(())
    }

    /// Get free slots count
    pub fn free_slots(&self) -> usize {
        let total = self.grid_width * self.grid_height;
        let occupied = self.grid.iter().flatten().filter(|&&v| v).count();
        total - occupied
    }

    /// Get used slots count
    pub fn used_slots(&self) -> usize {
        self.grid.iter().flatten().filter(|&&v| v).count()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get utilization percentage
    pub fn utilization_percent(&self) -> f32 {
        let total = (self.grid_width * self.grid_height) as f32;
        let used = self.used_slots() as f32;
        (used / total) * 100.0
    }

    /// Get weight utilization percentage
    pub fn weight_utilization_percent(&self) -> f32 {
        if self.max_weight_kg <= 0.0 {
            return 0.0;
        }
        (self.current_weight_kg / self.max_weight_kg) * 100.0
    }
}

/// Storage system manager (handles multiple containers)
#[derive(Debug)]
pub struct StorageSystem {
    pub containers: HashMap<u64, StorageContainer>,
    pub next_container_id: u64,
    pub next_item_id: u64,
}

impl Default for StorageSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageSystem {
    pub fn new() -> Self {
        Self {
            containers: HashMap::new(),
            next_container_id: 1,
            next_item_id: 1,
        }
    }

    /// Create new container
    pub fn create_container(
        &mut self,
        name: &str,
        container_type: ContainerType,
        grid_width: usize,
        grid_height: usize,
        max_weight_kg: f32,
    ) -> u64 {
        let id = self.next_container_id;
        self.next_container_id += 1;

        let container = StorageContainer::new(
            id,
            name,
            container_type,
            grid_width,
            grid_height,
            max_weight_kg,
        );

        self.containers.insert(id, container);
        id
    }

    /// Get container by ID
    pub fn get_container(&self, container_id: u64) -> Option<&StorageContainer> {
        self.containers.get(&container_id)
    }

    /// Get mutable container by ID
    pub fn get_container_mut(&mut self, container_id: u64) -> Option<&mut StorageContainer> {
        self.containers.get_mut(&container_id)
    }

    /// Generate new item ID
    pub fn generate_item_id(&mut self) -> u64 {
        let id = self.next_item_id;
        self.next_item_id += 1;
        id
    }

    /// Transfer item between containers
    pub fn transfer_item(
        &mut self,
        item_id: u64,
        from_container: u64,
        to_container: u64,
    ) -> Result<(), &'static str> {
        // Get item from source
        let item = {
            let src = self
                .containers
                .get_mut(&from_container)
                .ok_or("Контейнер-источник не найден")?;
            src.remove_item(item_id)
                .ok_or("Предмет не найден в контейнере")?
        };

        // Clone item before moving for potential error handling
        let item_clone = item.clone();

        // Add to destination
        let dst = self
            .containers
            .get_mut(&to_container)
            .ok_or("Контейнер-назначение не найден")?;

        match dst.add_item(item) {
            Ok(_) => Ok(()),
            Err(e) => {
                // Return item to source on failure
                if let Some(src) = self.containers.get_mut(&from_container) {
                    let mut returned_item = src.items.pop().unwrap_or(item_clone.clone());
                    returned_item.id = item_clone.id;
                    src.items.push(returned_item);
                    src.rebuild_grid();
                }
                Err(e)
            }
        }
    }

    /// Get total items across all containers
    pub fn total_items(&self) -> usize {
        self.containers.values().map(|c| c.items.len()).sum()
    }

    /// Get total weight across all containers
    pub fn total_weight(&self) -> f32 {
        self.containers.values().map(|c| c.current_weight_kg).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_dimensions() {
        let tire = ItemDimensions::TIRE;
        assert_eq!(tire.width, 3);
        assert_eq!(tire.height, 3);
        assert_eq!(tire.total_slots(), 9);
    }

    #[test]
    fn test_storage_container_creation() {
        let container =
            StorageContainer::new(1, "Test Backpack", ContainerType::Backpack, 5, 4, 20.0);

        assert_eq!(container.grid_width, 5);
        assert_eq!(container.grid_height, 4);
        assert_eq!(container.max_weight_kg, 20.0);
        assert!(container.is_empty());
    }

    #[test]
    fn test_add_item_auto_position() {
        let mut container = StorageContainer::new(1, "Test", ContainerType::Backpack, 5, 4, 50.0);

        let item = StoredItem::new(1, "Medkit", "medical", ItemDimensions::MEDKIT, 0.5);

        assert!(container.add_item(item).is_ok());
        assert_eq!(container.items.len(), 1);
        assert_eq!(container.used_slots(), 2); // 1x2 item
    }

    #[test]
    fn test_no_space_for_item() {
        let mut container = StorageContainer::new(1, "Small Box", ContainerType::Crate, 2, 2, 10.0);

        // Add large item
        let big_item = StoredItem::new(1, "Big Box", "misc", ItemDimensions::new(2, 2), 5.0);
        assert!(container.add_item(big_item).is_ok());

        // Try to add another item - should fail
        let small_item = StoredItem::new(2, "Small", "misc", ItemDimensions::new(1, 1), 0.5);
        assert!(container.add_item(small_item).is_err());
    }

    #[test]
    fn test_weight_limit() {
        let mut container =
            StorageContainer::new(1, "Light Box", ContainerType::Crate, 10, 10, 5.0);

        let heavy_item = StoredItem::new(1, "Heavy", "misc", ItemDimensions::new(1, 1), 10.0);
        assert!(container.add_item(heavy_item).is_err()); // Over weight limit
    }

    #[test]
    fn test_item_rotation() {
        let mut container = StorageContainer::new(1, "Test", ContainerType::Backpack, 3, 4, 20.0);

        // Item that only fits when rotated
        let item = StoredItem::new(1, "Long", "misc", ItemDimensions::new(1, 4), 1.0);

        // Without rotation - doesn't fit in 3-wide grid with 4-tall item... wait it does
        // Let's try different scenario
        let mut container2 = StorageContainer::new(2, "Test2", ContainerType::Backpack, 4, 3, 20.0);

        // 4-wide item in 4x3 grid - fits without rotation
        assert!(container2.can_place_at(ItemDimensions::new(4, 1), 0, 0, false));
        // Same item rotated (1x4) - doesn't fit in 3-tall grid
        assert!(!container2.can_place_at(ItemDimensions::new(4, 1), 0, 0, true));
    }

    #[test]
    fn test_storage_system() {
        let mut system = StorageSystem::new();

        let backpack_id = system.create_container("Backpack", ContainerType::Backpack, 5, 4, 20.0);
        let stash_id = system.create_container("Home Stash", ContainerType::Stash, 10, 8, 100.0);

        assert_eq!(system.containers.len(), 2);
        assert!(system.get_container(backpack_id).is_some());
        assert!(system.get_container(stash_id).is_some());
    }

    #[test]
    fn test_collision_detection() {
        let mut item1 = StoredItem::new(1, "Item1", "misc", ItemDimensions::new(2, 2), 1.0);
        item1.x = 0;
        item1.y = 0;

        let mut item2 = StoredItem::new(2, "Item2", "misc", ItemDimensions::new(2, 2), 1.0);
        item2.x = 1; // Overlaps with item1
        item2.y = 1;

        assert!(item1.collides_with(&item2));

        item2.x = 3; // No overlap
        assert!(!item1.collides_with(&item2));
    }
}
