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
            max_population: 5, // Start with nursery capacity
            current_population: 0,
            idle_workers: 0,
        }
    }
}

impl PlayerResources {
    pub fn can_afford(&self, cost: &[(crate::components::ResourceType, f32)]) -> bool {
        for (resource_type, amount) in cost {
            match resource_type {
                crate::components::ResourceType::Nectar => {
                    if self.nectar < *amount { return false; }
                },
                crate::components::ResourceType::Chitin => {
                    if self.chitin < *amount { return false; }
                },
                crate::components::ResourceType::Minerals => {
                    if self.minerals < *amount { return false; }
                },
                crate::components::ResourceType::Pheromones => {
                    if self.pheromones < *amount { return false; }
                },
            }
        }
        true
    }

    pub fn spend_resources(&mut self, cost: &[(crate::components::ResourceType, f32)]) -> bool {
        if !self.can_afford(cost) {
            return false;
        }

        for (resource_type, amount) in cost {
            match resource_type {
                crate::components::ResourceType::Nectar => self.nectar -= amount,
                crate::components::ResourceType::Chitin => self.chitin -= amount,
                crate::components::ResourceType::Minerals => self.minerals -= amount,
                crate::components::ResourceType::Pheromones => self.pheromones -= amount,
            }
        }
        true
    }

    pub fn add_resource(&mut self, resource_type: crate::components::ResourceType, amount: f32) {
        match resource_type {
            crate::components::ResourceType::Nectar => self.nectar += amount,
            crate::components::ResourceType::Chitin => self.chitin += amount,
            crate::components::ResourceType::Minerals => self.minerals += amount,
            crate::components::ResourceType::Pheromones => self.pheromones += amount,
        }
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
        // Add AI player 2
        resources.insert(2, PlayerResources {
            nectar: 500.0,
            chitin: 500.0, 
            minerals: 300.0,
            pheromones: 300.0,
            max_population: 10,
            current_population: 0,
            idle_workers: 0,
        });
        Self { resources }
    }
}

#[derive(Resource, Debug, Default)]
pub struct GameCosts {
    pub unit_costs: std::collections::HashMap<crate::components::UnitType, Vec<(crate::components::ResourceType, f32)>>,
    pub building_costs: std::collections::HashMap<crate::components::BuildingType, Vec<(crate::components::ResourceType, f32)>>,
    pub tech_costs: std::collections::HashMap<crate::components::TechnologyType, Vec<(crate::components::ResourceType, f32)>>,
}

impl GameCosts {
    pub fn initialize() -> Self {
        use crate::components::{UnitType, BuildingType, TechnologyType, ResourceType};
        
        let mut costs = Self::default();
        
        // Unit costs
        costs.unit_costs.insert(UnitType::WorkerAnt, vec![(ResourceType::Nectar, 50.0)]);
        costs.unit_costs.insert(UnitType::SoldierAnt, vec![(ResourceType::Nectar, 60.0), (ResourceType::Pheromones, 20.0)]);
        costs.unit_costs.insert(UnitType::HunterWasp, vec![(ResourceType::Chitin, 25.0), (ResourceType::Pheromones, 45.0)]);
        costs.unit_costs.insert(UnitType::BeetleKnight, vec![(ResourceType::Nectar, 60.0), (ResourceType::Pheromones, 75.0)]);
        costs.unit_costs.insert(UnitType::SpearMantis, vec![(ResourceType::Nectar, 35.0), (ResourceType::Chitin, 25.0)]);
        
        // Building costs
        costs.building_costs.insert(BuildingType::Nursery, vec![(ResourceType::Chitin, 25.0)]);
        costs.building_costs.insert(BuildingType::WarriorChamber, vec![(ResourceType::Chitin, 175.0)]);
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