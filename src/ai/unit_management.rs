use bevy::prelude::*;
use crate::components::*;

pub fn ai_unit_management_system(
    mut ai_units: Query<(&mut Movement, &RTSUnit), (With<RTSUnit>, Without<Combat>)>,
    resources: Query<&Transform, With<ResourceSource>>,
    _time: Res<Time>,
) {
    for (mut movement, unit) in ai_units.iter_mut() {
        if unit.player_id == 2 { // Only AI player 2
            handle_idle_worker_ai(&mut movement, &resources);
        }
    }
}

fn handle_idle_worker_ai(
    movement: &mut Movement,
    resources: &Query<&Transform, With<ResourceSource>>,
) {
    if movement.target_position.is_none() {
        if let Some(nearest_resource) = find_nearest_resource(resources) {
            movement.target_position = Some(nearest_resource);
        }
    }
}

fn find_nearest_resource(resources: &Query<&Transform, With<ResourceSource>>) -> Option<Vec3> {
    resources.iter().next().map(|transform| transform.translation)
}