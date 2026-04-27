//! First Mission System for RTGC-0.9
//! Handles the initial contract from "Серега" - tutorial mission

use crate::game::events::{publish_event, GameEvent};
use crate::game::skills::SkillType;
use crate::network::GameState;

/// Mission states for the first contract
#[derive(Debug, Clone, PartialEq)]
pub enum FirstMissionState {
    /// Not yet triggered
    NotStarted,
    /// Timer counting down to phone call
    WaitingForTrigger(f32), // seconds remaining
    /// Phone notification received, waiting for player acceptance
    PhoneRinging,
    /// Mission accepted, loading cargo
    LoadingCargo,
    /// Cargo loaded, need to deliver
    InProgress {
        distance_remaining_km: f64,
        cargo_weight_kg: f64,
    },
    /// Delivered successfully
    Completed,
    /// Reward claimed
    Claimed,
    /// Failed (timeout or cargo lost)
    Failed,
}

/// First mission data (Серега's contract)
#[derive(Debug, Clone)]
pub struct FirstMission {
    pub state: FirstMissionState,
    pub client_name: String,
    pub description: String,
    pub origin: String,
    pub destination: String,
    pub cargo_type: String,
    pub cargo_weight_kg: f64,
    pub reward_rub: f64,
    pub distance_km: f64,
    pub timeout_hours: f32,
    pub time_elapsed_hours: f32,
}

impl Default for FirstMission {
    fn default() -> Self {
        Self::new()
    }
}

impl FirstMission {
    pub fn new() -> Self {
        Self {
            state: FirstMissionState::NotStarted,
            client_name: "Серёга".to_string(),
            description: "Привет! Нужна помощь, доставить кирпич в Бердск. Заберу на складе в Новосибе, отвезёшь мне. 800 кг, справишься?".to_string(),
            origin: "Новосибирск, ул. Станционная".to_string(),
            destination: "Бердск, ул. Кирпичная".to_string(),
            cargo_type: "Кирпич строительный".to_string(),
            cargo_weight_kg: 800.0,
            reward_rub: 18_000.0,
            distance_km: 32.0,
            timeout_hours: 4.0,
            time_elapsed_hours: 0.0,
        }
    }

    /// Update mission timer
    pub fn update(&mut self, delta_seconds: f32) {
        match &mut self.state {
            FirstMissionState::WaitingForTrigger(time_left) => {
                *time_left -= delta_seconds;
                if *time_left <= 0.0 {
                    self.state = FirstMissionState::PhoneRinging;
                    publish_event(GameEvent::FirstMissionPhoneCall);
                }
            }
            FirstMissionState::InProgress { .. } => {
                self.time_elapsed_hours += delta_seconds / 3600.0;
                if self.time_elapsed_hours >= self.timeout_hours {
                    self.state = FirstMissionState::Failed;
                    publish_event(GameEvent::FirstMissionFailed {
                        reason: "timeout".to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    /// Accept the mission after phone call
    pub fn accept(&mut self) {
        if self.state == FirstMissionState::PhoneRinging {
            self.state = FirstMissionState::LoadingCargo;
            publish_event(GameEvent::FirstMissionAccepted);
        }
    }

    /// Start delivery (cargo loaded)
    pub fn start_delivery(&mut self) {
        if self.state == FirstMissionState::LoadingCargo {
            self.state = FirstMissionState::InProgress {
                distance_remaining_km: self.distance_km,
                cargo_weight_kg: self.cargo_weight_kg,
            };
            publish_event(GameEvent::FirstMissionDeliveryStarted);
        }
    }

    /// Update distance to destination
    pub fn update_distance(&mut self, current_distance_km: f64) {
        if let FirstMissionState::InProgress {
            distance_remaining_km,
            ..
        } = &mut self.state
        {
            *distance_remaining_km = current_distance_km.max(0.0);

            // Check if delivered
            if *distance_remaining_km < 0.5 {
                self.complete();
            }
        }
    }

    /// Complete the mission
    fn complete(&mut self) {
        self.state = FirstMissionState::Completed;
        publish_event(GameEvent::FirstMissionCompleted {
            reward: self.reward_rub,
            time_taken_hours: self.time_elapsed_hours,
        });
    }

    /// Get tutorial hints based on current state
    pub fn get_tutorial_hint(&self) -> Option<&'static str> {
        match self.state {
            FirstMissionState::PhoneRinging => Some("Вам звонят! Нажмите F чтобы принять вызов"),
            FirstMissionState::LoadingCargo => {
                Some("Подъедьте к складу задним ходом и нажмите F для погрузки кирпича")
            }
            FirstMissionState::InProgress { .. } => {
                Some("Двигайтесь к точке доставки. Следите за расходом топлива и состоянием дороги")
            }
            _ => None,
        }
    }

    /// Check if mission is active
    pub fn is_active(&self) -> bool {
        matches!(
            self.state,
            FirstMissionState::PhoneRinging
                | FirstMissionState::LoadingCargo
                | FirstMissionState::InProgress { .. }
        )
    }

    /// Get formatted status for HUD
    pub fn get_status_text(&self) -> String {
        match &self.state {
            FirstMissionState::NotStarted => String::new(),
            FirstMissionState::WaitingForTrigger(_) => String::new(),
            FirstMissionState::PhoneRinging => format!("📞 {} звонит!", self.client_name),
            FirstMissionState::LoadingCargo => {
                format!(
                    "📦 Погрузка: {} ({:.0} кг)",
                    self.cargo_type, self.cargo_weight_kg
                )
            }
            FirstMissionState::InProgress {
                distance_remaining_km,
                ..
            } => {
                format!(
                    "🚚 Доставка: осталось {:.1} км | Награда: {}₽",
                    distance_remaining_km, self.reward_rub as u32
                )
            }
            FirstMissionState::Completed => "✅ Задание выполнено!".to_string(),
            FirstMissionState::Failed => "❌ Задание провалено".to_string(),
            FirstMissionState::Claimed => "✅ Награда получена!".to_string(),
        }
    }
}

/// First mission manager
#[derive(Debug)]
pub struct FirstMissionManager {
    pub mission: FirstMission,
    pub trigger_delay_seconds: f32,
    pub contacts_unlocked: Vec<String>,
}

impl Default for FirstMissionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FirstMissionManager {
    pub fn new() -> Self {
        Self {
            mission: FirstMission::new(),
            trigger_delay_seconds: 30.0,
            contacts_unlocked: Vec::new(),
        }
    }

    /// Initialize and start the trigger timer
    pub fn initialize(&mut self) {
        self.mission.state = FirstMissionState::WaitingForTrigger(self.trigger_delay_seconds);
    }

    /// Update mission state
    pub fn update(&mut self, delta_seconds: f32) {
        self.mission.update(delta_seconds);

        // Unlock contacts when mission completes
        if self.mission.state == FirstMissionState::Completed && self.contacts_unlocked.is_empty() {
            self.contacts_unlocked = vec![
                "Серёга (заказчик)".to_string(),
                "АЗС Газпромнефть (партнёр)".to_string(),
                "ООО СтройТранс (логистика)".to_string(),
            ];
            publish_event(GameEvent::ContactsUnlocked {
                count: self.contacts_unlocked.len(),
            });
        }
    }

    /// Accept incoming call
    pub fn accept_call(&mut self) -> bool {
        if self.mission.state == FirstMissionState::PhoneRinging {
            self.mission.accept();
            true
        } else {
            false
        }
    }

    /// Load cargo at origin
    pub fn load_cargo(&mut self) -> bool {
        if self.mission.state == FirstMissionState::LoadingCargo {
            self.mission.start_delivery();
            true
        } else {
            false
        }
    }

    /// Update distance to destination
    pub fn update_distance(&mut self, distance_km: f64) {
        self.mission.update_distance(distance_km);
    }

    /// Get mission reward if completed
    pub fn claim_reward(&mut self, game_state: &mut GameState) -> Option<f64> {
        if self.mission.state == FirstMissionState::Completed {
            let reward = self.mission.reward_rub;
            self.mission.state = FirstMissionState::Claimed;
            
            // Add money to player wallet using economy system
            if let Some(ref mut economy) = game_state.economy {
                economy.wallet.add_rub(reward);
            }
            
            Some(reward)
        } else {
            None
        }
    }

    /// Get skill XP rewards for completion
    pub fn get_skill_rewards(&self) -> Vec<(SkillType, f32)> {
        vec![
            (SkillType::Driving, 2.5),    // 2.5 hours of driving
            (SkillType::Logistics, 1.0),  // Planning route
            (SkillType::Navigation, 0.5), // Finding destination
        ]
    }
}

/// Phone UI state for first mission
#[derive(Debug, Clone)]
pub struct PhoneNotification {
    pub caller: String,
    pub message: String,
    pub timestamp: String,
    pub is_unread: bool,
}

impl PhoneNotification {
    pub fn new(caller: &str, message: &str) -> Self {
        Self {
            caller: caller.to_string(),
            message: message.to_string(),
            timestamp: "10:30".to_string(), // Would be dynamic in real game
            is_unread: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_initialization() {
        let mut manager = FirstMissionManager::new();
        manager.initialize();

        assert!(matches!(
            manager.mission.state,
            FirstMissionState::WaitingForTrigger(_)
        ));
        assert_eq!(manager.mission.client_name, "Серёга");
        assert_eq!(manager.mission.cargo_weight_kg, 800.0);
        assert_eq!(manager.mission.reward_rub, 18_000.0);
    }

    #[test]
    fn test_trigger_timer() {
        let mut manager = FirstMissionManager::new();
        manager.initialize();

        // Simulate 30 seconds passing
        manager.update(30.0);

        assert!(matches!(
            manager.mission.state,
            FirstMissionState::PhoneRinging
        ));
    }

    #[test]
    fn test_accept_mission() {
        let mut manager = FirstMissionManager::new();
        manager.initialize();
        manager.update(30.0); // Trigger phone call

        assert!(manager.accept_call());
        assert!(matches!(
            manager.mission.state,
            FirstMissionState::LoadingCargo
        ));
    }

    #[test]
    fn test_delivery_progress() {
        let mut manager = FirstMissionManager::new();
        manager.initialize();
        manager.update(30.0);
        manager.accept_call();
        manager.load_cargo();

        assert!(matches!(
            manager.mission.state,
            FirstMissionState::InProgress { .. }
        ));

        // Simulate approaching destination
        manager.update_distance(15.0);
        if let FirstMissionState::InProgress {
            distance_remaining_km,
            ..
        } = manager.mission.state
        {
            assert_eq!(distance_remaining_km, 15.0);
        }

        // Arrive at destination
        manager.update_distance(0.3);
        assert!(matches!(
            manager.mission.state,
            FirstMissionState::Completed
        ));
    }

    #[test]
    fn test_timeout_failure() {
        let mut manager = FirstMissionManager::new();
        manager.initialize();
        manager.update(30.0);
        manager.accept_call();
        manager.load_cargo();

        // Simulate 4+ hours passing (timeout)
        manager.update(4.0 * 3600.0 + 1.0);

        assert!(matches!(manager.mission.state, FirstMissionState::Failed));
    }

    #[test]
    fn test_skill_rewards() {
        let manager = FirstMissionManager::new();
        let rewards = manager.get_skill_rewards();

        assert_eq!(rewards.len(), 3);
        assert!(rewards
            .iter()
            .any(|(skill, _)| *skill == SkillType::Driving));
        assert!(rewards
            .iter()
            .any(|(skill, _)| *skill == SkillType::Logistics));
        assert!(rewards
            .iter()
            .any(|(skill, _)| *skill == SkillType::Navigation));
    }
}
