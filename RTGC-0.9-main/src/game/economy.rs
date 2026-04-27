//! Economy System for RTGC-0.8
//! Handles player wallet, market prices, shops, wages, and contracts

use crate::game::skills::{PlayerSkills, SkillType};
use std::collections::HashMap;

/// Player wallet with multiple currencies
#[derive(Debug, Clone)]
pub struct PlayerWallet {
    /// Russian Rubles (primary currency)
    pub rub: f64,
    /// Chinese Yuan
    pub cny: f64,
    /// US Dollars
    pub usd: f64,
}

impl Default for PlayerWallet {
    fn default() -> Self {
        Self {
            rub: 50_000.0, // Starting money after education
            cny: 0.0,
            usd: 0.0,
        }
    }
}

impl PlayerWallet {
    pub fn new(rub: f64, cny: f64, usd: f64) -> Self {
        Self { rub, cny, usd }
    }

    /// Add rubles to wallet
    pub fn add_rub(&mut self, amount: f64) {
        self.rub += amount;
    }

    /// Remove rubles from wallet (returns false if insufficient funds)
    pub fn remove_rub(&mut self, amount: f64) -> bool {
        if self.rub >= amount {
            self.rub -= amount;
            true
        } else {
            false
        }
    }

    /// Transfer between currencies (simplified exchange rates)
    pub fn exchange_rub_to_usd(&mut self, rub_amount: f64) -> Option<f64> {
        let rate = 90.0; // 1 USD = 90 RUB
        if self.remove_rub(rub_amount) {
            let usd_amount = rub_amount / rate;
            self.usd += usd_amount;
            Some(usd_amount)
        } else {
            None
        }
    }

    pub fn exchange_usd_to_rub(&mut self, usd_amount: f64) -> Option<f64> {
        let rate = 90.0; // 1 USD = 90 RUB
        if self.usd >= usd_amount {
            self.usd -= usd_amount;
            let rub_amount = usd_amount * rate;
            self.rub += rub_amount;
            Some(rub_amount)
        } else {
            None
        }
    }

    pub fn exchange_rub_to_cny(&mut self, rub_amount: f64) -> Option<f64> {
        let rate = 12.5; // 1 CNY = 12.5 RUB
        if self.remove_rub(rub_amount) {
            let cny_amount = rub_amount / rate;
            self.cny += cny_amount;
            Some(cny_amount)
        } else {
            None
        }
    }

    pub fn total_rub_equivalent(&self) -> f64 {
        let rate_usd = 90.0;
        let rate_cny = 12.5;
        self.rub + (self.usd * rate_usd) + (self.cny * rate_cny)
    }
}

/// Market price for a resource
#[derive(Debug, Clone)]
pub struct MarketPrice {
    /// Resource identifier
    pub resource: String,
    /// Base price in rubles
    pub base_price_rub: f64,
    /// Location modifier (0.8 - 1.5, remote areas more expensive)
    pub location_modifier: f64,
    /// Supply/demand modifier (0.5 - 2.0)
    pub supply_modifier: f64,
}

impl MarketPrice {
    pub fn new(resource: &str, base_price: f64) -> Self {
        Self {
            resource: resource.to_string(),
            base_price_rub: base_price,
            location_modifier: 1.0,
            supply_modifier: 1.0,
        }
    }

    /// Calculate final price with modifiers
    pub fn final_price(&self) -> f64 {
        self.base_price_rub * self.location_modifier * self.supply_modifier
    }

    /// Update supply modifier based on transactions
    pub fn adjust_supply(&mut self, sold: bool, amount: f64) {
        if sold {
            // Sold to market -> oversupply -> price drops
            self.supply_modifier = (self.supply_modifier - 0.01 * amount).max(0.5);
        } else {
            // Bought from market -> undersupply -> price rises
            self.supply_modifier = (self.supply_modifier + 0.01 * amount).min(2.0);
        }
    }

    /// Set location modifier (called when player moves)
    pub fn set_location_modifier(&mut self, modifier: f64) {
        self.location_modifier = modifier.clamp(0.8, 1.5);
    }
}

/// Shop types available in settlements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShopType {
    /// Gas station - sells fuel
    GasStation,
    /// Service station/garage - sells parts, repairs
    ServiceStation,
    /// Warehouse/production - accepts/issues cargo
    Warehouse,
    /// Construction materials
    ConstructionSupply,
    /// General store
    GeneralStore,
}

impl ShopType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ShopType::GasStation => "АЗС",
            ShopType::ServiceStation => "СТО/Гараж",
            ShopType::Warehouse => "Склад",
            ShopType::ConstructionSupply => "Стройматериалы",
            ShopType::GeneralStore => "Магазин",
        }
    }
}

/// Shop data
#[derive(Debug, Clone)]
pub struct Shop {
    pub shop_type: ShopType,
    pub name: String,
    pub location: String,
    pub inventory: HashMap<String, ShopItem>,
    pub buy_orders: HashMap<String, BuyOrder>,
}

#[derive(Debug, Clone)]
pub struct ShopItem {
    pub name: String,
    pub quantity: u32,
    pub price_per_unit: f64,
    pub max_stack: u32,
}

#[derive(Debug, Clone)]
pub struct BuyOrder {
    pub resource: String,
    pub quantity_needed: u32,
    pub price_per_unit: f64,
    pub deadline_hours: f32,
}

impl Shop {
    pub fn new(shop_type: ShopType, name: &str, location: &str) -> Self {
        Self {
            shop_type,
            name: name.to_string(),
            location: location.to_string(),
            inventory: HashMap::new(),
            buy_orders: HashMap::new(),
        }
    }

    /// Add item to shop inventory
    pub fn add_item(&mut self, name: &str, quantity: u32, price: f64) {
        self.inventory.insert(
            name.to_string(),
            ShopItem {
                name: name.to_string(),
                quantity,
                price_per_unit: price,
                max_stack: 100,
            },
        );
    }

    /// Add buy order
    pub fn add_buy_order(&mut self, resource: &str, quantity: u32, price: f64, deadline: f32) {
        self.buy_orders.insert(
            resource.to_string(),
            BuyOrder {
                resource: resource.to_string(),
                quantity_needed: quantity,
                price_per_unit: price,
                deadline_hours: deadline,
            },
        );
    }

    /// Buy item from shop
    pub fn buy_item(
        &mut self,
        item_name: &str,
        quantity: u32,
        wallet: &mut PlayerWallet,
    ) -> Option<u32> {
        let item = self.inventory.get_mut(item_name)?;
        let total_cost = item.price_per_unit * quantity as f64;

        if wallet.remove_rub(total_cost) && item.quantity >= quantity {
            item.quantity -= quantity;
            Some(quantity)
        } else {
            None
        }
    }

    /// Sell item to shop (fulfill buy order)
    pub fn sell_item(
        &mut self,
        resource: &str,
        quantity: u32,
        wallet: &mut PlayerWallet,
    ) -> Option<f64> {
        let order = self.buy_orders.get(resource)?;
        let sell_qty = quantity.min(order.quantity_needed);
        let earnings = order.price_per_unit * sell_qty as f64;

        wallet.add_rub(earnings);
        Some(earnings)
    }
}

/// Wage calculation based on skill rank
pub fn calculate_wage(skill_rank: u8, base_salary: f64) -> f64 {
    let rank_multiplier = match skill_rank {
        0 => 0.0,
        1 => 0.3,
        2 => 0.5,
        3 => 0.7,
        4 => 1.0,
        5 => 1.3,
        6 => 1.6,
        7 => 2.0,
        8 => 3.0,
        9 => 5.0,
        10 => 8.0,
        11 => 12.0,
        12 => 25.0,
        _ => 1.0,
    };
    base_salary * rank_multiplier
}

/// Base salaries for different professions (rub/month at rank 4)
pub const BASE_SALARIES: &[(&str, f64)] = &[
    ("driver_rank2", 45_000.0),    // Driver rank 2
    ("pilot_rank4", 80_000.0),     // Pilot rank 4
    ("mechanic_rank3", 35_000.0),  // Mechanic rank 3
    ("welder_rank3", 40_000.0),    // Welder rank 3
    ("builder_rank3", 38_000.0),   // Builder rank 3
    ("geologist_rank4", 55_000.0), // Geologist rank 4
    ("logger_rank3", 42_000.0),    // Logger rank 3
    ("miner_rank4", 60_000.0),     // Miner rank 4
    ("business_rank5", 100_000.0), // Business owner rank 5
    ("logistics_rank6", 70_000.0), // Logistics manager rank 6
];

/// Get base salary for a profession
pub fn get_base_salary(profession: &str) -> f64 {
    BASE_SALARIES
        .iter()
        .find(|(name, _)| *name == profession)
        .map(|(_, salary)| *salary)
        .unwrap_or(30_000.0) // Default minimum wage
}

/// Contract job board (биржа заказов)
#[derive(Debug, Clone)]
pub struct JobBoard {
    pub jobs: Vec<ContractJob>,
}

#[derive(Debug, Clone)]
pub struct ContractJob {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub required_skill: SkillType,
    pub min_skill_rank: u8,
    pub reward_rub: f64,
    pub duration_hours: f32,
    pub location: String,
    pub employer: String,
    pub deadline_game_days: u32,
}

impl JobBoard {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    /// Add a job to the board
    pub fn add_job(&mut self, job: ContractJob) {
        self.jobs.push(job);
    }

    /// Get available jobs filtered by player skill
    pub fn get_available_jobs(&self, skills: &PlayerSkills) -> Vec<&ContractJob> {
        self.jobs
            .iter()
            .filter(|job| {
                let skill = skills.get_skill(job.required_skill);
                skill.rank >= job.min_skill_rank
            })
            .collect()
    }

    /// Accept a job (removes from board)
    pub fn accept_job(&mut self, job_id: u32) -> Option<ContractJob> {
        if let Some(pos) = self.jobs.iter().position(|j| j.id == job_id) {
            Some(self.jobs.remove(pos))
        } else {
            None
        }
    }

    /// Generate sample jobs
    pub fn generate_sample_jobs() -> Vec<ContractJob> {
        vec![
            ContractJob {
                id: 1,
                title: "Доставка стройматериалов".to_string(),
                description: "Перевезти 2 тонны цемента со склада в Академгородок".to_string(),
                required_skill: SkillType::Driving,
                min_skill_rank: 2,
                reward_rub: 15_000.0,
                duration_hours: 4.0,
                location: "Академгородок".to_string(),
                employer: "ООО СтройТранс".to_string(),
                deadline_game_days: 3,
            },
            ContractJob {
                id: 2,
                title: "Ремонт двигателя УАЗ".to_string(),
                description: "Заменить поршневую группу на ЗМЗ-409".to_string(),
                required_skill: SkillType::Mechanics,
                min_skill_rank: 3,
                reward_rub: 25_000.0,
                duration_hours: 8.0,
                location: "Ленинский район".to_string(),
                employer: "ИП Петров".to_string(),
                deadline_game_days: 5,
            },
            ContractJob {
                id: 3,
                title: "Геологическая разведка".to_string(),
                description: "Исследовать участок на наличие песка и гравия".to_string(),
                required_skill: SkillType::Geology,
                min_skill_rank: 4,
                reward_rub: 45_000.0,
                duration_hours: 12.0,
                location: "Заельцовский район".to_string(),
                employer: "НовосибирскГеоРазведка".to_string(),
                deadline_game_days: 7,
            },
            ContractJob {
                id: 4,
                title: "Сварка металлоконструкций".to_string(),
                description: "Сварить каркас для ангара 10x15м".to_string(),
                required_skill: SkillType::Welding,
                min_skill_rank: 3,
                reward_rub: 35_000.0,
                duration_hours: 16.0,
                location: "Кировский район".to_string(),
                employer: "МеталлСервис".to_string(),
                deadline_game_days: 10,
            },
            ContractJob {
                id: 5,
                title: "Логистика партии товара".to_string(),
                description: "Организовать доставку 5 тонн товаров из Китая".to_string(),
                required_skill: SkillType::Logistics,
                min_skill_rank: 5,
                reward_rub: 80_000.0,
                duration_hours: 24.0,
                location: "Центральный район".to_string(),
                employer: "Торговый Дом Восток".to_string(),
                deadline_game_days: 14,
            },
        ]
    }
}

impl Default for JobBoard {
    fn default() -> Self {
        Self {
            jobs: Self::generate_sample_jobs(),
        }
    }
}

/// Economy system manager
#[derive(Debug)]
pub struct EconomySystem {
    pub wallet: PlayerWallet,
    pub market_prices: HashMap<String, MarketPrice>,
    pub shops: HashMap<String, Shop>,
    pub job_board: JobBoard,
    /// Last processed game day for deadline tracking
    last_processed_day: u32,
}

impl Default for EconomySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl EconomySystem {
    pub fn new() -> Self {
        let mut system = Self {
            wallet: PlayerWallet::default(),
            market_prices: HashMap::new(),
            shops: HashMap::new(),
            job_board: JobBoard::default(),
            last_processed_day: 0,
        };

        // Initialize market prices for common resources
        system.init_market_prices();
        system.init_sample_shops();

        system
    }

    fn init_market_prices(&mut self) {
        let resources = [
            ("fuel_ai92", 52.0), // RUB per liter
            ("fuel_ai95", 56.0),
            ("fuel_diesel", 58.0),
            ("engine_oil", 800.0),       // RUB per 4L
            ("cement", 450.0),           // RUB per 50kg bag
            ("sand", 150.0),             // RUB per ton
            ("gravel", 280.0),           // RUB per ton
            ("metal_scrap", 12_000.0),   // RUB per ton
            ("logs", 3500.0),            // RUB per m³
            ("spark_plug", 450.0),       // RUB per piece
            ("brake_pads", 2500.0),      // RUB per set
            ("tire_235_75_r16", 8500.0), // RUB per tire
        ];

        for (resource, base_price) in resources.iter() {
            self.market_prices.insert(
                resource.to_string(),
                MarketPrice::new(resource, *base_price),
            );
        }
    }

    fn init_sample_shops(&mut self) {
        // Gas station
        let mut gas_station = Shop::new(ShopType::GasStation, "Газпромнефть", "ул. Ленина");
        gas_station.add_item("fuel_ai92", 10000, 52.0);
        gas_station.add_item("fuel_ai95", 8000, 56.0);
        gas_station.add_item("fuel_diesel", 6000, 58.0);
        gas_station.add_item("engine_oil", 50, 800.0);
        self.shops.insert("gas_station_1".to_string(), gas_station);

        // Service station
        let mut service_station =
            Shop::new(ShopType::ServiceStation, "АвтоСервис Профи", "ул. Гоголя");
        service_station.add_item("spark_plug", 100, 450.0);
        service_station.add_item("brake_pads", 30, 2500.0);
        service_station.add_item("tire_235_75_r16", 40, 8500.0);
        service_station.add_item("engine_oil", 80, 800.0);
        service_station.add_buy_order("metal_scrap", 500, 11_000.0, 168.0); // 1 week
        self.shops
            .insert("service_station_1".to_string(), service_station);

        // Construction supply
        let mut construction =
            Shop::new(ShopType::ConstructionSupply, "СтройБаза", "ул. Станционная");
        construction.add_item("cement", 500, 450.0);
        construction.add_item("sand", 200, 150.0);
        construction.add_item("gravel", 150, 280.0);
        construction.add_buy_order("logs", 100, 3200.0, 336.0); // 2 weeks
        self.shops
            .insert("construction_1".to_string(), construction);
    }

    /// Calculate wage for a profession based on skill rank
    pub fn calculate_profession_wage(&self, profession: &str, skill_rank: u8) -> f64 {
        let base = get_base_salary(profession);
        calculate_wage(skill_rank, base)
    }

    /// Get logistics bonus multiplier (rank 6+ gives up to +52%)
    pub fn get_logistics_bonus(&self, logistics_rank: u8) -> f64 {
        if logistics_rank >= 6 {
            1.0 + ((logistics_rank - 5) as f64 * 0.08).min(0.52)
        } else {
            1.0
        }
    }

    /// Apply contract reward with bonuses
    pub fn apply_contract_reward(&mut self, base_reward: f64, logistics_rank: u8) -> f64 {
        let bonus_multiplier = self.get_logistics_bonus(logistics_rank);
        let final_reward = base_reward * bonus_multiplier;
        self.wallet.add_rub(final_reward);
        final_reward
    }

    /// Update economy system - process deadlines and market changes
    pub fn update(&mut self, delta_hours: f32, current_game_day: u32) {
        // Process buy order deadlines in shops
        for shop in self.shops.values_mut() {
            shop.buy_orders.retain(|_key, order| {
                if order.deadline_hours <= 0.0 {
                    // Order expired - remove it
                    tracing::debug!("Buy order for {} expired", order.resource);
                    false
                } else {
                    order.deadline_hours -= delta_hours;
                    true
                }
            });
        }
  
        // Process contract job deadlines - decrement only when day changes
        let days_passed = if current_game_day > self.last_processed_day {
            current_game_day - self.last_processed_day
        } else {
            0
        };
        
        if days_passed > 0 {
            self.job_board.jobs.retain(|job| {
                if job.deadline_game_days == 0 {
                    // Job already expired
                    tracing::debug!("Contract job '{}' expired", job.title);
                    false
                } else if days_passed >= job.deadline_game_days {
                    // Job expires now
                    tracing::debug!("Contract job '{}' expired after {} days", job.title, days_passed);
                    false
                } else {
                    // Decrement deadline by days passed
                    // Note: We don't modify the job here since retain expects immutable reference
                    // The actual decrement happens below
                    true
                }
            });
            
            // Decrement deadlines for remaining jobs
            for job in &mut self.job_board.jobs {
                if job.deadline_game_days > 0 {
                    job.deadline_game_days = job.deadline_game_days.saturating_sub(days_passed);
                }
            }
        }
        
        self.last_processed_day = current_game_day;

        // Update market prices based on supply/demand fluctuations
        for price in self.market_prices.values_mut() {
             // Small random fluctuation (±2% per hour)
             let fluctuation = (delta_hours as f64 * 0.02).min(0.1);
             price.supply_modifier = (price.supply_modifier + (fluctuation * 0.1)).max(0.5).min(2.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_operations() {
        let mut wallet = PlayerWallet::new(100_000.0, 0.0, 0.0);

        assert!(wallet.remove_rub(50_000.0));
        assert_eq!(wallet.rub, 50_000.0);

        assert!(!wallet.remove_rub(100_000.0)); // Insufficient funds

        wallet.add_rub(25_000.0);
        assert_eq!(wallet.rub, 75_000.0);
    }

    #[test]
    fn test_currency_exchange() {
        let mut wallet = PlayerWallet::new(90_000.0, 0.0, 0.0);

        let usd_received = wallet.exchange_rub_to_usd(90_000.0);
        assert_eq!(usd_received, Some(1000.0));
        assert_eq!(wallet.rub, 0.0);
        assert_eq!(wallet.usd, 1000.0);
    }

    #[test]
    fn test_wage_calculation() {
        assert_eq!(calculate_wage(2, 45_000.0), 22_500.0); // 0.5x
        assert_eq!(calculate_wage(4, 45_000.0), 45_000.0); // 1.0x
        assert_eq!(calculate_wage(8, 45_000.0), 135_000.0); // 3.0x
        assert_eq!(calculate_wage(12, 45_000.0), 1_125_000.0); // 25.0x
    }

    #[test]
    fn test_logistics_bonus() {
        let econ = EconomySystem::new();

        assert_eq!(econ.get_logistics_bonus(5), 1.0); // No bonus
        assert_eq!(econ.get_logistics_bonus(6), 1.08); // +8%
        assert_eq!(econ.get_logistics_bonus(10), 1.40); // +40%
        assert_eq!(econ.get_logistics_bonus(12), 1.52); // +52% max
    }

    #[test]
    fn test_market_price_modifiers() {
        let mut price = MarketPrice::new("fuel_ai92", 52.0);

        assert_eq!(price.final_price(), 52.0);

        price.set_location_modifier(1.3); // Remote area
        price.adjust_supply(true, 50.0); // Sold 50 units

        assert!(price.final_price() < 52.0 * 1.3); // Price dropped due to oversupply
    }
}
