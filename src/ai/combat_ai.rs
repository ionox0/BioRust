use crate::ai::tactics::{TacticalManager, TacticalStance};
use crate::core::components::*;
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
    all_units: Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
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
            &all_units,
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
    all_units: &Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
    buildings: &Query<(&Transform, &CollisionRadius), (With<Building>, Without<RTSUnit>)>,
    environment_objects: &Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<RTSUnit>),
    >,
    stance: TacticalStance,
    current_time: f32,
) {
    let target_info = find_best_target(unit_transform, unit, all_units, state.target_entity);

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
        handle_no_target_behavior(
            movement,
            unit_transform,
            unit,
            all_units,
            buildings,
            environment_objects,
            stance,
            current_time,
            state,
        );
    }
}


/// Find the best target with prioritization
fn find_best_target(
    unit_transform: &Transform,
    unit: &RTSUnit,
    all_units: &Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
    target_entity: Option<Entity>,
) -> Option<(Entity, Vec3, f32, u32)> {
    let detection_range = 200.0; // Increased from 120.0 to 200.0 for more aggressive seeking
    let mut best_target: Option<(Entity, Vec3, f32, u32)> = None;
    let mut best_priority = 0u32;

    // Check if current target is still valid
    if let Some(target_entity) = target_entity {
        if let Ok((_entity, target_transform, target_unit, target_health)) =
            all_units.get(target_entity)
        {
            if target_unit.player_id != unit.player_id && target_health.current > 0.0 {
                let distance = unit_transform
                    .translation
                    .distance(target_transform.translation);
                if distance < detection_range {
                    let priority = calculate_target_priority(target_unit, target_health);
                    best_target = Some((
                        target_entity,
                        target_transform.translation,
                        distance,
                        priority,
                    ));
                    best_priority = priority;
                }
            }
        }
    }

    // Find all enemies in range and prioritize
    for (entity, enemy_transform, enemy_unit, enemy_health) in all_units.iter() {
        if enemy_unit.player_id != unit.player_id && enemy_health.current > 0.0 {
            let distance = unit_transform
                .translation
                .distance(enemy_transform.translation);

            if distance < detection_range {
                let priority = calculate_target_priority(enemy_unit, enemy_health);

                // Prefer higher priority targets, or closer targets of same priority
                if priority > best_priority
                    || (priority == best_priority
                        && (best_target.is_none() || distance < best_target.as_ref().unwrap().2))
                {
                    best_priority = priority;
                    best_target = Some((entity, enemy_transform.translation, distance, priority));
                }
            }
        }
    }

    best_target
}

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

/// Calculate a smart attack position that prevents units from converging on the same point
fn calculate_smart_attack_position(
    current_pos: Vec3,
    unit_id: u32,
    all_units: &Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
    buildings: &Query<(&Transform, &CollisionRadius), (With<Building>, Without<RTSUnit>)>,
    environment_objects: &Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<RTSUnit>),
    >,
    player_id: u8,
    current_time: f32,
) -> Option<Vec3> {
    // Find enemy units and buildings to attack
    let mut enemy_positions = Vec::new();
    let mut ally_positions = Vec::new();

    for (_, transform, unit, health) in all_units.iter() {
        if health.current > 0.0 {
            if unit.player_id != player_id {
                enemy_positions.push(transform.translation);
            } else if unit.unit_id != unit_id {
                ally_positions.push(transform.translation);
            }
        }
    }

    // If no enemies found, check if all enemies are truly eliminated
    if enemy_positions.is_empty() {
        // Check if there are any enemy buildings or units anywhere on the map
        let has_any_enemies = all_units
            .iter()
            .any(|(_, _, unit, health)| unit.player_id != player_id && health.current > 0.0);

        if !has_any_enemies {
            // VICTORY! All enemies eliminated - units should transition to resource gathering
            info!("🏆 AI unit {} transitioning from combat to resource gathering - all enemies eliminated!", unit_id);
            return None; // Return None to clear combat movement target and allow resource gathering system to take over
        } else {
            // Enemies exist somewhere but not in range - stay near our base instead of endless chasing
            let our_base = estimate_home_position(player_id);
            let distance_to_our_base = current_pos.distance(our_base);

            // Only move if we're very far from our base (avoid endless movement)
            if distance_to_our_base > 80.0 {
                return Some(our_base + (current_pos - our_base).normalize_or_zero() * 60.0);
            } else {
                // Close enough to base - stay put and let enemies come to us
                return None;
            }
        }
    }

    // Find closest enemy
    let closest_enemy = enemy_positions
        .iter()
        .min_by(|a, b| {
            current_pos
                .distance(**a)
                .partial_cmp(&current_pos.distance(**b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()?;

    // Don't chase enemies that are too far away to avoid endless pursuit
    let distance_to_closest = current_pos.distance(closest_enemy);
    if distance_to_closest > 150.0 {
        // Enemy too far - return to our base instead of chasing
        let our_base = estimate_home_position(player_id);
        let distance_to_our_base = current_pos.distance(our_base);
        
        if distance_to_our_base > 60.0 {
            return Some(our_base + (current_pos - our_base).normalize_or_zero() * 40.0);
        } else {
            return None; // Stay put near base
        }
    }

    // Calculate formation position around the closest enemy
    Some(calculate_formation_position_around_target(
        closest_enemy,
        current_pos,
        unit_id,
        &ally_positions,
        buildings,
        environment_objects,
        current_time,
    ))
}

/// Calculate a formation position around a target that avoids clustering
fn calculate_formation_position_around_target(
    target: Vec3,
    current_pos: Vec3,
    unit_id: u32,
    ally_positions: &[Vec3],
    buildings: &Query<(&Transform, &CollisionRadius), (With<Building>, Without<RTSUnit>)>,
    environment_objects: &Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<RTSUnit>),
    >,
    current_time: f32,
) -> Vec3 {
    // Use unit ID and time to create unique positioning
    let unit_offset = (unit_id % 100) as f32 * 0.1;
    let time_offset = (current_time * 0.2).sin() * 0.3;
    let unique_angle = (unit_id as f32 * 2.17) + unit_offset + time_offset; // 2.17 creates good distribution

    // Create looser ring-based formation around target
    let ring_number = ((unit_id % 24) / 8) as f32; // Units in rings of 8 (more spread out)
    let position_in_ring = (unit_id % 8) as f32;

    let base_radius = 30.0 + (ring_number * 20.0); // Larger rings at 30, 50, 70 units from target
    let angle_in_ring = (position_in_ring * std::f32::consts::PI * 2.0 / 8.0) + unique_angle;

    // Calculate ideal formation position
    let ideal_formation_pos = target
        + Vec3::new(
            angle_in_ring.cos() * base_radius,
            0.0,
            angle_in_ring.sin() * base_radius,
        );

    // Check if formation position is too crowded
    let mut final_position = ideal_formation_pos;

    // Avoid clustering with allies - much more aggressive spacing
    for &ally_pos in ally_positions {
        let distance_to_ally = final_position.distance(ally_pos);
        if distance_to_ally < 15.0 {
            // Larger minimum distance from allies
            // Push away from ally with more spacing
            let push_direction = (final_position - ally_pos).normalize_or_zero();
            final_position = ally_pos + push_direction * 20.0;
        }
    }

    // Ensure minimum distance from target for tactical positioning
    let distance_to_target = final_position.distance(target);
    if distance_to_target < 20.0 {
        let direction_to_target = (target - final_position).normalize_or_zero();
        final_position = target - direction_to_target * 20.0;
    }

    // OBSTACLE AVOIDANCE: Check if position conflicts with buildings or environment objects
    final_position = avoid_obstacles(final_position, current_pos, buildings, environment_objects);

    // MAP BOUNDARY ENFORCEMENT: Keep units within map boundaries
    final_position = clamp_to_map_boundary(final_position);

    final_position
}

/// Clamp position to map boundaries to prevent units from going outside the playable area
fn clamp_to_map_boundary(pos: Vec3) -> Vec3 {
    use crate::core::constants::movement::MAP_BOUNDARY;
    Vec3::new(
        pos.x.clamp(-MAP_BOUNDARY, MAP_BOUNDARY),
        pos.y,
        pos.z.clamp(-MAP_BOUNDARY, MAP_BOUNDARY),
    )
}

/// Adjust position to avoid obstacles like buildings and environment objects
fn avoid_obstacles(
    desired_position: Vec3,
    current_position: Vec3,
    buildings: &Query<(&Transform, &CollisionRadius), (With<Building>, Without<RTSUnit>)>,
    environment_objects: &Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<RTSUnit>),
    >,
) -> Vec3 {
    let mut final_position = desired_position;
    let unit_radius = 2.0; // Approximate unit radius
    let safety_margin = 3.0;

    // Check against buildings
    for (building_transform, building_collision) in buildings.iter() {
        let distance = final_position.distance(building_transform.translation);
        let min_safe_distance = unit_radius + building_collision.radius + safety_margin;

        if distance < min_safe_distance {
            // Push unit away from building
            let push_direction =
                (final_position - building_transform.translation).normalize_or_zero();

            // If units are stuck inside building, push toward their current position instead
            if push_direction.length() < 0.1 {
                let fallback_direction =
                    (current_position - building_transform.translation).normalize_or_zero();
                final_position =
                    building_transform.translation + fallback_direction * min_safe_distance;
            } else {
                final_position =
                    building_transform.translation + push_direction * min_safe_distance;
            }

            debug!("Avoided building obstacle, moved to safe position");
        }
    }

    // Check against environment objects
    for (env_transform, env_collision) in environment_objects.iter() {
        let distance = final_position.distance(env_transform.translation);
        let min_safe_distance = unit_radius + env_collision.radius + safety_margin;

        if distance < min_safe_distance {
            // Push unit away from environment object
            let push_direction = (final_position - env_transform.translation).normalize_or_zero();

            // If units are stuck inside object, push toward their current position instead
            if push_direction.length() < 0.1 {
                let fallback_direction =
                    (current_position - env_transform.translation).normalize_or_zero();
                final_position = env_transform.translation + fallback_direction * min_safe_distance;
            } else {
                final_position = env_transform.translation + push_direction * min_safe_distance;
            }

            debug!("Avoided environment obstacle, moved to safe position");
        }
    }

    final_position
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

fn handle_no_target_behavior(
    movement: &mut Movement,
    unit_transform: &Transform,
    unit: &RTSUnit,
    all_units: &Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
    buildings: &Query<(&Transform, &CollisionRadius), (With<Building>, Without<RTSUnit>)>,
    environment_objects: &Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<RTSUnit>),
    >,
    stance: TacticalStance,
    current_time: f32,
    state: &mut CombatState,
) {
    state.target_entity = None;

    if (stance == TacticalStance::Aggressive || stance == TacticalStance::Defensive)
        && unit.player_id == 2
    {
        handle_ai_aggressive_positioning(
            movement,
            unit_transform,
            unit,
            all_units,
            buildings,
            environment_objects,
            current_time,
        );
    }
}

fn handle_ai_aggressive_positioning(
    movement: &mut Movement,
    unit_transform: &Transform,
    unit: &RTSUnit,
    all_units: &Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
    buildings: &Query<(&Transform, &CollisionRadius), (With<Building>, Without<RTSUnit>)>,
    environment_objects: &Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<RTSUnit>),
    >,
    current_time: f32,
) {
    let smart_target_pos = calculate_smart_attack_position(
        unit_transform.translation,
        unit.unit_id,
        all_units,
        buildings,
        environment_objects,
        unit.player_id,
        current_time,
    );

    if let Some(target_pos) = smart_target_pos {
        let distance_to_target = unit_transform.translation.distance(target_pos);

        if distance_to_target > 10.0 {
            if should_update_target_position(&movement.target_position, target_pos, 5.0) {
                movement.target_position = Some(target_pos);
                debug!(
                    "AI unit {} moving to smart attack position at distance {:.1}",
                    unit.unit_id, distance_to_target
                );
            }
        } else {
            movement.target_position = None;
        }
    } else {
        movement.target_position = None;
        debug!(
            "AI unit {} clearing combat movement target - transitioning to resource gathering",
            unit.unit_id
        );
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
    // Check if victory conditions are met for each AI player
    for player_id in 2..=4 {
        // AI players 2-4
        let has_enemies = all_units
            .iter()
            .any(|unit| unit.player_id != player_id && unit.player_id == 1);

        if !has_enemies {
            // Handle AI workers returning to resource gathering
            let mut workers_reassigned = 0;
            for (_entity, mut movement, mut combat, unit, mut gatherer, mut combat_state_opt) in ai_workers.iter_mut() {
                if unit.player_id == player_id {
                    // Clear combat state and movement
                    movement.target_position = None;
                    movement.current_velocity = Vec3::ZERO;
                    combat.target = None;
                    combat.is_attacking = false;
                    combat.last_attack_time = 0.0;

                    // Clear combat state
                    if let Some(ref mut combat_state) = combat_state_opt {
                        combat_state.target_entity = None;
                        combat_state.state = crate::core::components::CombatStateType::Idle;
                        combat_state.target_position = None;
                    }

                    // If worker doesn't have a resource assignment, find one
                    if gatherer.target_resource.is_none() {
                        if let Some((resource_entity, resource_pos)) = find_nearest_resource(&unit, &resource_sources) {
                            gatherer.target_resource = Some(resource_entity);
                            gatherer.resource_type = Some(resource_pos.1.clone());
                            gatherer.carried_amount = 0.0;
                            gatherer.drop_off_building = find_nearest_building_for_worker(unit, &buildings);
                            movement.target_position = Some(resource_pos.0);
                            
                            info!("🔄 AI Worker {} (player {}) returning to gather {:?} after combat", 
                                  unit.unit_id, unit.player_id, resource_pos.1);
                            workers_reassigned += 1;
                        }
                    } else {
                        // Worker already has assignment, just ensure they're moving to it
                        if let Ok((_, resource_source, resource_transform)) = resource_sources.get(gatherer.target_resource.unwrap()) {
                            if resource_source.amount > 0.0 {
                                movement.target_position = Some(resource_transform.translation);
                                info!("🔄 AI Worker {} (player {}) resuming resource gathering after combat", 
                                      unit.unit_id, unit.player_id);
                                workers_reassigned += 1;
                            } else {
                                // Resource depleted, find a new one
                                gatherer.target_resource = None;
                            }
                        }
                    }
                }
            }

            // Handle pure military units (convert to workers if desired)
            let mut units_converted = 0;
            for (entity, mut movement, mut combat, unit, mut combat_state_opt) in military_units.iter_mut() {
                if unit.player_id == player_id {
                    // Clear combat state and movement
                    movement.target_position = None;
                    movement.current_velocity = Vec3::ZERO;
                    combat.target = None;
                    combat.is_attacking = false;
                    combat.last_attack_time = 0.0;

                    // Clear combat state
                    if let Some(ref mut combat_state) = combat_state_opt {
                        combat_state.target_entity = None;
                        combat_state.state = crate::core::components::CombatStateType::Idle;
                        combat_state.target_position = None;
                    }

                    // Convert to resource gatherer
                    if let Some((resource_entity, resource_pos)) = find_nearest_resource(&unit, &resource_sources) {
                        commands.entity(entity).insert(ResourceGatherer {
                            gather_rate: 8.0,
                            capacity: 4.0,
                            carried_amount: 0.0,
                            resource_type: Some(resource_pos.1.clone()),
                            target_resource: Some(resource_entity),
                            drop_off_building: find_nearest_building_for_worker(unit, &buildings),
                        });
                        movement.target_position = Some(resource_pos.0);
                        units_converted += 1;
                    }
                }
            }

            // Log summary
            if workers_reassigned > 0 || units_converted > 0 {
                info!("🏆 VICTORY! AI Player {} - {} workers resumed gathering, {} military converted to workers", 
                      player_id, workers_reassigned, units_converted);
            }
        }
    }
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
