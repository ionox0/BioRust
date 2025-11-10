use bevy::prelude::*;
use crate::core::components::*;

pub fn ai_combat_system(
    mut ai_units: Query<(&mut Movement, &mut Combat, &Transform, &RTSUnit), With<Combat>>,
    enemy_units: Query<(&Transform, &RTSUnit), (With<RTSUnit>, Without<Combat>)>,
) {
    for (mut movement, combat, unit_transform, unit) in ai_units.iter_mut() {
        if unit.player_id == 2 && combat.auto_attack { // AI player 2 combat units
            handle_combat_ai(&mut movement, &combat, unit_transform, unit, &enemy_units);
        }
    }
}

fn handle_combat_ai(
    movement: &mut Movement,
    combat: &Combat,
    unit_transform: &Transform,
    unit: &RTSUnit,
    enemy_units: &Query<(&Transform, &RTSUnit), (With<RTSUnit>, Without<Combat>)>,
) {
    if let Some((nearest_enemy_pos, distance)) = find_nearest_enemy(unit_transform, unit, enemy_units) {
        if distance > combat.attack_range {
            // Move towards enemy
            movement.target_position = Some(nearest_enemy_pos);
        } else {
            // In range - stop and attack
            movement.target_position = None;
        }
    }
}

fn find_nearest_enemy(
    unit_transform: &Transform,
    unit: &RTSUnit,
    enemy_units: &Query<(&Transform, &RTSUnit), (With<RTSUnit>, Without<Combat>)>,
) -> Option<(Vec3, f32)> {
    let mut nearest_enemy = None;
    let mut nearest_distance = f32::INFINITY;
    
    for (enemy_transform, enemy_unit) in enemy_units.iter() {
        if enemy_unit.player_id != unit.player_id {
            let distance = unit_transform.translation.distance(enemy_transform.translation);
            if distance < nearest_distance && distance < 100.0 { // Within detection range
                nearest_distance = distance;
                nearest_enemy = Some(enemy_transform.translation);
            }
        }
    }
    
    nearest_enemy.map(|pos| (pos, nearest_distance))
}