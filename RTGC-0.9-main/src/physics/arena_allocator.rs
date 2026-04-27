use std::ops::Index;
use std::ops::IndexMut;
use std::vec::Vec;

/// A simple arena allocator for efficient memory management with generation tracking to prevent use-after-free bugs
#[derive(Clone)]
pub struct ArenaAllocator<T: Clone> {
    items: Vec<Option<T>>,
    free_indices: Vec<usize>,
    generations: Vec<u64>, // Generation counter for each slot to detect use-after-free
    count: usize,
}

impl<T: Clone> ArenaAllocator<T> {
    /// Creates a new arena allocator with initial capacity
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            free_indices: Vec::new(),
            generations: Vec::new(),
            count: 0,
        }
    }

    /// Creates a new arena allocator with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            free_indices: Vec::with_capacity(capacity),
            generations: Vec::with_capacity(capacity),
            count: 0,
        }
    }

    /// Allocates a new item in the arena and returns its index
    pub fn allocate(&mut self, item: T) -> usize {
        if let Some(index) = self.free_indices.pop() {
            // Reuse a previously freed slot - increment generation to invalidate old references
            self.generations[index] = self.generations[index].wrapping_add(1);
            self.items[index] = Some(item);
            self.count += 1;
            index
        } else {
            // Add a new slot
            let index = self.items.len();
            self.items.push(Some(item));
            self.generations.push(0);
            self.count += 1;
            index
        }
    }

    /// Deallocates an item by index
    pub fn deallocate(&mut self, index: usize) {
        if index < self.items.len() {
            if self.items[index].is_some() {
                self.items[index] = None;
                self.free_indices.push(index);
                self.count -= 1;
            }
        }
    }

    /// Gets a reference to an item by index with generation check
    pub fn get(&self, index: usize, generation: u64) -> Option<&T> {
        if index < self.items.len() && self.generations.get(index) == Some(&generation) {
            self.items[index].as_ref()
        } else {
            None
        }
    }

    /// Gets a mutable reference to an item by index with generation check
    pub fn get_mut(&mut self, index: usize, generation: u64) -> Option<&mut T> {
        if index < self.items.len() && self.generations.get(index) == Some(&generation) {
            self.items[index].as_mut()
        } else {
            None
        }
    }

    /// Gets a reference without generation check (legacy compatibility)
    pub fn get_unchecked(&self, index: usize) -> Option<&T> {
        if index < self.items.len() {
            self.items[index].as_ref()
        } else {
            None
        }
    }

    /// Gets a mutable reference without generation check (legacy compatibility)
    pub fn get_mut_unchecked(&mut self, index: usize) -> Option<&mut T> {
        if index < self.items.len() {
            self.items[index].as_mut()
        } else {
            None
        }
    }

    /// Gets a reference by index only (backward compatibility alias)
    pub fn get_by_index(&self, index: usize) -> Option<&T> {
        self.get_unchecked(index)
    }

    /// Gets a mutable reference by index only (backward compatibility alias)
    pub fn get_mut_by_index(&mut self, index: usize) -> Option<&mut T> {
        self.get_mut_unchecked(index)
    }

    /// Gets a mutable reference by index (alias for get_mut_by_index)
    pub fn get_by_index_mut(&mut self, index: usize) -> Option<&mut T> {
        self.get_mut_by_index(index)
    }

    /// Checks if an index is valid and allocated
    pub fn is_allocated(&self, index: usize) -> bool {
        if index < self.items.len() {
            self.items[index].is_some()
        } else {
            false
        }
    }

    /// Returns the number of allocated items
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns the generation for a given index
    pub fn get_generation(&self, index: usize) -> Option<u64> {
        self.generations.get(index).copied()
    }

    /// Clears the arena, deallocating all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.free_indices.clear();
        self.generations.clear();
        self.count = 0;
    }

    /// Returns capacity hint for pre-allocation
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    /// Returns an iterator over allocated items
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().filter_map(|opt| opt.as_ref())
    }

    /// Returns a mutable iterator over allocated items
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().filter_map(|opt| opt.as_mut())
    }

    /// Returns a mutable pointer to the underlying data
    pub fn as_mut_ptr(&mut self) -> *mut Option<T> {
        self.items.as_mut_ptr()
    }

    /// Returns a mutable slice of the underlying data
    pub fn as_mut_slice(&mut self) -> &mut [Option<T>] {
        &mut self.items
    }

    /// Splits the arena into two mutable slices at the given index
    pub fn split_at_mut(&mut self, mid: usize) -> (&mut [Option<T>], &mut [Option<T>]) {
        self.items.split_at_mut(mid)
    }

    /// Returns the length of the arena
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the arena is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T: Clone> Default for ArenaAllocator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Index<usize> for ArenaAllocator<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        // SAFETY: This will panic if index is out of bounds or not allocated.
        // Prefer using get() or get_unchecked() for safer access patterns.
        self.items[index]
            .as_ref()
            .expect("Index out of bounds or not allocated")
    }
}

impl<T: Clone> IndexMut<usize> for ArenaAllocator<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // SAFETY: This will panic if index is out of bounds or not allocated.
        // Prefer using get_mut() or get_mut_unchecked() for safer access patterns.
        self.items[index]
            .as_mut()
            .expect("Index out of bounds or not allocated")
    }
}
