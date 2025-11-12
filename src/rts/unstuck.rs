use bevy::prelude::*;
use crate::core::components::*;

/// System to detect when units are stuck and help them get unstuck
pub fn unstuck_system(
    _commands: Commands,
    mut units: Query<(Entity, &mut Transform, &mut Movement, &mut StuckDetection, &CollisionRadius, &RTSUnit), With<RTSUnit>>,
    // Static obstacles that units might be stuck against
    buildings: Query<(&Position, &CollisionRadius), (With<Building>, Without<Movement>)>,
    environment_objects: Query<(&Transform, &CollisionRadius), (With<EnvironmentObject>, Without<Movement>)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    let dt = time.delta_secs();

    for (_entity, mut transform, mut movement, mut stuck_detection, collision_radius, rts_unit) in units.iter_mut() {
        // Add StuckDetection component if it doesn't exist
        if stuck_detection.as_ref().last_position == Vec3::ZERO {
            stuck_detection.last_position = transform.translation;
            stuck_detection.last_movement_time = current_time;
            continue;
        }

        let current_position = transform.translation;
        let distance_moved = current_position.distance(stuck_detection.last_position);
        let has_target = movement.target_position.is_some();
        let velocity_magnitude = movement.current_velocity.length();

        // Update position history for better stuck detection
        stuck_detection.position_history.push(current_position);
        if stuck_detection.position_history.len() > 20 {
            stuck_detection.position_history.remove(0);
        }

        // Check if unit is stuck
        let is_stuck = detect_if_stuck(
            &stuck_detection,
            distance_moved,
            has_target,
            velocity_magnitude,
            dt,
            current_time,
        );

        if is_stuck {
            stuck_detection.stuck_timer += dt;
            
            // Try to unstuck the unit if it's been stuck for a while
            if stuck_detection.stuck_timer > 2.0 && current_time - stuck_detection.last_unstuck_time > 1.0 {
                let unstuck_success = attempt_unstuck(
                    &mut transform,
                    &mut movement,
                    &mut stuck_detection,
                    collision_radius,
                    rts_unit,
                    &buildings,
                    &environment_objects,
                    current_time,
                );

                if unstuck_success {
                    info!("Successfully unstuck unit {} (player {})", rts_unit.unit_id, rts_unit.player_id);
                    stuck_detection.stuck_timer = 0.0;
                    stuck_detection.unstuck_attempts = 0;
                    stuck_detection.last_unstuck_time = current_time;
                } else {
                    stuck_detection.unstuck_attempts += 1;
                    stuck_detection.last_unstuck_time = current_time;
                    
                    // If we've tried many times, teleport to safety
                    if stuck_detection.unstuck_attempts >= 5 {
                        teleport_to_safety(&mut transform, &mut movement, &mut stuck_detection, rts_unit);
                        info!("Teleported severely stuck unit {} (player {}) to safety", rts_unit.unit_id, rts_unit.player_id);
                    }
                }
            }
        } else {
            // Unit is not stuck, reset stuck timer
            if stuck_detection.stuck_timer > 0.0 {
                stuck_detection.stuck_timer = 0.0;
                stuck_detection.unstuck_attempts = 0;
            }
            stuck_detection.last_movement_time = current_time;
        }

        // Update last position for next frame
        stuck_detection.last_position = current_position;
    }
}

/// Detect if a unit is stuck based on movement patterns
fn detect_if_stuck(
    stuck_detection: &StuckDetection,
    distance_moved: f32,
    has_target: bool,
    velocity_magnitude: f32,
    dt: f32,
    current_time: f32,
) -> bool {
    // If unit has no target, it's not stuck (it's supposed to be stationary)
    if !has_target {
        return false;
    }

    // Check if unit hasn't moved much despite having a target and velocity
    let movement_threshold = 0.5; // Very small movement threshold
    let time_since_last_movement = current_time - stuck_detection.last_movement_time;

    // Unit is stuck if:
    // 1. It has a target to move to
    // 2. It has velocity (trying to move)
    // 3. It hasn't moved much in the last few frames
    // 4. It's been in this state for a while
    let is_trying_to_move = velocity_magnitude > 1.0;
    let hasnt_moved_much = distance_moved < movement_threshold * dt;
    let been_stuck_long = time_since_last_movement > 1.5;

    // Additional check: oscillating in place (moving back and forth)
    let is_oscillating = if stuck_detection.position_history.len() >= 10 {
        let recent_positions = &stuck_detection.position_history[stuck_detection.position_history.len()-10..];
        let avg_movement = recent_positions.windows(2)
            .map(|pair| pair[1].distance(pair[0]))
            .sum::<f32>() / (recent_positions.len() - 1) as f32;
        
        // High movement but no net progress indicates oscillation
        avg_movement > 1.0 && distance_moved < 0.2
    } else {
        false
    };

    (is_trying_to_move && hasnt_moved_much && been_stuck_long) || is_oscillating
}

/// Attempt to unstuck a unit using various strategies
fn attempt_unstuck(
    transform: &mut Transform,
    movement: &mut Movement,
    stuck_detection: &mut StuckDetection,
    collision_radius: &CollisionRadius,
    _rts_unit: &RTSUnit,
    buildings: &Query<(&Position, &CollisionRadius), (With<Building>, Without<Movement>)>,
    environment_objects: &Query<(&Transform, &CollisionRadius), (With<EnvironmentObject>, Without<Movement>)>,
    current_time: f32,
) -> bool {
    let current_position = transform.translation;
    let target_position = movement.target_position.unwrap_or(current_position);

    // Strategy 1: Find a clear path around obstacles
    if let Some(clear_position) = find_clear_position_around_obstacles(
        current_position,
        target_position,
        collision_radius.radius,
        buildings,
        environment_objects,
    ) {
        movement.target_position = Some(clear_position);
        movement.current_velocity = Vec3::ZERO; // Reset velocity to avoid conflicting forces
        debug!("Found clear position for stuck unit: {:?}", clear_position);
        return true;
    }

    // Strategy 2: Move in a random direction to break free
    if stuck_detection.unstuck_attempts < 3 {
        let random_direction = generate_random_direction(stuck_detection.unstuck_attempts, current_time);
        let unstuck_position = current_position + random_direction * (collision_radius.radius * 3.0);
        
        if is_position_clear(unstuck_position, collision_radius.radius, buildings, environment_objects) {
            transform.translation = unstuck_position;
            movement.current_velocity = Vec3::ZERO;
            debug!("Moved stuck unit in random direction: {:?}", random_direction);
            return true;
        }
    }

    // Strategy 3: Step back from the direction of the target
    if let Some(target) = movement.target_position {
        let direction_to_target = (target - current_position).normalize_or_zero();
        let step_back_position = current_position - direction_to_target * (collision_radius.radius * 2.0);
        
        if is_position_clear(step_back_position, collision_radius.radius, buildings, environment_objects) {
            transform.translation = step_back_position;
            movement.current_velocity = Vec3::ZERO;
            debug!("Stepped stuck unit back from target");
            return true;
        }
    }

    false
}

/// Find a clear position around obstacles to reach the target
fn find_clear_position_around_obstacles(
    current_position: Vec3,
    target_position: Vec3,
    unit_radius: f32,
    buildings: &Query<(&Position, &CollisionRadius), (With<Building>, Without<Movement>)>,
    environment_objects: &Query<(&Transform, &CollisionRadius), (With<EnvironmentObject>, Without<Movement>)>,
) -> Option<Vec3> {
    // Try positions in a circle around the current position to find a clear path
    let search_radius = unit_radius * 4.0;
    
    for i in 0..8 {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 8.0;
        let test_position = current_position + Vec3::new(
            angle.cos() * search_radius,
            0.0,
            angle.sin() * search_radius,
        );
        
        if is_position_clear(test_position, unit_radius, buildings, environment_objects) {
            // Check if this position gives us a better line to the target
            let distance_to_target = test_position.distance(target_position);
            let current_distance_to_target = current_position.distance(target_position);
            
            // Only use this position if it's closer to the target or at least not much farther
            if distance_to_target <= current_distance_to_target + search_radius {
                return Some(test_position);
            }
        }
    }
    
    None
}

/// Check if a position is clear of obstacles
fn is_position_clear(
    position: Vec3,
    unit_radius: f32,
    buildings: &Query<(&Position, &CollisionRadius), (With<Building>, Without<Movement>)>,
    environment_objects: &Query<(&Transform, &CollisionRadius), (With<EnvironmentObject>, Without<Movement>)>,
) -> bool {
    let safety_margin = 1.0; // Extra space for safety

    // Check against buildings
    for (building_position, building_radius) in buildings.iter() {
        let distance = position.distance(building_position.translation);
        let min_distance = unit_radius + building_radius.radius + safety_margin;
        
        if distance < min_distance {
            return false;
        }
    }

    // Check against environment objects
    for (env_transform, env_radius) in environment_objects.iter() {
        let distance = position.distance(env_transform.translation);
        let min_distance = unit_radius + env_radius.radius + safety_margin;
        
        if distance < min_distance {
            return false;
        }
    }

    true
}

/// Generate a random direction based on attempt number for deterministic but varied unstuck attempts
fn generate_random_direction(attempt: u32, current_time: f32) -> Vec3 {
    let seed = (attempt as f32 + current_time.fract()) * 1000.0;
    let angle = seed % (2.0 * std::f32::consts::PI);
    
    Vec3::new(angle.cos(), 0.0, angle.sin()).normalize()
}

/// Last resort: teleport unit to a safe location
fn teleport_to_safety(
    transform: &mut Transform,
    movement: &mut Movement,
    stuck_detection: &mut StuckDetection,
    rts_unit: &RTSUnit,
) {
    // Teleport to player's base area
    let base_position = match rts_unit.player_id {
        1 => Vec3::new(0.0, 10.0, 0.0),
        2 => Vec3::new(200.0, 10.0, 0.0),
        _ => Vec3::new((rts_unit.player_id as f32 - 1.0) * 200.0, 10.0, 0.0),
    };
    
    transform.translation = base_position;
    movement.current_velocity = Vec3::ZERO;
    movement.target_position = None; // Clear target to stop trying to reach impossible location
    stuck_detection.stuck_timer = 0.0;
    stuck_detection.unstuck_attempts = 0;
}

/// System to automatically add StuckDetection component to moving units that don't have it
pub fn add_stuck_detection_system(
    mut commands: Commands,
    units_without_stuck_detection: Query<Entity, (With<RTSUnit>, With<Movement>, Without<StuckDetection>)>,
) {
    for entity in units_without_stuck_detection.iter() {
        commands.entity(entity).insert(StuckDetection::default());
    }
}