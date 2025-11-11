use bevy::prelude::*;
use bevy::ecs::system::ParamSet;
use crate::core::components::*;
use crate::core::resources::*;

// System to spawn initial resources for AI strategy testing
pub fn ai_resource_initialization_system(
    _commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    resource_query: Query<&ResourceSource>,
    _terrain_manager: Res<crate::world::terrain_v2::TerrainChunkManager>,
    _terrain_settings: Res<crate::world::terrain_v2::TerrainSettings>,
    mut initialized: Local<bool>,
) {
    // Only run once at startup
    if *initialized {
        return;
    }
    *initialized = true;
    
    // Check if resources already exist
    if resource_query.iter().count() > 0 {
        info!("Resources already exist (environment objects with ResourceSource components), skipping AI primitive resource initialization");
        return;
    }
    
    info!("No resources found - environment objects should be providing resources via ResourceSource components");
    // The environment objects (mushrooms for Nectar, rocks for Minerals) now provide all needed resources
    // No need to spawn additional primitive cube/sphere resources
    
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
    mut ai_resources: ResMut<AIResources>,
    game_costs: Res<GameCosts>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    units: Query<&RTSUnit>,
    mut buildings: Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    mut worker_queries: ParamSet<(
        Query<(Entity, &mut ResourceGatherer, &mut Movement, &Transform, &RTSUnit), (With<ResourceGatherer>, With<RTSUnit>)>,
        Query<(Entity, &mut Movement, &RTSUnit), (With<RTSUnit>, Without<ConstructionTask>)>,
    )>,
    resource_sources: Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    mut building_sites: Query<(Entity, &mut BuildingSite), With<BuildingSite>>,
    terrain_manager: Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::world::terrain_v2::TerrainSettings>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    // Collect player IDs to avoid borrowing issues
    let player_ids: Vec<u8> = ai_strategy.strategies.keys().cloned().collect();
    
    for player_id in player_ids {
        if let Some(strategy) = ai_strategy.strategies.get_mut(&player_id) {
            if let Some(player_resources) = ai_resources.resources.get(&player_id).cloned() {
                execute_strategy(
                    player_id,
                    strategy,
                    &player_resources,
                    &game_costs,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &units,
                    &mut buildings,
                    &mut ai_resources,
                    &terrain_manager,
                    &terrain_settings,
                    current_time,
                );
            
                // Assign workers to resource gathering
                assign_workers_to_resources(
                    player_id,
                    strategy,
                    &mut worker_queries.p0(),
                    &resource_sources,
                );
                
                // Assign workers to construction tasks
                assign_workers_to_construction(
                    player_id,
                    &mut commands,
                    &mut worker_queries.p1(),
                    &mut building_sites,
                );
            }
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
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    ai_resources: &mut ResMut<AIResources>,
    terrain_manager: &Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: &Res<crate::world::terrain_v2::TerrainSettings>,
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
            ai_resources,
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
    _units: &Query<&RTSUnit>,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    ai_resources: &mut ResMut<AIResources>,
    terrain_manager: &Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: &Res<crate::world::terrain_v2::TerrainSettings>,
    current_time: f32,
) {
    if strategy.priority_queue.is_empty() {
        return;
    }
    
    let goal = strategy.priority_queue[0].clone();
    
    match goal {
        StrategyGoal::BuildWorker => {
            if try_build_unit(player_id, UnitType::WorkerAnt, resources, &game_costs, commands, meshes, materials, buildings) {
                strategy.priority_queue.remove(0);
                strategy.last_unit_time = current_time;
                info!("AI Player {} queued WorkerAnt for production", player_id);
            }
        },
        StrategyGoal::BuildMilitaryUnit(unit_type) => {
            if try_build_unit(player_id, unit_type.clone(), resources, &game_costs, commands, meshes, materials, buildings) {
                strategy.priority_queue.remove(0);
                strategy.last_unit_time = current_time;
                info!("AI Player {} queued {:?} for production", player_id, unit_type);
            }
        },
        StrategyGoal::ConstructBuilding(building_type) => {
            if try_build_building(player_id, building_type.clone(), resources, game_costs, commands, meshes, materials, ai_resources, terrain_manager, terrain_settings) {
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
    _commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
) -> bool {
    // Check if we can afford the unit
    if let Some(cost) = game_costs.unit_costs.get(&unit_type) {
        if !resources.can_afford(cost) || !resources.has_population_space() {
            return false;
        }
    }
    
    // Find a suitable production building
    let required_building = match unit_type {
        // Ant units from Queen (Ant Hill)
        UnitType::WorkerAnt => BuildingType::Queen,
        UnitType::SoldierAnt => BuildingType::Queen,
        UnitType::ScoutAnt => BuildingType::Queen,
        UnitType::SpearMantis => BuildingType::Queen,
        // Bee/Flying units from Nursery (Bee Hive)
        UnitType::HunterWasp => BuildingType::Nursery,
        UnitType::DragonFly => BuildingType::Nursery,
        UnitType::AcidSpitter => BuildingType::Nursery,
        // Beetle/Heavy units from WarriorChamber (Pine Cone)
        UnitType::BeetleKnight => BuildingType::WarriorChamber,
        UnitType::BatteringBeetle => BuildingType::WarriorChamber,
        _ => return false,
    };
    
    // Find a suitable building to queue the unit in
    for (mut queue, building, unit) in buildings.iter_mut() {
        if unit.player_id == player_id && 
           building.building_type == required_building && 
           building.is_complete &&
           queue.queue.len() < 5 { // Don't overfill the queue
            
            queue.queue.push(unit_type.clone());
            info!("AI Player {} queued {:?} in {:?}", player_id, unit_type, required_building);
            return true;
        }
    }
    
    false
}

fn try_build_building(
    player_id: u8,
    building_type: BuildingType,
    resources: &PlayerResources,
    game_costs: &GameCosts,
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    ai_resources: &mut ResMut<AIResources>,
    terrain_manager: &Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: &Res<crate::world::terrain_v2::TerrainSettings>,
) -> bool {
    // Check if we can afford the building
    if let Some(cost) = game_costs.building_costs.get(&building_type) {
        if !resources.can_afford(cost) {
            return false;
        }
        
        // Deduct resources immediately when creating building site
        if let Some(player_resources) = ai_resources.resources.get_mut(&player_id) {
            if !player_resources.spend_resources(cost) {
                warn!("AI Player {} failed to spend resources for {:?} - insufficient resources", player_id, building_type);
                return false;
            }
            info!("💰 AI Player {} spent resources for {:?}: {:?}", player_id, building_type, cost);
        } else {
            warn!("AI Player {} not found in resource manager", player_id);
            return false;
        }
    }
    
    // Helper function to get terrain-aware position
    let get_terrain_position = |x: f32, z: f32, height_offset: f32| -> Vec3 {
        let terrain_height = crate::world::terrain_v2::sample_terrain_height(
            x, z, &terrain_manager.noise_generator, &terrain_settings
        );
        Vec3::new(x, terrain_height + height_offset, z)
    };
    
    // Find a suitable building position using a grid-based approach with randomization
    // Calculate base position based on player ID
    let base_x = (player_id as f32 - 1.0) * 200.0;
    let base_z = 0.0;
    
    // Add some randomness to make each building placement unique
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    building_type.hash(&mut hasher);
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
    let seed = hasher.finish();
    
    // Use the seed to generate a unique offset for this building
    let random_offset_x = (seed % 150) as f32 - 75.0; // -75 to 75 range
    let random_offset_z = ((seed / 150) % 150) as f32 - 75.0; // -75 to 75 range
    
    // Try multiple positions in expanding rings around the base
    let mut building_position = None;
    for ring in 0..5 {
        for attempt in 0..8 {
            let angle = (attempt as f32) * std::f32::consts::PI / 4.0; // 8 positions around circle
            let radius = 20.0 + (ring as f32 * 25.0); // Expanding rings
            
            let pos_x = base_x + random_offset_x + radius * angle.cos();
            let pos_z = base_z + random_offset_z + radius * angle.sin();
            
            let position = get_terrain_position(pos_x, pos_z, 0.0);
            
            // Check if position is reasonable and not too close to existing building sites
            if position.x.abs() < 1000.0 && position.z.abs() < 1000.0 {
                // TODO: Add proper collision checking with existing buildings/sites
                // For now, just use the first reasonable position found
                building_position = Some(position);
                break;
            }
        }
        if building_position.is_some() {
            break;
        }
    }
    
    let Some(building_position) = building_position else {
        warn!("AI Player {} could not find suitable position for {:?}", player_id, building_type);
        return false;
    };
    
    // Create a building site instead of directly spawning the building
    let _building_site = commands.spawn(BuildingSite {
        building_type: building_type.clone(),
        position: building_position,
        player_id,
        assigned_worker: None,
        construction_started: false,
        site_reserved: false,
    }).id();
    
    info!("AI Player {} created building site for {:?} at {:?} (unique position)", player_id, building_type, building_position);
    
    true
}


fn add_dynamic_goals(
    strategy: &mut PlayerStrategy,
    _resources: &PlayerResources,
    units: &Query<&RTSUnit>,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
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
        let has_building = buildings.iter().any(|(_, building, unit)| {
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
    workers: &mut Query<(Entity, &mut ResourceGatherer, &mut Movement, &Transform, &RTSUnit), (With<ResourceGatherer>, With<RTSUnit>)>,
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
) {
    // Find suitable resource sources for each resource type
    for priority_resource in &strategy.economy_targets.resource_priorities {
        let suitable_sources: Vec<(Entity, Vec3)> = resource_sources
            .iter()
            .filter(|(_, source, _)| source.resource_type == *priority_resource && source.amount > 0.0)
            .map(|(entity, _, transform)| (entity, transform.translation))
            .collect();
        
        if suitable_sources.is_empty() {
            continue;
        }
        
        // Find idle workers for this player and assign them to nearest resource
        for (_worker_entity, mut gatherer, mut movement, worker_transform, unit) in workers.iter_mut() {
            if unit.player_id == player_id && gatherer.target_resource.is_none() {
                // Find the closest resource source
                let mut closest_resource: Option<Entity> = None;
                let mut closest_distance = f32::MAX;
                
                for &(source_entity, source_position) in &suitable_sources {
                    let distance = worker_transform.translation.distance(source_position);
                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_resource = Some(source_entity);
                    }
                }
                
                if let Some(source_entity) = closest_resource {
                    gatherer.target_resource = Some(source_entity);
                    gatherer.resource_type = Some(priority_resource.clone());
                    
                    // Set movement target to the resource location
                    if let Ok((_, _, source_transform)) = resource_sources.get(source_entity) {
                        movement.target_position = Some(source_transform.translation);
                        info!("AI Player {} assigned worker {} to harvest {:?} at distance {:.1}", 
                              player_id, unit.unit_id, priority_resource, closest_distance);
                    }
                    
                    // Only assign one worker per iteration to avoid conflicts
                    break;
                }
            }
        }
    }
}

/// System to assign newly spawned AI workers to nearby resources
pub fn ai_worker_initialization_system(
    mut workers: Query<(Entity, &mut ResourceGatherer, &mut Movement, &Transform, &RTSUnit), (With<ResourceGatherer>, With<RTSUnit>, Added<ResourceGatherer>)>,
    resource_sources: Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
) {
    for (_worker_entity, mut gatherer, mut movement, worker_transform, unit) in workers.iter_mut() {
        // Only process AI workers (not player 1)
        if unit.player_id == 1 {
            continue;
        }
        
        // Find the closest resource source (prioritize Nectar for new workers)
        let mut closest_resource: Option<Entity> = None;
        let mut closest_distance = f32::MAX;
        
        for (source_entity, source, source_transform) in resource_sources.iter() {
            if source.amount > 0.0 {
                let distance = worker_transform.translation.distance(source_transform.translation);
                // Prioritize Nectar resources for new workers
                let priority_bonus = if source.resource_type == ResourceType::Nectar { -50.0 } else { 0.0 };
                let effective_distance = distance + priority_bonus;
                
                if effective_distance < closest_distance {
                    closest_distance = effective_distance;
                    closest_resource = Some(source_entity);
                }
            }
        }
        
        // Assign to closest resource if found
        if let Some(source_entity) = closest_resource {
            if let Ok((_, source, source_transform)) = resource_sources.get(source_entity) {
                gatherer.target_resource = Some(source_entity);
                gatherer.resource_type = Some(source.resource_type.clone());
                movement.target_position = Some(source_transform.translation);
                
                info!("AI Player {} newly spawned worker {} assigned to harvest {:?} at distance {:.1}", 
                      unit.player_id, unit.unit_id, source.resource_type, closest_distance);
            }
        }
    }
}

/// System to manage AI worker drop-off building assignment
pub fn ai_worker_dropoff_system(
    mut workers: Query<(&mut ResourceGatherer, &Transform, &RTSUnit), With<RTSUnit>>,
    buildings: Query<(Entity, &Building, &Transform, &RTSUnit), Without<ResourceGatherer>>,
) {
    for (mut gatherer, worker_transform, unit) in workers.iter_mut() {
        // Only process AI workers (not player 1)
        if unit.player_id == 1 {
            continue;
        }
        
        // If gatherer doesn't have a drop-off building or is carrying resources, find the closest one
        if gatherer.drop_off_building.is_none() || gatherer.carried_amount > 0.0 {
            // Find the closest building of the same player
            let mut closest_building: Option<Entity> = None;
            let mut closest_distance = f32::MAX;
            
            for (building_entity, building, building_transform, building_unit) in buildings.iter() {
                // Only consider buildings owned by the same player
                if building_unit.player_id != unit.player_id {
                    continue;
                }
                
                // Only consider completed buildings that can accept resources
                if !building.is_complete {
                    continue;
                }
                
                // Check if building type can accept resources (Queen, StorageChamber, etc.)
                match building.building_type {
                    BuildingType::Queen | 
                    BuildingType::StorageChamber |
                    BuildingType::Nursery => {
                        let distance = worker_transform.translation.distance(building_transform.translation);
                        if distance < closest_distance {
                            closest_distance = distance;
                            closest_building = Some(building_entity);
                        }
                    }
                    _ => continue,
                }
            }
            
            if let Some(building) = closest_building {
                gatherer.drop_off_building = Some(building);
            }
        }
    }
}

/// Assign available workers to construction tasks
fn assign_workers_to_construction(
    player_id: u8,
    commands: &mut Commands,
    construction_workers: &mut Query<(Entity, &mut Movement, &RTSUnit), (With<RTSUnit>, Without<ConstructionTask>)>,
    building_sites: &mut Query<(Entity, &mut BuildingSite), With<BuildingSite>>,
) {
    // Find unassigned building sites for this player that haven't started construction yet
    for (site_entity, mut site) in building_sites.iter_mut() {
        if site.player_id == player_id && site.assigned_worker.is_none() && !site.site_reserved && !site.construction_started {
            // Find an available worker for this player (can be resource gatherers too)
            for (worker_entity, mut movement, unit) in construction_workers.iter_mut() {
                if unit.player_id == player_id {
                    // Assign this worker to the construction site
                    site.assigned_worker = Some(worker_entity);
                    site.site_reserved = true;
                    
                    // Give the worker a construction task and movement target
                    movement.target_position = Some(site.position);
                    
                    // Add construction task component to worker
                    commands.entity(worker_entity).insert(ConstructionTask {
                        building_site: site_entity,
                        building_type: site.building_type.clone(),
                        target_position: site.position,
                        is_moving_to_site: true,
                        construction_progress: 0.0,
                        total_build_time: get_building_construction_time(&site.building_type),
                    });
                    
                    info!("✅ AI Player {} assigned worker {} to construct {:?} at {:?}", 
                          player_id, unit.unit_id, site.building_type, site.position);
                    
                    break; // Only assign one worker per iteration
                }
            }
        }
    }
}

/// Get the construction time for a building type
fn get_building_construction_time(building_type: &BuildingType) -> f32 {
    match building_type {
        BuildingType::Queen => 120.0,
        BuildingType::Nursery => 80.0,
        BuildingType::WarriorChamber => 100.0,
        BuildingType::HunterChamber => 100.0,
        BuildingType::FungalGarden => 90.0,
        BuildingType::StorageChamber => 60.0,
        _ => 100.0, // Default construction time
    }
}