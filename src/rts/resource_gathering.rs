use bevy::prelude::*;
use crate::components::*;

pub fn resource_gathering_system(
    mut gatherers: Query<(Entity, &mut ResourceGatherer, &Transform, &RTSUnit), With<RTSUnit>>,
    mut resources: Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    mut player_resources: ResMut<crate::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::resources::AIResources>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (_gatherer_entity, mut gatherer, gatherer_transform, unit) in gatherers.iter_mut() {
        process_resource_gathering(&mut gatherer, gatherer_transform, unit, &mut resources, delta_time, &mut commands);
        process_resource_delivery(&mut gatherer, gatherer_transform, unit, &resources, &mut player_resources, &mut ai_resources);
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
    gatherer_transform: &Transform,
    unit: &RTSUnit,
    resources: &Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    player_resources: &mut ResMut<crate::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::resources::AIResources>,
) {
    if gatherer.carried_amount < gatherer.capacity {
        return;
    }
    
    let Some(dropoff_entity) = gatherer.drop_off_building else { return };
    
    let Ok(dropoff_query) = resources.get(dropoff_entity) else { return };
    
    let distance = gatherer_transform.translation.distance(dropoff_query.2.translation);
    if distance > 10.0 {
        return;
    }
    
    if let Some(resource_type) = gatherer.resource_type.clone() {
        deliver_resources_to_player(unit.player_id, resource_type, gatherer.carried_amount, player_resources, ai_resources);
        reset_gatherer_cargo(gatherer);
    }
}

fn deliver_resources_to_player(
    player_id: u8,
    resource_type: ResourceType,
    amount: f32,
    player_resources: &mut ResMut<crate::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::resources::AIResources>,
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