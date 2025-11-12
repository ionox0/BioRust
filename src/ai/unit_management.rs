use crate::core::components::*;
use bevy::prelude::*;

pub fn ai_unit_management_system(
    mut ai_units: Query<
        (&Transform, &mut Movement, &mut ResourceGatherer, &RTSUnit),
        (With<RTSUnit>, With<ResourceGatherer>, Without<Combat>),
    >,
    resources: Query<(Entity, &Transform), With<ResourceSource>>,
    _time: Res<Time>,
) {
    for (transform, mut movement, mut gatherer, unit) in ai_units.iter_mut() {
        if unit.player_id != 1 {
            // Handle all AI players (not human player 1)
            handle_idle_worker_ai(transform, &mut movement, &mut gatherer, &resources);
        }
    }
}

fn handle_idle_worker_ai(
    worker_transform: &Transform,
    movement: &mut Movement,
    gatherer: &mut ResourceGatherer,
    resources: &Query<(Entity, &Transform), With<ResourceSource>>,
) {
    // Only assign new targets if worker is truly idle:
    // - No movement target
    // - No target resource
    // - Not carrying resources
    // - No resource type assigned (meaning economy system hasn't handled this worker yet)
    if movement.target_position.is_none()
        && gatherer.target_resource.is_none()
        && gatherer.carried_amount == 0.0
        && gatherer.resource_type.is_none()
    {
        // This is a fallback for workers not yet assigned by the economy system
        if let Some((resource_entity, resource_pos)) =
            find_nearest_resource(worker_transform.translation, resources)
        {
            movement.target_position = Some(resource_pos);
            gatherer.target_resource = Some(resource_entity);
        }
    }
}

fn find_nearest_resource(
    worker_position: Vec3,
    resources: &Query<(Entity, &Transform), With<ResourceSource>>,
) -> Option<(Entity, Vec3)> {
    resources
        .iter()
        .map(|(entity, transform)| {
            let distance = worker_position.distance(transform.translation);
            (entity, transform.translation, distance)
        })
        .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(entity, position, _distance)| (entity, position))
}
