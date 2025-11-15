use crate::core::components::*;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

// Use centralized constants from core module
use crate::constants::resource_interaction::{DROPOFF_TRAVEL_DISTANCE, GATHERING_DISTANCE};

pub fn resource_gathering_system(
    mut gatherers: Query<
        (
            Entity,
            &mut ResourceGatherer,
            &mut Movement,
            &Transform,
            &RTSUnit,
        ),
        With<RTSUnit>,
    >,
    mut resources: Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    buildings: Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
    mut player_resources: ResMut<crate::core::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::core::resources::AIResources>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (_gatherer_entity, mut gatherer, mut movement, gatherer_transform, unit) in
        gatherers.iter_mut()
    {
        // Ensure worker has a drop-off building assigned
        ensure_dropoff_building_assigned(&mut gatherer, gatherer_transform, unit, &buildings);

        process_resource_gathering(
            &mut gatherer,
            gatherer_transform,
            unit,
            &mut resources,
            delta_time,
            &mut commands,
        );
        process_resource_delivery(
            &mut gatherer,
            &mut movement,
            gatherer_transform,
            unit,
            &resources,
            &buildings,
            &mut player_resources,
            &mut ai_resources,
        );
    }
}

/// Ensure a worker has a drop-off building assigned with load balancing
fn ensure_dropoff_building_assigned(
    gatherer: &mut ResourceGatherer,
    worker_transform: &Transform,
    unit: &RTSUnit,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) {
    // If gatherer already has a drop-off building, verify it still exists and is valid
    if let Some(current_dropoff) = gatherer.drop_off_building {
        if let Ok((_, _building_transform, building, building_unit)) =
            buildings.get(current_dropoff)
        {
            // Check if building is still valid (same player, complete, can accept resources)
            let is_correct_type = matches!(
                building.building_type,
                BuildingType::Queen | BuildingType::StorageChamber | BuildingType::Nursery
            );
            let is_valid = building_unit.player_id == unit.player_id
                && building.is_complete
                && is_correct_type;

            if is_valid {
                // Building is valid - keep it assigned!
                // Workers can be thousands of units away while gathering - that's totally fine
                return; // Current drop-off is valid, keep it
            }
        }

        // Only clear if building is invalid (destroyed, wrong player, incomplete, wrong type)
        // Do NOT clear just because worker is far away!
        gatherer.drop_off_building = None;
    }

    // Find the closest suitable building for this worker's player
    let mut closest_building: Option<Entity> = None;
    let mut closest_distance = f32::MAX;

    // DEBUG: Count buildings to diagnose the issue
    let total_buildings = buildings.iter().count();
    let player_buildings = buildings
        .iter()
        .filter(|(_, _, _, b)| b.player_id == unit.player_id)
        .count();
    let complete_buildings = buildings
        .iter()
        .filter(|(_, _, b, u)| u.player_id == unit.player_id && b.is_complete)
        .count();

    // Throttle the building debugging logs to avoid spam
    static BUILDING_DEBUG_LOG: LazyLock<Mutex<HashMap<u8, f32>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    if player_buildings == 0 && gatherer.drop_off_building.is_none() {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();

        let mut log = BUILDING_DEBUG_LOG.lock().unwrap();
        let last_time = log.get(&unit.player_id).copied().unwrap_or(0.0);
        if current_time - last_time > 5.0 {
            // Only log every 5 seconds per player
            warn!("🏛️ DEBUG: Player {} - Total buildings on map: {}, Player's buildings: {}, Complete: {}",
                  unit.player_id, total_buildings, player_buildings, complete_buildings);
            log.insert(unit.player_id, current_time);
        }
    }

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
            BuildingType::Queen | BuildingType::StorageChamber | BuildingType::Nursery => {
                let distance = worker_transform
                    .translation
                    .distance(building_transform.translation);

                // Find the closest building
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_building = Some(building_entity);
                }
            }
            _ => continue,
        }
    }

    // Assign the closest building if we found a significantly better option
    if let Some(building) = closest_building {
        gatherer.drop_off_building = Some(building);
        // Only log if worker has resources or is actively gathering (not at startup)
        if gatherer.carried_amount > 0.0 || gatherer.target_resource.is_some() {
            info!(
                "✅ Assigned drop-off building to worker {} (player {}) at distance {:.1}",
                unit.unit_id, unit.player_id, closest_distance
            );
        }
    } else if gatherer.drop_off_building.is_none() {
        // Worker needs dropoff but none found - log this as it's a problem
        // Only log once per second to avoid spam
        static LAST_WARNING_TIME: LazyLock<Mutex<f32>> = LazyLock::new(|| Mutex::new(0.0));
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();

        let mut last_time = LAST_WARNING_TIME.lock().unwrap();
        if current_time - *last_time > 1.0 {
            warn!("⚠️  Worker {} (player {}) needs drop-off building but none available (no Queen/StorageChamber/Nursery found)",
                  unit.unit_id, unit.player_id);
            warn!(
                "   Buildings check: {} total, {} for player {}, {} complete",
                total_buildings, player_buildings, unit.player_id, complete_buildings
            );
            *last_time = current_time;
        }
    }
}

fn process_resource_gathering(
    gatherer: &mut ResourceGatherer,
    gatherer_transform: &Transform,
    unit: &RTSUnit,
    resources: &mut Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    delta_time: f32,
    commands: &mut Commands,
) {
    let Some(resource_entity) = gatherer.target_resource else {
        return;
    };

    let Ok((_, mut resource, resource_transform)) = resources.get_mut(resource_entity) else {
        gatherer.target_resource = None;
        return;
    };

    let distance = gatherer_transform
        .translation
        .distance(resource_transform.translation);

    // Log distance for debugging collision issues
    static DISTANCE_LOG: LazyLock<Mutex<HashMap<u32, f32>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f32();

    let mut log = DISTANCE_LOG.lock().unwrap();
    let last_time = log.get(&unit.unit_id).copied().unwrap_or(0.0);
    if current_time - last_time > 2.0 {
        info!(
            "🎯 Worker {} (player {}) distance to resource: {:.1} units (need < {:.1})",
            unit.unit_id, unit.player_id, distance, GATHERING_DISTANCE
        );
        log.insert(unit.unit_id, current_time);
    }

    if distance > GATHERING_DISTANCE {
        return;
    }

    // If at full capacity, stop gathering but keep target_resource so we can return after delivery
    if gatherer.carried_amount >= gatherer.capacity {
        // Log when worker reaches full capacity and stops gathering
        static LAST_FULL_LOG: LazyLock<Mutex<HashMap<u32, f32>>> =
            LazyLock::new(|| Mutex::new(HashMap::new()));
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();

        let mut log = LAST_FULL_LOG.lock().unwrap();
        let last_time = log.get(&unit.unit_id).copied().unwrap_or(0.0);
        if current_time - last_time > 2.0 {
            info!(
                "📦 Worker {} (player {}) FULL: {:.1}/{:.1} - waiting to return",
                unit.unit_id, unit.player_id, gatherer.carried_amount, gatherer.capacity
            );
            log.insert(unit.unit_id, current_time);
        }
        return;
    }

    let gather_amount = calculate_gather_amount(gatherer, &resource, delta_time);

    // Log gathering progress periodically
    if gather_amount > 0.0 {
        static LAST_GATHER_LOG: LazyLock<Mutex<HashMap<u32, f32>>> =
            LazyLock::new(|| Mutex::new(HashMap::new()));
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();

        let mut log = LAST_GATHER_LOG.lock().unwrap();
        let last_time = log.get(&unit.unit_id).copied().unwrap_or(0.0);
        if current_time - last_time > 1.0 {
            info!(
                "⛏️  Worker {} (player {}) gathering {:?}: {:.1}/{:.1} (resource has {:.1} left)",
                unit.unit_id,
                unit.player_id,
                gatherer.resource_type,
                gatherer.carried_amount,
                gatherer.capacity,
                resource.amount
            );
            log.insert(unit.unit_id, current_time);
        }
    }

    apply_gathering(gatherer, &mut resource, gather_amount);

    if resource.amount <= 0.0 {
        info!(
            "🔴 Resource depleted! Worker {} (player {}) has {:.1}/{:.1} resources",
            unit.unit_id, unit.player_id, gatherer.carried_amount, gatherer.capacity
        );
        commands.entity(resource_entity).despawn();
        gatherer.target_resource = None;
    }
}

fn calculate_gather_amount(
    gatherer: &ResourceGatherer,
    resource: &ResourceSource,
    delta_time: f32,
) -> f32 {
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
    if !should_start_delivery(gatherer) {
        return;
    }

    log_delivery_decision(gatherer, unit);

    let Some(dropoff_entity) = gatherer.drop_off_building else {
        handle_no_dropoff_building(gatherer, movement, unit);
        return;
    };

    let Ok((_, building_transform, _, _)) = buildings.get(dropoff_entity) else {
        handle_invalid_dropoff_building(gatherer, unit);
        return;
    };

    let distance = gatherer_transform
        .translation
        .distance(building_transform.translation);
    log_dropoff_distance(distance, unit);

    if distance > DROPOFF_TRAVEL_DISTANCE {
        handle_travel_to_dropoff(movement, building_transform, gatherer, unit, distance);
        return;
    }

    handle_resource_delivery_at_dropoff(
        gatherer,
        movement,
        unit,
        resources,
        player_resources,
        ai_resources,
    );
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

    // Removed resource delivery logging for performance
}

fn should_start_delivery(gatherer: &ResourceGatherer) -> bool {
    if gatherer.carried_amount <= 0.0 {
        return false;
    }

    // If still gathering from a resource, don't return yet (unless at full capacity)
    !(gatherer.target_resource.is_some() && gatherer.carried_amount < gatherer.capacity)
}

fn log_delivery_decision(gatherer: &ResourceGatherer, unit: &RTSUnit) {
    static DELIVERY_DECISION_LOGGED: LazyLock<Mutex<(HashSet<u32>, f32)>> =
        LazyLock::new(|| Mutex::new((HashSet::new(), 0.0)));
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f32();

    let mut logged = DELIVERY_DECISION_LOGGED.lock().unwrap();
    // Clear the set every 60 seconds to prevent memory leaks and allow repeated logging
    if current_time - logged.1 > 60.0 {
        logged.0.clear();
        logged.1 = current_time;
    }

    if !logged.0.contains(&unit.unit_id) {
        let reason = if gatherer.carried_amount >= gatherer.capacity {
            "FULL CAPACITY"
        } else if gatherer.target_resource.is_none() {
            "RESOURCE DEPLETED"
        } else {
            "UNKNOWN"
        };
        info!(
            "🔵 Worker {} (player {}) READY TO RETURN: {:.1}/{:.1} resources (reason: {})",
            unit.unit_id, unit.player_id, gatherer.carried_amount, gatherer.capacity, reason
        );
        logged.0.insert(unit.unit_id);
    }
}

fn handle_no_dropoff_building(
    gatherer: &mut ResourceGatherer,
    movement: &mut Movement,
    unit: &RTSUnit,
) {
    // Worker has resources but no drop-off building (likely no buildings built yet)
    // Clear target_resource and movement so worker waits idle until a building is assigned
    if gatherer.target_resource.is_some() || movement.target_position.is_some() {
        gatherer.target_resource = None;
        movement.target_position = None;

        // Rate limit this warning to once every 30 seconds per worker to prevent spam
        use std::collections::HashMap;
        use std::sync::Mutex;
        static LAST_WARN_TIMES: std::sync::LazyLock<Mutex<HashMap<u32, f32>>> =
            std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

        if let Ok(mut warn_times) = LAST_WARN_TIMES.lock() {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f32();

            let last_warn = warn_times.get(&unit.unit_id).copied().unwrap_or(0.0);
            if current_time - last_warn > 30.0 {
                debug!("Worker {} (player {}) has {:.1} resources but no drop-off building available - waiting for building to be constructed",
                      unit.unit_id, unit.player_id, gatherer.carried_amount);
                warn_times.insert(unit.unit_id, current_time);
            }
        }
    }
}

fn handle_invalid_dropoff_building(gatherer: &mut ResourceGatherer, unit: &RTSUnit) {
    warn!(
        "Drop-off building not found for worker {} (player {})!",
        unit.unit_id, unit.player_id
    );
    gatherer.drop_off_building = None;
}

fn log_dropoff_distance(distance: f32, unit: &RTSUnit) {
    static DROPOFF_DISTANCE_LOG: LazyLock<Mutex<HashMap<u32, f32>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f32();
    let mut log = DROPOFF_DISTANCE_LOG.lock().unwrap();
    let last_time = log.get(&unit.unit_id).copied().unwrap_or(0.0);
    if current_time - last_time > 2.0 {
        info!(
            "🏠 Worker {} (player {}) distance to dropoff: {:.1} units (need < {:.1} to deliver)",
            unit.unit_id, unit.player_id, distance, DROPOFF_TRAVEL_DISTANCE
        );
        log.insert(unit.unit_id, current_time);
    }
}

fn handle_travel_to_dropoff(
    movement: &mut Movement,
    building_transform: &Transform,
    gatherer: &ResourceGatherer,
    unit: &RTSUnit,
    distance: f32,
) {
    // Set movement target to building if not already moving there
    if movement.target_position.is_none()
        || movement
            .target_position
            .unwrap()
            .distance(building_transform.translation)
            > 5.0
    {
        movement.target_position = Some(building_transform.translation);
        // Removed worker return logging for performance
    }
}

fn handle_resource_delivery_at_dropoff(
    gatherer: &mut ResourceGatherer,
    movement: &mut Movement,
    unit: &RTSUnit,
    resources: &Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    // Close enough to deliver
    if let Some(resource_type) = gatherer.resource_type.clone() {
        deliver_resources_to_player(
            unit.player_id,
            resource_type,
            gatherer.carried_amount,
            player_resources,
            ai_resources,
        );
        reset_gatherer_cargo(gatherer);

        handle_post_delivery_movement(gatherer, movement, unit, resources);
    }
}

fn handle_post_delivery_movement(
    gatherer: &mut ResourceGatherer,
    movement: &mut Movement,
    unit: &RTSUnit,
    resources: &Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
) {
    // Automatically return to resource gathering if target still exists
    if let Some(resource_entity) = gatherer.target_resource {
        if let Ok((_, _, resource_transform)) = resources.get(resource_entity) {
            movement.target_position = Some(resource_transform.translation);
            // Removed worker return-to-gather logging for performance
        } else {
            // Resource no longer exists, clear target and movement so AI can assign new resource
            gatherer.target_resource = None;
            movement.target_position = None;
            info!(
                "🏁 Worker {:?} completed delivery, resource depleted, becoming idle",
                unit.unit_id
            );
        }
    } else {
        // No target resource (was depleted before delivery), clear movement so AI can assign new resource
        movement.target_position = None;
        info!(
            "🏁 Worker {:?} completed delivery, becoming idle",
            unit.unit_id
        );
    }
}

fn reset_gatherer_cargo(gatherer: &mut ResourceGatherer) {
    gatherer.carried_amount = 0.0;
    gatherer.resource_type = None;
}
