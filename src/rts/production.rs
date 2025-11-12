use bevy::prelude::*;
use crate::core::components::*;

pub fn production_system(
    mut buildings: Query<(&mut ProductionQueue, &Building, &RTSUnit, &Transform), With<Building>>,
    mut player_resources: ResMut<crate::core::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::core::resources::AIResources>,
    game_costs: Res<crate::core::resources::GameCosts>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (mut queue, building, unit, transform) in buildings.iter_mut() {
        if building.is_complete && !queue.queue.is_empty() {
            process_production_queue(
                &mut queue,
                building,
                unit,
                transform,
                &mut player_resources,
                &mut ai_resources,
                &game_costs,
                &mut commands,
                &mut meshes,
                &mut materials,
                delta_time
            );
        }
    }
}

fn process_production_queue(
    queue: &mut ProductionQueue,
    building: &Building,
    unit: &RTSUnit,
    transform: &Transform,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
    game_costs: &Res<crate::core::resources::GameCosts>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    delta_time: f32,
) {
    queue.current_progress += delta_time;

    if queue.current_progress < queue.production_time {
        return;
    }

    let unit_type = queue.queue[0].clone();

    if can_afford_production(unit.player_id, &unit_type, game_costs, player_resources, ai_resources) {
        complete_production(queue, building, unit, transform, unit_type, player_resources, ai_resources, game_costs, commands, meshes, materials);
    } else {
        handle_production_failure(unit.player_id);
    }
}

fn can_afford_production(
    player_id: u8,
    unit_type: &UnitType,
    game_costs: &Res<crate::core::resources::GameCosts>,
    player_resources: &ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &ResMut<crate::core::resources::AIResources>,
) -> bool {
    let Some(cost) = game_costs.unit_costs.get(unit_type) else {
        return true;
    };

    // Use direct access for read-only check (no mutation needed)
    if player_id == 1 {
        player_resources.can_afford(cost) && player_resources.has_population_space()
    } else {
        ai_resources.resources.get(&player_id)
            .map(|resources| resources.can_afford(cost) && resources.has_population_space())
            .unwrap_or(true)
    }
}

fn complete_production(
    queue: &mut ProductionQueue,
    building: &Building,
    unit: &RTSUnit,
    transform: &Transform,
    unit_type: UnitType,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
    _game_costs: &Res<crate::core::resources::GameCosts>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    queue.queue.remove(0);
    queue.current_progress = 0.0;

    // Don't pay production cost here - resources were already deducted when queueing
    // Only add population since the unit is now produced
    let mut manager = crate::core::resources::ResourceManager::new(player_resources, ai_resources);
    manager.add_population(unit.player_id, 1);
    
    spawn_produced_unit(&unit_type, building, transform, unit.player_id, commands, meshes, materials);

    info!("Player {} produced {:?} unit", unit.player_id, unit_type);
}


fn spawn_produced_unit(
    unit_type: &UnitType,
    building: &Building,
    building_transform: &Transform,
    player_id: u8,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    use crate::entities::entity_factory::{EntityFactory, SpawnConfig, EntityType};

    // Calculate smart rally point to prevent units from getting stuck near buildings
    let rally_position = if let Some(rally) = building.rally_point {
        rally
    } else {
        calculate_auto_rally_point(building_transform.translation, player_id)
    };

    // Spawn unit with movement toward rally point to spread out immediately
    let config = SpawnConfig::unit(EntityType::Unit(unit_type.clone()), building_transform.translation, player_id);
    let entity = EntityFactory::spawn(commands, meshes, materials, config, None);
    
    // Set immediate movement target to rally point to spread units away from spawn building
    commands.entity(entity).insert(Movement {
        target_position: Some(rally_position),
        current_velocity: Vec3::ZERO,
        max_speed: 25.0,
        acceleration: 8.0,
        turning_speed: 5.0,
        path: Vec::new(),
        path_index: 0,
    });
    
    info!("🚀 AI Player {} unit {:?} spawned with auto-rally to {:?} (spreading from building)", 
          player_id, unit_type, rally_position);
}

/// Calculate an intelligent auto-rally point that spreads units away from their spawn building
fn calculate_auto_rally_point(building_position: Vec3, player_id: u8) -> Vec3 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Create deterministic but varied rally points based on building position and player
    let mut hasher = DefaultHasher::new();
    building_position.x.to_bits().hash(&mut hasher);
    building_position.z.to_bits().hash(&mut hasher);
    player_id.hash(&mut hasher);
    let seed = hasher.finish();
    
    // Generate multiple candidate rally points and select the best one
    let base_distance = 25.0; // Base distance from building to prevent clustering
    let max_distance = 45.0;  // Max distance to keep units reasonably close to base
    
    for ring in 0..3 {  // Try up to 3 rings of different distances
        for attempt in 0..8 {  // 8 attempts per ring
            // Calculate angle with some randomness
            let base_angle = (attempt as f32) * std::f32::consts::PI / 4.0; // 45° segments
            let angle_variation = ((seed + (ring * 8 + attempt) as u64) % 100) as f32 / 100.0 * 0.5; // ±22.5° variation
            let final_angle = base_angle + angle_variation;
            
            // Calculate distance with ring-based progression
            let distance = base_distance + (ring as f32 * 10.0); // 25, 35, 45 units away
            
            // Calculate candidate position
            let candidate_x = building_position.x + distance * final_angle.cos();
            let candidate_z = building_position.z + distance * final_angle.sin();
            let candidate_position = Vec3::new(candidate_x, building_position.y, candidate_z);
            
            // Basic validation: ensure position is reasonable and within map bounds
            if candidate_position.x.abs() < 800.0 && candidate_position.z.abs() < 800.0 && 
               building_position.distance(candidate_position) <= max_distance {
                
                // For now, return first valid position (could add collision checking later)
                return candidate_position;
            }
        }
    }
    
    // Fallback: simple offset if all attempts fail
    let fallback_offset = match player_id {
        1 => Vec3::new(-30.0, 0.0, 15.0),  // Player 1 units rally to the left-front
        2 => Vec3::new(30.0, 0.0, -15.0),  // AI Player 2 units rally to the right-back
        _ => Vec3::new(25.0, 0.0, 0.0),    // Other players rally to the right
    };
    
    building_position + fallback_offset
}

fn handle_production_failure(player_id: u8) {
    if player_id == 1 {
        info!("Cannot produce unit: insufficient resources or population space");
    }
}

pub fn building_completion_system(
    mut buildings: Query<(&mut Building, &mut crate::core::components::RTSHealth, &crate::core::components::RTSUnit), With<Building>>,
    mut player_resources: ResMut<crate::core::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::core::resources::AIResources>,
    time: Res<Time>,
) {
    for (mut building, mut health, unit) in buildings.iter_mut() {
        if building.is_complete {
            continue;
        }
        
        update_construction_progress(&mut building, &mut health, time.delta_secs());
        
        if building.construction_progress >= building.max_construction {
            complete_building(&mut building, &mut health, unit, &mut player_resources, &mut ai_resources);
        }
    }
}

pub fn population_management_system(
    mut player_resources: ResMut<crate::core::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::core::resources::AIResources>,
    units: Query<&RTSUnit, With<RTSUnit>>,
) {
    reset_population_counts(&mut player_resources, &mut ai_resources);
    count_active_units(&units, &mut player_resources, &mut ai_resources);
}

fn reset_population_counts(
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    player_resources.current_population = 0;
    
    for resources in ai_resources.resources.values_mut() {
        resources.current_population = 0;
    }
}

fn update_construction_progress(
    building: &mut Building,
    health: &mut crate::core::components::RTSHealth,
    delta_time: f32,
) {
    building.construction_progress += delta_time;
    let completion_ratio = (building.construction_progress / building.max_construction).min(1.0);
    health.current = health.max * completion_ratio;
}

fn complete_building(
    building: &mut Building,
    health: &mut crate::core::components::RTSHealth,
    unit: &crate::core::components::RTSUnit,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    building.is_complete = true;
    health.current = health.max;
    
    let housing_amount = get_housing_amount(&building.building_type);
    
    if housing_amount > 0 {
        add_housing_capacity(unit.player_id, housing_amount, player_resources, ai_resources);
        log_building_completion_with_housing(unit.player_id, housing_amount, player_resources, ai_resources);
    } else {
        info!("Player {} building {:?} completed!", unit.player_id, building.building_type);
    }
}

fn get_housing_amount(building_type: &crate::core::components::BuildingType) -> u32 {
    match building_type {
        crate::core::components::BuildingType::Nursery => 15,
        crate::core::components::BuildingType::Queen => 10,
        crate::core::components::BuildingType::StorageChamber => 5,
        _ => 0,
    }
}

fn add_housing_capacity(
    player_id: u8,
    housing_amount: u32,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    if player_id == 1 {
        player_resources.add_housing(housing_amount);
    } else if let Some(ai_player_resources) = ai_resources.resources.get_mut(&player_id) {
        ai_player_resources.add_housing(housing_amount);
    }
}

fn log_building_completion_with_housing(
    player_id: u8,
    housing_amount: u32,
    player_resources: &ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &ResMut<crate::core::resources::AIResources>,
) {
    if player_id == 1 {
        info!("Player {} building completed! Added {} housing capacity (total: {})", 
              player_id, housing_amount, player_resources.max_population);
    } else if let Some(ai_player_resources) = ai_resources.resources.get(&player_id) {
        info!("AI Player {} building completed! Added {} housing capacity (total: {})", 
              player_id, housing_amount, ai_player_resources.max_population);
    }
}

fn count_active_units(
    units: &Query<&RTSUnit, With<RTSUnit>>,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    for unit in units.iter() {
        if unit.player_id == 1 {
            player_resources.current_population += 1;
        } else if let Some(ai_player_resources) = ai_resources.resources.get_mut(&unit.player_id) {
            ai_player_resources.current_population += 1;
        }
    }
}

/// Try to queue a unit with overflow support - finds buildings with available queue space
/// This is shared between UI and AI systems
pub fn try_queue_unit_with_overflow(
    unit_type: UnitType,
    building_type: BuildingType,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_id: u8,
) -> bool {
    const MAX_QUEUE_SIZE: usize = 8;
    
    // First pass: try to find a building with available queue space
    for (mut queue, building, unit) in buildings.iter_mut() {
        if unit.player_id == player_id && 
           building.building_type == building_type && 
           building.is_complete &&
           queue.queue.len() < MAX_QUEUE_SIZE {
            
            queue.queue.push(unit_type.clone());
            info!("🏭 Player {} queued {:?} in {:?} (queue: {}/{}, direct placement)", 
                  player_id, unit_type, building_type, queue.queue.len(), MAX_QUEUE_SIZE);
            return true;
        }
    }
    
    // Second pass: find the building with the smallest queue for overflow (but still under max)
    let mut min_queue_size = usize::MAX;
    let mut found_building_under_max = false;
    
    // First find the minimum queue size among buildings that are still under max
    for (queue, building, unit) in buildings.iter() {
        if unit.player_id == player_id && 
           building.building_type == building_type && 
           building.is_complete && 
           queue.queue.len() < MAX_QUEUE_SIZE {
            min_queue_size = min_queue_size.min(queue.queue.len());
            found_building_under_max = true;
        }
    }
    
    // If we found buildings under max capacity, queue to the first one with minimum queue size
    if found_building_under_max {
        for (mut queue, building, unit) in buildings.iter_mut() {
            if unit.player_id == player_id && 
               building.building_type == building_type && 
               building.is_complete &&
               queue.queue.len() == min_queue_size &&
               queue.queue.len() < MAX_QUEUE_SIZE {
                
                queue.queue.push(unit_type.clone());
                info!("🔄 Player {} overflow: Queued {:?} in different {:?} (queue: {}/{}, was least busy at {})", 
                      player_id, unit_type, building_type, queue.queue.len(), MAX_QUEUE_SIZE, min_queue_size);
                return true;
            }
        }
    }
    
    // All buildings are at max capacity - cannot queue
    info!("❌ Player {} cannot queue {:?}: All {:?} buildings at max capacity ({}/{})", 
          player_id, unit_type, building_type, MAX_QUEUE_SIZE, MAX_QUEUE_SIZE);
    
    false
}