use crate::ai::tactics::{TacticalManager, TacticalStance};
use crate::core::components::*;
use crate::core::query_cache::UnitQueryCache;
use bevy::prelude::*;

// CombatState component is now defined in core::components

pub fn ai_combat_system(
    mut commands: Commands,
    mut ai_units: Query<
        (
            Entity,
            &mut Movement,
            &mut Combat,
            &Transform,
            &RTSUnit,
            &RTSHealth,
            Option<&mut CombatState>,
        ),
        With<Combat>,
    >,
    // Use query cache instead of expensive repeated queries
    query_cache: Res<UnitQueryCache>,
    // Add queries for obstacles
    buildings: Query<(&Transform, &CollisionRadius), (With<Building>, Without<RTSUnit>)>,
    environment_objects: Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<RTSUnit>),
    >,
    tactical_manager: Option<Res<TacticalManager>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, mut movement, mut combat, unit_transform, unit, health, mut combat_state_opt) in
        ai_units.iter_mut()
    {
        // Skip player 1 units and units that don't auto-attack (like workers)
        if unit.player_id == 1 || !combat.auto_attack {
            continue;
        }

        // Ensure unit has combat state
        let mut state = match &combat_state_opt {
            Some(state) => (**state).clone(),
            None => {
                commands.entity(entity).insert(CombatState::default());
                CombatState::default()
            }
        };

        // Update state timing
        update_state_timing(&mut state, current_time);

        // Get tactical stance - make AI more aggressive by default
        let stance = if let Some(ref manager) = tactical_manager {
            manager
                .player_tactics
                .get(&unit.player_id)
                .map(|t| t.current_stance.clone())
                .unwrap_or(TacticalStance::Aggressive) // Changed from Defensive to Aggressive
        } else {
            TacticalStance::Aggressive // Changed from Defensive to Aggressive
        };

        // Advanced combat AI with tactics
        handle_advanced_combat_ai(
            entity,
            &mut movement,
            &mut combat,
            unit_transform,
            unit,
            health,
            &mut state,
            &query_cache,
            &buildings,
            &environment_objects,
            stance,
            current_time,
        );

        // Update combat state
        if let Some(ref mut cs) = combat_state_opt {
            **cs = state;
        }
    }
}

fn update_state_timing(state: &mut CombatState, current_time: f32) {
    // Keep existing timing fields updated
    state.last_attack_attempt = current_time;
}

/// Update combat state based on target engagement
fn update_combat_state_for_engagement(
    state: &mut CombatState,
    target_entity: Entity,
    target_pos: Vec3,
    distance: f32,
    combat: &Combat,
    current_time: f32,
) {
    use crate::core::components::CombatStateType;

    // Update target information
    if state.target_entity != Some(target_entity) {
        state.target_entity = Some(target_entity);
        state.target_position = Some(target_pos);
        state.last_state_change = current_time;

        // Starting engagement
        if matches!(state.state, CombatStateType::Idle) {
            state.engagement_start_time = current_time;
        }
    }

    // Determine appropriate state based on distance to target
    let new_state = if distance <= combat.attack_range {
        CombatStateType::InCombat
    } else if distance <= combat.attack_range * 2.0 {
        CombatStateType::CombatMoving
    } else if matches!(state.state, CombatStateType::Idle) {
        // Initial movement toward combat - use MovingToCombat
        CombatStateType::MovingToCombat
    } else {
        // Already in combat sequence, continue with MovingToAttack
        CombatStateType::MovingToAttack
    };

    // Update state if changed
    if state.state != new_state {
        state.state = new_state;
        state.last_state_change = current_time;
    }
}

/// Transition unit to idle state when no target
fn transition_to_idle_state(state: &mut CombatState, current_time: f32) {
    use crate::core::components::CombatStateType;

    if !matches!(state.state, CombatStateType::Idle) {
        state.state = CombatStateType::Idle;
        state.target_entity = None;
        state.target_position = None;
        state.last_state_change = current_time;
    }
}



fn handle_advanced_combat_ai(
    _self_entity: Entity,
    movement: &mut Movement,
    combat: &mut Combat,
    unit_transform: &Transform,
    unit: &RTSUnit,
    _health: &RTSHealth,
    state: &mut CombatState,
    query_cache: &UnitQueryCache,
    buildings: &Query<(&Transform, &CollisionRadius), (With<Building>, Without<RTSUnit>)>,
    environment_objects: &Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<RTSUnit>),
    >,
    stance: TacticalStance,
    current_time: f32,
) {
    let target_info = find_best_target_cached(unit_transform, unit, query_cache, state.target_entity);

    if let Some((target_entity, target_pos, target_distance, _target_priority)) = target_info {
        update_combat_state_for_engagement(
            state,
            target_entity,
            target_pos,
            target_distance,
            combat,
            current_time,
        );
        handle_target_engagement(
            target_entity,
            target_pos,
            target_distance,
            movement,
            combat,
            unit_transform,
            unit,
            state,
            current_time,
        );
    } else {
        transition_to_idle_state(state, current_time);
        handle_no_target_behavior_cached(
            movement,
            unit_transform,
            unit,
            query_cache,
            buildings,
            environment_objects,
            stance,
            current_time,
            state,
        );
    }
}


/// Find the best target with prioritization
// Optimized version using query cache
fn find_best_target_cached(
    unit_transform: &Transform,
    unit: &RTSUnit,
    query_cache: &UnitQueryCache,
    target_entity: Option<Entity>,
) -> Option<(Entity, Vec3, f32, u32)> {
    let detection_range = 200.0; // Increased from 120.0 to 200.0 for more aggressive seeking
    let mut best_target: Option<(Entity, Vec3, f32, u32)> = None;
    let mut best_priority = 0u32;

    // Check if current target is still valid using cache
    if let Some(target_entity) = target_entity {
        if let Some(target_data) = query_cache.health_units.get(&target_entity) {
            if target_data.basic.player_id != unit.player_id && target_data.health.current > 0.0 {
                let distance = unit_transform.translation.distance(target_data.basic.transform.translation);
                if distance <= detection_range {
                    let priority = calculate_target_priority(&target_data.basic.unit, &target_data.health);
                    return Some((target_entity, target_data.basic.transform.translation, distance, priority));
                }
            }
        }
    }

    // Find best target using cached enemy units
    for enemy_data in query_cache.get_enemy_units(unit.player_id) {
        if let Some(health_data) = query_cache.health_units.get(&enemy_data.entity) {
            if health_data.health.current <= 0.0 {
                continue; // Skip dead units
            }

            let distance = unit_transform.translation.distance(enemy_data.transform.translation);
            if distance <= detection_range {
                let priority = calculate_target_priority(&enemy_data.unit, &health_data.health);
                if priority > best_priority {
                    best_priority = priority;
                    best_target = Some((enemy_data.entity, enemy_data.transform.translation, distance, priority));
                }
            }
        }
    }

    best_target
}

// Keep the original function for backwards compatibility

/// Calculate target priority (higher = more important to attack)
fn calculate_target_priority(target_unit: &RTSUnit, target_health: &RTSHealth) -> u32 {
    let mut priority = 5; // Base priority

    // Prioritize by unit type
    if let Some(unit_type) = &target_unit.unit_type {
        match unit_type {
            UnitType::WorkerAnt => priority = 8, // Kill workers first (disrupt economy)
            UnitType::SoldierAnt => priority = 6,
            UnitType::BeetleKnight => priority = 5, // Tanks are lower priority
            _ => {}
        }
    }

    // Prioritize low health targets (easier to finish off)
    if target_health.current < target_health.max * 0.5 {
        priority += 2;
    }

    priority
}

/// Check if target position should be updated (avoids micro-adjustments)
fn should_update_target_position(
    current_target: &Option<Vec3>,
    new_target: Vec3,
    threshold: f32,
) -> bool {
    match current_target {
        None => true, // No target set, always update
        Some(current) => current.distance(new_target) > threshold, // Only update if significantly different
    }
}


/// Melee unit aggressive behavior
fn handle_melee_combat(
    movement: &mut Movement,
    combat: &Combat,
    _unit_transform: &Transform,
    target_pos: Vec3,
    distance: f32,
) {
    let threshold = 1.0; // Minimum distance change before updating

    if distance > combat.attack_range + threshold {
        // Too far - chase target
        if should_update_target_position(&movement.target_position, target_pos, threshold) {
            movement.target_position = Some(target_pos);
        }
    } else if distance <= combat.attack_range {
        // In range - stop and attack
        if movement.current_velocity.length() < 1.0 {
            movement.target_position = None;
        }
    }
    // Else: in buffer zone, maintain current movement to avoid jitter
}





fn handle_target_engagement(
    target_entity: Entity,
    target_pos: Vec3,
    target_distance: f32,
    movement: &mut Movement,
    combat: &mut Combat,
    unit_transform: &Transform,
    unit: &RTSUnit,
    state: &mut CombatState,
    current_time: f32,
) {
    state.target_entity = Some(target_entity);

    match unit.unit_type.as_ref() {
        Some(UnitType::SoldierAnt | UnitType::BeetleKnight) => {
            handle_melee_combat(
                movement,
                combat,
                unit_transform,
                target_pos,
                target_distance,
            );
        }
        _ => {
            handle_default_combat(movement, combat, target_pos, target_distance);
        }
    }

    state.last_attack_attempt = current_time;
}

fn handle_default_combat(
    movement: &mut Movement,
    combat: &Combat,
    target_pos: Vec3,
    target_distance: f32,
) {
    let threshold = 1.0;
    if target_distance > combat.attack_range + threshold {
        if should_update_target_position(&movement.target_position, target_pos, threshold) {
            movement.target_position = Some(target_pos);
        }
    } else if target_distance <= combat.attack_range && movement.current_velocity.length() < 1.0 {
        movement.target_position = None;
    }
}

// Cached version for performance
fn handle_no_target_behavior_cached(
    movement: &mut Movement,
    unit_transform: &Transform,
    unit: &RTSUnit,
    query_cache: &UnitQueryCache,
    _buildings: &Query<(&Transform, &CollisionRadius), (With<Building>, Without<RTSUnit>)>,
    _environment_objects: &Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<RTSUnit>),
    >,
    stance: TacticalStance,
    current_time: f32,
    state: &mut CombatState,
) {
    state.target_entity = None;
    
    match stance {
        TacticalStance::Aggressive => {
            // Aggressively search for targets using cached data
            let patrol_range = 150.0;
            let enemy_units = query_cache.get_units_in_range(unit_transform.translation, patrol_range);
            
            if !enemy_units.is_empty() {
                // Move towards detected enemies
                if let Some(closest_enemy) = enemy_units
                    .into_iter()
                    .filter(|u| u.player_id != unit.player_id)
                    .min_by(|a, b| {
                        let dist_a = unit_transform.translation.distance(a.transform.translation);
                        let dist_b = unit_transform.translation.distance(b.transform.translation);
                        dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                {
                    movement.target_position = Some(closest_enemy.transform.translation);
                    state.last_state_change = current_time;
                }
            } else {
                patrol_behavior_optimized(movement, unit_transform, current_time, state);
            }
        }
        TacticalStance::Defensive => {
            // Stay near friendly units or patrol a smaller area
            defensive_behavior_optimized(movement, unit_transform, query_cache, unit, current_time, state);
        }
        TacticalStance::Harass => {
            // Harass behavior - hit and run tactics
            patrol_behavior_optimized(movement, unit_transform, current_time, state);
        }
        TacticalStance::Expand => {
            // Expansion behavior - secure new areas
            patrol_behavior_optimized(movement, unit_transform, current_time, state);
        }
    }
}

fn defensive_behavior_optimized(
    movement: &mut Movement,
    unit_transform: &Transform,
    query_cache: &UnitQueryCache,
    unit: &RTSUnit,
    current_time: f32,
    state: &mut CombatState,
) {
    // Find nearby friendly units
    let friendly_units = query_cache.get_player_units(unit.player_id);
    let nearby_friendlies: Vec<_> = friendly_units
        .into_iter()
        .filter(|u| {
            let distance = unit_transform.translation.distance(u.transform.translation);
            distance < 100.0 && distance > 5.0 // Not too close, not too far
        })
        .collect();

    if !nearby_friendlies.is_empty() {
        // Stay near friendly units
        let center = nearby_friendlies
            .iter()
            .fold(Vec3::ZERO, |acc, u| acc + u.transform.translation) / nearby_friendlies.len() as f32;
        
        let distance_to_center = unit_transform.translation.distance(center);
        if distance_to_center > 50.0 {
            movement.target_position = Some(center);
            state.last_state_change = current_time;
        }
    } else {
        // No friendlies nearby, patrol defensively
        patrol_behavior_optimized(movement, unit_transform, current_time, state);
    }
}

fn patrol_behavior_optimized(
    movement: &mut Movement,
    unit_transform: &Transform,
    current_time: f32,
    state: &mut CombatState,
) {
    if movement.target_position.is_none() || 
       (current_time - state.last_state_change > 10.0 && 
        movement.target_position.map_or(true, |pos| unit_transform.translation.distance(pos) < 5.0)) {
        
        // Set a new patrol target
        let patrol_radius = 80.0;
        let angle = (current_time * 0.5).sin() * std::f32::consts::TAU;
        let patrol_offset = Vec3::new(
            angle.cos() * patrol_radius,
            0.0,
            angle.sin() * patrol_radius,
        );
        movement.target_position = Some(unit_transform.translation + patrol_offset);
        state.last_state_change = current_time;
    }
}



/// Estimate home base position
fn estimate_home_position(player_id: u8) -> Vec3 {
    match player_id {
        1 => Vec3::new(0.0, 0.0, 0.0),
        2 => Vec3::new(200.0, 0.0, 0.0),
        _ => Vec3::new((player_id as f32 - 1.0) * 200.0, 0.0, 0.0),
    }
}

/// System to handle smooth transition from combat to resource gathering when victory is achieved
pub fn combat_to_resource_transition_system(
    mut commands: Commands,
    mut military_units: Query<
        (Entity, &mut Movement, &mut Combat, &RTSUnit, Option<&mut CombatState>),
        (With<Combat>, Without<ResourceGatherer>),
    >,
    mut ai_workers: Query<
        (Entity, &mut Movement, &mut Combat, &RTSUnit, &mut ResourceGatherer, Option<&mut CombatState>),
        (With<Combat>, With<ResourceGatherer>),
    >,
    all_units: Query<&RTSUnit, With<RTSUnit>>,
    resource_sources: Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    buildings: Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) {
    for player_id in 2..=4 {
        if !has_enemies_for_player(player_id, &all_units) {
            let workers_reassigned = reassign_workers_to_resources(
                player_id, 
                &mut ai_workers, 
                &resource_sources, 
                &buildings
            );
            
            let units_converted = convert_military_to_workers(
                player_id,
                &mut commands,
                &mut military_units,
                &resource_sources,
                &buildings,
            );

            if workers_reassigned > 0 || units_converted > 0 {
                info!("🏆 VICTORY! AI Player {} - {} workers resumed gathering, {} military converted to workers", 
                      player_id, workers_reassigned, units_converted);
            }
        }
    }
}

fn has_enemies_for_player(player_id: u8, all_units: &Query<&RTSUnit, With<RTSUnit>>) -> bool {
    all_units
        .iter()
        .any(|unit| unit.player_id != player_id && unit.player_id == 1)
}

fn clear_combat_state(
    movement: &mut Movement,
    combat: &mut Combat,
    combat_state_opt: Option<&mut CombatState>
) {
    movement.target_position = None;
    movement.current_velocity = Vec3::ZERO;
    combat.target = None;
    combat.is_attacking = false;
    combat.last_attack_time = 0.0;

    if let Some(combat_state) = combat_state_opt {
        combat_state.target_entity = None;
        combat_state.state = crate::core::components::CombatStateType::Idle;
        combat_state.target_position = None;
    }
}

fn reassign_workers_to_resources(
    player_id: u8,
    ai_workers: &mut Query<
        (Entity, &mut Movement, &mut Combat, &RTSUnit, &mut ResourceGatherer, Option<&mut CombatState>),
        (With<Combat>, With<ResourceGatherer>),
    >,
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) -> i32 {
    let mut workers_reassigned = 0;
    
    for (_entity, mut movement, mut combat, unit, mut gatherer, mut combat_state_opt) in ai_workers.iter_mut() {
        if unit.player_id != player_id {
            continue;
        }

        clear_combat_state(&mut movement, &mut combat, combat_state_opt.as_deref_mut());

        if gatherer.target_resource.is_none() {
            if let Some((resource_entity, resource_pos)) = find_nearest_resource(&unit, &resource_sources) {
                assign_worker_to_resource(&mut gatherer, &mut movement, resource_entity, resource_pos, unit, buildings);
                workers_reassigned += 1;
            }
        } else {
            workers_reassigned += resume_existing_resource_assignment(&mut gatherer, &mut movement, unit, resource_sources);
        }
    }
    
    workers_reassigned
}

fn assign_worker_to_resource(
    gatherer: &mut ResourceGatherer,
    movement: &mut Movement,
    resource_entity: Entity,
    resource_pos: (Vec3, ResourceType),
    unit: &RTSUnit,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) {
    gatherer.target_resource = Some(resource_entity);
    gatherer.resource_type = Some(resource_pos.1.clone());
    gatherer.carried_amount = 0.0;
    gatherer.drop_off_building = find_nearest_building_for_worker(unit, buildings);
    movement.target_position = Some(resource_pos.0);
    
    info!("🔄 AI Worker {} (player {}) returning to gather {:?} after combat", 
          unit.unit_id, unit.player_id, resource_pos.1);
}

fn resume_existing_resource_assignment(
    gatherer: &mut ResourceGatherer,
    movement: &mut Movement,
    unit: &RTSUnit,
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
) -> i32 {
    if let Ok((_, resource_source, resource_transform)) = resource_sources.get(gatherer.target_resource.unwrap()) {
        if resource_source.amount > 0.0 {
            movement.target_position = Some(resource_transform.translation);
            info!("🔄 AI Worker {} (player {}) resuming resource gathering after combat", 
                  unit.unit_id, unit.player_id);
            return 1;
        } else {
            gatherer.target_resource = None;
        }
    }
    0
}

fn convert_military_to_workers(
    player_id: u8,
    commands: &mut Commands,
    military_units: &mut Query<
        (Entity, &mut Movement, &mut Combat, &RTSUnit, Option<&mut CombatState>),
        (With<Combat>, Without<ResourceGatherer>),
    >,
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) -> i32 {
    let mut units_converted = 0;
    
    for (entity, mut movement, mut combat, unit, mut combat_state_opt) in military_units.iter_mut() {
        if unit.player_id != player_id {
            continue;
        }

        clear_combat_state(&mut movement, &mut combat, combat_state_opt.as_deref_mut());

        if let Some((resource_entity, resource_pos)) = find_nearest_resource(&unit, &resource_sources) {
            commands.entity(entity).insert(ResourceGatherer {
                gather_rate: 8.0,
                capacity: 4.0,
                carried_amount: 0.0,
                resource_type: Some(resource_pos.1.clone()),
                target_resource: Some(resource_entity),
                drop_off_building: find_nearest_building_for_worker(unit, buildings),
            });
            movement.target_position = Some(resource_pos.0);
            units_converted += 1;
        }
    }
    
    units_converted
}

/// Find the nearest available resource for a unit
fn find_nearest_resource(
    unit: &RTSUnit,
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
) -> Option<(Entity, (Vec3, ResourceType))> {
    let home_base = estimate_home_position(unit.player_id);
    let mut nearest_resource = None;
    let mut nearest_distance = f32::MAX;

    for (entity, source, transform) in resource_sources.iter() {
        if source.amount > 0.0 {
            let distance = home_base.distance(transform.translation);
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest_resource = Some((
                    entity,
                    (transform.translation, source.resource_type.clone()),
                ));
            }
        }
    }

    nearest_resource
}

/// Find the nearest building for a worker to drop off resources
fn find_nearest_building_for_worker(
    unit: &RTSUnit,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) -> Option<Entity> {
    buildings
        .iter()
        .filter(|(_, _, building, building_unit)| {
            building_unit.player_id == unit.player_id
                && building.is_complete
                && matches!(
                    building.building_type,
                    BuildingType::Queen | BuildingType::StorageChamber | BuildingType::Nursery
                )
        })
        .min_by(|a, b| {
            let home_base = estimate_home_position(unit.player_id);
            let dist_a = home_base.distance(a.1.translation);
            let dist_b = home_base.distance(b.1.translation);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _, _, _)| entity)
}
