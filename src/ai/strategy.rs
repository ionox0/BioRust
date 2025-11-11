use bevy::prelude::*;
use bevy::ecs::system::ParamSet;
use crate::core::components::*;
use crate::core::resources::*;
use crate::ui::placement::get_building_collision_radius;

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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    Economic,  // Focus on resource gathering and infrastructure
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub next_military_building: Option<BuildingType>,
}

#[derive(Debug, Clone)]
pub enum StrategyGoal {
    BuildWorker,
    BuildMilitaryUnit(UnitType),
    ConstructBuilding(BuildingType),
    #[allow(dead_code)]
    GatherResource(ResourceType),
    #[allow(dead_code)]
    ExpandTerritory,
    #[allow(dead_code)]
    AttackEnemy(u8), // Attack specific player
}

impl Default for AIStrategy {
    fn default() -> Self {
        let mut strategies = std::collections::HashMap::new();
        
        // Initialize AI player 2 with POPULATION-FOCUSED strategy: 1. Increase Population → 2. Eliminate Enemy
        strategies.insert(2, PlayerStrategy {
            strategy_type: StrategyType::Economic, // Population growth requires economic focus initially
            last_building_time: 0.0,
            last_unit_time: 0.0,
            priority_queue: vec![
                // PHASE 1: POPULATION GROWTH - Massive worker expansion
                StrategyGoal::BuildWorker,
                StrategyGoal::BuildWorker,
                StrategyGoal::BuildWorker,
                StrategyGoal::BuildWorker,
                StrategyGoal::BuildWorker,
                // Housing infrastructure for population support
                StrategyGoal::ConstructBuilding(BuildingType::Nursery),
                StrategyGoal::ConstructBuilding(BuildingType::Nursery), // Multiple housing
                // Resource economy to sustain population growth
                StrategyGoal::ConstructBuilding(BuildingType::FungalGarden),
                StrategyGoal::ConstructBuilding(BuildingType::StorageChamber),
                // Continue worker spam for economic dominance
                StrategyGoal::BuildWorker,
                StrategyGoal::BuildWorker,
                StrategyGoal::BuildWorker,
                StrategyGoal::BuildWorker,
                StrategyGoal::BuildWorker,
            ],
            phase: StrategyPhase::EarlyGame,
            economy_targets: EconomyTargets {
                desired_workers: 25, // Massive population target - increased from 15 to 25
                next_building: Some(BuildingType::Nursery), // Housing priority for population
                resource_priorities: vec![
                    ResourceType::Nectar,    // For population growth (worker production)
                    ResourceType::Chitin,    // For housing construction
                    ResourceType::Minerals,  // For infrastructure
                    ResourceType::Pheromones, // Support resource
                ],
            },
            military_targets: MilitaryTargets {
                desired_military_units: 20, // Large army for enemy elimination (increased from 12 to 20)
                preferred_unit_types: vec![
                    UnitType::SoldierAnt,    // Core army unit
                    UnitType::BeetleKnight,  // Heavy assault for enemy elimination
                    UnitType::HunterWasp,    // Fast raiders
                    UnitType::SpearMantis,   // Anti-building siege units
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
    collision_query: Query<(&Transform, &CollisionRadius, Option<&Building>, Option<&RTSUnit>, Option<&EnvironmentObject>)>,
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
                // Extract collision data for different entity types
                let building_collisions: Vec<(Vec3, f32)> = collision_query.iter()
                    .filter_map(|(transform, collision, building, _, _)| {
                        building.map(|_| (transform.translation, collision.radius))
                    })
                    .collect();
                
                let unit_collisions: Vec<(Vec3, f32)> = collision_query.iter()
                    .filter_map(|(transform, collision, _, unit, _)| {
                        unit.map(|_| (transform.translation, collision.radius))
                    })
                    .collect();
                
                let environment_collisions: Vec<(Vec3, f32, EnvironmentObjectType)> = collision_query.iter()
                    .filter_map(|(transform, collision, _, _, env_obj)| {
                        env_obj.map(|e| (transform.translation, collision.radius, e.object_type.clone()))
                    })
                    .collect();

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
                    &resource_sources,
                    &building_collisions,
                    &unit_collisions,
                    &environment_collisions,
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
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    building_collisions: &Vec<(Vec3, f32)>,
    unit_collisions: &Vec<(Vec3, f32)>,
    environment_collisions: &Vec<(Vec3, f32, EnvironmentObjectType)>,
    current_time: f32,
) {
    // Update strategy phase based on resources and units
    update_strategy_phase(strategy, resources, units, player_id);
    
    // Execute next goal if enough time has passed
    let time_since_last_action = current_time - strategy.last_building_time.max(strategy.last_unit_time);
    let should_execute = should_execute_next_goal(strategy, current_time);
    debug!("AI Player {} timing check: time_since_last={:.1}s, should_execute={}, queue_size={}", 
           player_id, time_since_last_action, should_execute, strategy.priority_queue.len());
    
    if should_execute {
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
            resource_sources,
            building_collisions,
            unit_collisions,
            environment_collisions,
            current_time,
        );
    }
    
    // Add new goals based on current situation
    add_dynamic_goals(strategy, resources, units, buildings, player_id);
}

fn update_strategy_phase(
    strategy: &mut PlayerStrategy,
    _resources: &PlayerResources,
    units: &Query<&RTSUnit>,
    player_id: u8,
) {
    let total_units = count_player_units(units, player_id);
    let worker_count = count_player_units_of_type(units, player_id, |unit_type| {
        matches!(unit_type, UnitType::WorkerAnt)
    });
    let military_count = count_player_units_of_type(units, player_id, |unit_type| {
        matches!(unit_type, UnitType::SoldierAnt | UnitType::BeetleKnight | UnitType::HunterWasp | UnitType::SpearMantis)
    });
    
    // GOAL-BASED STRATEGY PHASES: 1. Population Growth → 2. Enemy Elimination
    strategy.phase = match (worker_count, military_count, total_units) {
        // EARLY GAME: Population Growth Phase - Focus on workers and economy
        // Stay in early game until we have substantial worker population
        (workers, _, _) if workers < 20 => {
            // Switch to Economic strategy during population growth phase
            strategy.strategy_type = StrategyType::Economic;
            StrategyPhase::EarlyGame
        },
        
        // MID GAME: Transition Phase - Population achieved, building military  
        // Begin military buildup once we have strong worker base
        (workers, military, _) if workers >= 20 && military < 15 => {
            // Switch to Balanced strategy during transition
            strategy.strategy_type = StrategyType::Balanced;
            StrategyPhase::MidGame
        },
        
        // LATE GAME: Enemy Elimination Phase - Large army, focus on attack
        // Full military focus once both population and army goals are met
        (workers, military, total) if workers >= 20 && military >= 15 || total >= 40 => {
            // Switch to Military strategy for enemy elimination
            strategy.strategy_type = StrategyType::Military;
            StrategyPhase::LateGame
        },
        
        // Default fallback
        _ => StrategyPhase::EarlyGame,
    };
    
    // Log phase transitions for debugging
    match strategy.phase {
        StrategyPhase::EarlyGame => debug!("AI Player {} in POPULATION GROWTH phase: {} workers, {} military", 
                                         player_id, worker_count, military_count),
        StrategyPhase::MidGame => info!("🔄 AI Player {} transitioning: {} workers, {} military - Building army", 
                                       player_id, worker_count, military_count),
        StrategyPhase::LateGame => info!("⚔️ AI Player {} in ENEMY ELIMINATION phase: {} workers, {} military - ATTACKING!", 
                                        player_id, worker_count, military_count),
    }
}

fn should_execute_next_goal(strategy: &PlayerStrategy, current_time: f32) -> bool {
    let time_since_last_action = current_time - strategy.last_building_time.max(strategy.last_unit_time);
    
    // Very aggressive timing for rapid unit production
    match strategy.phase {
        StrategyPhase::EarlyGame => time_since_last_action > 0.8, // Much faster worker production
        StrategyPhase::MidGame => time_since_last_action > 0.6,   // Fast military buildup
        StrategyPhase::LateGame => time_since_last_action > 0.4,  // Constant unit production
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
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    building_collisions: &Vec<(Vec3, f32)>,
    unit_collisions: &Vec<(Vec3, f32)>,
    environment_collisions: &Vec<(Vec3, f32, EnvironmentObjectType)>,
    current_time: f32,
) {
    if strategy.priority_queue.is_empty() {
        debug!("AI Player {} priority queue is empty - no goals to execute", player_id);
        return;
    }
    
    let goal = strategy.priority_queue[0].clone();
    debug!("🎯 AI Player {} executing goal: {:?} (queue length: {})", player_id, goal, strategy.priority_queue.len());
    
    match goal {
        StrategyGoal::BuildWorker => {
            if try_build_unit(player_id, UnitType::WorkerAnt, resources, &game_costs, commands, meshes, materials, buildings) {
                strategy.priority_queue.remove(0);
                strategy.last_unit_time = current_time;
                info!("✅ AI Player {} queued WorkerAnt for production", player_id);
            } else {
                warn!("❌ AI Player {} failed to queue WorkerAnt - resources: N:{:.0} C:{:.0} M:{:.0} P:{:.0}, pop: {}/{}. This is blocking exponential growth!", 
                      player_id, resources.nectar, resources.chitin, resources.minerals, resources.pheromones,
                      resources.current_population, resources.max_population);
                // Remove the failed goal to prevent infinite retry - it will be re-added by add_dynamic_goals if conditions improve
                strategy.priority_queue.remove(0);
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
            if try_build_building(player_id, building_type.clone(), resources, game_costs, commands, meshes, materials, ai_resources, terrain_manager, terrain_settings, resource_sources, building_collisions, unit_collisions, environment_collisions) {
                strategy.priority_queue.remove(0);
                strategy.last_building_time = current_time;
                info!("✅ AI Player {} started building {:?}", player_id, building_type);
            } else {
                warn!("❌ AI Player {} failed to start building {:?} - removing from queue", player_id, building_type);
                strategy.priority_queue.remove(0); // Remove failed building attempts to prevent infinite retry
            }
        },
        StrategyGoal::AttackEnemy(target_player_id) => {
            // Execute attack strategy - mark goal as processed
            strategy.priority_queue.remove(0);
            info!("⚔️ AI Player {} executing ATTACK on Player {} - ENEMY ELIMINATION PHASE!", player_id, target_player_id);
            
            // Attack goal is handled by the combat AI system which will make units seek enemies
            // The combat AI already has aggressive behavior for Late Game phase
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
        if !resources.can_afford(cost) {
            debug!("❌ AI Player {} cannot afford {:?} - cost: {:?}, resources: N:{:.0} C:{:.0} M:{:.0} P:{:.0}", 
                   player_id, unit_type, cost, resources.nectar, resources.chitin, resources.minerals, resources.pheromones);
            return false;
        }
        if !resources.has_population_space() {
            debug!("❌ AI Player {} no population space for {:?} - pop: {}/{}", 
                   player_id, unit_type, resources.current_population, resources.max_population);
            return false;
        }
    } else {
        warn!("❌ AI Player {} no cost found for {:?} in game_costs", player_id, unit_type);
        return false;
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
        _ => {
            warn!("❌ AI Player {} unsupported unit type: {:?}", player_id, unit_type);
            return false;
        }
    };
    
    // Debug: Count available buildings
    let mut total_buildings = 0;
    let mut player_buildings = 0;
    let mut matching_buildings = 0;
    let mut complete_matching_buildings = 0;
    let mut available_buildings = 0;
    
    for (queue, building, unit) in buildings.iter() {
        total_buildings += 1;
        if unit.player_id == player_id {
            player_buildings += 1;
            if building.building_type == required_building {
                matching_buildings += 1;
                if building.is_complete {
                    complete_matching_buildings += 1;
                    if queue.queue.len() < 8 {
                        available_buildings += 1;
                    }
                }
            }
        }
    }
    
    debug!("🏗️ AI Player {} building check for {:?}: total={}, player={}, type_match={}, complete={}, available={}", 
           player_id, required_building, total_buildings, player_buildings, matching_buildings, complete_matching_buildings, available_buildings);
    
    // Find a suitable building to queue the unit in
    for (mut queue, building, unit) in buildings.iter_mut() {
        if unit.player_id == player_id && 
           building.building_type == required_building && 
           building.is_complete &&
           queue.queue.len() < 8 { // Allow larger queue for military production
            
            queue.queue.push(unit_type.clone());
            info!("✅ AI Player {} queued {:?} in {:?} (queue: {}/8)", player_id, unit_type, required_building, queue.queue.len());
            return true;
        }
    }
    
    warn!("❌ AI Player {} failed to find suitable {:?} building for {:?} - need: complete building owned by player with queue < 8", 
          player_id, required_building, unit_type);
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
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    building_collisions: &Vec<(Vec3, f32)>,
    unit_collisions: &Vec<(Vec3, f32)>,
    environment_collisions: &Vec<(Vec3, f32, EnvironmentObjectType)>,
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
    
    // STRATEGIC BUILDING PLACEMENT: Place buildings near resources for faster collection
    let base_x = (player_id as f32 - 1.0) * 200.0;
    let base_z = 0.0;
    let base_position = Vec3::new(base_x, 0.0, base_z);
    
    let mut building_position = None;
    
    // For ALL buildings, prioritize placement near relevant resources for better economy
    let should_place_near_resources = true; // Always try to place near resources first
    
    if should_place_near_resources {
        // Find the best resource cluster for this building type - prioritize Nectar for most buildings
        let target_resource_types = match building_type {
            BuildingType::FungalGarden => vec![ResourceType::Nectar], // Nectar processing
            BuildingType::StorageChamber => vec![ResourceType::Nectar, ResourceType::Minerals, ResourceType::Chitin], // General storage
            BuildingType::Queen => vec![ResourceType::Nectar], // Queen is critical - prioritize Nectar (food) access
            BuildingType::Nursery => vec![ResourceType::Nectar, ResourceType::Chitin], // Housing needs Nectar and building materials
            BuildingType::WarriorChamber => vec![ResourceType::Nectar, ResourceType::Pheromones], // Military units need food and special resources
            BuildingType::HunterChamber => vec![ResourceType::Nectar, ResourceType::Pheromones], // Flying units need food and special resources
            _ => vec![ResourceType::Nectar], // Default: all buildings benefit from Nectar access
        };
        
        // Find relevant resource sources
        let mut relevant_resources = Vec::new();
        for (_, source, transform) in resource_sources.iter() {
            if target_resource_types.contains(&source.resource_type) && source.amount > 0.0 {
                let distance_to_base = transform.translation.distance(base_position);
                // Expanded search radius to find more resources for better placement options
                if distance_to_base < 250.0 { // Increased from 150 to 250 for more resource options
                    relevant_resources.push((transform.translation, source.resource_type.clone(), distance_to_base));
                }
            }
        }
        
        // Sort by distance to base to prioritize closer resources
        relevant_resources.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
        
        // Try to place building near the best resource clusters
        for (resource_pos, _resource_type, _distance) in relevant_resources.iter().take(5) { // Try top 5 closest resources (increased from 3)
            // Try positions around this resource at optimal distance (not too close, not too far)
            for ring in 0..4 { // Increased attempts (0..4 instead of 0..3)
                for attempt in 0..12 { // More placement attempts per ring (12 instead of 8)
                    let angle = (attempt as f32) * std::f32::consts::PI / 6.0; // More precise angles
                    let radius = 20.0 + (ring as f32 * 12.0); // 20-56 range - closer to resources for faster gathering
                    
                    let pos_x = resource_pos.x + radius * angle.cos();
                    let pos_z = resource_pos.z + radius * angle.sin();
                    
                    let position = get_terrain_position(pos_x, pos_z, 0.0);
                    
                    // Validate position is reasonable and not too far from base
                    let distance_to_base = position.distance(base_position);
                    if position.x.abs() < 1000.0 && position.z.abs() < 1000.0 && distance_to_base < 300.0 { // Increased from 200 to 300 for more flexibility
                        // Get building radius for collision checking
                        let building_radius = get_building_collision_radius(&building_type);
                        
                        // Validate that this position doesn't overlap with existing buildings/units
                        if validate_ai_building_placement(
                            position,
                            building_radius,
                            building_collisions,
                            unit_collisions,
                            environment_collisions,
                        ) {
                            building_position = Some(position);
                            info!("🏗️ AI Player {} placing {:?} near {:?} resource at distance {:.1} from base", 
                                  player_id, building_type, _resource_type, distance_to_base);
                            break;
                        } else {
                            debug!("AI Player {} resource-based position validation failed for {:?} at {:?}", player_id, building_type, position);
                        }
                    }
                }
                if building_position.is_some() {
                    break;
                }
            }
            if building_position.is_some() {
                break;
            }
        }
    }
    
    // Fallback: if no resource-based position found, use traditional base-centered placement
    if building_position.is_none() {
        // Add some randomness to make each building placement unique
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        building_type.hash(&mut hasher);
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let seed = hasher.finish();
        
        let random_offset_x = (seed % 100) as f32 - 50.0; // Smaller randomization
        let random_offset_z = ((seed / 100) % 100) as f32 - 50.0;
        
        // Try positions in expanding rings around the base
        for ring in 0..4 {
            for attempt in 0..8 {
                let angle = (attempt as f32) * std::f32::consts::PI / 4.0;
                let radius = 30.0 + (ring as f32 * 20.0); // Closer spacing around base
                
                let pos_x = base_x + random_offset_x + radius * angle.cos();
                let pos_z = base_z + random_offset_z + radius * angle.sin();
                
                let position = get_terrain_position(pos_x, pos_z, 0.0);
                
                if position.x.abs() < 1000.0 && position.z.abs() < 1000.0 {
                    // Get building radius for collision checking
                    let building_radius = get_building_collision_radius(&building_type);
                    
                    // Validate that this position doesn't overlap with existing buildings/units
                    if validate_ai_building_placement(
                        position,
                        building_radius,
                        building_collisions,
                        unit_collisions,
                        environment_collisions,
                    ) {
                        building_position = Some(position);
                        info!("🏗️ AI Player {} placing {:?} near base (validated position)", player_id, building_type);
                        break;
                    } else {
                        debug!("AI Player {} position validation failed for {:?} at {:?}", player_id, building_type, position);
                    }
                }
            }
            if building_position.is_some() {
                break;
            }
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
        matches!(unit_type, UnitType::SoldierAnt | UnitType::HunterWasp | UnitType::BeetleKnight | UnitType::SpearMantis)
    });
    
    // GOAL-BASED DYNAMIC PLANNING: 1. Population Growth → 2. Enemy Elimination
    match strategy.phase {
        
        // PHASE 1: POPULATION GROWTH - Maximum worker priority, housing support
        StrategyPhase::EarlyGame => {
            // CRITICAL PREREQUISITE: Queen buildings for worker production
            let queen_count = buildings.iter().filter(|(_, building, unit)| {
                unit.player_id == player_id && building.building_type == BuildingType::Queen
            }).count();
            
            // Absolute priority: Must have Queen buildings to produce workers
            if queen_count == 0 && !strategy.priority_queue.iter().any(|goal| {
                matches!(goal, StrategyGoal::ConstructBuilding(BuildingType::Queen))
            }) {
                strategy.priority_queue.insert(0, StrategyGoal::ConstructBuilding(BuildingType::Queen));
                info!("🔥 CRITICAL: AI Player {} has no Queen buildings - adding to priority queue", player_id);
            }
            
            // Build multiple Queen buildings for exponential worker production
            if queen_count < 3 && !strategy.priority_queue.iter().any(|goal| { // Reduced from 5 to 3 for balance
                matches!(goal, StrategyGoal::ConstructBuilding(BuildingType::Queen))
            }) {
                strategy.priority_queue.insert(0, StrategyGoal::ConstructBuilding(BuildingType::Queen));
                info!("📈 AI Player {} needs more Queen buildings ({} < 3) - adding to priority queue", player_id, queen_count);
            }
            
            // SUPREME PRIORITY: Workers for exponential population growth
            if worker_count < 40 { // Massive population target for true exponential growth
                let workers_needed = (40 - worker_count).min(10); // Add up to 10 workers at once for exponential growth
                let current_worker_goals = strategy.priority_queue.iter().filter(|goal| 
                    matches!(goal, StrategyGoal::BuildWorker)).count() as u32;
                
                let goals_to_add = workers_needed.saturating_sub(current_worker_goals);
                if goals_to_add > 0 {
                    info!("📈 AI Player {} adding {} worker goals (current: {}, needed: {}, in queue: {})", 
                          player_id, goals_to_add, worker_count, workers_needed, current_worker_goals);
                }
                
                // Spam workers at the very front of the queue for exponential growth
                for _ in 0..goals_to_add {
                    strategy.priority_queue.insert(0, StrategyGoal::BuildWorker);
                }
            }
            
            // Housing for population support - prioritize Nursery buildings
            let nursery_count = buildings.iter().filter(|(_, building, unit)| {
                unit.player_id == player_id && building.building_type == BuildingType::Nursery
            }).count();
            
            if nursery_count < 3 && !strategy.priority_queue.iter().any(|goal| {
                matches!(goal, StrategyGoal::ConstructBuilding(BuildingType::Nursery))
            }) {
                strategy.priority_queue.insert(1, StrategyGoal::ConstructBuilding(BuildingType::Nursery));
            }
            
            // Basic resource economy for population sustenance
            let economic_buildings = [BuildingType::FungalGarden, BuildingType::StorageChamber];
            for building_type in &economic_buildings {
                let has_building = buildings.iter().any(|(_, building, unit)| {
                    unit.player_id == player_id && building.building_type == *building_type
                });
                
                if !has_building && !strategy.priority_queue.iter().any(|goal| {
                    matches!(goal, StrategyGoal::ConstructBuilding(ref bt) if bt == building_type)
                }) {
                    strategy.priority_queue.push(StrategyGoal::ConstructBuilding(building_type.clone()));
                }
            }
        },
        
        // PHASE 2: TRANSITION - Balance population maintenance with military buildup
        StrategyPhase::MidGame => {
            // Ensure Queen buildings for continued worker production
            let queen_count = buildings.iter().filter(|(_, building, unit)| {
                unit.player_id == player_id && building.building_type == BuildingType::Queen
            }).count();
            
            if queen_count < 3 && !strategy.priority_queue.iter().any(|goal| {
                matches!(goal, StrategyGoal::ConstructBuilding(BuildingType::Queen))
            }) {
                strategy.priority_queue.insert(0, StrategyGoal::ConstructBuilding(BuildingType::Queen));
            }
            
            // Maintain worker population while building military infrastructure
            if worker_count < 20 {
                let workers_needed = (20 - worker_count).min(2);
                for _ in 0..workers_needed {
                    strategy.priority_queue.insert(0, StrategyGoal::BuildWorker);
                }
            }
            
            // Military infrastructure for army buildup
            let military_buildings = [BuildingType::WarriorChamber, BuildingType::HunterChamber];
            for building_type in &military_buildings {
                let has_building = buildings.iter().any(|(_, building, unit)| {
                    unit.player_id == player_id && building.building_type == *building_type
                });
                
                if !has_building && !strategy.priority_queue.iter().any(|goal| {
                    matches!(goal, StrategyGoal::ConstructBuilding(ref bt) if bt == building_type)
                }) {
                    // High priority for military buildings in transition phase
                    strategy.priority_queue.insert(1, StrategyGoal::ConstructBuilding(building_type.clone()));
                }
            }
            
            // Begin military unit production
            if military_count < 15 {
                let units_needed = (15 - military_count).min(3);
                let mut units_added = 0;
                for unit_type in &strategy.military_targets.preferred_unit_types {
                    if units_added >= units_needed { break; }
                    strategy.priority_queue.push(StrategyGoal::BuildMilitaryUnit(unit_type.clone()));
                    units_added += 1;
                }
            }
        },
        
        // PHASE 3: ENEMY ELIMINATION - Full military focus, mass army production
        StrategyPhase::LateGame => {
            // Ensure Queen buildings remain available for worker production
            let queen_count = buildings.iter().filter(|(_, building, unit)| {
                unit.player_id == player_id && building.building_type == BuildingType::Queen
            }).count();
            
            if queen_count < 1 && !strategy.priority_queue.iter().any(|goal| {
                matches!(goal, StrategyGoal::ConstructBuilding(BuildingType::Queen))
            }) {
                strategy.priority_queue.insert(0, StrategyGoal::ConstructBuilding(BuildingType::Queen));
            }
            
            // Maintain minimal worker population for economy
            if worker_count < 15 {
                strategy.priority_queue.insert(0, StrategyGoal::BuildWorker);
            }
            
            // MASSIVE MILITARY PRODUCTION for enemy elimination
            if military_count < strategy.military_targets.desired_military_units {
                let units_needed = (strategy.military_targets.desired_military_units - military_count).min(6); // Up to 6 units at once
                
                // Prioritize siege units for enemy elimination
                let elimination_unit_priority = vec![
                    UnitType::BeetleKnight,  // Heavy assault
                    UnitType::SpearMantis,   // Anti-building
                    UnitType::SoldierAnt,    // Core army
                    UnitType::HunterWasp,    // Fast raiders
                ];
                
                let mut units_added = 0;
                for unit_type in &elimination_unit_priority {
                    if units_added >= units_needed { break; }
                    
                    let current_count = strategy.priority_queue.iter().filter(|goal| {
                        matches!(goal, StrategyGoal::BuildMilitaryUnit(ref ut) if ut == unit_type)
                    }).count();
                    
                    if current_count < 3 { // Allow up to 3 of each unit type for massive army
                        strategy.priority_queue.insert(0, StrategyGoal::BuildMilitaryUnit(unit_type.clone())); // High priority
                        units_added += 1;
                    }
                }
            }
            
            // Additional military infrastructure if needed
            let warrior_chambers = buildings.iter().filter(|(_, building, unit)| {
                unit.player_id == player_id && building.building_type == BuildingType::WarriorChamber
            }).count();
            
            if warrior_chambers < 2 && !strategy.priority_queue.iter().any(|goal| {
                matches!(goal, StrategyGoal::ConstructBuilding(BuildingType::WarriorChamber))
            }) {
                strategy.priority_queue.insert(1, StrategyGoal::ConstructBuilding(BuildingType::WarriorChamber));
            }
            
            // ENEMY ELIMINATION GOAL: Add attack goals when we have sufficient army
            // First check if there are any enemies left to attack
            let enemy_count = count_player_units(units, 1); // Count Player 1 units
            
            if enemy_count == 0 {
                // VICTORY! All enemies eliminated - clear attack goals and celebrate
                strategy.priority_queue.retain(|goal| !matches!(goal, StrategyGoal::AttackEnemy(_)));
                info!("🏆 VICTORY! AI Player {} has eliminated all enemies - {} workers, {} military units celebrating!", 
                      player_id, worker_count, military_count);
            } else if military_count >= 15 && !strategy.priority_queue.iter().any(|goal| {
                matches!(goal, StrategyGoal::AttackEnemy(_))
            }) {
                // AI Player 2 attacks Player 1 (the human player)
                strategy.priority_queue.insert(0, StrategyGoal::AttackEnemy(1));
                info!("🎯 AI Player {} adding ATTACK GOAL - targeting Player 1 for elimination! (Enemy units remaining: {})", 
                      player_id, enemy_count);
            }
        },
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

/// AI-specific building placement validation that works with Vec data instead of queries
fn validate_ai_building_placement(
    position: Vec3,
    building_radius: f32,
    building_collisions: &Vec<(Vec3, f32)>,
    unit_collisions: &Vec<(Vec3, f32)>,
    environment_collisions: &Vec<(Vec3, f32, EnvironmentObjectType)>,
) -> bool {
    use crate::core::constants::building_placement::*;
    
    // Check against existing buildings - prevent overlap
    for (building_pos, collision_radius) in building_collisions.iter() {
        let distance = position.distance(*building_pos);
        let min_distance = building_radius + collision_radius + MIN_SPACING_BETWEEN_BUILDINGS;
        
        if distance < min_distance {
            return false; // Too close to another building
        }
    }
    
    // Check against units - buildings shouldn't be placed on top of units
    for (unit_pos, collision_radius) in unit_collisions.iter() {
        let distance = position.distance(*unit_pos);
        let min_distance = building_radius + collision_radius + MIN_SPACING_FROM_UNITS;
        
        if distance < min_distance {
            return false; // Too close to a unit
        }
    }
    
    // Check against environment objects - avoid placing buildings on obstacles
    for (env_pos, collision_radius, object_type) in environment_collisions.iter() {
        let distance = position.distance(*env_pos);
        
        // Apply larger spacing for mushrooms to prevent buildings from being placed too close
        let extra_spacing = match object_type {
            EnvironmentObjectType::Mushrooms => 80.0, // Much wider radius for mushrooms
            _ => MIN_SPACING_FROM_ENVIRONMENT, // Normal spacing for other objects
        };
        let min_distance = building_radius + collision_radius + extra_spacing;
        
        if distance < min_distance {
            return false; // Too close to environment object
        }
    }
    
    true // Position is valid
}