use bevy::prelude::*;
use crate::core::components::*;

pub struct MovementContext {
    pub delta_time: f32,
    pub unit_positions: Vec<(Entity, Vec3, f32)>,
}

impl MovementContext {
    pub fn new(time: &Res<Time>, units_query: &Query<(Entity, &mut Transform, &mut Movement, &CollisionRadius), With<RTSUnit>>) -> Self {
        let unit_positions: Vec<(Entity, Vec3, f32)> = units_query
            .iter()
            .map(|(entity, transform, _, collision_radius)| (entity, transform.translation, collision_radius.radius))
            .collect();

        Self {
            delta_time: time.delta_secs().min(0.1), // Cap delta time
            unit_positions,
        }
    }
}

pub fn movement_system(
    mut units_query: Query<(Entity, &mut Transform, &mut Movement, &CollisionRadius, &RTSUnit), With<RTSUnit>>,
    terrain_manager: Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::world::terrain_v2::TerrainSettings>,
    time: Res<Time>,
) {
    // Create context without RTSUnit (use simpler query for context)
    let unit_positions: Vec<(Entity, Vec3, f32)> = units_query
        .iter()
        .map(|(entity, transform, _, collision_radius, _)| (entity, transform.translation, collision_radius.radius))
        .collect();

    let context = MovementContext {
        delta_time: time.delta_secs().min(0.1),
        unit_positions,
    };

    for (entity, mut transform, mut movement, collision_radius, rts_unit) in units_query.iter_mut() {
        process_unit_movement(entity, &mut transform, &mut movement, &collision_radius, rts_unit, &context, &terrain_manager, &terrain_settings);
    }
}

fn process_unit_movement(
    entity: Entity,
    transform: &mut Transform,
    movement: &mut Movement,
    collision_radius: &CollisionRadius,
    rts_unit: &RTSUnit,
    context: &MovementContext,
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

        let direction = (target - current_pos).normalize_or_zero();
        let distance = current_pos.distance(target);

        if !distance.is_finite() || distance > MAX_DISTANCE {
            warn!("Invalid distance {:.1}, clearing target", distance);
            movement.target_position = None;
            return;
        }

        if distance > ARRIVAL_THRESHOLD {
            let new_position = calculate_new_position(current_pos, target, movement, context);
            apply_collision_avoidance(entity, new_position, movement, collision_radius, context);
            update_position_with_terrain(transform, new_position, terrain_manager, terrain_settings);
            update_rotation(transform, direction, movement, rts_unit, context.delta_time);
        } else {
            // Apply collision avoidance even when stopped to prevent overlap at destination
            apply_collision_avoidance(entity, current_pos, movement, collision_radius, context);
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
    pos.x.abs() <= MAX_POSITION && pos.z.abs() <= MAX_POSITION && pos.x.is_finite() && pos.z.is_finite()
}

fn reset_unit_to_origin(transform: &mut Transform, movement: &mut Movement) {
    use crate::constants::movement::DEFAULT_SPAWN_HEIGHT;
    warn!("Unit at extreme position, resetting to origin");
    transform.translation = Vec3::new(0.0, DEFAULT_SPAWN_HEIGHT, 0.0);
    movement.current_velocity = Vec3::ZERO;
    movement.target_position = None;
}

fn calculate_new_position(current_pos: Vec3, target: Vec3, movement: &mut Movement, context: &MovementContext) -> Vec3 {
    use crate::constants::movement::*;
    
    let direction = (target - current_pos).normalize_or_zero();
    let target_velocity = direction * movement.max_speed;
    let clamped_velocity = clamp_velocity(target_velocity);
    
    movement.current_velocity = movement.current_velocity.lerp(
        clamped_velocity,
        movement.acceleration * context.delta_time
    );
    
    movement.current_velocity = clamp_velocity(movement.current_velocity);
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
) {
    use crate::constants::movement::*;
    
    if !is_valid_position(new_position) {
        warn!("New position would be invalid, stopping movement");
        movement.current_velocity = Vec3::ZERO;
        movement.target_position = None;
        return;
    }
    
    let separation_data = calculate_separation_force(entity, new_position, collision_radius, context);
    
    if separation_data.force.length() > 0.001 {
        let velocity_factor = (1.0 - movement.current_velocity.length() / movement.max_speed).max(0.2);
        movement.current_velocity += separation_data.force * context.delta_time * velocity_factor;
    }

    if separation_data.has_collision {
        movement.current_velocity *= 0.5;  // Stronger damping to reduce jerkiness
    }
    
    movement.current_velocity = clamp_velocity(movement.current_velocity);
}

struct SeparationData {
    force: Vec3,
    has_collision: bool,
}

fn calculate_separation_force(
    entity: Entity,
    position: Vec3,
    collision_radius: &CollisionRadius,
    context: &MovementContext,
) -> SeparationData {
    use crate::constants::movement::*;
    
    let mut separation_force = Vec3::ZERO;
    let separation_radius = collision_radius.radius * SEPARATION_MULTIPLIER;
    let mut has_collision = false;
    
    for &(other_entity, other_position, other_radius) in &context.unit_positions {
        if entity == other_entity {
            continue;
        }
        
        let to_other = other_position - position;
        let distance = to_other.length();
        let min_distance = collision_radius.radius + other_radius + 0.2;
        
        if distance < separation_radius && distance > 0.001 {
            let separation_strength = ((separation_radius - distance) / separation_radius).powf(2.0);
            let separation_direction = -to_other.normalize_or_zero();
            separation_force += separation_direction * separation_strength * SEPARATION_FORCE_STRENGTH;
        }
        
        if distance < min_distance && distance > 0.001 {
            has_collision = true;
        }
    }
    
    SeparationData {
        force: separation_force,
        has_collision,
    }
}

fn update_position_with_terrain(
    transform: &mut Transform,
    new_position: Vec3,
    terrain_manager: &crate::world::terrain_v2::TerrainChunkManager,
    terrain_settings: &crate::world::terrain_v2::TerrainSettings,
) {
    use crate::constants::movement::TERRAIN_SAMPLE_LIMIT;
    
    let terrain_height = if new_position.x.abs() < TERRAIN_SAMPLE_LIMIT && new_position.z.abs() < TERRAIN_SAMPLE_LIMIT {
        crate::world::terrain_v2::sample_terrain_height(
            new_position.x,
            new_position.z,
            &terrain_manager.noise_generator,
            terrain_settings,
        )
    } else {
        0.0
    };
    
    let mut final_position = new_position;
    final_position.y = terrain_height + 2.0;
    transform.translation = final_position;
}

fn update_rotation(transform: &mut Transform, direction: Vec3, movement: &Movement, rts_unit: &RTSUnit, delta_time: f32) {
    use crate::core::components::UnitType;

    if direction.length() <= 0.1 {
        return;
    }

    // Calculate rotation based on model type
    // Some models (Fourmi/Ant) naturally face forward and don't need pre-rotation
    // Others face backward and are pre-rotated 180° in entity_factory
    let target_rotation = match rts_unit.unit_type.as_ref() {
        Some(UnitType::WorkerAnt) | Some(UnitType::SoldierAnt) => {
            // Ant models naturally face backward in GLB, no pre-rotation applied
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