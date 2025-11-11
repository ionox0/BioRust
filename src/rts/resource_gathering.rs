use bevy::prelude::*;
use crate::core::components::*;

// Configurable gathering parameters
const GATHERING_DISTANCE: f32 = 5.0;  // Distance within which gathering occurs
const DROPOFF_TRAVEL_DISTANCE: f32 = 10.0;  // Distance threshold for traveling to dropoff
const DROPOFF_REASSIGNMENT_THRESHOLD: f32 = 50.0;  // Only reassign if new building is this much closer

pub fn resource_gathering_system(
    mut gatherers: Query<(Entity, &mut ResourceGatherer, &mut Movement, &Transform, &RTSUnit), With<RTSUnit>>,
    mut resources: Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    buildings: Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
    mut player_resources: ResMut<crate::core::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::core::resources::AIResources>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (_gatherer_entity, mut gatherer, mut movement, gatherer_transform, unit) in gatherers.iter_mut() {
        // Ensure worker has a drop-off building assigned
        ensure_dropoff_building_assigned(&mut gatherer, gatherer_transform, unit, &buildings);

        process_resource_gathering(&mut gatherer, gatherer_transform, unit, &mut resources, delta_time, &mut commands);
        process_resource_delivery(&mut gatherer, &mut movement, gatherer_transform, unit, &resources, &buildings, &mut player_resources, &mut ai_resources);
    }
}

/// Ensure a worker has a drop-off building assigned with load balancing
fn ensure_dropoff_building_assigned(
    gatherer: &mut ResourceGatherer,
    worker_transform: &Transform,
    unit: &RTSUnit,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) {
    let mut current_distance = f32::MAX;

    // If gatherer already has a drop-off building, verify it still exists and is valid
    if let Some(current_dropoff) = gatherer.drop_off_building {
        if let Ok((_, building_transform, building, building_unit)) = buildings.get(current_dropoff) {
            // Check if building is still valid (same player, complete, can accept resources)
            if building_unit.player_id == unit.player_id && building.is_complete {
                current_distance = worker_transform.translation.distance(building_transform.translation);

                // Keep current assignment if it's reasonably close (within reassignment threshold)
                // This provides load balancing by reducing reassignment churn
                if current_distance < DROPOFF_REASSIGNMENT_THRESHOLD {
                    return; // Current drop-off is still valid and close enough
                }
            }
        }
        // Current drop-off is invalid or too far, clear it
        gatherer.drop_off_building = None;
    }

    // Find the closest suitable building for this worker's player
    let mut closest_building: Option<Entity> = None;
    let mut closest_distance = f32::MAX;

    for (building_entity, building_transform, building, building_unit) in buildings.iter() {
        // Only consider buildings owned by the same player
        if building_unit.player_id != unit.player_id {
            continue;
        }

        // Only consider completed buildings that can accept resources
        if !building.is_complete {
            continue;
        }

        // Check if building type can accept resources
        match building.building_type {
            BuildingType::Queen |
            BuildingType::StorageChamber |
            BuildingType::Nursery => {
                let distance = worker_transform.translation.distance(building_transform.translation);

                // Only consider this building if it's significantly closer than current assignment
                if current_distance == f32::MAX || distance < current_distance - DROPOFF_REASSIGNMENT_THRESHOLD {
                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_building = Some(building_entity);
                    }
                }
            }
            _ => continue,
        }
    }

    // Assign the closest building if we found a significantly better option
    if let Some(building) = closest_building {
        gatherer.drop_off_building = Some(building);
        info!("Assigned drop-off building to worker {} (player {}) at distance {:.1}",
              unit.unit_id, unit.player_id, closest_distance);
    }
}

fn process_resource_gathering(
    gatherer: &mut ResourceGatherer,
    gatherer_transform: &Transform,
    _unit: &RTSUnit,
    resources: &mut Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    delta_time: f32,
    commands: &mut Commands,
) {
    let Some(resource_entity) = gatherer.target_resource else { return };
    
    let Ok((_, mut resource, resource_transform)) = resources.get_mut(resource_entity) else {
        gatherer.target_resource = None;
        return;
    };
    
    let distance = gatherer_transform.translation.distance(resource_transform.translation);

    if distance > GATHERING_DISTANCE {
        return;
    }

    // If at full capacity, stop gathering but keep target_resource so we can return after delivery
    if gatherer.carried_amount >= gatherer.capacity {
        return;
    }

    let gather_amount = calculate_gather_amount(gatherer, &resource, delta_time);
    apply_gathering(gatherer, &mut resource, gather_amount);
    
    if resource.amount <= 0.0 {
        commands.entity(resource_entity).despawn();
        gatherer.target_resource = None;
    }
}

fn calculate_gather_amount(gatherer: &ResourceGatherer, resource: &ResourceSource, delta_time: f32) -> f32 {
    let base_amount = gatherer.gather_rate * delta_time;
    let capacity_limited = gatherer.capacity - gatherer.carried_amount;
    let resource_limited = resource.amount;
    
    base_amount.min(capacity_limited).min(resource_limited)
}

fn apply_gathering(gatherer: &mut ResourceGatherer, resource: &mut ResourceSource, amount: f32) {
    gatherer.carried_amount += amount;
    resource.amount -= amount;
    gatherer.resource_type = Some(resource.resource_type.clone());
}

fn process_resource_delivery(
    gatherer: &mut ResourceGatherer,
    movement: &mut Movement,
    gatherer_transform: &Transform,
    unit: &RTSUnit,
    resources: &Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    // Check if carrying any resources AND no active resource target
    // This ensures units return to base even if resource depletes before reaching full capacity
    if gatherer.carried_amount <= 0.0 {
        return;
    }

    // If still gathering from a resource, don't return yet (unless at full capacity)
    if gatherer.target_resource.is_some() && gatherer.carried_amount < gatherer.capacity {
        return;
    }

    let Some(dropoff_entity) = gatherer.drop_off_building else {
        warn!("Worker {} (player {}) has full load but no drop-off building assigned!", unit.unit_id, unit.player_id);
        return;
    };

    // Get building transform
    let Ok((_, building_transform, _, _)) = buildings.get(dropoff_entity) else {
        warn!("Drop-off building not found for worker {} (player {})!", unit.unit_id, unit.player_id);
        gatherer.drop_off_building = None;
        return;
    };

    let distance = gatherer_transform.translation.distance(building_transform.translation);

    // If far from dropoff, move towards it
    if distance > DROPOFF_TRAVEL_DISTANCE {
        // Set movement target to building if not already moving there
        if movement.target_position.is_none() ||
           movement.target_position.unwrap().distance(building_transform.translation) > 5.0 {
            movement.target_position = Some(building_transform.translation);
            info!("🚚 Worker {:?} returning to base with resources", unit.unit_id);
        }
        return;
    }

    // Close enough to deliver
    if let Some(resource_type) = gatherer.resource_type.clone() {
        deliver_resources_to_player(unit.player_id, resource_type, gatherer.carried_amount, player_resources, ai_resources);
        reset_gatherer_cargo(gatherer);

        // Automatically return to resource gathering if target still exists
        if let Some(resource_entity) = gatherer.target_resource {
            if let Ok((_, _, resource_transform)) = resources.get(resource_entity) {
                movement.target_position = Some(resource_transform.translation);
                info!("🔄 Worker {:?} returning to gather more resources", unit.unit_id);
            } else {
                // Resource no longer exists, clear target and movement so AI can assign new resource
                gatherer.target_resource = None;
                movement.target_position = None;
                info!("🏁 Worker {:?} completed delivery, resource depleted, becoming idle", unit.unit_id);
            }
        } else {
            // No target resource (was depleted before delivery), clear movement so AI can assign new resource
            movement.target_position = None;
            info!("🏁 Worker {:?} completed delivery, becoming idle", unit.unit_id);
        }
    }
}

fn deliver_resources_to_player(
    player_id: u8,
    resource_type: ResourceType,
    amount: f32,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    let mut manager = crate::core::resources::ResourceManager::new(player_resources, ai_resources);
    manager.add_resource(player_id, resource_type.clone(), amount);

    if player_id == 1 {
        info!("Player delivered {:.1} {:?}", amount, resource_type);
    } else {
        info!("AI Player {} delivered {:.1} {:?}", player_id, amount, resource_type);
    }
}

fn reset_gatherer_cargo(gatherer: &mut ResourceGatherer) {
    gatherer.carried_amount = 0.0;
    gatherer.resource_type = None;
}