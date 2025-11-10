use bevy::prelude::*;
use crate::core::components::*;

pub fn ai_unit_management_system(
    mut ai_units: Query<(&mut Movement, &mut ResourceGatherer, &RTSUnit), (With<RTSUnit>, With<ResourceGatherer>, Without<Combat>)>,
    resources: Query<(Entity, &Transform), With<ResourceSource>>,
    _time: Res<Time>,
) {
    for (mut movement, mut gatherer, unit) in ai_units.iter_mut() {
        if unit.player_id != 1 { // Handle all AI players (not human player 1)
            handle_idle_worker_ai(&mut movement, &mut gatherer, &resources);
        }
    }
}

fn handle_idle_worker_ai(
    movement: &mut Movement,
    gatherer: &mut ResourceGatherer,
    resources: &Query<(Entity, &Transform), With<ResourceSource>>,
) {
    // Only assign new targets if worker is idle and not carrying resources
    if movement.target_position.is_none() && gatherer.target_resource.is_none() && gatherer.carried_amount == 0.0 {
        if let Some((resource_entity, resource_pos)) = find_nearest_resource(resources) {
            movement.target_position = Some(resource_pos);
            gatherer.target_resource = Some(resource_entity);
        }
    }
}

fn find_nearest_resource(resources: &Query<(Entity, &Transform), With<ResourceSource>>) -> Option<(Entity, Vec3)> {
    resources.iter().next().map(|(entity, transform)| (entity, transform.translation))
}