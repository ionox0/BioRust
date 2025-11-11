use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub player_speed: f32,
    pub enemy_spawn_rate: f32,
    pub difficulty_scaling: f32,
    pub debug_mode: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            player_speed: 30.0,
            enemy_spawn_rate: 2.0,
            difficulty_scaling: 1.1,
            debug_mode: false,
        }
    }
}

#[allow(dead_code)]
#[derive(Resource, Default, Debug)]
pub struct Score {
    pub value: u32,
    pub high_score: u32,
    pub multiplier: f32,
}

#[allow(dead_code)]
#[derive(Resource, Default, Debug)]
pub struct GameTimer {
    pub elapsed: f32,
    pub level_time: f32,
}

#[allow(dead_code)]
#[derive(Resource, Default, Debug)]
pub struct InputState {
    pub movement: Vec2,
    pub jump: bool,
    pub action: bool,
    pub pause: bool,
}

#[allow(dead_code)]
#[derive(Resource, Debug)]
pub struct AudioHandles {
    pub jump: Handle<AudioSource>,
    pub collect: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub background_music: Handle<AudioSource>,
}

#[allow(dead_code)]
#[derive(Resource, Debug)]
pub struct TextureHandles {
    pub player: Handle<Image>,
    pub enemy: Handle<Image>,
    pub collectible: Handle<Image>,
    pub background: Handle<Image>,
}

#[allow(dead_code)]
#[derive(Resource, Default, Debug)]
pub struct LevelData {
    pub current_level: u32,
    pub spawn_points: Vec<Vec2>,
    pub collectible_positions: Vec<Vec2>,
    pub enemy_positions: Vec<Vec2>,
}

#[allow(dead_code)]
#[derive(Resource, Default, Debug)]
pub struct GameStats {
    pub enemies_defeated: u32,
    pub collectibles_gathered: u32,
    pub distance_traveled: f32,
    pub play_time: f32,
}

// RTS Resource Management
#[derive(Resource, Debug, Clone)]
pub struct PlayerResources {
    pub nectar: f32,    // Food -> Nectar (insect theme)
    pub chitin: f32,    // Wood -> Chitin (insect theme) 
    pub minerals: f32,  // Stone -> Minerals (insect theme)
    pub pheromones: f32, // Gold -> Pheromones (insect theme)
    pub max_population: u32,
    pub current_population: u32,
    pub idle_workers: u32,  // villagers -> workers (insect theme)
}

impl Default for PlayerResources {
    fn default() -> Self {
        Self {
            nectar: 200.0,
            chitin: 200.0,
            minerals: 100.0,
            pheromones: 100.0,
            max_population: 500, // Start with nursery capacity
            current_population: 0,
            idle_workers: 0,
        }
    }
}

impl PlayerResources {
    pub fn can_afford(&self, cost: &[(crate::core::components::ResourceType, f32)]) -> bool {
        cost.iter().all(|(resource_type, amount)| {
            resource_type.get_from(self) >= *amount
        })
    }

    pub fn spend_resources(&mut self, cost: &[(crate::core::components::ResourceType, f32)]) -> bool {
        if !self.can_afford(cost) {
            return false;
        }

        for (resource_type, amount) in cost {
            resource_type.subtract_from(self, *amount);
        }
        true
    }

    pub fn add_resource(&mut self, resource_type: crate::core::components::ResourceType, amount: f32) {
        resource_type.add_to(self, amount);
    }

    pub fn has_population_space(&self) -> bool {
        self.current_population < self.max_population
    }

    pub fn add_population(&mut self, amount: u32) -> bool {
        if self.current_population + amount <= self.max_population {
            self.current_population += amount;
            true
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn remove_population(&mut self, amount: u32) {
        self.current_population = self.current_population.saturating_sub(amount);
    }

    pub fn add_housing(&mut self, amount: u32) {
        self.max_population += amount;
    }
}

// AI Player resource management
#[derive(Resource, Debug, Clone)]
pub struct AIResources {
    pub resources: std::collections::HashMap<u8, PlayerResources>,
}

impl Default for AIResources {
    fn default() -> Self {
        let mut resources = std::collections::HashMap::new();
        // Add AI player 2 with better starting resources
        resources.insert(2, PlayerResources {
            nectar: 800.0,  // More food for unit production
            chitin: 800.0,  // More building materials
            minerals: 500.0, // More minerals
            pheromones: 500.0, // More pheromones for special units
            max_population: 20, // Higher pop cap to support more units
            current_population: 0,
            idle_workers: 0,
        });
        Self { resources }
    }
}

/// Helper struct to provide unified access to player resources regardless of player ID
/// This eliminates the need for repeated "if player_id == 1" checks throughout the codebase
pub struct ResourceManager<'a> {
    player_resources: &'a mut PlayerResources,
    ai_resources: &'a mut AIResources,
}

impl<'a> ResourceManager<'a> {
    pub fn new(player_resources: &'a mut PlayerResources, ai_resources: &'a mut AIResources) -> Self {
        Self { player_resources, ai_resources }
    }

    /// Get immutable reference to resources for a specific player
    pub fn get(&self, player_id: u8) -> Option<&PlayerResources> {
        if player_id == 1 {
            Some(self.player_resources)
        } else {
            self.ai_resources.resources.get(&player_id)
        }
    }

    /// Get mutable reference to resources for a specific player
    pub fn get_mut(&mut self, player_id: u8) -> Option<&mut PlayerResources> {
        if player_id == 1 {
            Some(self.player_resources)
        } else {
            self.ai_resources.resources.get_mut(&player_id)
        }
    }

    /// Check if a player can afford a cost
    pub fn can_afford(&self, player_id: u8, cost: &[(crate::core::components::ResourceType, f32)]) -> bool {
        self.get(player_id)
            .map(|res| res.can_afford(cost))
            .unwrap_or(false)
    }

    /// Check if a player can afford a cost and has population space
    pub fn can_afford_unit(&self, player_id: u8, cost: &[(crate::core::components::ResourceType, f32)]) -> bool {
        self.get(player_id)
            .map(|res| res.can_afford(cost) && res.has_population_space())
            .unwrap_or(false)
    }

    /// Spend resources for a player
    pub fn spend_resources(&mut self, player_id: u8, cost: &[(crate::core::components::ResourceType, f32)]) -> bool {
        self.get_mut(player_id)
            .map(|res| res.spend_resources(cost))
            .unwrap_or(false)
    }

    /// Add resources for a player
    pub fn add_resource(&mut self, player_id: u8, resource_type: crate::core::components::ResourceType, amount: f32) {
        if let Some(res) = self.get_mut(player_id) {
            res.add_resource(resource_type, amount);
        }
    }

    /// Add population for a player
    pub fn add_population(&mut self, player_id: u8, amount: u32) -> bool {
        self.get_mut(player_id)
            .map(|res| res.add_population(amount))
            .unwrap_or(false)
    }

    /// Add housing for a player
    pub fn add_housing(&mut self, player_id: u8, amount: u32) {
        if let Some(res) = self.get_mut(player_id) {
            res.add_housing(amount);
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct GameCosts {
    pub unit_costs: std::collections::HashMap<crate::core::components::UnitType, Vec<(crate::core::components::ResourceType, f32)>>,
    pub building_costs: std::collections::HashMap<crate::core::components::BuildingType, Vec<(crate::core::components::ResourceType, f32)>>,
    pub tech_costs: std::collections::HashMap<crate::core::components::TechnologyType, Vec<(crate::core::components::ResourceType, f32)>>,
}

impl GameCosts {
    pub fn initialize() -> Self {
        use crate::core::components::{UnitType, BuildingType, TechnologyType, ResourceType};
        
        let mut costs = Self::default();
        
        // Unit costs
        costs.unit_costs.insert(UnitType::WorkerAnt, vec![(ResourceType::Nectar, 50.0)]);
        costs.unit_costs.insert(UnitType::SoldierAnt, vec![(ResourceType::Nectar, 60.0), (ResourceType::Pheromones, 20.0)]);
        costs.unit_costs.insert(UnitType::HunterWasp, vec![(ResourceType::Chitin, 25.0), (ResourceType::Pheromones, 45.0)]);
        costs.unit_costs.insert(UnitType::BeetleKnight, vec![(ResourceType::Nectar, 60.0), (ResourceType::Pheromones, 75.0)]);
        costs.unit_costs.insert(UnitType::SpearMantis, vec![(ResourceType::Nectar, 35.0), (ResourceType::Chitin, 25.0)]);
        costs.unit_costs.insert(UnitType::ScoutAnt, vec![(ResourceType::Nectar, 40.0), (ResourceType::Pheromones, 15.0)]);
        costs.unit_costs.insert(UnitType::BatteringBeetle, vec![(ResourceType::Nectar, 80.0), (ResourceType::Chitin, 40.0)]);
        costs.unit_costs.insert(UnitType::AcidSpitter, vec![(ResourceType::Chitin, 50.0), (ResourceType::Minerals, 30.0)]);

        // Elite units (existing types)
        costs.unit_costs.insert(UnitType::DragonFly, vec![(ResourceType::Nectar, 100.0), (ResourceType::Chitin, 75.0), (ResourceType::Minerals, 50.0)]);
        costs.unit_costs.insert(UnitType::DefenderBug, vec![(ResourceType::Nectar, 70.0), (ResourceType::Chitin, 60.0)]);
        costs.unit_costs.insert(UnitType::EliteSpider, vec![(ResourceType::Chitin, 80.0), (ResourceType::Pheromones, 60.0)]);

        // Worker/Economy units
        costs.unit_costs.insert(UnitType::TermiteWorker, vec![(ResourceType::Nectar, 55.0)]);

        // Basic flying units
        costs.unit_costs.insert(UnitType::HoneyBee, vec![(ResourceType::Nectar, 45.0), (ResourceType::Chitin, 20.0)]);
        costs.unit_costs.insert(UnitType::Housefly, vec![(ResourceType::Nectar, 35.0), (ResourceType::Pheromones, 10.0)]);

        // Light predators
        costs.unit_costs.insert(UnitType::SpiderHunter, vec![(ResourceType::Chitin, 40.0), (ResourceType::Pheromones, 25.0)]);
        costs.unit_costs.insert(UnitType::LadybugScout, vec![(ResourceType::Nectar, 40.0), (ResourceType::Pheromones, 20.0)]);

        // Mid-tier units
        costs.unit_costs.insert(UnitType::Ladybug, vec![(ResourceType::Nectar, 55.0), (ResourceType::Chitin, 35.0)]);
        costs.unit_costs.insert(UnitType::LegBeetle, vec![(ResourceType::Nectar, 50.0), (ResourceType::Chitin, 30.0)]);
        costs.unit_costs.insert(UnitType::Stinkbug, vec![(ResourceType::Chitin, 45.0), (ResourceType::Pheromones, 35.0)]);

        // Heavy units
        costs.unit_costs.insert(UnitType::Scorpion, vec![(ResourceType::Nectar, 75.0), (ResourceType::Chitin, 50.0)]);
        costs.unit_costs.insert(UnitType::WolfSpider, vec![(ResourceType::Chitin, 70.0), (ResourceType::Pheromones, 50.0)]);
        costs.unit_costs.insert(UnitType::TermiteWarrior, vec![(ResourceType::Nectar, 90.0), (ResourceType::Chitin, 60.0), (ResourceType::Minerals, 40.0)]);

        // Building costs
        costs.building_costs.insert(BuildingType::Nursery, vec![(ResourceType::Chitin, 25.0)]);
        costs.building_costs.insert(BuildingType::WarriorChamber, vec![(ResourceType::Chitin, 175.0)]);
        costs.building_costs.insert(BuildingType::HunterChamber, vec![(ResourceType::Chitin, 150.0), (ResourceType::Pheromones, 50.0)]);
        costs.building_costs.insert(BuildingType::Queen, vec![(ResourceType::Chitin, 275.0), (ResourceType::Minerals, 100.0)]);
        costs.building_costs.insert(BuildingType::WoodProcessor, vec![(ResourceType::Chitin, 100.0)]);
        costs.building_costs.insert(BuildingType::MineralProcessor, vec![(ResourceType::Chitin, 100.0)]);
        costs.building_costs.insert(BuildingType::StorageChamber, vec![(ResourceType::Chitin, 100.0)]);
        costs.building_costs.insert(BuildingType::EvolutionChamber, vec![(ResourceType::Chitin, 150.0), (ResourceType::Minerals, 50.0)]);
        
        // Technology costs
        costs.tech_costs.insert(TechnologyType::ChitinWeaving, vec![(ResourceType::Nectar, 50.0), (ResourceType::Pheromones, 50.0)]);
        costs.tech_costs.insert(TechnologyType::SharpMandibles, vec![(ResourceType::Nectar, 100.0), (ResourceType::Chitin, 50.0)]);
        costs.tech_costs.insert(TechnologyType::PheromoneBoost, vec![(ResourceType::Nectar, 75.0), (ResourceType::Chitin, 75.0)]);
        costs.tech_costs.insert(TechnologyType::CargoSacs, vec![(ResourceType::Chitin, 175.0), (ResourceType::Nectar, 50.0)]);
        
        costs
    }
}