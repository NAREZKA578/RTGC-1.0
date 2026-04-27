use crate::physics::{PhysicsWorld, SpringConstraint};
use nalgebra::Vector3;
// Re-export InventoryItem for backwards compatibility
pub use crate::game::inventory::InventoryItem;

/// Груз — физический объект который можно прицепить к машине
pub struct Cargo {
    pub id: String,
    pub body_index: usize, // Индекс RigidBody в PhysicsWorld
    pub weight_kg: f32,
    pub is_attached: bool,
    pub attachment_body: Option<usize>, // К чему прицеплено (индекс тела машины)
    pub health: f32,                    // 1.0 = целый, 0.0 = разрушен
    pub position: Vector3<f32>,         // Текущая позиция (кэш из physics)
}

impl Cargo {
    pub fn new(id: String, body_index: usize, weight_kg: f32) -> Self {
        Self {
            id,
            body_index,
            weight_kg,
            is_attached: false,
            attachment_body: None,
            health: 1.0,
            position: Vector3::zeros(),
        }
    }

    /// Прицепить груз к машине
    pub fn attach(&mut self, vehicle_body_index: usize, _constraints: &mut Vec<SpringConstraint>) {
        // В реальной реализации здесь создается жесткая связь (SpringConstraint с rest_length=0)
        // между cargo body и vehicle body
        self.is_attached = true;
        self.attachment_body = Some(vehicle_body_index);
    }

    /// Отцепить груз
    pub fn detach(&mut self, _constraints: &mut Vec<SpringConstraint>) {
        // Удаляем constraint
        self.is_attached = false;
        self.attachment_body = None;
    }

    /// Обновить позицию из физического мира
    pub fn sync_position(&mut self, physics_world: &PhysicsWorld) {
        if let Some(body) = physics_world.get_body(self.body_index) {
            self.position = body.position;
        }
    }

    /// Получить текущий вес с учетом здоровья
    pub fn get_effective_weight(&self) -> f32 {
        self.weight_kg * self.health
    }

    /// Нанести урон грузу (при ударе)
    pub fn take_damage(&mut self, damage: f32) {
        self.health = (self.health - damage).max(0.0);
    }

    /// Разрушен ли груз полностью
    pub fn is_destroyed(&self) -> bool {
        self.health <= 0.0
    }
}
