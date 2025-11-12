use crate::core::components::*;
use crate::core::resources::*;
use crate::ui::placement::get_building_collision_radius;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use std::collections::HashMap;

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

#[derive(Resource, Default)]
pub struct LogRateLimiter {
    last_log_times: HashMap<String, f32>,
}

impl LogRateLimiter {
    pub fn should_log(&mut self, key: &str, current_time: f32, interval: f32) -> bool {
        if let Some(last_time) = self.last_log_times.get(key) {
            if current_time - last_time < interval {
                return false;
            }
        }
        self.last_log_times.insert(key.to_string(), current_time);
        true
    }
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
    pub last_building_failure_time: f32,
    pub priority_queue: Vec<StrategyGoal>,
    pub phase: StrategyPhase,
    pub previous_phase: Option<StrategyPhase>,
    pub economy_targets: EconomyTargets,
    pub military_targets: MilitaryTargets,
}

#[derive(Debug, Clone)]
pub enum StrategyType {
    #[allow(dead_code)]
    Economic, // Focus on resource gathering and infrastructure
    #[allow(dead_code)]
    Military, // Focus on unit production and combat
    Balanced, // Mix of both
}

#[derive(Debug, Clone, PartialEq)]
pub enum StrategyPhase {
    EarlyGame, // Build workers, basic economy
    MidGame,   // Expand, military production
    LateGame,  // Advanced units, technology
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
        strategies.insert(
            2,
            PlayerStrategy {
                strategy_type: StrategyType::Economic, // Population growth requires economic focus initially
                last_building_time: 0.0,
                last_unit_time: 0.0,
                last_building_failure_time: 0.0,
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
                previous_phase: None,
                economy_targets: EconomyTargets {
                    desired_workers: 25, // Massive population target - increased from 15 to 25
                    next_building: Some(BuildingType::Nursery), // Housing priority for population
                    resource_priorities: vec![
                        ResourceType::Nectar,     // For population growth (worker production)
                        ResourceType::Chitin,     // For housing construction
                        ResourceType::Minerals,   // For infrastructure
                        ResourceType::Pheromones, // Support resource
                    ],
                },
                military_targets: MilitaryTargets {
                    desired_military_units: 35, // MASSIVE army for enemy elimination (increased from 20 to 35)
                    preferred_unit_types: vec![
                        UnitType::DragonFly,       // STRONGEST - 5x speed air superiority
                        UnitType::BeetleKnight,    // Heavy assault for enemy elimination
                        UnitType::SpearMantis,     // Anti-building siege units
                        UnitType::BatteringBeetle, // Siege warfare specialists
                        UnitType::HunterWasp,      // Fast air raiders
                        UnitType::EliteSpider,     // Elite predator units
                        UnitType::SoldierAnt,      // Core army unit
                    ],
                    next_military_building: Some(BuildingType::WarriorChamber),
                },
            },
        );

        Self { strategies }
    }
}

pub fn ai_strategy_system(
    mut ai_strategy: ResMut<AIStrategy>,
    mut ai_resources: ResMut<AIResources>,
    mut log_limiter: ResMut<LogRateLimiter>,
    game_costs: Res<GameCosts>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    units: Query<&RTSUnit>,
    mut buildings: Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    mut worker_queries: ParamSet<(
        Query<
            (
                Entity,
                &mut ResourceGatherer,
                &mut Movement,
                &Transform,
                &RTSUnit,
            ),
            (With<ResourceGatherer>, With<RTSUnit>),
        >,
        Query<(Entity, &mut Movement, &RTSUnit), (With<RTSUnit>, Without<ConstructionTask>)>,
    )>,
    resource_sources: Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    collision_query: Query<(
        &Transform,
        &CollisionRadius,
        Option<&Building>,
        Option<&RTSUnit>,
        Option<&EnvironmentObject>,
    )>,
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
                let building_collisions: Vec<(Vec3, f32)> = collision_query
                    .iter()
                    .filter_map(|(transform, collision, building, _, _)| {
                        building.map(|_| (transform.translation, collision.radius))
                    })
                    .collect();

                let unit_collisions: Vec<(Vec3, f32)> = collision_query
                    .iter()
                    .filter_map(|(transform, collision, _, unit, _)| {
                        unit.map(|_| (transform.translation, collision.radius))
                    })
                    .collect();

                let environment_collisions: Vec<(Vec3, f32, EnvironmentObjectType)> =
                    collision_query
                        .iter()
                        .filter_map(|(transform, collision, _, _, env_obj)| {
                            env_obj.map(|e| {
                                (
                                    transform.translation,
                                    collision.radius,
                                    e.object_type.clone(),
                                )
                            })
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
                    &mut log_limiter,
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
    log_limiter: &mut LogRateLimiter,
) {
    // Update strategy phase based on resources and units
    update_strategy_phase(strategy, resources, units, player_id);

    // Execute next goal if enough time has passed
    let time_since_last_action =
        current_time - strategy.last_building_time.max(strategy.last_unit_time);
    let should_execute = should_execute_next_goal(strategy, current_time);
    debug!(
        "AI Player {} timing check: time_since_last={:.1}s, should_execute={}, queue_size={}",
        player_id,
        time_since_last_action,
        should_execute,
        strategy.priority_queue.len()
    );

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
            log_limiter,
        );
    }

    // Add new goals based on current situation
    add_dynamic_goals(
        strategy,
        resources,
        units,
        buildings,
        player_id,
        current_time,
        log_limiter,
    );
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
        matches!(
            unit_type,
            UnitType::SoldierAnt
                | UnitType::BeetleKnight
                | UnitType::HunterWasp
                | UnitType::SpearMantis
                | UnitType::DragonFly
                | UnitType::BatteringBeetle
                | UnitType::EliteSpider
                | UnitType::ScoutAnt
                | UnitType::AcidSpitter
                | UnitType::DefenderBug
                | UnitType::SpiderHunter
                | UnitType::WolfSpider
                | UnitType::Ladybug
                | UnitType::LegBeetle
                | UnitType::Scorpion
                | UnitType::TermiteWarrior
                | UnitType::Stinkbug
        )
    });

    // GOAL-BASED STRATEGY PHASES: 1. Population Growth → 2. Enemy Elimination
    strategy.phase = match (worker_count, military_count, total_units) {
        // EARLY GAME: Population Growth Phase - Focus on workers and economy
        // Stay in early game until we have substantial worker population
        (workers, _, _) if workers < 20 => {
            // Switch to Economic strategy during population growth phase
            strategy.strategy_type = StrategyType::Economic;
            StrategyPhase::EarlyGame
        }

        // MID GAME: Transition Phase - Population achieved, building military
        // Begin military buildup once we have strong worker base
        (workers, military, _) if workers >= 15 && military < 20 => {
            // Lower worker requirement, higher military target
            // Switch to Balanced strategy during transition
            strategy.strategy_type = StrategyType::Balanced;
            StrategyPhase::MidGame
        }

        // LATE GAME: Enemy Elimination Phase - Large army, focus on attack
        // Full military focus once both population and army goals are met - ENHANCED for faster aggression
        (workers, military, total) if (workers >= 15 && military >= 12) || total >= 30 => {
            // Lower thresholds for faster attack
            // Switch to Military strategy for enemy elimination
            strategy.strategy_type = StrategyType::Military;
            StrategyPhase::LateGame
        }

        // Default fallback
        _ => StrategyPhase::EarlyGame,
    };

    // Rate limit phase logging to every 10 seconds to reduce spam
    use std::collections::HashMap;
    use std::sync::Mutex;
    static PHASE_LOG_TIMES: std::sync::LazyLock<Mutex<HashMap<u8, f32>>> =
        std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

    if let Ok(mut log_times) = PHASE_LOG_TIMES.lock() {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f32();

        let last_log = log_times.get(&player_id).copied().unwrap_or(0.0);
        if current_time - last_log > 10.0 {
            match strategy.phase {
                StrategyPhase::EarlyGame => debug!("AI Player {} in POPULATION GROWTH phase: {} workers, {} military", 
                                                 player_id, worker_count, military_count),
                StrategyPhase::MidGame => debug!("🔄 AI Player {} transitioning: {} workers, {} military - Building army", 
                                                player_id, worker_count, military_count),
                StrategyPhase::LateGame => debug!("⚔️ AI Player {} in ENEMY ELIMINATION phase: {} workers, {} military - ATTACKING!", 
                                                player_id, worker_count, military_count),
            }
            log_times.insert(player_id, current_time);
        }
    }
}

fn should_execute_next_goal(strategy: &PlayerStrategy, current_time: f32) -> bool {
    let time_since_last_action =
        current_time - strategy.last_building_time.max(strategy.last_unit_time);

    // SUPER AGGRESSIVE timing for rapid military unit production
    match strategy.phase {
        StrategyPhase::EarlyGame => time_since_last_action > 0.6, // Faster worker production
        StrategyPhase::MidGame => time_since_last_action > 0.4,   // Very fast military buildup
        StrategyPhase::LateGame => time_since_last_action > 0.2, // CONSTANT military production for overwhelming force
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
    log_limiter: &mut LogRateLimiter,
) {
    if strategy.priority_queue.is_empty() {
        debug!(
            "AI Player {} priority queue is empty - no goals to execute",
            player_id
        );
        return;
    }

    let goal = strategy.priority_queue[0].clone();
    debug!(
        "🎯 AI Player {} executing goal: {:?} (queue length: {})",
        player_id,
        goal,
        strategy.priority_queue.len()
    );

    match goal {
        StrategyGoal::BuildWorker => {
            if try_build_unit(
                player_id,
                UnitType::WorkerAnt,
                resources,
                &game_costs,
                commands,
                meshes,
                materials,
                buildings,
                current_time,
                log_limiter,
            ) {
                strategy.priority_queue.remove(0);
                strategy.last_unit_time = current_time;
                info!("✅ AI Player {} queued WorkerAnt for production", player_id);
            } else {
                let log_key = format!("worker_queue_failed_player_{}", player_id);
                if log_limiter.should_log(&log_key, current_time, 10.0) {
                    warn!("❌ AI Player {} failed to queue WorkerAnt - resources: N:{:.0} C:{:.0} M:{:.0} P:{:.0}, pop: {}/{}. This is blocking exponential growth!", 
                          player_id, resources.nectar, resources.chitin, resources.minerals, resources.pheromones,
                          resources.current_population, resources.max_population);
                }
                // Remove the failed goal to prevent infinite retry - it will be re-added by add_dynamic_goals if conditions improve
                strategy.priority_queue.remove(0);
            }
        }
        StrategyGoal::BuildMilitaryUnit(unit_type) => {
            if try_build_unit(
                player_id,
                unit_type.clone(),
                resources,
                &game_costs,
                commands,
                meshes,
                materials,
                buildings,
                current_time,
                log_limiter,
            ) {
                strategy.priority_queue.remove(0);
                strategy.last_unit_time = current_time;
                info!(
                    "AI Player {} queued {:?} for production",
                    player_id, unit_type
                );
            } else {
                // Remove failed military unit goal to prevent blocking the queue
                strategy.priority_queue.remove(0);
                debug!(
                    "❌ AI Player {} failed to queue {:?} - removing from queue",
                    player_id, unit_type
                );
            }
        }
        StrategyGoal::ConstructBuilding(building_type) => {
            if try_build_building(
                player_id,
                building_type.clone(),
                resources,
                game_costs,
                commands,
                meshes,
                materials,
                ai_resources,
                terrain_manager,
                terrain_settings,
                resource_sources,
                building_collisions,
                unit_collisions,
                environment_collisions,
            ) {
                strategy.priority_queue.remove(0);
                strategy.last_building_time = current_time;
                info!(
                    "✅ AI Player {} started building {:?}",
                    player_id, building_type
                );
            } else {
                // Rate limit building failure logging to prevent spam AND record failure for cooldown
                use std::collections::HashMap;
                use std::sync::Mutex;
                static BUILDING_FAILURE_LOG: std::sync::LazyLock<
                    Mutex<HashMap<(u8, BuildingType), f32>>,
                > = std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
                static BUILDING_COOLDOWNS: std::sync::LazyLock<
                    Mutex<HashMap<(u8, BuildingType), f32>>,
                > = std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

                let key = (player_id, building_type.clone());

                // Record failure time for cooldown mechanism
                if let Ok(mut cooldowns) = BUILDING_COOLDOWNS.lock() {
                    cooldowns.insert(key.clone(), current_time);
                }

                let should_log = if let Ok(mut log_times) = BUILDING_FAILURE_LOG.lock() {
                    let last_log = log_times.get(&key).copied().unwrap_or(0.0);
                    if current_time - last_log > 5.0 {
                        // Only log every 5 seconds per building type
                        log_times.insert(key, current_time);
                        true
                    } else {
                        false
                    }
                } else {
                    true // Log if mutex fails
                };

                if should_log {
                    warn!(
                        "❌ AI Player {} failed to start building {:?} - removing from queue",
                        player_id, building_type
                    );
                }

                strategy.priority_queue.remove(0); // Remove failed building attempts to prevent infinite retry
                strategy.last_building_failure_time = current_time; // Track failure time to prevent immediate retries
            }
        }
        StrategyGoal::AttackEnemy(target_player_id) => {
            // Execute attack strategy - mark goal as processed
            strategy.priority_queue.remove(0);
            debug!(
                "⚔️ AI Player {} executing ATTACK on Player {} - ENEMY ELIMINATION PHASE!",
                player_id, target_player_id
            );

            // Attack goal is handled by the combat AI system which will make units seek enemies
            // The combat AI already has aggressive behavior for Late Game phase
        }
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
    current_time: f32,
    log_limiter: &mut LogRateLimiter,
) -> bool {
    // Check if we can afford the unit
    if let Some(cost) = game_costs.unit_costs.get(&unit_type) {
        if !resources.can_afford(cost) {
            debug!("❌ AI Player {} cannot afford {:?} - cost: {:?}, resources: N:{:.0} C:{:.0} M:{:.0} P:{:.0}", 
                   player_id, unit_type, cost, resources.nectar, resources.chitin, resources.minerals, resources.pheromones);
            return false;
        }
        if !resources.has_population_space() {
            debug!(
                "❌ AI Player {} no population space for {:?} - pop: {}/{}",
                player_id, unit_type, resources.current_population, resources.max_population
            );
            return false;
        }
    } else {
        warn!(
            "❌ AI Player {} no cost found for {:?} in game_costs",
            player_id, unit_type
        );
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
        UnitType::LegBeetle => BuildingType::WarriorChamber,
        UnitType::TermiteWarrior => BuildingType::WarriorChamber,
        UnitType::Scorpion => BuildingType::WarriorChamber,
        UnitType::Stinkbug => BuildingType::WarriorChamber,
        // Spider/Predator units from HunterChamber
        UnitType::EliteSpider => BuildingType::HunterChamber,
        UnitType::DefenderBug => BuildingType::HunterChamber,
        UnitType::SpiderHunter => BuildingType::HunterChamber,
        UnitType::WolfSpider => BuildingType::HunterChamber,
        UnitType::Ladybug => BuildingType::HunterChamber,
        UnitType::LadybugScout => BuildingType::HunterChamber,
        // Flying units from Nursery
        UnitType::HoneyBee => BuildingType::Nursery,
        UnitType::Housefly => BuildingType::Nursery,
        // Worker variants
        UnitType::TermiteWorker => BuildingType::Queen,
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

    // Use the shared overflow system for AI unit queuing
    if crate::rts::production::try_queue_unit_with_overflow(
        unit_type.clone(),
        required_building.clone(),
        buildings,
        player_id,
    ) {
        return true;
    }

    let log_key = format!(
        "building_not_found_{}_{:?}_{:?}",
        player_id, required_building, unit_type
    );
    if log_limiter.should_log(&log_key, current_time, 10.0) {
        warn!(
            "❌ AI Player {} failed to find suitable {:?} building for {:?} - no buildings available",
            player_id, required_building, unit_type
        );
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
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    building_collisions: &Vec<(Vec3, f32)>,
    unit_collisions: &Vec<(Vec3, f32)>,
    environment_collisions: &Vec<(Vec3, f32, EnvironmentObjectType)>,
) -> bool {
    if !can_afford_building(resources, game_costs, &building_type) {
        return false;
    }

    let terrain_helper = TerrainHelper::new(terrain_manager, terrain_settings);
    let placement_context = BuildingPlacementContext::new(
        player_id,
        building_type.clone(),
        building_collisions,
        resource_sources,
    );

    let Some(building_position) = find_optimal_building_position(
        placement_context,
        &terrain_helper,
        unit_collisions,
        environment_collisions,
    ) else {
        warn!(
            "AI Player {} could not find suitable position for {:?}",
            player_id, building_type
        );
        return false;
    };

    if !deduct_building_resources(player_id, &building_type, game_costs, ai_resources) {
        return false;
    }

    create_building_site(commands, building_type, building_position, player_id);
    true
}

fn add_dynamic_goals(
    strategy: &mut PlayerStrategy,
    _resources: &PlayerResources,
    units: &Query<&RTSUnit>,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_id: u8,
    current_time: f32,
    log_limiter: &mut LogRateLimiter,
) {
    let unit_counts = calculate_unit_counts(units, player_id);

    match strategy.phase {
        StrategyPhase::EarlyGame => {
            handle_early_game_goals(
                strategy,
                buildings,
                player_id,
                current_time,
                &unit_counts,
                log_limiter,
            );
        }
        StrategyPhase::MidGame => {
            handle_mid_game_goals(
                strategy,
                buildings,
                player_id,
                current_time,
                &unit_counts,
                log_limiter,
            );
        }
        StrategyPhase::LateGame => {
            handle_late_game_goals(
                strategy,
                buildings,
                units,
                player_id,
                current_time,
                &unit_counts,
                log_limiter,
            );
        }
    }
}

fn count_player_units(units: &Query<&RTSUnit>, player_id: u8) -> u32 {
    units
        .iter()
        .filter(|unit| unit.player_id == player_id)
        .count() as u32
}

fn count_player_units_of_type<F>(units: &Query<&RTSUnit>, player_id: u8, filter: F) -> u32
where
    F: Fn(&UnitType) -> bool,
{
    units
        .iter()
        .filter(|unit| {
            unit.player_id == player_id && unit.unit_type.as_ref().map_or(false, |ut| filter(ut))
        })
        .count() as u32
}

fn assign_workers_to_resources(
    player_id: u8,
    strategy: &PlayerStrategy,
    workers: &mut Query<
        (
            Entity,
            &mut ResourceGatherer,
            &mut Movement,
            &Transform,
            &RTSUnit,
        ),
        (With<ResourceGatherer>, With<RTSUnit>),
    >,
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
) {
    // Find suitable resource sources for each resource type
    for priority_resource in &strategy.economy_targets.resource_priorities {
        let suitable_sources: Vec<(Entity, Vec3)> = resource_sources
            .iter()
            .filter(|(_, source, _)| {
                source.resource_type == *priority_resource && source.amount > 0.0
            })
            .map(|(entity, _, transform)| (entity, transform.translation))
            .collect();

        if suitable_sources.is_empty() {
            continue;
        }

        // Find idle workers for this player and assign them to nearest resource
        for (_worker_entity, mut gatherer, mut movement, worker_transform, unit) in
            workers.iter_mut()
        {
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
                        debug!(
                            "AI Player {} assigned worker {} to harvest {:?} at distance {:.1}",
                            player_id, unit.unit_id, priority_resource, closest_distance
                        );
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
    mut workers: Query<
        (
            Entity,
            &mut ResourceGatherer,
            &mut Movement,
            &Transform,
            &RTSUnit,
        ),
        (
            With<ResourceGatherer>,
            With<RTSUnit>,
            Added<ResourceGatherer>,
        ),
    >,
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
                let distance = worker_transform
                    .translation
                    .distance(source_transform.translation);
                // Prioritize Nectar resources for new workers
                let priority_bonus = if source.resource_type == ResourceType::Nectar {
                    -50.0
                } else {
                    0.0
                };
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
                    BuildingType::Queen | BuildingType::StorageChamber | BuildingType::Nursery => {
                        let distance = worker_transform
                            .translation
                            .distance(building_transform.translation);
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
    construction_workers: &mut Query<
        (Entity, &mut Movement, &RTSUnit),
        (With<RTSUnit>, Without<ConstructionTask>),
    >,
    building_sites: &mut Query<(Entity, &mut BuildingSite), With<BuildingSite>>,
) {
    // Find unassigned building sites for this player that haven't started construction yet
    for (site_entity, mut site) in building_sites.iter_mut() {
        if site.player_id == player_id
            && site.assigned_worker.is_none()
            && !site.site_reserved
            && !site.construction_started
        {
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

                    info!(
                        "✅ AI Player {} assigned worker {} to construct {:?} at {:?}",
                        player_id, unit.unit_id, site.building_type, site.position
                    );

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
            _ => MIN_SPACING_FROM_ENVIRONMENT,        // Normal spacing for other objects
        };
        let min_distance = building_radius + collision_radius + extra_spacing;

        if distance < min_distance {
            return false; // Too close to environment object
        }
    }

    true // Position is valid
}

struct TerrainHelper<'a> {
    terrain_manager: &'a crate::world::terrain_v2::TerrainChunkManager,
    terrain_settings: &'a crate::world::terrain_v2::TerrainSettings,
}

impl<'a> TerrainHelper<'a> {
    fn new(
        terrain_manager: &'a crate::world::terrain_v2::TerrainChunkManager,
        terrain_settings: &'a crate::world::terrain_v2::TerrainSettings,
    ) -> Self {
        Self {
            terrain_manager,
            terrain_settings,
        }
    }

    fn get_terrain_position(&self, x: f32, z: f32, height_offset: f32) -> Vec3 {
        let terrain_height = crate::world::terrain_v2::sample_terrain_height(
            x,
            z,
            &self.terrain_manager.noise_generator,
            &self.terrain_settings,
        );
        Vec3::new(x, terrain_height + height_offset, z)
    }
}

struct BuildingPlacementContext {
    player_id: u8,
    building_type: BuildingType,
    base_position: Vec3,
    existing_buildings: Vec<(Vec3, BuildingType)>,
    target_resource_types: Vec<ResourceType>,
    relevant_resources: Vec<(Vec3, ResourceType, f32)>,
}

impl BuildingPlacementContext {
    fn new(
        player_id: u8,
        building_type: BuildingType,
        building_collisions: &[(Vec3, f32)],
        resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    ) -> Self {
        let base_x = if player_id == 1 { -800.0 } else { 800.0 };
        let base_position = Vec3::new(base_x, 0.0, 0.0);

        let existing_buildings = building_collisions
            .iter()
            .map(|(pos, _)| (*pos, BuildingType::Queen))
            .collect();

        let target_resource_types = get_target_resource_types(&building_type);
        let relevant_resources =
            collect_relevant_resources(resource_sources, &target_resource_types, base_position);

        Self {
            player_id,
            building_type,
            base_position,
            existing_buildings,
            target_resource_types,
            relevant_resources,
        }
    }
}

fn can_afford_building(
    resources: &PlayerResources,
    game_costs: &GameCosts,
    building_type: &BuildingType,
) -> bool {
    if let Some(cost) = game_costs.building_costs.get(building_type) {
        resources.can_afford(cost)
    } else {
        false
    }
}

fn get_target_resource_types(building_type: &BuildingType) -> Vec<ResourceType> {
    match building_type {
        BuildingType::FungalGarden => vec![ResourceType::Nectar],
        BuildingType::StorageChamber => vec![
            ResourceType::Nectar,
            ResourceType::Minerals,
            ResourceType::Chitin,
        ],
        BuildingType::Queen => vec![ResourceType::Nectar],
        BuildingType::Nursery => vec![ResourceType::Nectar, ResourceType::Chitin],
        BuildingType::WarriorChamber => vec![ResourceType::Nectar, ResourceType::Pheromones],
        BuildingType::HunterChamber => vec![ResourceType::Nectar, ResourceType::Pheromones],
        _ => vec![ResourceType::Nectar],
    }
}

fn collect_relevant_resources(
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    target_resource_types: &[ResourceType],
    base_position: Vec3,
) -> Vec<(Vec3, ResourceType, f32)> {
    let mut relevant_resources = Vec::new();
    for (_, source, transform) in resource_sources.iter() {
        if target_resource_types.contains(&source.resource_type) && source.amount > 0.0 {
            let distance_to_base = transform.translation.distance(base_position);
            if distance_to_base < 250.0 {
                relevant_resources.push((
                    transform.translation,
                    source.resource_type.clone(),
                    distance_to_base,
                ));
            }
        }
    }
    relevant_resources.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    relevant_resources
}

fn find_optimal_building_position(
    context: BuildingPlacementContext,
    terrain_helper: &TerrainHelper,
    unit_collisions: &[(Vec3, f32)],
    environment_collisions: &[(Vec3, f32, EnvironmentObjectType)],
) -> Option<Vec3> {
    if let Some(position) = try_resource_based_placement(
        &context,
        terrain_helper,
        unit_collisions,
        environment_collisions,
    ) {
        return Some(position);
    }

    try_base_area_placement(
        &context,
        terrain_helper,
        unit_collisions,
        environment_collisions,
    )
}

fn try_resource_based_placement(
    context: &BuildingPlacementContext,
    terrain_helper: &TerrainHelper,
    unit_collisions: &[(Vec3, f32)],
    environment_collisions: &[(Vec3, f32, EnvironmentObjectType)],
) -> Option<Vec3> {
    for (resource_pos, resource_type, _distance) in context.relevant_resources.iter().take(5) {
        let nearby_buildings = context
            .existing_buildings
            .iter()
            .filter(|(pos, _)| pos.distance(*resource_pos) < 60.0)
            .count();

        if nearby_buildings >= 2 {
            debug!(
                "AI Player {} skipping overcrowded {:?} resource ({} nearby buildings)",
                context.player_id, resource_type, nearby_buildings
            );
            continue;
        }

        if let Some(position) = try_positions_around_resource(
            context,
            resource_pos,
            nearby_buildings,
            terrain_helper,
            unit_collisions,
            environment_collisions,
        ) {
            return Some(position);
        }
    }
    None
}

fn try_positions_around_resource(
    context: &BuildingPlacementContext,
    resource_pos: &Vec3,
    nearby_buildings: usize,
    terrain_helper: &TerrainHelper,
    unit_collisions: &[(Vec3, f32)],
    environment_collisions: &[(Vec3, f32, EnvironmentObjectType)],
) -> Option<Vec3> {
    for ring in 0..5 {
        for attempt in 0..16 {
            let angle = (attempt as f32) * std::f32::consts::PI / 8.0;
            let base_radius = 25.0 + (nearby_buildings as f32 * 8.0);
            let radius = base_radius + (ring as f32 * 15.0);

            let pos_x = resource_pos.x + radius * angle.cos();
            let pos_z = resource_pos.z + radius * angle.sin();
            let position = terrain_helper.get_terrain_position(pos_x, pos_z, 0.0);

            if is_valid_resource_position(
                context,
                position,
                unit_collisions,
                environment_collisions,
            ) {
                return Some(position);
            }
        }
    }
    None
}

fn is_valid_resource_position(
    context: &BuildingPlacementContext,
    position: Vec3,
    unit_collisions: &[(Vec3, f32)],
    environment_collisions: &[(Vec3, f32, EnvironmentObjectType)],
) -> bool {
    let distance_to_base = position.distance(context.base_position);
    if !(position.x.abs() < 1000.0
        && position.z.abs() < 1000.0
        && distance_to_base > 40.0
        && distance_to_base < 350.0)
    {
        return false;
    }

    let min_building_distance = context
        .existing_buildings
        .iter()
        .map(|(pos, _)| pos.distance(position))
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1000.0);

    if min_building_distance < 35.0 {
        return false;
    }

    let building_radius = get_building_collision_radius(&context.building_type);
    validate_ai_building_placement(
        position,
        building_radius,
        &context
            .existing_buildings
            .iter()
            .map(|(pos, _)| (*pos, building_radius))
            .collect::<Vec<_>>(),
        &unit_collisions.to_vec(),
        &environment_collisions.to_vec(),
    )
}

fn try_base_area_placement(
    context: &BuildingPlacementContext,
    terrain_helper: &TerrainHelper,
    unit_collisions: &[(Vec3, f32)],
    environment_collisions: &[(Vec3, f32, EnvironmentObjectType)],
) -> Option<Vec3> {
    info!(
        "AI Player {} using fallback placement for {:?} - no suitable resource locations found",
        context.player_id, context.building_type
    );

    let base_area_buildings = context
        .existing_buildings
        .iter()
        .filter(|(pos, _)| pos.distance(context.base_position) < 80.0)
        .count();
    let base_crowding_offset = (base_area_buildings as f32 * 10.0).min(40.0);

    let (random_offset_x, random_offset_z) =
        generate_placement_offsets(&context.building_type, context.player_id);

    for ring in 0..6 {
        for attempt in 0..12 {
            let angle = (attempt as f32) * std::f32::consts::PI / 6.0;
            let radius = 50.0 + base_crowding_offset + (ring as f32 * 25.0);

            let pos_x = context.base_position.x + random_offset_x + radius * angle.cos();
            let pos_z = context.base_position.z + random_offset_z + radius * angle.sin();
            let position = terrain_helper.get_terrain_position(pos_x, pos_z, 0.0);

            if is_valid_base_position(context, position, unit_collisions, environment_collisions) {
                return Some(position);
            }
        }
    }
    None
}

fn generate_placement_offsets(building_type: &BuildingType, player_id: u8) -> (f32, f32) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    building_type.hash(&mut hasher);
    player_id.hash(&mut hasher);
    let seed = hasher.finish();

    let random_offset_x = ((seed % 80) as f32 - 40.0) * 0.5;
    let random_offset_z = (((seed / 80) % 80) as f32 - 40.0) * 0.5;
    (random_offset_x, random_offset_z)
}

fn is_valid_base_position(
    context: &BuildingPlacementContext,
    position: Vec3,
    unit_collisions: &[(Vec3, f32)],
    environment_collisions: &[(Vec3, f32, EnvironmentObjectType)],
) -> bool {
    if !(position.x.abs() < 1000.0 && position.z.abs() < 1000.0) {
        return false;
    }

    let min_building_distance = context
        .existing_buildings
        .iter()
        .map(|(pos, _)| pos.distance(position))
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1000.0);

    if min_building_distance < 30.0 {
        return false;
    }

    let building_radius = get_building_collision_radius(&context.building_type);
    validate_ai_building_placement(
        position,
        building_radius,
        &context
            .existing_buildings
            .iter()
            .map(|(pos, _)| (*pos, building_radius))
            .collect::<Vec<_>>(),
        &unit_collisions.to_vec(),
        &environment_collisions.to_vec(),
    )
}

fn deduct_building_resources(
    player_id: u8,
    building_type: &BuildingType,
    game_costs: &GameCosts,
    ai_resources: &mut ResMut<AIResources>,
) -> bool {
    if let Some(cost) = game_costs.building_costs.get(building_type) {
        if let Some(player_resources) = ai_resources.resources.get_mut(&player_id) {
            if !player_resources.spend_resources(cost) {
                warn!("AI Player {} failed to spend resources for {:?} - insufficient resources after placement", 
                      player_id, building_type);
                return false;
            }
            info!(
                "💰 AI Player {} spent resources for {:?}: {:?}",
                player_id, building_type, cost
            );
            return true;
        } else {
            warn!("AI Player {} not found in resource manager", player_id);
        }
    } else {
        warn!(
            "AI Player {} no cost found for building {:?}",
            player_id, building_type
        );
    }
    false
}

fn create_building_site(
    commands: &mut Commands,
    building_type: BuildingType,
    building_position: Vec3,
    player_id: u8,
) {
    commands.spawn(BuildingSite {
        building_type: building_type.clone(),
        position: building_position,
        player_id,
        assigned_worker: None,
        construction_started: false,
        site_reserved: false,
    });

    info!(
        "AI Player {} created building site for {:?} at {:?} (unique position)",
        player_id, building_type, building_position
    );
}

struct UnitCounts {
    worker_count: u32,
    military_count: u32,
}

fn calculate_unit_counts(units: &Query<&RTSUnit>, player_id: u8) -> UnitCounts {
    let worker_count = count_player_units_of_type(units, player_id, |unit_type| {
        matches!(unit_type, UnitType::WorkerAnt)
    });

    let military_count = count_player_units_of_type(units, player_id, |unit_type| {
        matches!(
            unit_type,
            UnitType::SoldierAnt
                | UnitType::BeetleKnight
                | UnitType::HunterWasp
                | UnitType::SpearMantis
                | UnitType::DragonFly
                | UnitType::BatteringBeetle
                | UnitType::EliteSpider
                | UnitType::ScoutAnt
                | UnitType::AcidSpitter
                | UnitType::DefenderBug
                | UnitType::SpiderHunter
                | UnitType::WolfSpider
                | UnitType::Ladybug
                | UnitType::LegBeetle
                | UnitType::Scorpion
                | UnitType::TermiteWarrior
                | UnitType::Stinkbug
        )
    });

    UnitCounts {
        worker_count,
        military_count,
    }
}

fn handle_early_game_goals(
    strategy: &mut PlayerStrategy,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_id: u8,
    current_time: f32,
    unit_counts: &UnitCounts,
    log_limiter: &mut LogRateLimiter,
) {
    ensure_queen_buildings(strategy, buildings, player_id, current_time, 1, 4, log_limiter); // More Queens for expansion
    add_worker_goals(
        strategy,
        unit_counts.worker_count,
        40,
        10,
        player_id,
        log_limiter,
        current_time,
    );
    add_housing_goals(strategy, buildings, player_id);
    add_economic_buildings(strategy, buildings, player_id);
}

fn handle_mid_game_goals(
    strategy: &mut PlayerStrategy,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_id: u8,
    current_time: f32,
    unit_counts: &UnitCounts,
    log_limiter: &mut LogRateLimiter,
) {
    ensure_queen_buildings(strategy, buildings, player_id, current_time, 1, 4, log_limiter); // More Queens for expansion
    add_worker_goals(
        strategy,
        unit_counts.worker_count,
        25,
        3,
        player_id,
        log_limiter,
        current_time,
    ); // Increased mid-game worker target
    add_military_infrastructure(strategy, buildings, player_id);
    add_military_units(strategy, unit_counts.military_count, 15, 3);
}

fn handle_late_game_goals(
    strategy: &mut PlayerStrategy,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    units: &Query<&RTSUnit>,
    player_id: u8,
    current_time: f32,
    unit_counts: &UnitCounts,
    log_limiter: &mut LogRateLimiter,
) {
    ensure_queen_buildings(strategy, buildings, player_id, current_time, 1, 5, log_limiter); // Need more Queens for massive worker production
    add_worker_goals(
        strategy,
        unit_counts.worker_count,
        40,
        3,
        player_id,
        log_limiter,
        current_time,
    ); // Massive exponential worker growth
    add_massive_military_production(strategy, unit_counts.military_count);
    add_additional_military_infrastructure(strategy, buildings, player_id);
    handle_enemy_elimination_goals(strategy, units, player_id, unit_counts);
}

fn ensure_queen_buildings(
    strategy: &mut PlayerStrategy,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_id: u8,
    current_time: f32,
    min_urgent: usize,
    min_target: usize,
    log_limiter: &mut LogRateLimiter,
) {
    let queen_count = buildings
        .iter()
        .filter(|(_, building, unit)| {
            unit.player_id == player_id && building.building_type == BuildingType::Queen
        })
        .count();

    let has_queen_goal = strategy
        .priority_queue
        .iter()
        .any(|goal| matches!(goal, StrategyGoal::ConstructBuilding(BuildingType::Queen)));

    if queen_count <= min_urgent && !has_queen_goal {
        strategy
            .priority_queue
            .insert(0, StrategyGoal::ConstructBuilding(BuildingType::Queen));
        let log_key = format!("queen_critical_player_{}", player_id);
        if log_limiter.should_log(&log_key, current_time, 5.0) {
            info!(
                "🔥 CRITICAL: AI Player {} has {} Queen buildings - adding to priority queue",
                player_id, queen_count
            );
        }
    } else if queen_count < min_target && !has_queen_goal {
        let time_since_failure = current_time - strategy.last_building_failure_time;
        if strategy.last_building_failure_time == 0.0 || time_since_failure > 5.0 {
            strategy
                .priority_queue
                .insert(0, StrategyGoal::ConstructBuilding(BuildingType::Queen));
            let log_key = format!("queen_needed_player_{}", player_id);
            if log_limiter.should_log(&log_key, current_time, 5.0) {
                info!(
                    "📈 AI Player {} needs more Queen buildings ({} < {}) - adding to priority queue",
                    player_id, queen_count, min_target
                );
            }
        }
    }
}

fn add_worker_goals(
    strategy: &mut PlayerStrategy,
    current_workers: u32,
    target: u32,
    max_batch: u32,
    player_id: u8,
    log_limiter: &mut LogRateLimiter,
    current_time: f32,
) {
    if current_workers < target {
        let workers_needed = (target - current_workers).min(max_batch);
        let current_worker_goals = strategy
            .priority_queue
            .iter()
            .filter(|goal| matches!(goal, StrategyGoal::BuildWorker))
            .count() as u32;

        let goals_to_add = workers_needed.saturating_sub(current_worker_goals);
        if goals_to_add > 0 {
            let log_key = format!("worker_goals_player_{}", player_id);
            if log_limiter.should_log(&log_key, current_time, 5.0) {
                info!(
                    "📈 Adding {} worker goals (current: {}, target: {})",
                    goals_to_add, current_workers, target
                );
            }
        }

        for _ in 0..goals_to_add {
            strategy.priority_queue.insert(0, StrategyGoal::BuildWorker);
        }
    }
}

fn add_housing_goals(
    strategy: &mut PlayerStrategy,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_id: u8,
) {
    let nursery_count = buildings
        .iter()
        .filter(|(_, building, unit)| {
            unit.player_id == player_id && building.building_type == BuildingType::Nursery
        })
        .count();

    if nursery_count < 3
        && !strategy
            .priority_queue
            .iter()
            .any(|goal| matches!(goal, StrategyGoal::ConstructBuilding(BuildingType::Nursery)))
    {
        strategy
            .priority_queue
            .insert(1, StrategyGoal::ConstructBuilding(BuildingType::Nursery));
    }
}

fn add_economic_buildings(
    strategy: &mut PlayerStrategy,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_id: u8,
) {
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
}

fn add_military_infrastructure(
    strategy: &mut PlayerStrategy,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_id: u8,
) {
    let military_buildings = [BuildingType::WarriorChamber, BuildingType::HunterChamber];

    for building_type in &military_buildings {
        let has_building = buildings.iter().any(|(_, building, unit)| {
            unit.player_id == player_id && building.building_type == *building_type
        });

        let already_queued = strategy.priority_queue.iter().any(
            |goal| matches!(goal, StrategyGoal::ConstructBuilding(ref bt) if bt == building_type),
        );

        if !has_building && !already_queued {
            // Add cooldown mechanism to prevent immediate re-queuing of failed buildings
            use std::collections::HashMap;
            use std::sync::Mutex;
            static BUILDING_COOLDOWNS: std::sync::LazyLock<
                Mutex<HashMap<(u8, BuildingType), f32>>,
            > = std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f32();

            let can_retry = if let Ok(cooldowns) = BUILDING_COOLDOWNS.lock() {
                let key = (player_id, building_type.clone());
                let last_failure = cooldowns.get(&key).copied().unwrap_or(0.0);
                current_time - last_failure > 30.0 // 30 second cooldown between retries
            } else {
                true // Allow if mutex fails
            };

            if can_retry {
                strategy
                    .priority_queue
                    .insert(1, StrategyGoal::ConstructBuilding(building_type.clone()));
            }
        }
    }
}

fn add_military_units(
    strategy: &mut PlayerStrategy,
    current_military: u32,
    target: u32,
    max_batch: u32,
) {
    if current_military < target {
        let units_needed = (target - current_military).min(max_batch);
        let mut units_added = 0;
        for unit_type in &strategy.military_targets.preferred_unit_types {
            if units_added >= units_needed {
                break;
            }
            strategy
                .priority_queue
                .push(StrategyGoal::BuildMilitaryUnit(unit_type.clone()));
            units_added += 1;
        }
    }
}

fn add_massive_military_production(strategy: &mut PlayerStrategy, current_military: u32) {
    if current_military < strategy.military_targets.desired_military_units {
        let units_needed =
            (strategy.military_targets.desired_military_units - current_military).min(12); // Increased from 6 to 12

        // ENHANCED - Prioritize strongest attacking units with DragonFly at the top
        let elite_elimination_priority = vec![
            UnitType::DragonFly,       // STRONGEST - 5x speed, air superiority
            UnitType::BeetleKnight,    // Heavy assault tank
            UnitType::SpearMantis,     // Anti-building specialist
            UnitType::BatteringBeetle, // Siege warfare
            UnitType::HunterWasp,      // Fast air raiders
            UnitType::EliteSpider,     // Elite predator
            UnitType::SoldierAnt,      // Core ground army
        ];

        let mut units_added = 0;
        for unit_type in &elite_elimination_priority {
            if units_added >= units_needed {
                break;
            }

            let current_count = strategy.priority_queue.iter().filter(|goal| {
                matches!(goal, StrategyGoal::BuildMilitaryUnit(ref ut) if ut == unit_type)
            }).count();

            // Allow more of the strongest units in queue - DragonFlies get priority
            let max_in_queue = if *unit_type == UnitType::DragonFly {
                6
            } else {
                4
            }; // More DragonFlies

            if current_count < max_in_queue {
                strategy
                    .priority_queue
                    .insert(0, StrategyGoal::BuildMilitaryUnit(unit_type.clone()));
                units_added += 1;

                // Double-queue DragonFlies for maximum air superiority
                if *unit_type == UnitType::DragonFly && current_count < 3 {
                    strategy
                        .priority_queue
                        .insert(0, StrategyGoal::BuildMilitaryUnit(unit_type.clone()));
                    units_added += 1;
                }
            }
        }
    }
}

fn add_additional_military_infrastructure(
    strategy: &mut PlayerStrategy,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_id: u8,
) {
    let warrior_chambers = buildings
        .iter()
        .filter(|(_, building, unit)| {
            unit.player_id == player_id && building.building_type == BuildingType::WarriorChamber
        })
        .count();

    if warrior_chambers < 2
        && !strategy.priority_queue.iter().any(|goal| {
            matches!(
                goal,
                StrategyGoal::ConstructBuilding(BuildingType::WarriorChamber)
            )
        })
    {
        strategy.priority_queue.insert(
            1,
            StrategyGoal::ConstructBuilding(BuildingType::WarriorChamber),
        );
    }
}

fn handle_enemy_elimination_goals(
    strategy: &mut PlayerStrategy,
    units: &Query<&RTSUnit>,
    player_id: u8,
    unit_counts: &UnitCounts,
) {
    let enemy_count = count_player_units(units, 1);

    if enemy_count == 0 {
        strategy
            .priority_queue
            .retain(|goal| !matches!(goal, StrategyGoal::AttackEnemy(_)));
        info!("🏆 VICTORY! AI Player {} has eliminated all enemies - {} workers, {} military units celebrating!", 
              player_id, unit_counts.worker_count, unit_counts.military_count);
    } else if unit_counts.military_count >= 15
        && !strategy
            .priority_queue
            .iter()
            .any(|goal| matches!(goal, StrategyGoal::AttackEnemy(_)))
    {
        strategy
            .priority_queue
            .insert(0, StrategyGoal::AttackEnemy(1));
        debug!("🎯 AI Player {} adding ATTACK GOAL - targeting Player 1 for elimination! (Enemy units remaining: {})", 
              player_id, enemy_count);
    }
}
