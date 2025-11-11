use bevy::prelude::*;
use crate::core::components::*;
use std::sync::{LazyLock, Mutex};
use std::collections::{HashMap, HashSet};

// Configurable gathering parameters
const GATHERING_DISTANCE: f32 = 20.0;  // Distance within which gathering occurs (increased for collision constraints)
const DROPOFF_TRAVEL_DISTANCE: f32 = 30.0;  // Distance threshold for delivering resources (increased for building collision radii)
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
    // Validate existing assignment
    if is_current_dropoff_valid(gatherer, unit, buildings) {
        return;
    }

    // Clear invalid assignment
    if gatherer.drop_off_building.is_some() {
        gatherer.drop_off_building = None;
    }

    // Find and assign new building
    if let Some((building_entity, distance)) = find_closest_dropoff_building(worker_transform, unit, buildings) {
        assign_new_dropoff_building(gatherer, unit, building_entity, distance);
    } else {
        log_no_dropoff_available(gatherer, unit, buildings);
    }
}

/// Check if current dropoff building is still valid
fn is_current_dropoff_valid(
    gatherer: &ResourceGatherer,
    unit: &RTSUnit,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) -> bool {
    let Some(current_dropoff) = gatherer.drop_off_building else {
        return false;
    };

    let Ok((_, _, building, building_unit)) = buildings.get(current_dropoff) else {
        return false;
    };

    let is_correct_type = matches!(building.building_type,
        BuildingType::Queen | BuildingType::StorageChamber | BuildingType::Nursery);

    building_unit.player_id == unit.player_id && building.is_complete && is_correct_type
}

/// Find the closest suitable dropoff building for a worker
fn find_closest_dropoff_building(
    worker_transform: &Transform,
    unit: &RTSUnit,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) -> Option<(Entity, f32)> {
    let mut closest_building: Option<Entity> = None;
    let mut closest_distance = f32::MAX;

    for (building_entity, building_transform, building, building_unit) in buildings.iter() {
        // Only consider buildings owned by the same player
        if building_unit.player_id != unit.player_id || !building.is_complete {
            continue;
        }

        // Check if building type can accept resources
        let is_resource_building = matches!(building.building_type,
            BuildingType::Queen | BuildingType::StorageChamber | BuildingType::Nursery);

        if is_resource_building {
            let distance = worker_transform.translation.distance(building_transform.translation);
            if distance < closest_distance {
                closest_distance = distance;
                closest_building = Some(building_entity);
            }
        }
    }

    closest_building.map(|building| (building, closest_distance))
}

/// Assign a new dropoff building to a worker
fn assign_new_dropoff_building(
    gatherer: &mut ResourceGatherer,
    unit: &RTSUnit,
    building_entity: Entity,
    distance: f32,
) {
    gatherer.drop_off_building = Some(building_entity);
    // Only log if worker has resources or is actively gathering (not at startup)
    if gatherer.carried_amount > 0.0 || gatherer.target_resource.is_some() {
        info!("✅ Assigned drop-off building to worker {} (player {}) at distance {:.1}",
              unit.unit_id, unit.player_id, distance);
    }
}

/// Log warning when no dropoff building is available
fn log_no_dropoff_available(
    gatherer: &ResourceGatherer,
    unit: &RTSUnit,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) {
    if gatherer.drop_off_building.is_some() {
        return; // Already has a building, no need to log
    }

    // Only log once per second to avoid spam
    static LAST_WARNING_TIME: LazyLock<Mutex<f32>> = LazyLock::new(|| Mutex::new(0.0));
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f32();

    let mut last_time = LAST_WARNING_TIME.lock().unwrap();
    if current_time - *last_time > 1.0 {
        let total_buildings = buildings.iter().count();
        let player_buildings = buildings.iter().filter(|(_, _, _, b)| b.player_id == unit.player_id).count();
        let complete_buildings = buildings.iter().filter(|(_, _, b, u)| u.player_id == unit.player_id && b.is_complete).count();

        warn!("⚠️  Worker {} (player {}) needs drop-off building but none available (no Queen/StorageChamber/Nursery found)",
              unit.unit_id, unit.player_id);
        warn!("   Buildings check: {} total, {} for player {}, {} complete",
              total_buildings, player_buildings, unit.player_id, complete_buildings);
        *last_time = current_time;
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
    let Some(resource_entity) = gatherer.target_resource else { return };

    let Ok((_, mut resource, resource_transform)) = resources.get_mut(resource_entity) else {
        gatherer.target_resource = None;
        return;
    };

    let distance = gatherer_transform.translation.distance(resource_transform.translation);

    // Log distance for debugging collision issues
    static DISTANCE_LOG: LazyLock<Mutex<HashMap<u32, f32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f32();

    let mut log = DISTANCE_LOG.lock().unwrap();
    let last_time = log.get(&unit.unit_id).copied().unwrap_or(0.0);
    if current_time - last_time > 2.0 {
        info!("🎯 Worker {} (player {}) distance to resource: {:.1} units (need < {:.1})",
              unit.unit_id, unit.player_id, distance, GATHERING_DISTANCE);
        log.insert(unit.unit_id, current_time);
    }

    if distance > GATHERING_DISTANCE {
        return;
    }

    // If at full capacity, stop gathering but keep target_resource so we can return after delivery
    if gatherer.carried_amount >= gatherer.capacity {
        // Log when worker reaches full capacity and stops gathering
        static LAST_FULL_LOG: LazyLock<Mutex<HashMap<u32, f32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();

        let mut log = LAST_FULL_LOG.lock().unwrap();
        let last_time = log.get(&unit.unit_id).copied().unwrap_or(0.0);
        if current_time - last_time > 2.0 {
            info!("📦 Worker {} (player {}) FULL: {:.1}/{:.1} - waiting to return",
                  unit.unit_id, unit.player_id, gatherer.carried_amount, gatherer.capacity);
            log.insert(unit.unit_id, current_time);
        }
        return;
    }

    let gather_amount = calculate_gather_amount(gatherer, &resource, delta_time);

    // Log gathering progress periodically
    if gather_amount > 0.0 {
        static LAST_GATHER_LOG: LazyLock<Mutex<HashMap<u32, f32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32();

        let mut log = LAST_GATHER_LOG.lock().unwrap();
        let last_time = log.get(&unit.unit_id).copied().unwrap_or(0.0);
        if current_time - last_time > 1.0 {
            info!("⛏️  Worker {} (player {}) gathering {:?}: {:.1}/{:.1} (resource has {:.1} left)",
                  unit.unit_id, unit.player_id, gatherer.resource_type,
                  gatherer.carried_amount, gatherer.capacity, resource.amount);
            log.insert(unit.unit_id, current_time);
        }
    }

    apply_gathering(gatherer, &mut resource, gather_amount);

    if resource.amount <= 0.0 {
        info!("🔴 Resource depleted! Worker {} (player {}) has {:.1}/{:.1} resources",
              unit.unit_id, unit.player_id, gatherer.carried_amount, gatherer.capacity);
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
    // Check if worker is ready to return to base
    if !should_worker_return_to_base(gatherer, unit) {
        return;
    }

    // Ensure worker has a dropoff building
    let Some(dropoff_entity) = gatherer.drop_off_building else {
        handle_no_dropoff_building(gatherer, movement, unit);
        return;
    };

    // Get building location
    let Ok((_, building_transform, _, _)) = buildings.get(dropoff_entity) else {
        warn!("Drop-off building not found for worker {} (player {})!", unit.unit_id, unit.player_id);
        gatherer.drop_off_building = None;
        return;
    };

    let distance = gatherer_transform.translation.distance(building_transform.translation);
    log_dropoff_distance(unit, distance);

    // If far from dropoff, move towards it
    if distance > DROPOFF_TRAVEL_DISTANCE {
        move_worker_to_dropoff(gatherer, movement, unit, building_transform, distance);
        return;
    }

    // Close enough to deliver resources
    complete_resource_delivery(gatherer, movement, unit, resources, player_resources, ai_resources);
}

/// Check if worker should return to base with resources
fn should_worker_return_to_base(gatherer: &ResourceGatherer, unit: &RTSUnit) -> bool {
    // No resources to deliver
    if gatherer.carried_amount <= 0.0 {
        return false;
    }

    // Still gathering from a resource, don't return yet (unless at full capacity)
    if gatherer.target_resource.is_some() && gatherer.carried_amount < gatherer.capacity {
        return false;
    }

    // Log when worker decides to return (once per worker)
    static DELIVERY_DECISION_LOGGED: LazyLock<Mutex<HashSet<u32>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
    let mut logged = DELIVERY_DECISION_LOGGED.lock().unwrap();
    if !logged.contains(&unit.unit_id) {
        let reason = if gatherer.carried_amount >= gatherer.capacity {
            "FULL CAPACITY"
        } else if gatherer.target_resource.is_none() {
            "RESOURCE DEPLETED"
        } else {
            "UNKNOWN"
        };
        info!("🔵 Worker {} (player {}) READY TO RETURN: {:.1}/{:.1} resources (reason: {})",
              unit.unit_id, unit.player_id, gatherer.carried_amount, gatherer.capacity, reason);
        logged.insert(unit.unit_id);
    }

    true
}

/// Handle case where worker has resources but no dropoff building
fn handle_no_dropoff_building(gatherer: &mut ResourceGatherer, movement: &mut Movement, unit: &RTSUnit) {
    // Clear target_resource and movement so worker waits idle until a building is assigned
    if gatherer.target_resource.is_some() || movement.target_position.is_some() {
        gatherer.target_resource = None;
        movement.target_position = None;
        warn!("Worker {} (player {}) has {:.1} resources but no drop-off building available - waiting for building to be constructed",
              unit.unit_id, unit.player_id, gatherer.carried_amount);
    }
}

/// Log the distance to dropoff building for debugging
fn log_dropoff_distance(unit: &RTSUnit, distance: f32) {
    static DROPOFF_DISTANCE_LOG: LazyLock<Mutex<HashMap<u32, f32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f32();

    let mut log = DROPOFF_DISTANCE_LOG.lock().unwrap();
    let last_time = log.get(&unit.unit_id).copied().unwrap_or(0.0);
    if current_time - last_time > 2.0 {
        info!("🏠 Worker {} (player {}) distance to dropoff: {:.1} units (need < {:.1} to deliver)",
              unit.unit_id, unit.player_id, distance, DROPOFF_TRAVEL_DISTANCE);
        log.insert(unit.unit_id, current_time);
    }
}

/// Move worker towards the dropoff building
fn move_worker_to_dropoff(
    gatherer: &ResourceGatherer,
    movement: &mut Movement,
    unit: &RTSUnit,
    building_transform: &Transform,
    distance: f32,
) {
    // Set movement target to building if not already moving there
    if movement.target_position.is_none() ||
       movement.target_position.unwrap().distance(building_transform.translation) > 5.0 {
        movement.target_position = Some(building_transform.translation);
        info!("🚚 Worker {} (player {}) returning to base with {:.1} resources (distance: {:.1})",
              unit.unit_id, unit.player_id, gatherer.carried_amount, distance);
    }
}

/// Complete the resource delivery and decide next action
fn complete_resource_delivery(
    gatherer: &mut ResourceGatherer,
    movement: &mut Movement,
    unit: &RTSUnit,
    resources: &Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    let Some(resource_type) = gatherer.resource_type.clone() else {
        return;
    };

    // Deliver resources to player
    deliver_resources_to_player(unit.player_id, resource_type, gatherer.carried_amount, player_resources, ai_resources);
    reset_gatherer_cargo(gatherer);

    // Decide next action: return to gathering or become idle
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