use bevy::prelude::*;
use crate::core::components::*;
use crate::ai::tactics::{TacticalManager, TacticalStance};

/// Component to track combat state for individual units
#[derive(Component, Debug, Clone)]
pub struct CombatState {
    pub last_attack_time: f32,
    pub current_target: Option<Entity>,
    pub is_retreating: bool,
    pub retreat_position: Option<Vec3>,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            last_attack_time: 0.0,
            current_target: None,
            is_retreating: false,
            retreat_position: None,
        }
    }
}

pub fn ai_combat_system(
    mut commands: Commands,
    mut ai_units: Query<(Entity, &mut Movement, &mut Combat, &Transform, &RTSUnit, &RTSHealth, Option<&mut CombatState>), With<Combat>>,
    all_units: Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
    tactical_manager: Option<Res<TacticalManager>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, mut movement, mut combat, unit_transform, unit, health, mut combat_state_opt) in ai_units.iter_mut() {
        // Skip player 1 units
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

        // Get tactical stance
        let stance = if let Some(ref manager) = tactical_manager {
            manager.player_tactics.get(&unit.player_id)
                .map(|t| t.current_stance.clone())
                .unwrap_or(TacticalStance::Defensive)
        } else {
            TacticalStance::Defensive
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
            stance,
            current_time,
        );

        // Update combat state
        if let Some(ref mut cs) = combat_state_opt {
            **cs = state;
        }
    }
}

fn handle_advanced_combat_ai(
    _self_entity: Entity,
    movement: &mut Movement,
    combat: &mut Combat,
    unit_transform: &Transform,
    unit: &RTSUnit,
    health: &RTSHealth,
    state: &mut CombatState,
    all_units: &Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
    stance: TacticalStance,
    current_time: f32,
) {
    // 1. Check if we should retreat (low health or outnumbered)
    if should_retreat(health, unit_transform, unit, all_units, &stance) {
        if !state.is_retreating {
            state.is_retreating = true;
            // Retreat towards home base
            let home_pos = estimate_home_position(unit.player_id);
            state.retreat_position = Some(home_pos);
        }

        if let Some(retreat_pos) = state.retreat_position {
            // Only update if significantly different to avoid jitter
            if should_update_target_position(&movement.target_position, retreat_pos, 2.0) {
                movement.target_position = Some(retreat_pos);
            }
        }
        return;
    } else {
        state.is_retreating = false;
        state.retreat_position = None;
    }

    // 2. Find best target (with target prioritization)
    let target_info = find_best_target(unit_transform, unit, all_units, state.current_target);

    if let Some((target_entity, target_pos, target_distance, _target_priority)) = target_info {
        state.current_target = Some(target_entity);

        // 3. Handle different unit types with different tactics
        match unit.unit_type.as_ref() {
            Some(UnitType::HunterWasp) => {
                // Ranged units: Kiting behavior - stay at max range
                handle_ranged_combat(movement, combat, unit_transform, target_pos, target_distance);
            }
            Some(UnitType::SoldierAnt | UnitType::BeetleKnight) => {
                // Melee units: Aggressive approach
                handle_melee_combat(movement, combat, unit_transform, target_pos, target_distance);
            }
            _ => {
                // Default combat behavior with threshold to prevent jitter
                let threshold = 1.0;
                if target_distance > combat.attack_range + threshold {
                    if should_update_target_position(&movement.target_position, target_pos, threshold) {
                        movement.target_position = Some(target_pos);
                    }
                } else if target_distance <= combat.attack_range && movement.current_velocity.length() < 1.0 {
                    movement.target_position = None;
                }
            }
        }

        state.last_attack_time = current_time;
    } else {
        // No target found - follow group or return to rally
        state.current_target = None;

        // If no enemy nearby and defensive stance, stay near home
        if stance == TacticalStance::Defensive {
            let home_pos = estimate_home_position(unit.player_id);
            if unit_transform.translation.distance(home_pos) > 40.0 {
                // Only update if significantly different to avoid jitter
                if should_update_target_position(&movement.target_position, home_pos, 2.0) {
                    movement.target_position = Some(home_pos);
                }
            }
        }
    }
}

/// Determines if unit should retreat based on health and enemy presence
fn should_retreat(
    health: &RTSHealth,
    unit_transform: &Transform,
    unit: &RTSUnit,
    all_units: &Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
    stance: &TacticalStance,
) -> bool {
    // Retreat if health is below 30%
    if health.current < health.max * 0.3 {
        return true;
    }

    // If stance is retreat, always retreat
    if *stance == TacticalStance::Retreat {
        return true;
    }

    // Count nearby allies and enemies
    let mut nearby_allies = 0;
    let mut nearby_enemies = 0;
    let detection_range = 40.0;

    for (_entity, enemy_transform, enemy_unit, _enemy_health) in all_units.iter() {
        let distance = unit_transform.translation.distance(enemy_transform.translation);
        if distance < detection_range {
            if enemy_unit.player_id == unit.player_id {
                nearby_allies += 1;
            } else {
                nearby_enemies += 1;
            }
        }
    }

    // Retreat if significantly outnumbered (3:1 or worse)
    if nearby_enemies >= 3 && nearby_allies < nearby_enemies / 3 {
        return true;
    }

    false
}

/// Find the best target with prioritization
fn find_best_target(
    unit_transform: &Transform,
    unit: &RTSUnit,
    all_units: &Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
    current_target: Option<Entity>,
) -> Option<(Entity, Vec3, f32, u32)> {
    let detection_range = 120.0;
    let mut best_target: Option<(Entity, Vec3, f32, u32)> = None;
    let mut best_priority = 0u32;

    // Check if current target is still valid
    if let Some(target_entity) = current_target {
        if let Ok((_entity, target_transform, target_unit, target_health)) = all_units.get(target_entity) {
            if target_unit.player_id != unit.player_id && target_health.current > 0.0 {
                let distance = unit_transform.translation.distance(target_transform.translation);
                if distance < detection_range {
                    let priority = calculate_target_priority(target_unit, target_health);
                    best_target = Some((target_entity, target_transform.translation, distance, priority));
                    best_priority = priority;
                }
            }
        }
    }

    // Find all enemies in range and prioritize
    for (entity, enemy_transform, enemy_unit, enemy_health) in all_units.iter() {
        if enemy_unit.player_id != unit.player_id && enemy_health.current > 0.0 {
            let distance = unit_transform.translation.distance(enemy_transform.translation);

            if distance < detection_range {
                let priority = calculate_target_priority(enemy_unit, enemy_health);

                // Prefer higher priority targets, or closer targets of same priority
                if priority > best_priority ||
                   (priority == best_priority && (best_target.is_none() || distance < best_target.as_ref().unwrap().2)) {
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
            UnitType::HunterWasp => priority = 7, // Kill ranged units
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
fn should_update_target_position(current_target: &Option<Vec3>, new_target: Vec3, threshold: f32) -> bool {
    match current_target {
        None => true, // No target set, always update
        Some(current) => current.distance(new_target) > threshold, // Only update if significantly different
    }
}

/// Ranged unit kiting behavior - maintain optimal distance
fn handle_ranged_combat(
    movement: &mut Movement,
    combat: &Combat,
    unit_transform: &Transform,
    target_pos: Vec3,
    distance: f32,
) {
    let optimal_range = combat.attack_range * 0.9; // Stay at 90% of max range
    let min_range = combat.attack_range * 0.7; // Don't get closer than 70%
    let threshold = 1.5; // Minimum distance change before updating position

    if distance < min_range - threshold {
        // Too close - back away (kiting)
        let retreat_direction = (unit_transform.translation - target_pos).normalize();
        let new_target = unit_transform.translation + retreat_direction * 10.0;

        // Only update if significantly different from current target
        if should_update_target_position(&movement.target_position, new_target, threshold) {
            movement.target_position = Some(new_target);
        }
    } else if distance > optimal_range + threshold {
        // Too far - move closer
        if should_update_target_position(&movement.target_position, target_pos, threshold) {
            movement.target_position = Some(target_pos);
        }
    } else {
        // In optimal range - maintain current position without micro-adjustments
        if movement.current_velocity.length() < 1.0 {
            movement.target_position = None;
        }
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

/// Estimate home base position for retreat
fn estimate_home_position(player_id: u8) -> Vec3 {
    match player_id {
        1 => Vec3::new(0.0, 0.0, 0.0),
        2 => Vec3::new(200.0, 0.0, 0.0),
        _ => Vec3::new((player_id as f32 - 1.0) * 200.0, 0.0, 0.0),
    }
}