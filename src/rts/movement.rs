use crate::core::components::*;
use crate::core::resources::SpatialGrids;
use bevy::prelude::*;

pub struct MovementContext {
    pub delta_time: f32,
    pub unit_positions: Vec<(Entity, Vec3, f32)>,
    pub building_obstacles: Vec<(Vec3, f32)>, // position, radius
    pub environment_obstacles: Vec<(Vec3, f32)>, // position, radius
    // Removed obstacle_grid - now using persistent grid from resource
}

pub fn movement_system(
    mut units_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Movement,
            &CollisionRadius,
            &RTSUnit,
            Option<&CombatState>,
            Option<&ResourceGatherer>,
        ),
        With<RTSUnit>,
    >,
    buildings_query: Query<(Entity, &Transform, &CollisionRadius), (With<Building>, Without<RTSUnit>)>,
    environment_query: Query<
        (Entity, &Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<RTSUnit>),
    >,
    mut spatial_grids: ResMut<SpatialGrids>,
    terrain_manager: Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::world::terrain_v2::TerrainSettings>,
    time: Res<Time>,
) {
    // Update obstacle grid incrementally
    let current_obstacles: std::collections::HashSet<Entity> = buildings_query
        .iter()
        .chain(environment_query.iter().map(|(e, t, c)| (e, t, c)))
        .map(|(entity, _, _)| entity)
        .collect();
    
    let grid_obstacles: std::collections::HashSet<Entity> = spatial_grids.obstacle_grid.entity_positions.keys().copied().collect();
    
    // Remove obstacles that no longer exist
    for entity in grid_obstacles.difference(&current_obstacles) {
        spatial_grids.obstacle_grid.remove_item(*entity);
    }
    
    // Update building obstacles
    for (entity, transform, collision_radius) in buildings_query.iter() {
        spatial_grids.obstacle_grid.update_obstacle(entity, transform.translation, collision_radius.radius);
    }
    
    // Update environment obstacles  
    for (entity, transform, collision_radius) in environment_query.iter() {
        spatial_grids.obstacle_grid.update_obstacle(entity, transform.translation, collision_radius.radius);
    }

    // Create context with unit data
    let unit_positions: Vec<(Entity, Vec3, f32)> = units_query
        .iter()
        .map(|(entity, transform, _, collision_radius, _, _, _)| {
            (entity, transform.translation, collision_radius.radius)
        })
        .collect();

    let building_obstacles: Vec<(Vec3, f32)> = buildings_query
        .iter()
        .map(|(_, transform, collision_radius)| (transform.translation, collision_radius.radius))
        .collect();

    let environment_obstacles: Vec<(Vec3, f32)> = environment_query
        .iter()
        .map(|(_, transform, collision_radius)| (transform.translation, collision_radius.radius))
        .collect();

    // Use actual delta time for movement speed, but add stability measures
    let raw_delta = time.delta_secs();

    // Clamp delta time to prevent extreme jumps (but allow speed increases)
    let stable_delta = raw_delta.min(0.1); // Cap at 100ms to prevent huge jumps

    let context = MovementContext {
        delta_time: stable_delta,
        unit_positions,
        building_obstacles,
        environment_obstacles,
    };

    for (
        entity,
        mut transform,
        mut movement,
        collision_radius,
        rts_unit,
        combat_state,
        resource_gatherer,
    ) in units_query.iter_mut()
    {
        process_unit_movement(
            entity,
            &mut transform,
            &mut movement,
            &collision_radius,
            rts_unit,
            combat_state,
            resource_gatherer,
            &context,
            &spatial_grids.obstacle_grid,
            &terrain_manager,
            &terrain_settings,
        );
    }
}

fn process_unit_movement(
    entity: Entity,
    transform: &mut Transform,
    movement: &mut Movement,
    collision_radius: &CollisionRadius,
    rts_unit: &RTSUnit,
    combat_state: Option<&CombatState>,
    resource_gatherer: Option<&ResourceGatherer>,
    context: &MovementContext,
    obstacle_grid: &crate::core::spatial_grid::IncrementalObstacleSpatialGrid,
    terrain_manager: &crate::world::terrain_v2::TerrainChunkManager,
    terrain_settings: &crate::world::terrain_v2::TerrainSettings,
) {
    use crate::constants::movement::*;

    let current_pos = transform.translation;

    if !is_valid_position(current_pos) {
        reset_unit_to_origin(transform, movement);
        return;
    }

    if let Some(target) = movement.target_position {
        if !is_valid_position(target) {
            movement.target_position = None;
            return;
        }

        // Clamp target to map boundaries if it's outside
        let clamped_target = clamp_to_map_boundary(target);
        if clamped_target != target {
            movement.target_position = Some(clamped_target);
        }
        let target = clamped_target;

        // Check for destination congestion and apply spreading (skip for construction tasks)
        let adjusted_target = if should_skip_destination_spreading(entity, target, context) {
            target // Don't apply spreading for construction tasks
        } else {
            apply_destination_spreading(entity, target, current_pos, context)
        };

        // Apply smart pathfinding that avoids all obstacles (but allows gatherers to approach their target resources)
        let smart_direction = calculate_smart_direction(
            entity,
            current_pos,
            adjusted_target,
            resource_gatherer,
            context,
            obstacle_grid,
            rts_unit,
        );
        let direction = smart_direction.normalize_or_zero();
        let distance = current_pos.distance(adjusted_target);

        if !distance.is_finite() || distance > MAX_DISTANCE {
            warn!("Invalid distance {:.1}, clearing target", distance);
            movement.target_position = None;
            return;
        }

        if distance > ARRIVAL_THRESHOLD {
            let new_position =
                calculate_new_position(current_pos, adjusted_target, movement, context);
            apply_collision_avoidance(
                entity,
                new_position,
                movement,
                collision_radius,
                context,
                combat_state,
            );
            update_position_with_terrain(
                transform,
                new_position,
                terrain_manager,
                terrain_settings,
            );
            update_rotation(transform, direction, movement, rts_unit, context.delta_time);
        } else {
            // Skip collision avoidance when stopped to prevent jitter at destination
            if movement.current_velocity.length() < 0.1 {
                stop_unit_movement(movement);
            }
        }
    } else {
        apply_movement_dampening(movement);
    }
}

fn is_valid_position(pos: Vec3) -> bool {
    use crate::constants::movement::*;
    pos.x.abs() <= MAX_POSITION
        && pos.z.abs() <= MAX_POSITION
        && pos.x.is_finite()
        && pos.z.is_finite()
}

/// Check if position is within map boundaries (2000x2000 unit area)
fn is_within_map_boundary(pos: Vec3) -> bool {
    use crate::constants::movement::MAP_BOUNDARY;
    pos.x.abs() <= MAP_BOUNDARY && pos.z.abs() <= MAP_BOUNDARY
}

/// Clamp position to map boundaries
fn clamp_to_map_boundary(pos: Vec3) -> Vec3 {
    use crate::constants::movement::MAP_BOUNDARY;
    Vec3::new(
        pos.x.clamp(-MAP_BOUNDARY, MAP_BOUNDARY),
        pos.y,
        pos.z.clamp(-MAP_BOUNDARY, MAP_BOUNDARY),
    )
}

/// Calculate boundary push force to keep units within map
fn calculate_boundary_push_force(pos: Vec3) -> Vec3 {
    use crate::constants::movement::{BOUNDARY_BUFFER, BOUNDARY_PUSH_FORCE, MAP_BOUNDARY};

    let mut push_force = Vec3::ZERO;

    // X-axis boundary checking
    if pos.x > MAP_BOUNDARY - BOUNDARY_BUFFER {
        let distance_from_boundary = pos.x - (MAP_BOUNDARY - BOUNDARY_BUFFER);
        let force_strength = (distance_from_boundary / BOUNDARY_BUFFER).clamp(0.0, 1.0);
        push_force.x -= BOUNDARY_PUSH_FORCE * force_strength;
    } else if pos.x < -(MAP_BOUNDARY - BOUNDARY_BUFFER) {
        let distance_from_boundary = (-pos.x) - (MAP_BOUNDARY - BOUNDARY_BUFFER);
        let force_strength = (distance_from_boundary / BOUNDARY_BUFFER).clamp(0.0, 1.0);
        push_force.x += BOUNDARY_PUSH_FORCE * force_strength;
    }

    // Z-axis boundary checking
    if pos.z > MAP_BOUNDARY - BOUNDARY_BUFFER {
        let distance_from_boundary = pos.z - (MAP_BOUNDARY - BOUNDARY_BUFFER);
        let force_strength = (distance_from_boundary / BOUNDARY_BUFFER).clamp(0.0, 1.0);
        push_force.z -= BOUNDARY_PUSH_FORCE * force_strength;
    } else if pos.z < -(MAP_BOUNDARY - BOUNDARY_BUFFER) {
        let distance_from_boundary = (-pos.z) - (MAP_BOUNDARY - BOUNDARY_BUFFER);
        let force_strength = (distance_from_boundary / BOUNDARY_BUFFER).clamp(0.0, 1.0);
        push_force.z += BOUNDARY_PUSH_FORCE * force_strength;
    }

    push_force
}

fn reset_unit_to_origin(transform: &mut Transform, movement: &mut Movement) {
    use crate::constants::movement::DEFAULT_SPAWN_HEIGHT;
    warn!("Unit at extreme position, resetting to origin");
    transform.translation = Vec3::new(0.0, DEFAULT_SPAWN_HEIGHT, 0.0);
    movement.current_velocity = Vec3::ZERO;
    movement.target_position = None;
}

fn calculate_new_position(
    current_pos: Vec3,
    target: Vec3,
    movement: &mut Movement,
    context: &MovementContext,
) -> Vec3 {
    let direction = (target - current_pos).normalize_or_zero();
    let mut target_velocity = direction * movement.max_speed;

    // Add boundary push force to keep units within map limits
    let boundary_force = calculate_boundary_push_force(current_pos);
    target_velocity += boundary_force;

    let clamped_velocity = clamp_velocity(target_velocity);

    // Debug logging for DragonFly movement speed
    if movement.max_speed > 350.0 {
        debug!("DragonFly movement - max_speed: {:.1}, target_velocity magnitude: {:.1}, clamped_velocity magnitude: {:.1}", 
               movement.max_speed, target_velocity.length(), clamped_velocity.length());
    }

    // Use smooth acceleration that works well at all speeds
    let acceleration_factor = (movement.acceleration * context.delta_time).min(0.95); // Allow fast acceleration but prevent instant changes
    movement.current_velocity = movement
        .current_velocity
        .lerp(clamped_velocity, acceleration_factor);

    movement.current_velocity = clamp_velocity(movement.current_velocity);

    // Calculate movement with proper delta time scaling (units get faster with game speed)
    current_pos + movement.current_velocity * context.delta_time
}

fn clamp_velocity(velocity: Vec3) -> Vec3 {
    use crate::constants::movement::MAX_VELOCITY;
    Vec3::new(
        velocity.x.clamp(-MAX_VELOCITY, MAX_VELOCITY),
        velocity.y.clamp(-MAX_VELOCITY, MAX_VELOCITY),
        velocity.z.clamp(-MAX_VELOCITY, MAX_VELOCITY),
    )
}

fn apply_collision_avoidance(
    entity: Entity,
    new_position: Vec3,
    movement: &mut Movement,
    collision_radius: &CollisionRadius,
    context: &MovementContext,
    combat_state: Option<&CombatState>,
) {
    if !is_valid_position(new_position) {
        warn!("New position would be invalid, stopping movement");
        movement.current_velocity = Vec3::ZERO;
        movement.target_position = None;
        return;
    }

    let separation_data = calculate_separation_force(
        entity,
        new_position,
        collision_radius,
        context,
        combat_state,
    );

    // Combat units have different movement behavior
    let (separation_modifier, _velocity_modifier) = get_combat_movement_modifiers(combat_state);

    // Only apply separation forces if they're significant and unit is moving
    if separation_data.force.length() > 0.01 && movement.current_velocity.length() > 1.0 {
        // Scale separation force based on speed and combat state
        let speed_factor = (movement.current_velocity.length() / movement.max_speed).max(0.2);

        // Use stable separation force calculation that scales with delta time but prevents oscillation
        // The key is to limit the force magnitude relative to current speed, not absolute values
        let base_separation = separation_data.force * speed_factor * separation_modifier;
        let separation_magnitude = base_separation.length();

        // Scale the separation force by delta time, but limit it to prevent overcorrection
        let delta_scaled_magnitude = separation_magnitude * context.delta_time;
        let max_separation_ratio = 0.2; // Max 20% of current velocity change per frame
        let current_speed = movement.current_velocity.length();
        let max_allowed_separation = current_speed * max_separation_ratio;

        let final_magnitude = delta_scaled_magnitude.min(max_allowed_separation);
        let mut separation_strength = base_separation.normalize_or_zero() * final_magnitude;

        // Further reduce separation for units near construction sites to prevent jiggling
        if is_near_construction_site(new_position, context) {
            separation_strength *= 0.3; // Significantly reduce separation near construction
        }

        movement.current_velocity += separation_strength;
    }

    // Only apply velocity damping if there's actual collision, not just proximity
    if separation_data.has_hard_collision {
        movement.current_velocity *= 0.5; // Strong damping only for hard collisions
    } else if separation_data.has_collision {
        movement.current_velocity *= 0.95; // Very light damping for soft collisions
    }

    movement.current_velocity = clamp_velocity(movement.current_velocity);
}

struct SeparationData {
    force: Vec3,
    has_collision: bool,
    has_hard_collision: bool,
}

struct CollisionResult {
    distance: f32,
    is_collision: bool,
    is_hard_collision: bool,
    needs_separation: bool,
}

fn check_unit_collision(
    position: Vec3,
    other_position: Vec3,
    radius: f32,
    other_radius: f32,
) -> CollisionResult {
    let to_other = other_position - position;
    let distance = to_other.length();

    // Allow closer approach for combat - reduce minimum spacing slightly
    let min_distance = radius + other_radius + 0.3; // Reduced from 0.5
    let hard_collision_distance = radius + other_radius + 0.05; // Reduced from 0.1
    let separation_radius = radius * crate::constants::movement::SEPARATION_MULTIPLIER;

    let is_hard_collision = distance < hard_collision_distance && distance > 0.001;
    let is_collision = distance < min_distance && distance > 0.001;
    let needs_separation = distance < separation_radius && distance > 0.001;

    CollisionResult {
        distance,
        is_collision,
        is_hard_collision,
        needs_separation,
    }
}

fn calculate_unit_separation_force(
    position: Vec3,
    other_position: Vec3,
    distance: f32,
    separation_radius: f32,
) -> Vec3 {
    use crate::constants::movement::SEPARATION_FORCE_STRENGTH;

    let to_other = other_position - position;
    let normalized_distance = distance / separation_radius;
    let separation_strength = (1.0 - normalized_distance).powf(2.0);
    let separation_direction = -to_other.normalize_or_zero();

    let force_scale = calculate_force_scale(distance, separation_radius);
    separation_direction * separation_strength * SEPARATION_FORCE_STRENGTH * force_scale
}

fn calculate_force_scale(distance: f32, separation_radius: f32) -> f32 {
    if distance > separation_radius * 0.8 {
        0.1 // Very gentle force for loose spacing
    } else if distance > separation_radius * 0.5 {
        0.3 // Light force for medium spacing
    } else {
        0.8 // Stronger force only when quite close
    }
}

fn calculate_separation_force(
    entity: Entity,
    position: Vec3,
    collision_radius: &CollisionRadius,
    context: &MovementContext,
    combat_state: Option<&CombatState>,
) -> SeparationData {
    use crate::constants::movement::*;

    let mut separation_force = Vec3::ZERO;
    let separation_radius = collision_radius.radius * SEPARATION_MULTIPLIER;
    let mut has_collision = false;
    let mut has_hard_collision = false;

    for &(other_entity, other_position, other_radius) in &context.unit_positions {
        if entity == other_entity {
            continue;
        }

        let collision_result = check_unit_collision(
            position,
            other_position,
            collision_radius.radius,
            other_radius,
        );

        if collision_result.is_hard_collision {
            has_hard_collision = true;
            has_collision = true;
        } else if collision_result.is_collision {
            has_collision = true;
        }

        if collision_result.needs_separation {
            let mut force = calculate_unit_separation_force(
                position,
                other_position,
                collision_result.distance,
                separation_radius,
            );

            // Reduce separation force when in combat to allow tighter formations and melee attacks
            if let Some(combat) = combat_state {
                if matches!(
                    combat.state,
                    crate::core::components::CombatStateType::InCombat
                        | crate::core::components::CombatStateType::MovingToCombat
                        | crate::core::components::CombatStateType::MovingToAttack
                ) {
                    force *= 0.2; // Drastically reduce separation forces during combat
                }
            }

            separation_force += force;
        }
    }

    SeparationData {
        force: separation_force,
        has_collision,
        has_hard_collision,
    }
}

fn update_position_with_terrain(
    transform: &mut Transform,
    new_position: Vec3,
    terrain_manager: &crate::world::terrain_v2::TerrainChunkManager,
    terrain_settings: &crate::world::terrain_v2::TerrainSettings,
) {
    use crate::constants::movement::TERRAIN_SAMPLE_LIMIT;

    let terrain_height = if new_position.x.abs() < TERRAIN_SAMPLE_LIMIT
        && new_position.z.abs() < TERRAIN_SAMPLE_LIMIT
    {
        crate::world::terrain_v2::sample_terrain_height(
            new_position.x,
            new_position.z,
            &terrain_manager.noise_generator,
            terrain_settings,
        )
    } else {
        0.0
    };

    // Enforce map boundaries - clamp position to map limits
    let mut final_position = clamp_to_map_boundary(new_position);
    final_position.y = terrain_height + 2.0;
    transform.translation = final_position;
}

fn update_rotation(
    transform: &mut Transform,
    direction: Vec3,
    movement: &Movement,
    rts_unit: &RTSUnit,
    delta_time: f32,
) {
    use crate::core::components::UnitType;

    if direction.length() <= 0.1 {
        return;
    }

    // Calculate rotation based on model type
    // Some models (Fourmi/Ant, Butterfly) naturally face backward in GLB and don't need pre-rotation
    // Others face backward and are pre-rotated 180° in entity_factory
    let target_rotation = match rts_unit.unit_type.as_ref() {
        Some(UnitType::WorkerAnt) | Some(UnitType::SoldierAnt) | Some(UnitType::ScoutAnt) => {
            // Ant and butterfly models naturally face backward in GLB, no pre-rotation applied
            // Use negated formula for backward-facing models
            Quat::from_rotation_y(-direction.x.atan2(-direction.z))
        }
        _ => {
            // All other models are pre-rotated 180° to face forward
            // Use standard formula for forward-facing models
            Quat::from_rotation_y(direction.x.atan2(direction.z))
        }
    };

    let turn_speed = (movement.turning_speed * delta_time).min(0.1);
    transform.rotation = transform.rotation.slerp(target_rotation, turn_speed);
}

fn stop_unit_movement(movement: &mut Movement) {
    movement.target_position = None;
    movement.current_velocity *= 0.9;
}

fn apply_movement_dampening(movement: &mut Movement) {
    movement.current_velocity *= 0.9;
}

/// Calculate smart direction that avoids all types of obstacles (units, buildings, environment)
/// Special case: allow resource gatherers to approach their target resources
fn calculate_smart_direction(
    entity: Entity,
    current_pos: Vec3,
    target_pos: Vec3,
    resource_gatherer: Option<&ResourceGatherer>,
    context: &MovementContext,
    obstacle_grid: &crate::core::spatial_grid::IncrementalObstacleSpatialGrid,
    rts_unit: &RTSUnit,
) -> Vec3 {
    let base_direction = (target_pos - current_pos).normalize_or_zero();

    // Check if we have a clear path to target (with special allowance for resource gathering)
    if has_clear_path(current_pos, target_pos, resource_gatherer, context, obstacle_grid) {
        // Add small pathfinding randomness for ant units to prevent identical paths
        if is_ant_unit(rts_unit) {
            return add_ant_pathfinding_randomness(entity, base_direction, base_direction.length());
        }
        return base_direction * base_direction.length();
    }

    // Find best direction that avoids obstacles
    let mut best_direction = base_direction;
    let mut best_score = score_direction(
        current_pos,
        target_pos,
        base_direction,
        resource_gatherer,
        context,
    );

    // Test multiple directions in a cone around the target direction
    let test_angles = [-1.57, -0.78, -0.39, 0.39, 0.78, 1.57]; // -90°, -45°, -22.5°, 22.5°, 45°, 90°

    for &angle_offset in &test_angles {
        let test_direction = rotate_direction_y(base_direction, angle_offset);
        let score = score_direction(
            current_pos,
            target_pos,
            test_direction,
            resource_gatherer,
            context,
        );

        if score > best_score {
            best_score = score;
            best_direction = test_direction;
        }
    }

    // Blend with unit avoidance for smooth movement
    let unit_avoidance = calculate_unit_avoidance(entity, current_pos, base_direction, context);
    let mut final_direction = (best_direction + unit_avoidance * 0.4).normalize_or_zero();

    // Add pathfinding randomness for ant units to prevent single-file movement
    if is_ant_unit(rts_unit) {
        final_direction =
            add_ant_pathfinding_randomness(entity, final_direction, base_direction.length());
    }

    final_direction * base_direction.length()
}

/// Check if there's a clear path between two points
/// Special handling for resource gatherers approaching their target resources
/// OPTIMIZED: Uses spatial grid for O(1) obstacle lookup instead of O(N) linear search
fn has_clear_path(
    start: Vec3,
    end: Vec3,
    resource_gatherer: Option<&ResourceGatherer>,
    context: &MovementContext,
    obstacle_grid: &crate::core::spatial_grid::IncrementalObstacleSpatialGrid,
) -> bool {
    let direction = end - start;
    let distance = direction.length();
    if distance < 0.1 {
        return true;
    }

    let normalized_dir = direction / distance;
    let step_size = 4.0; // OPTIMIZATION: Increased from 2.0 to reduce iterations
    let steps = (distance / step_size).ceil() as i32;

    for i in 1..=steps {
        let test_pos = start + normalized_dir * (i as f32 * step_size).min(distance);

        // OPTIMIZATION: Use spatial grid to get only nearby obstacles instead of checking all obstacles
        let nearby_obstacles = obstacle_grid.query_nearby_obstacles(test_pos, 10.0); // 10.0 = max safety margin

        for (obstacle_pos, obstacle_radius) in nearby_obstacles {
            let obstacle_distance = test_pos.distance(obstacle_pos);
            
            // Special case: if this unit is gathering resources and this obstacle is their target, allow closer approach
            if let Some(gatherer) = resource_gatherer {
                if let Some(_target_resource) = gatherer.target_resource {
                    // We don't have the resource entity here, so use position proximity as a heuristic
                    // If the end point is very close to this obstacle, it's likely the target resource
                    if end.distance(obstacle_pos) < 5.0 {
                        // Close to the resource we're targeting
                        // Allow much closer approach for gathering
                        if obstacle_distance < obstacle_radius + 15.0 {
                            // 15.0 = close enough to gather
                            continue; // Skip blocking this obstacle - allow approach
                        }
                    }
                }
            }

            // Determine if this is a building or environment obstacle based on size heuristic
            let safety_margin = if obstacle_radius > 10.0 { 3.0 } else { 2.0 }; // Buildings are typically larger
            
            if obstacle_distance < obstacle_radius + safety_margin {
                return false;
            }
        }
    }

    true
}

/// Score a direction based on obstacle avoidance and progress toward target
fn score_direction(
    current_pos: Vec3,
    target_pos: Vec3,
    direction: Vec3,
    resource_gatherer: Option<&ResourceGatherer>,
    context: &MovementContext,
) -> f32 {
    let normalized_dir = direction.normalize_or_zero();
    let ideal_direction = (target_pos - current_pos).normalize_or_zero();

    // Base score: how well does this direction point toward target?
    let alignment_score = normalized_dir.dot(ideal_direction).max(0.0);

    // Penalty for getting too close to obstacles
    let mut obstacle_penalty = 0.0;
    let lookahead_distance = 8.0;
    let test_pos = current_pos + normalized_dir * lookahead_distance;

    // Check building obstacles
    for &(obstacle_pos, obstacle_radius) in &context.building_obstacles {
        let distance = test_pos.distance(obstacle_pos);
        let safe_distance = obstacle_radius + 5.0; // Larger safety margin
        if distance < safe_distance {
            obstacle_penalty += (safe_distance - distance) / safe_distance;
        }
    }

    // Check environment obstacles
    for &(obstacle_pos, obstacle_radius) in &context.environment_obstacles {
        // Special case: if this unit is gathering resources and this obstacle is their target, reduce penalty
        if let Some(gatherer) = resource_gatherer {
            if let Some(_target_resource) = gatherer.target_resource {
                // If the target position is very close to this obstacle, it's likely the target resource
                if target_pos.distance(obstacle_pos) < 5.0 {
                    // Close to the resource we're targeting
                    // Reduce penalty for approaching target resource
                    let distance = test_pos.distance(obstacle_pos);
                    let safe_distance = obstacle_radius + 15.0; // Allow closer approach
                    if distance < safe_distance {
                        obstacle_penalty += ((safe_distance - distance) / safe_distance) * 0.3;
                        // Much lower penalty
                    }
                    continue;
                }
            }
        }

        // Normal obstacle penalty
        let distance = test_pos.distance(obstacle_pos);
        let safe_distance = obstacle_radius + 3.0;
        if distance < safe_distance {
            obstacle_penalty += (safe_distance - distance) / safe_distance;
        }
    }

    // Final score: favor target alignment, penalize obstacle proximity
    (alignment_score - obstacle_penalty * 2.0).max(-1.0)
}

/// Rotate a direction vector around the Y axis
fn rotate_direction_y(direction: Vec3, angle: f32) -> Vec3 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    Vec3::new(
        direction.x * cos_a - direction.z * sin_a,
        direction.y,
        direction.x * sin_a + direction.z * cos_a,
    )
}

/// Calculate unit avoidance (separate from obstacle avoidance for smoother blending)
fn calculate_unit_avoidance(
    entity: Entity,
    current_pos: Vec3,
    base_direction: Vec3,
    context: &MovementContext,
) -> Vec3 {
    let mut avoidance = Vec3::ZERO;
    let detection_radius = 12.0;

    for &(other_entity, other_pos, _other_radius) in &context.unit_positions {
        if entity == other_entity {
            continue;
        }

        let to_other = other_pos - current_pos;
        let distance = to_other.length();

        if distance < detection_radius && distance > 0.1 {
            let dot_product = base_direction.dot(to_other.normalize_or_zero());

            // If unit is in front of us, add avoidance
            if dot_product > 0.2 {
                let avoidance_strength = (1.0 - distance / detection_radius) * 0.3;
                let perpendicular = Vec3::new(-base_direction.z, 0.0, base_direction.x);
                let side_preference = if (entity.index() % 2) == 0 { 1.0 } else { -1.0 };

                avoidance += perpendicular * side_preference * avoidance_strength;
            }
        }
    }

    avoidance
}

/// Get movement behavior modifiers based on combat state
fn get_combat_movement_modifiers(combat_state: Option<&CombatState>) -> (f32, f32) {
    use crate::core::components::CombatStateType;

    match combat_state.map(|cs| &cs.state) {
        Some(CombatStateType::Idle) => (0.5, 0.9), // Normal separation, normal damping
        Some(CombatStateType::MovingToCombat) => (0.4, 0.95), // Initial combat approach - slightly less separation, maintain speed
        Some(CombatStateType::MovingToAttack) => (0.3, 0.95), // Less separation (more aggressive), less damping (maintain speed)
        Some(CombatStateType::InCombat) => (0.1, 0.8), // Very low separation (tight formations), more damping (precise positioning)
        Some(CombatStateType::CombatMoving) => (0.4, 0.85), // Moderate separation, moderate damping
        None => (0.5, 0.9),                              // Default for units without combat state
    }
}

/// Check if destination spreading should be skipped (e.g., for construction tasks)
fn should_skip_destination_spreading(
    _entity: Entity,
    target: Vec3,
    context: &MovementContext,
) -> bool {
    // Skip spreading if target is very close to a building (likely a construction site)
    for &(building_pos, building_radius) in &context.building_obstacles {
        if target.distance(building_pos) < building_radius + 5.0 {
            return true; // This is likely a construction site, don't spread
        }
    }
    false
}

/// Check if a position is near a construction site
fn is_near_construction_site(position: Vec3, context: &MovementContext) -> bool {
    // Check if position is near any building (construction site)
    for &(building_pos, building_radius) in &context.building_obstacles {
        if position.distance(building_pos) < building_radius + 30.0 {
            return true; // Near a construction site
        }
    }
    false
}

/// Apply destination spreading to prevent units from clustering at the exact same target
fn apply_destination_spreading(
    entity: Entity,
    original_target: Vec3,
    _current_pos: Vec3,
    context: &MovementContext,
) -> Vec3 {
    let destination_radius = 25.0; // Radius around target to check for congestion
    let spread_distance = 12.0; // How far to spread units apart

    // Count how many other units are targeting the same general area
    let mut nearby_targets = 0;
    let mut congestion_center = Vec3::ZERO;

    for &(other_entity, other_pos, _) in &context.unit_positions {
        if entity == other_entity {
            continue;
        }

        // Check if other unit is heading to same general destination
        let distance_to_original_target = other_pos.distance(original_target);
        if distance_to_original_target < destination_radius {
            nearby_targets += 1;
            congestion_center += other_pos;
        }
    }

    // If there's congestion, spread this unit's destination
    if nearby_targets > 0 {
        congestion_center /= nearby_targets as f32;

        // Create a unique offset based on entity hash to ensure consistent spreading
        let entity_hash = entity.index() as f32;
        let angle = (entity_hash * 2.17) % (std::f32::consts::PI * 2.0); // Pseudo-random angle
        let ring_offset = (entity_hash % 3.0) * 4.0; // Vary distance slightly

        let spread_offset = Vec3::new(
            angle.cos() * (spread_distance + ring_offset),
            0.0,
            angle.sin() * (spread_distance + ring_offset),
        );

        // Apply spreading away from congestion center
        let spread_direction = (original_target - congestion_center).normalize_or_zero();
        if spread_direction.length() > 0.1 {
            original_target + spread_direction * spread_distance + spread_offset
        } else {
            // If congestion is exactly at target, use pure radial offset
            original_target + spread_offset
        }
    } else {
        original_target
    }
}

/// Check if a unit is an ant type that should use pathfinding randomness
fn is_ant_unit(rts_unit: &RTSUnit) -> bool {
    use crate::core::components::UnitType;
    matches!(
        rts_unit.unit_type.as_ref(),
        Some(UnitType::WorkerAnt) | Some(UnitType::SoldierAnt) | Some(UnitType::ScoutAnt)
    )
}

/// Add small pathfinding randomness to ant units to prevent identical paths
fn add_ant_pathfinding_randomness(entity: Entity, direction: Vec3, magnitude: f32) -> Vec3 {
    // Use entity hash for consistent but pseudo-random behavior per unit
    let entity_hash = entity.index() as f32;

    // Create a small random angle offset based on entity hash (between -5° and +5°)
    let max_random_angle = 0.087; // ~5 degrees in radians
    let random_angle = ((entity_hash * 3.14159) % (max_random_angle * 2.0)) - max_random_angle;

    // Apply small perpendicular offset to create path variation
    let perpendicular = Vec3::new(-direction.z, 0.0, direction.x).normalize_or_zero();
    let random_offset_strength = 0.15; // Small offset to avoid dramatic path changes
    let random_offset = perpendicular * (entity_hash % 1.0 - 0.5) * random_offset_strength;

    // Combine rotated direction with small offset
    let rotated_direction = rotate_direction_y(direction, random_angle);
    let final_direction = (rotated_direction + random_offset).normalize_or_zero();

    final_direction * magnitude
}
