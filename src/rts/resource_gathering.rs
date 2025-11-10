use bevy::prelude::*;
use crate::core::components::*;

pub fn resource_gathering_system(
    mut gatherers: Query<(Entity, &mut ResourceGatherer, &mut Movement, &Transform, &RTSUnit), With<RTSUnit>>,
    mut resources: Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    buildings: Query<(Entity, &Transform), With<Building>>,
    mut player_resources: ResMut<crate::core::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::core::resources::AIResources>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (_gatherer_entity, mut gatherer, mut movement, gatherer_transform, unit) in gatherers.iter_mut() {
        process_resource_gathering(&mut gatherer, gatherer_transform, unit, &mut resources, delta_time, &mut commands);
        process_resource_delivery(&mut gatherer, &mut movement, gatherer_transform, unit, &resources, &buildings, &mut player_resources, &mut ai_resources);
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
    
    if distance > 5.0 {
        return;
    }
    
    if gatherer.carried_amount >= gatherer.capacity {
        gatherer.target_resource = None;
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
    buildings: &Query<(Entity, &Transform), With<Building>>,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    // Check if carrying full load
    if gatherer.carried_amount < gatherer.capacity {
        return;
    }

    let Some(dropoff_entity) = gatherer.drop_off_building else { return };

    // Get building transform - FIX: use buildings query, not resources query
    let Ok((_, building_transform)) = buildings.get(dropoff_entity) else {
        warn!("Drop-off building not found!");
        gatherer.drop_off_building = None;
        return;
    };

    let distance = gatherer_transform.translation.distance(building_transform.translation);

    // If far from dropoff, move towards it
    if distance > 10.0 {
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

        // Automatically return to resource gathering
        if let Some(resource_entity) = gatherer.target_resource {
            if let Ok((_, _, resource_transform)) = resources.get(resource_entity) {
                movement.target_position = Some(resource_transform.translation);
                info!("🔄 Worker {:?} returning to gather more resources", unit.unit_id);
            }
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
    if player_id == 1 {
        player_resources.add_resource(resource_type.clone(), amount);
        info!("Player delivered {:.1} {:?}", amount, resource_type);
    } else if let Some(ai_player_resources) = ai_resources.resources.get_mut(&player_id) {
        ai_player_resources.add_resource(resource_type.clone(), amount);
        info!("AI Player {} delivered {:.1} {:?}", player_id, amount, resource_type);
    }
}

fn reset_gatherer_cargo(gatherer: &mut ResourceGatherer) {
    gatherer.carried_amount = 0.0;
    gatherer.resource_type = None;
}