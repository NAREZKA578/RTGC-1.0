//! Debug Menu - F3 overlay with performance and game state information
//! Displays FPS, CPU/RAM usage, physics stats, player info

use crate::game::{Player, PlayerState};
use crate::physics::PhysicsStats;

/// Debug menu state
#[derive(Clone)]
pub struct DebugMenu {
    /// Is debug menu visible
    pub visible: bool,

    /// Current FPS
    pub fps: f32,

    /// Frame time in milliseconds
    pub frame_time_ms: f32,

    /// Physics statistics
    pub physics_stats: PhysicsStats,

    /// Active rigid body count
    pub active_rigid_bodies: usize,

    /// Active collision pairs
    pub active_collisions: usize,

    /// Active chunks loaded
    pub active_chunks: usize,

    /// Memory usage (MB)
    pub ram_usage_mb: f32,

    /// VRAM usage (MB) - if available
    pub vram_usage_mb: Option<f32>,
}

impl DebugMenu {
    pub fn new() -> Self {
        Self {
            visible: false,
            fps: 0.0,
            frame_time_ms: 0.0,
            physics_stats: PhysicsStats::default(),
            active_rigid_bodies: 0,
            active_collisions: 0,
            active_chunks: 0,
            ram_usage_mb: 0.0,
            vram_usage_mb: None,
        }
    }

    /// Toggle debug menu visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Update FPS counter
    pub fn update_fps(&mut self, fps: f32, frame_time_ms: f32) {
        self.fps = fps;
        self.frame_time_ms = frame_time_ms;
    }

    /// Update physics statistics
    pub fn update_physics_stats(&mut self, stats: PhysicsStats) {
        self.active_rigid_bodies = stats.active_bodies;
        self.active_collisions = stats.broadphase_pairs;
        self.physics_stats = stats.clone();
    }

    /// Update chunk count
    pub fn update_chunks(&mut self, count: usize) {
        self.active_chunks = count;
    }

    /// Update RAM usage
    pub fn update_ram_usage(&mut self, mb: f32) {
        self.ram_usage_mb = mb;
    }

    /// Update VRAM usage
    pub fn update_vram_usage(&mut self, mb: f32) {
        self.vram_usage_mb = Some(mb);
    }

    /// Render debug overlay (call from renderer)
    pub fn render(&self, draw_text: &mut dyn FnMut(&str, f32, f32, [f32; 4])) {
        if !self.visible {
            return;
        }

        let mut y = 10.0;
        let x = 10.0;
        let line_height = 18.0;
        let white = [1.0, 1.0, 1.0, 1.0];
        let yellow = [1.0, 1.0, 0.0, 1.0];
        let green = [0.0, 1.0, 0.0, 1.0];
        let cyan = [0.0, 1.0, 1.0, 1.0];

        // Header
        draw_text("=== DEBUG MENU (F3) ===", x, y, yellow);
        y += line_height + 5.0;

        // Performance
        draw_text("--- PERFORMANCE ---", x, y, cyan);
        y += line_height;

        let fps_color = if self.fps >= 60.0 {
            green
        } else if self.fps >= 30.0 {
            yellow
        } else {
            white
        };
        draw_text(&format!("FPS: {:.1}", self.fps), x, y, fps_color);
        y += line_height;

        draw_text(
            &format!("Frame Time: {:.2} ms", self.frame_time_ms),
            x,
            y,
            white,
        );
        y += line_height;

        draw_text(&format!("RAM: {:.1} MB", self.ram_usage_mb), x, y, white);
        y += line_height;

        if let Some(vram) = self.vram_usage_mb {
            draw_text(&format!("VRAM: {:.1} MB", vram), x, y, white);
            y += line_height;
        }

        // Physics
        draw_text("--- PHYSICS ---", x, y, cyan);
        y += line_height;

        draw_text(
            &format!("Active RigidBodies: {}", self.active_rigid_bodies),
            x,
            y,
            white,
        );
        y += line_height;

        draw_text(
            &format!("Collision Pairs: {}", self.active_collisions),
            x,
            y,
            white,
        );
        y += line_height;

        draw_text(
            &format!(
                "Contacts Resolved: {}",
                self.physics_stats.contacts_resolved
            ),
            x,
            y,
            white,
        );
        y += line_height;

        draw_text(
            &format!("Physics Step: {} μs", self.physics_stats.step_time_us),
            x,
            y,
            white,
        );
        y += line_height;

        draw_text(
            &format!("Sleeping Bodies: {}", self.physics_stats.sleeping_bodies),
            x,
            y,
            white,
        );
        y += line_height;

        // World
        draw_text("--- WORLD ---", x, y, cyan);
        y += line_height;

        draw_text(
            &format!("Active Chunks: {}", self.active_chunks),
            x,
            y,
            white,
        );
        y += line_height;
    }
    pub fn render_player_info(
        &self,
        player: &Player,
        draw_text: &mut dyn FnMut(&str, f32, f32, [f32; 4]),
    ) {
        if !self.visible {
            return;
        }

        let mut y = 300.0;
        let x = 10.0;
        let line_height = 18.0;
        let white = [1.0, 1.0, 1.0, 1.0];
        let cyan = [0.0, 1.0, 1.0, 1.0];
        let green = [0.0, 1.0, 0.0, 1.0];

        draw_text("--- PLAYER ---", x, y, cyan);
        y += line_height + 5.0;

        draw_text(&format!("Name: {}", player.name), x, y, white);
        y += line_height;

        draw_text(&format!("Height: {:.2} m", player.height), x, y, white);
        y += line_height;

        let state_str = match player.state {
            PlayerState::OnFoot => "On Foot",
            PlayerState::InVehicle {
                vehicle_index,
                seat_index,
            } => &format!("In Vehicle ({}:{})", vehicle_index, seat_index),
            PlayerState::InHelicopter { .. } => "In Helicopter",
            PlayerState::InCrane => "In Crane",
        };
        draw_text(&format!("State: {}", state_str), x, y, white);
        y += line_height;

        draw_text(
            &format!("Stamina: {:.1}/{:.1}", player.stamina, player.max_stamina),
            x,
            y,
            white,
        );
        y += line_height;

        draw_text(
            &format!(
                "Inventory: {:.1}/{:.1} kg",
                player.inventory_weight, player.max_inventory_weight
            ),
            x,
            y,
            white,
        );
        y += line_height;

        draw_text(&format!("Money: {:.0} RUB", player.money.rub), x, y, green);
        y += line_height;

        // Skills summary
        draw_text("--- SKILLS (Top 5) ---", x, y, cyan);
        y += line_height + 5.0;

        let skills = [
            ("Mechanics", player.skills.mechanics.rank),
            ("Driving", player.skills.driving.rank),
            ("Piloting", player.skills.piloting.rank),
            ("Business", player.skills.business.rank),
            ("Fitness", player.skills.fitness.rank),
        ];

        for (name, rank) in skills.iter() {
            draw_text(&format!("{}: Rank {}", name, rank), x, y, white);
            y += line_height;
        }
    }

    /// Get debug info as string (for logging)
    pub fn get_debug_string(&self) -> String {
        format!(
            "FPS: {:.1} | Frame: {:.2}ms | Bodies: {} | Collisions: {} | Chunks: {} | RAM: {:.1}MB",
            self.fps,
            self.frame_time_ms,
            self.active_rigid_bodies,
            self.active_collisions,
            self.active_chunks,
            self.ram_usage_mb
        )
    }
}

impl Default for DebugMenu {
    fn default() -> Self {
        Self::new()
    }
}
