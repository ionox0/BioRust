use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::rts_entities::RTSEntityFactory;

// System to spawn initial resources for AI strategy testing
pub fn ai_resource_initialization_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    resource_query: Query<&ResourceSource>,
    terrain_manager: Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::terrain_v2::TerrainSettings>,
    mut initialized: Local<bool>,
) {
    // Only run once at startup
    if *initialized {
        return;
    }
    *initialized = true;
    
    // Check if resources already exist
    if resource_query.iter().count() > 0 {
        info!("Resources already exist, skipping AI resource initialization");
        return;
    }
    
    info!("Initializing resources for AI strategy testing");
    
    // Helper function to get terrain-aware position
    let get_terrain_position = |x: f32, z: f32, height_offset: f32| -> Vec3 {
        let terrain_height = crate::terrain_v2::sample_terrain_height(
            x, z, &terrain_manager.noise_generator, &terrain_settings
        );
        Vec3::new(x, terrain_height + height_offset, z)
    };
    
    // Spawn some basic resource sources for AI to collect (near Player 2's area)
    let resource_positions_2d = [
        ((60.0, -60.0), ResourceType::Nectar),
        ((80.0, -40.0), ResourceType::Chitin),
        ((100.0, -60.0), ResourceType::Minerals),
        ((120.0, -80.0), ResourceType::Pheromones),
    ];
    
    for ((x, z), resource_type) in resource_positions_2d.iter() {
        let position = get_terrain_position(*x, *z, 1.0); // Use same height offset as existing resources
        
        match resource_type {
            ResourceType::Nectar => {
                RTSEntityFactory::spawn_nectar_source(&mut commands, &mut meshes, &mut materials, position);
            },
            ResourceType::Chitin => {
                RTSEntityFactory::spawn_chitin_source(&mut commands, &mut meshes, &mut materials, position);
            },
            ResourceType::Minerals => {
                RTSEntityFactory::spawn_mineral_deposit(&mut commands, &mut meshes, &mut materials, position);
            },
            ResourceType::Pheromones => {
                RTSEntityFactory::spawn_pheromone_cache(&mut commands, &mut meshes, &mut materials, position);
            },
        }
    }
    
    info!("AI resource initialization complete");
}

#[derive(Resource, Debug, Clone)]
pub struct AIStrategy {
    pub strategies: std::collections::HashMap<u8, PlayerStrategy>,
}

#[derive(Debug, Clone)]
pub struct PlayerStrategy {
    pub strategy_type: StrategyType,
    pub last_building_time: f32,
    pub last_unit_time: f32,
    pub priority_queue: Vec<StrategyGoal>,
    pub phase: StrategyPhase,
    pub economy_targets: EconomyTargets,
    pub military_targets: MilitaryTargets,
}

#[derive(Debug, Clone)]
pub enum StrategyType {
    Economic,  // Focus on resource gathering and infrastructure
    Military,  // Focus on unit production and combat
    Balanced,  // Mix of both
}

#[derive(Debug, Clone)]
pub enum StrategyPhase {
    EarlyGame,   // Build workers, basic economy
    MidGame,     // Expand, military production
    LateGame,    // Advanced units, technology
}

#[derive(Debug, Clone)]
pub struct EconomyTargets {
    pub desired_workers: u32,
    pub next_building: Option<BuildingType>,
    pub resource_priorities: Vec<ResourceType>,
}

#[derive(Debug, Clone)]
pub struct MilitaryTargets {
    pub desired_military_units: u32,
    pub preferred_unit_types: Vec<UnitType>,
    pub next_military_building: Option<BuildingType>,
}

#[derive(Debug, Clone)]
pub enum StrategyGoal {
    BuildWorker,
    BuildMilitaryUnit(UnitType),
    ConstructBuilding(BuildingType),
    GatherResource(ResourceType),
    ExpandTerritory,
    AttackEnemy,
}

impl Default for AIStrategy {
    fn default() -> Self {
        let mut strategies = std::collections::HashMap::new();
        
        // Initialize AI player 2 with balanced strategy
        strategies.insert(2, PlayerStrategy {
            strategy_type: StrategyType::Balanced,
            last_building_time: 0.0,
            last_unit_time: 0.0,
            priority_queue: vec![
                StrategyGoal::BuildWorker,
                StrategyGoal::BuildWorker,
                StrategyGoal::ConstructBuilding(BuildingType::FungalGarden),
                StrategyGoal::BuildWorker,
                StrategyGoal::ConstructBuilding(BuildingType::Nursery),
            ],
            phase: StrategyPhase::EarlyGame,
            economy_targets: EconomyTargets {
                desired_workers: 6,
                next_building: Some(BuildingType::FungalGarden),
                resource_priorities: vec![
                    ResourceType::Nectar,
                    ResourceType::Chitin,
                    ResourceType::Minerals,
                    ResourceType::Pheromones,
                ],
            },
            military_targets: MilitaryTargets {
                desired_military_units: 3,
                preferred_unit_types: vec![
                    UnitType::SoldierAnt,
                    UnitType::HunterWasp,
                ],
                next_military_building: Some(BuildingType::WarriorChamber),
            },
        });
        
        Self { strategies }
    }
}

pub fn ai_strategy_system(
    mut ai_strategy: ResMut<AIStrategy>,
    ai_resources: Res<AIResources>,
    game_costs: Res<GameCosts>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    units: Query<&RTSUnit>,
    buildings: Query<(&Building, &RTSUnit), With<Building>>,
    mut workers: Query<(Entity, &mut ResourceGatherer, &RTSUnit), With<ResourceGatherer>>,
    resource_sources: Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    terrain_manager: Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::terrain_v2::TerrainSettings>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    for (&player_id, strategy) in ai_strategy.strategies.iter_mut() {
        if let Some(player_resources) = ai_resources.resources.get(&player_id) {
            execute_strategy(
                player_id,
                strategy,
                player_resources,
                &game_costs,
                &mut commands,
                &mut meshes,
                &mut materials,
                &units,
                &buildings,
                &terrain_manager,
                &terrain_settings,
                current_time,
            );
            
            // Assign workers to resource gathering
            assign_workers_to_resources(
                player_id,
                strategy,
                &mut workers,
                &resource_sources,
            );
        }
    }
}

fn execute_strategy(
    player_id: u8,
    strategy: &mut PlayerStrategy,
    resources: &PlayerResources,
    game_costs: &GameCosts,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    units: &Query<&RTSUnit>,
    buildings: &Query<(&Building, &RTSUnit), With<Building>>,
    terrain_manager: &Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: &Res<crate::terrain_v2::TerrainSettings>,
    current_time: f32,
) {
    // Update strategy phase based on resources and units
    update_strategy_phase(strategy, resources, units, player_id);
    
    // Execute next goal if enough time has passed
    if should_execute_next_goal(strategy, current_time) {
        execute_next_goal(
            player_id,
            strategy,
            resources,
            game_costs,
            commands,
            meshes,
            materials,
            units,
            buildings,
            terrain_manager,
            terrain_settings,
            current_time,
        );
    }
    
    // Add new goals based on current situation
    add_dynamic_goals(strategy, resources, units, buildings, player_id);
}

fn update_strategy_phase(
    strategy: &mut PlayerStrategy,
    resources: &PlayerResources,
    units: &Query<&RTSUnit>,
    player_id: u8,
) {
    let unit_count = count_player_units(units, player_id);
    let total_resources = resources.nectar + resources.chitin + resources.minerals + resources.pheromones;
    
    strategy.phase = match (unit_count, total_resources) {
        (_, total) if total > 2000.0 => StrategyPhase::LateGame,
        (units, _) if units > 8 => StrategyPhase::MidGame,
        _ => StrategyPhase::EarlyGame,
    };
}

fn should_execute_next_goal(strategy: &PlayerStrategy, current_time: f32) -> bool {
    let time_since_last_action = current_time - strategy.last_building_time.max(strategy.last_unit_time);
    
    match strategy.phase {
        StrategyPhase::EarlyGame => time_since_last_action > 5.0,
        StrategyPhase::MidGame => time_since_last_action > 3.0,
        StrategyPhase::LateGame => time_since_last_action > 2.0,
    }
}

fn execute_next_goal(
    player_id: u8,
    strategy: &mut PlayerStrategy,
    resources: &PlayerResources,
    game_costs: &GameCosts,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    units: &Query<&RTSUnit>,
    buildings: &Query<(&Building, &RTSUnit), With<Building>>,
    terrain_manager: &Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: &Res<crate::terrain_v2::TerrainSettings>,
    current_time: f32,
) {
    if strategy.priority_queue.is_empty() {
        return;
    }
    
    let goal = strategy.priority_queue[0].clone();
    
    match goal {
        StrategyGoal::BuildWorker => {
            if try_build_unit(player_id, UnitType::WorkerAnt, resources, game_costs, commands, meshes, materials, buildings) {
                strategy.priority_queue.remove(0);
                strategy.last_unit_time = current_time;
                info!("AI Player {} built WorkerAnt", player_id);
            }
        },
        StrategyGoal::BuildMilitaryUnit(unit_type) => {
            if try_build_unit(player_id, unit_type.clone(), resources, game_costs, commands, meshes, materials, buildings) {
                strategy.priority_queue.remove(0);
                strategy.last_unit_time = current_time;
                info!("AI Player {} built {:?}", player_id, unit_type);
            }
        },
        StrategyGoal::ConstructBuilding(building_type) => {
            if try_build_building(player_id, building_type.clone(), resources, game_costs, commands, meshes, materials, terrain_manager, terrain_settings) {
                strategy.priority_queue.remove(0);
                strategy.last_building_time = current_time;
                info!("AI Player {} started building {:?}", player_id, building_type);
            }
        },
        _ => {
            // For goals we can't execute yet, skip them
            strategy.priority_queue.remove(0);
        }
    }
}

fn try_build_unit(
    player_id: u8,
    unit_type: UnitType,
    resources: &PlayerResources,
    game_costs: &GameCosts,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    buildings: &Query<(&Building, &RTSUnit), With<Building>>,
) -> bool {
    // Check if we can afford the unit
    if let Some(cost) = game_costs.unit_costs.get(&unit_type) {
        if !resources.can_afford(cost) || !resources.has_population_space() {
            return false;
        }
    }
    
    // Find a suitable production building
    let required_building = match unit_type {
        UnitType::WorkerAnt => BuildingType::Queen,
        UnitType::SoldierAnt => BuildingType::WarriorChamber,
        UnitType::HunterWasp => BuildingType::HunterChamber,
        UnitType::BeetleKnight => BuildingType::WarriorChamber,
        _ => return false,
    };
    
    // Check if we have the required building
    let has_building = buildings.iter().any(|(building, unit)| {
        unit.player_id == player_id && 
        building.building_type == required_building && 
        building.is_complete
    });
    
    if !has_building {
        return false;
    }
    
    // Spawn the unit directly (simplified approach)
    let spawn_position = Vec3::new(
        (player_id as f32 - 1.0) * 50.0,
        0.0,
        0.0,
    );
    let unit_id = rand::random();
    
    match unit_type {
        UnitType::WorkerAnt => {
            RTSEntityFactory::spawn_worker_ant(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        UnitType::SoldierAnt => {
            RTSEntityFactory::spawn_soldier_ant(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        UnitType::HunterWasp => {
            RTSEntityFactory::spawn_hunter_wasp(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        UnitType::BeetleKnight => {
            RTSEntityFactory::spawn_beetle_knight(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        _ => return false,
    }
    
    true
}

fn try_build_building(
    player_id: u8,
    building_type: BuildingType,
    resources: &PlayerResources,
    game_costs: &GameCosts,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    terrain_manager: &Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: &Res<crate::terrain_v2::TerrainSettings>,
) -> bool {
    // Check if we can afford the building
    if let Some(cost) = game_costs.building_costs.get(&building_type) {
        if !resources.can_afford(cost) {
            return false;
        }
    }
    
    // Helper function to get terrain-aware position
    let get_terrain_position = |x: f32, z: f32, height_offset: f32| -> Vec3 {
        let terrain_height = crate::terrain_v2::sample_terrain_height(
            x, z, &terrain_manager.noise_generator, &terrain_settings
        );
        Vec3::new(x, terrain_height + height_offset, z)
    };
    
    // Calculate building position based on player ID and add some randomness
    let base_x = (player_id as f32 - 1.0) * 200.0; // Use same spacing as in systems.rs
    let base_z = 0.0;
    let building_x = base_x + rand::random::<f32>() * 40.0 - 20.0; // Random offset within 40x40 area
    let building_z = base_z + rand::random::<f32>() * 40.0 - 20.0;
    
    // Spawn the building using terrain-aware positioning (same as in systems.rs)
    let building_position = get_terrain_position(building_x, building_z, 0.0); // Buildings use height offset 0.0
    
    match building_type {
        BuildingType::Queen => {
            RTSEntityFactory::spawn_queen_chamber(
                commands,
                meshes,
                materials,
                building_position,
                player_id,
            );
        },
        BuildingType::Nursery => {
            RTSEntityFactory::spawn_nursery(
                commands,
                meshes,
                materials,
                building_position,
                player_id,
            );
        },
        BuildingType::WarriorChamber => {
            RTSEntityFactory::spawn_warrior_chamber(
                commands,
                meshes,
                materials,
                building_position,
                player_id,
            );
        },
        BuildingType::HunterChamber => {
            // For now, use warrior chamber as fallback for hunter chamber
            RTSEntityFactory::spawn_warrior_chamber(
                commands,
                meshes,
                materials,
                building_position,
                player_id,
            );
        },
        BuildingType::FungalGarden => {
            // For now, use nursery as fallback for fungal garden
            RTSEntityFactory::spawn_nursery(
                commands,
                meshes,
                materials,
                building_position,
                player_id,
            );
        },
        _ => {
            // Use queen chamber as fallback for other building types
            RTSEntityFactory::spawn_queen_chamber(
                commands,
                meshes,
                materials,
                building_position,
                player_id,
            );
        }
    }
    
    true
}

fn add_dynamic_goals(
    strategy: &mut PlayerStrategy,
    resources: &PlayerResources,
    units: &Query<&RTSUnit>,
    buildings: &Query<(&Building, &RTSUnit), With<Building>>,
    player_id: u8,
) {
    let worker_count = count_player_units_of_type(units, player_id, |unit_type| {
        matches!(unit_type, UnitType::WorkerAnt)
    });
    
    let military_count = count_player_units_of_type(units, player_id, |unit_type| {
        matches!(unit_type, UnitType::SoldierAnt | UnitType::HunterWasp | UnitType::BeetleKnight)
    });
    
    // Add worker goals if we need more workers
    if worker_count < strategy.economy_targets.desired_workers {
        if !strategy.priority_queue.iter().any(|goal| matches!(goal, StrategyGoal::BuildWorker)) {
            strategy.priority_queue.push(StrategyGoal::BuildWorker);
        }
    }
    
    // Add military goals if we need more military units
    if military_count < strategy.military_targets.desired_military_units {
        for unit_type in &strategy.military_targets.preferred_unit_types {
            if !strategy.priority_queue.iter().any(|goal| {
                matches!(goal, StrategyGoal::BuildMilitaryUnit(ref ut) if ut == unit_type)
            }) {
                strategy.priority_queue.push(StrategyGoal::BuildMilitaryUnit(unit_type.clone()));
                break; // Only add one at a time
            }
        }
    }
    
    // Add building goals based on current needs
    if let Some(building_type) = strategy.economy_targets.next_building.clone() {
        let has_building = buildings.iter().any(|(building, unit)| {
            unit.player_id == player_id && building.building_type == building_type
        });
        
        if !has_building && !strategy.priority_queue.iter().any(|goal| {
            matches!(goal, StrategyGoal::ConstructBuilding(ref bt) if bt == &building_type)
        }) {
            strategy.priority_queue.push(StrategyGoal::ConstructBuilding(building_type));
        }
    }
}

fn count_player_units(units: &Query<&RTSUnit>, player_id: u8) -> u32 {
    units.iter().filter(|unit| unit.player_id == player_id).count() as u32
}

fn count_player_units_of_type<F>(units: &Query<&RTSUnit>, player_id: u8, filter: F) -> u32 
where
    F: Fn(&UnitType) -> bool,
{
    units.iter()
        .filter(|unit| {
            unit.player_id == player_id && 
            unit.unit_type.as_ref().map_or(false, |ut| filter(ut))
        })
        .count() as u32
}

fn assign_workers_to_resources(
    player_id: u8,
    strategy: &PlayerStrategy,
    workers: &mut Query<(Entity, &mut ResourceGatherer, &RTSUnit), With<ResourceGatherer>>,
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
) {
    // Find suitable resource sources for each resource type
    for priority_resource in &strategy.economy_targets.resource_priorities {
        let suitable_sources: Vec<Entity> = resource_sources
            .iter()
            .filter(|(_, source, _)| source.resource_type == *priority_resource)
            .map(|(entity, _, _)| entity)
            .collect();
        
        if suitable_sources.is_empty() {
            continue;
        }
        
        // Find idle workers for this player and assign them
        for (worker_entity, mut gatherer, unit) in workers.iter_mut() {
            if unit.player_id == player_id && gatherer.target_resource.is_none() {
                // Assign worker to the first available suitable resource source
                if let Some(source_entity) = suitable_sources.first() {
                    gatherer.target_resource = Some(*source_entity);
                    gatherer.resource_type = Some(priority_resource.clone());
                    
                    // Reduced worker assignment logging
                    
                    // Only assign one worker per iteration to avoid conflicts
                    break;
                }
            }
        }
    }
}