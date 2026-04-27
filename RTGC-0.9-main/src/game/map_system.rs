// Map system for RTGC-0.8
// Handles paper map item, full-screen map view, and custom markers

use serde::{Deserialize, Serialize};

/// Marker type for custom player markers
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MarkerType {
    Player,    // Игрок (синий)
    Objective, // Цель (красный)
    Friend,    // Друг/союзник (зелёный)
    Danger,    // Опасность (оранжевый)
    Custom1,   // Пользовательская 1 (жёлтый)
    Custom2,   // Пользовательская 2 (фиолетовый)
}

impl MarkerType {
    pub fn color(&self) -> [f32; 4] {
        match self {
            MarkerType::Player => [0.0, 0.5, 1.0, 1.0],    // Синий
            MarkerType::Objective => [1.0, 0.0, 0.0, 1.0], // Красный
            MarkerType::Friend => [0.0, 1.0, 0.0, 1.0],    // Зелёный
            MarkerType::Danger => [1.0, 0.5, 0.0, 1.0],    // Оранжевый
            MarkerType::Custom1 => [1.0, 1.0, 0.0, 1.0],   // Жёлтый
            MarkerType::Custom2 => [0.5, 0.0, 1.0, 1.0],   // Фиолетовый
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MarkerType::Player => "Игрок",
            MarkerType::Objective => "Цель",
            MarkerType::Friend => "Друг",
            MarkerType::Danger => "Опасность",
            MarkerType::Custom1 => "Метка 1",
            MarkerType::Custom2 => "Метка 2",
        }
    }
}

/// Custom marker placed by player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMarker {
    pub id: u32,
    pub marker_type: MarkerType,
    pub position: (f32, f32), // World coordinates (x, z)
    pub label: String,
    pub visible_on_compass: bool,
}

impl MapMarker {
    pub fn new(id: u32, marker_type: MarkerType, x: f32, z: f32, label: String) -> Self {
        Self {
            id,
            marker_type,
            position: (x, z),
            label,
            visible_on_compass: true,
        }
    }
}

/// Map system manager
#[derive(Clone)]
pub struct MapSystem {
    /// Is full-screen map open?
    pub map_open: bool,
    /// Is context menu open for paper map?
    pub context_menu_open: bool,
    /// Context menu position (screen coordinates)
    pub context_menu_pos: (f32, f32),
    /// Selected inventory slot for paper map
    pub selected_slot: Option<usize>,
    /// Custom markers (max 4)
    pub markers: Vec<MapMarker>,
    /// Next marker ID
    next_marker_id: u32,
    /// Map zoom level
    pub zoom: f32,
    /// Map offset (panning)
    pub offset: (f32, f32),
}

impl Default for MapSystem {
    fn default() -> Self {
        Self {
            map_open: false,
            context_menu_open: false,
            context_menu_pos: (0.0, 0.0),
            selected_slot: None,
            markers: Vec::new(),
            next_marker_id: 1,
            zoom: 1.0,
            offset: (0.0, 0.0),
        }
    }
}

impl MapSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open full-screen map
    pub fn open_map(&mut self) {
        self.map_open = true;
        self.context_menu_open = false;
    }

    /// Close full-screen map
    pub fn close_map(&mut self) {
        self.map_open = false;
    }

    /// Toggle full-screen map
    pub fn toggle_map(&mut self) {
        if self.map_open {
            self.close_map();
        } else {
            self.open_map();
        }
    }

    /// Open context menu for paper map
    pub fn open_context_menu(&mut self, slot: usize, screen_x: f32, screen_y: f32) {
        self.selected_slot = Some(slot);
        self.context_menu_pos = (screen_x, screen_y);
        self.context_menu_open = true;
    }

    /// Close context menu
    pub fn close_context_menu(&mut self) {
        self.context_menu_open = false;
        self.selected_slot = None;
    }

    /// Add custom marker (max 4)
    pub fn add_marker(
        &mut self,
        marker_type: MarkerType,
        x: f32,
        z: f32,
        label: String,
    ) -> Option<u32> {
        if self.markers.len() >= 4 {
            return None; // Max 4 markers
        }

        let id = self.next_marker_id;
        self.next_marker_id += 1;

        let marker = MapMarker::new(id, marker_type, x, z, label);
        self.markers.push(marker);

        Some(id)
    }

    /// Remove marker by ID
    pub fn remove_marker(&mut self, id: u32) {
        self.markers.retain(|m| m.id != id);
    }

    /// Get markers visible on compass
    pub fn get_compass_markers(&self) -> Vec<&MapMarker> {
        self.markers
            .iter()
            .filter(|m| m.visible_on_compass)
            .collect()
    }

    /// Clear all custom markers
    pub fn clear_markers(&mut self) {
        self.markers.clear();
    }

    /// Update marker position
    pub fn update_marker_position(&mut self, id: u32, x: f32, z: f32) {
        if let Some(marker) = self.markers.iter_mut().find(|m| m.id == id) {
            marker.position = (x, z);
        }
    }

    /// Zoom in
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + 0.1).min(4.0);
    }

    /// Zoom out
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - 0.1).max(0.25);
    }

    /// Reset zoom
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
    }
}
