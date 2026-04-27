//! Scene Management System - Handle different game states and transitions

use std::any::Any;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Unique identifier for a scene
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneId(u32);

impl SceneId {
    pub const fn null() -> Self {
        Self(0)
    }

    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Built-in scene types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneType {
    MainMenu,
    CharacterCreation,
    OpenWorld,
    Loading,
    Pause,
    Settings,
    Inventory,
    Mission,
    Cutscene,
    Custom(u32),
}

/// Scene state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneState {
    Active,
    Paused,
    Transitioning,
    Unloaded,
}

/// Trait for all scenes
pub trait Scene: Send {
    /// Get the scene type
    fn scene_type(&self) -> SceneType;

    /// Called when the scene is entered
    fn on_enter(&mut self) {
        debug!("Scene {:?} entered", self.scene_type());
    }

    /// Called when the scene is exited
    fn on_exit(&mut self) {
        debug!("Scene {:?} exited", self.scene_type());
    }

    /// Called when the scene is paused
    fn on_pause(&mut self) {}

    /// Called when the scene is resumed
    fn on_resume(&mut self) {}

    /// Update the scene
    fn update(&mut self, _delta_time: f32) {}

    /// Render the scene
    fn render(
        &mut self,
        _renderer: &mut crate::graphics::renderer::Renderer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Get scene name for debugging
    fn name(&self) -> &str {
        "Unnamed Scene"
    }

    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Cast to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Scene transition effect
#[derive(Debug, Clone)]
pub enum TransitionEffect {
    None,
    FadeOut(f32),
    FadeIn(f32),
    CrossFade(f32),
    SlideLeft(f32),
    SlideRight(f32),
    Zoom(f32),
}

impl TransitionEffect {
    pub fn duration(&self) -> f32 {
        match self {
            TransitionEffect::None => 0.0,
            TransitionEffect::FadeOut(d) => *d,
            TransitionEffect::FadeIn(d) => *d,
            TransitionEffect::CrossFade(d) => *d,
            TransitionEffect::SlideLeft(d) => *d,
            TransitionEffect::SlideRight(d) => *d,
            TransitionEffect::Zoom(d) => *d,
        }
    }
}

/// Scene manager configuration
#[derive(Debug, Clone)]
pub struct SceneManagerConfig {
    pub max_scenes: usize,
    pub enable_transitions: bool,
    pub default_transition_duration: f32,
    pub auto_unload_inactive: bool,
}

impl Default for SceneManagerConfig {
    fn default() -> Self {
        Self {
            max_scenes: 8,
            enable_transitions: true,
            default_transition_duration: 0.5,
            auto_unload_inactive: false,
        }
    }
}

/// Manages all scenes in the game
pub struct SceneManager {
    config: SceneManagerConfig,
    scenes: HashMap<SceneId, Box<dyn Scene>>,
    active_scene: Option<SceneId>,
    previous_scene: Option<SceneId>,
    next_scene: Option<SceneId>,
    current_transition: Option<(TransitionEffect, f32)>,
    next_id: u32,
    state: SceneState,
}

impl SceneManager {
    pub fn new(config: SceneManagerConfig) -> Self {
        let max_scenes = config.max_scenes;
        Self {
            config,
            scenes: HashMap::with_capacity(max_scenes),
            active_scene: None,
            previous_scene: None,
            next_scene: None,
            current_transition: None,
            next_id: 1,
            state: SceneState::Unloaded,
        }
    }

    /// Register a new scene
    pub fn register<S: Scene + 'static>(&mut self, scene: S) -> SceneId {
        let id = SceneId::new(self.next_id);
        self.next_id += 1;

        if self.scenes.len() >= self.config.max_scenes {
            warn!("Maximum scene count reached, consider unloading unused scenes");
        }

        self.scenes.insert(id, Box::new(scene));
        debug!("Registered scene with ID {:?}", id);
        id
    }

    /// Switch to a different scene
    pub fn switch_to(&mut self, scene_id: SceneId, transition: Option<TransitionEffect>) -> bool {
        if !self.scenes.contains_key(&scene_id) {
            warn!("Attempted to switch to unknown scene: {:?}", scene_id);
            return false;
        }

        let transition = transition.unwrap_or_else(|| {
            if self.config.enable_transitions {
                TransitionEffect::CrossFade(self.config.default_transition_duration)
            } else {
                TransitionEffect::None
            }
        });

        // If immediate transition, do it now
        if matches!(transition, TransitionEffect::None) {
            return self.switch_to_immediate(scene_id);
        }

        // Start transition
        self.previous_scene = self.active_scene;
        self.next_scene = Some(scene_id);
        self.current_transition = Some((transition, 0.0));
        self.state = SceneState::Transitioning;

        // Pause current scene
        if let Some(current_id) = self.active_scene {
            if let Some(current) = self.scenes.get_mut(&current_id) {
                current.on_pause();
            }
        }

        info!("Starting scene transition to {:?}", scene_id);
        true
    }

    /// Immediate scene switch without transition
    fn switch_to_immediate(&mut self, scene_id: SceneId) -> bool {
        if !self.scenes.contains_key(&scene_id) {
            return false;
        }

        // Exit current scene
        if let Some(current_id) = self.active_scene.take() {
            if let Some(current) = self.scenes.get_mut(&current_id) {
                current.on_exit();
            }
        }

        // Enter new scene
        self.active_scene = Some(scene_id);
        self.previous_scene = None;

        if let Some(new_scene) = self.scenes.get_mut(&scene_id) {
            new_scene.on_enter();
        }

        self.state = SceneState::Active;
        info!("Switched to scene {:?}", scene_id);
        true
    }

    /// Update the scene manager and current scene
    pub fn update(&mut self, delta_time: f32) {
        // Handle transitions
        if let Some((ref transition, ref mut elapsed)) = self.current_transition {
            *elapsed += delta_time;
            let duration = transition.duration();

            if *elapsed >= duration {
                // Transition complete
                let target_scene = self.next_scene.take();
                self.current_transition = None;

                if let Some(scene_id) = target_scene {
                    self.switch_to_immediate(scene_id);
                }
            }
            return;
        }

        // Update active scene
        if self.state == SceneState::Active {
            if let Some(active_id) = self.active_scene {
                if let Some(active) = self.scenes.get_mut(&active_id) {
                    active.update(delta_time);
                }
            }
        }
    }

    /// Render the current scene
    pub fn render(
        &mut self,
        renderer: &mut crate::graphics::renderer::Renderer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(active_id) = self.active_scene {
            if let Some(active) = self.scenes.get_mut(&active_id) {
                return active.render(renderer);
            }
        }
        Ok(())
    }

    /// Get the active scene
    pub fn active_scene(&self) -> Option<SceneId> {
        self.active_scene
    }

    /// Get the active scene as a specific type
    pub fn active_scene_as<T: Scene + 'static>(&self) -> Option<&T> {
        if let Some(id) = self.active_scene {
            if let Some(scene) = self.scenes.get(&id) {
                return scene.as_any().downcast_ref::<T>();
            }
        }
        None
    }

    /// Get mutable reference to active scene as specific type
    pub fn active_scene_as_mut<T: Scene + 'static>(&mut self) -> Option<&mut T> {
        if let Some(id) = self.active_scene {
            if let Some(scene) = self.scenes.get_mut(&id) {
                return scene.as_any_mut().downcast_mut::<T>();
            }
        }
        None
    }

    /// Get scene state
    pub fn state(&self) -> SceneState {
        self.state
    }

    /// Pause the current scene
    pub fn pause(&mut self) {
        if let Some(active_id) = self.active_scene {
            if let Some(active) = self.scenes.get_mut(&active_id) {
                active.on_pause();
            }
            self.state = SceneState::Paused;
            info!("Scene paused");
        }
    }

    /// Resume the current scene
    pub fn resume(&mut self) {
        if self.state == SceneState::Paused {
            if let Some(active_id) = self.active_scene {
                if let Some(active) = self.scenes.get_mut(&active_id) {
                    active.on_resume();
                }
                self.state = SceneState::Active;
                info!("Scene resumed");
            }
        }
    }

    /// Unload a scene
    pub fn unload(&mut self, scene_id: SceneId) -> bool {
        match self.scenes.remove(&scene_id) { Some(scene) => {
            if self.active_scene == Some(scene_id) {
                self.active_scene = None;
                self.state = SceneState::Unloaded;
            }
            debug!("Unloaded scene {:?}", scene_id);
            true
        } _ => {
            false
        }}
    }

    /// Get number of loaded scenes
    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }

    /// Check if a scene is loaded
    pub fn is_scene_loaded(&self, scene_id: SceneId) -> bool {
        self.scenes.contains_key(&scene_id)
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new(SceneManagerConfig::default())
    }
}
