use crate::ai::intelligence::IntelligenceSystem;
use crate::core::components::*;
use bevy::prelude::*;

/// Component to mark units as scouts
#[derive(Component, Debug, Clone)]
pub struct ScoutUnit {
    #[allow(dead_code)]
    pub assigned_player_id: u8,
    pub scout_target: Vec3,
    pub is_returning: bool,
    pub has_found_enemy: bool,
}

/// System to manage scouting behavior
pub fn scouting_system(
    mut commands: Commands,
    mut intelligence: ResMut<IntelligenceSystem>,
    mut scouts: Query<(Entity, &mut Movement, &mut ScoutUnit, &Transform, &RTSUnit)>,
    enemy_units: Query<(&Transform, &RTSUnit), (With<RTSUnit>, Without<ScoutUnit>)>,
    workers: Query<
        (Entity, &RTSUnit),
        (With<ResourceGatherer>, Without<ScoutUnit>, Without<Combat>),
    >,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Check if AI players need to send scouts
    for (ai_player_id, intel) in intelligence.player_intel.iter_mut() {
        // Send scout early in the game if we don't have one
        if intel.scout_unit.is_none() && current_time > 10.0 && current_time < 120.0 {
            // Find a worker to send as scout
            if let Some((worker_entity, _worker_unit)) = workers
                .iter()
                .find(|(_, unit)| unit.player_id == *ai_player_id)
            {
                // Estimate enemy base location (opposite side of map)
                let scout_target = estimate_enemy_base_location(*ai_player_id);

                commands.entity(worker_entity).insert(ScoutUnit {
                    assigned_player_id: *ai_player_id,
                    scout_target,
                    is_returning: false,
                    has_found_enemy: false,
                });

                intel.scout_unit = Some(worker_entity);
                info!(
                    "AI Player {} sending scout to {:?}",
                    ai_player_id, scout_target
                );
            }
        }
    }

    // Handle scout movement and behavior
    for (scout_entity, mut movement, mut scout, transform, unit) in scouts.iter_mut() {
        // Check if scout has found enemy units
        if !scout.has_found_enemy {
            for (enemy_transform, enemy_unit) in enemy_units.iter() {
                if enemy_unit.player_id != unit.player_id {
                    let distance = transform.translation.distance(enemy_transform.translation);
                    if distance < 50.0 {
                        scout.has_found_enemy = true;

                        // Update intelligence with enemy base location
                        if let Some(intel) = intelligence.get_intel_mut(unit.player_id) {
                            intel.enemy_base_location = Some(enemy_transform.translation);
                            info!(
                                "Scout from Player {} found enemy base at {:?}",
                                unit.player_id, enemy_transform.translation
                            );
                        }

                        // Start returning home
                        scout.is_returning = true;
                        break;
                    }
                }
            }
        }

        // Scout movement logic
        if !scout.is_returning {
            // Move to scout target
            movement.target_position = Some(scout.scout_target);

            // If reached target, start exploring nearby area
            if transform.translation.distance(scout.scout_target) < 5.0 {
                // Circle around the area
                let angle = (current_time * 0.5).sin() * std::f32::consts::PI;
                let offset = Vec3::new(angle.cos() * 30.0, 0.0, angle.sin() * 30.0);
                movement.target_position = Some(scout.scout_target + offset);
            }
        } else {
            // Return to base
            let home_position = estimate_home_base_location(unit.player_id);
            movement.target_position = Some(home_position);

            // If back at base, remove scout component
            if transform.translation.distance(home_position) < 20.0 {
                commands.entity(scout_entity).remove::<ScoutUnit>();

                // Clear scout reference in intelligence
                if let Some(intel) = intelligence.get_intel_mut(unit.player_id) {
                    intel.scout_unit = None;
                }
            }
        }
    }
}

/// Estimate enemy base location based on player ID
fn estimate_enemy_base_location(ai_player_id: u8) -> Vec3 {
    // Enemies are typically on opposite side of map
    // Player 1 is at x = 0, Player 2 is at x = 200
    match ai_player_id {
        2 => Vec3::new(0.0, 0.0, 0.0),   // AI player 2 scouts towards player 1
        _ => Vec3::new(200.0, 0.0, 0.0), // Other AI players scout opposite direction
    }
}

/// Estimate home base location based on player ID
fn estimate_home_base_location(player_id: u8) -> Vec3 {
    match player_id {
        1 => Vec3::new(0.0, 0.0, 0.0),
        2 => Vec3::new(200.0, 0.0, 0.0),
        _ => Vec3::new((player_id as f32 - 1.0) * 200.0, 0.0, 0.0),
    }
}

/// System to handle scout survival - retreat if under attack
pub fn scout_survival_system(
    mut scouts: Query<(
        &mut Movement,
        &mut ScoutUnit,
        &Transform,
        &RTSHealth,
        &RTSUnit,
    )>,
    enemy_units: Query<(&Transform, &RTSUnit, &Combat), (With<Combat>, Without<ScoutUnit>)>,
) {
    for (mut movement, mut scout, transform, health, unit) in scouts.iter_mut() {
        // If health is low or enemy military nearby, retreat immediately
        if health.current < health.max * 0.5 {
            scout.is_returning = true;
            continue;
        }

        // Check for nearby enemy military units
        for (enemy_transform, enemy_unit, _combat) in enemy_units.iter() {
            if enemy_unit.player_id != unit.player_id {
                let distance = transform.translation.distance(enemy_transform.translation);
                if distance < 30.0 {
                    // Enemy military nearby - retreat!
                    scout.is_returning = true;
                    let retreat_direction =
                        (transform.translation - enemy_transform.translation).normalize();
                    movement.target_position =
                        Some(transform.translation + retreat_direction * 50.0);
                    break;
                }
            }
        }
    }
}
