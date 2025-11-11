use bevy::prelude::*;
use crate::core::components::*;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unit_collision_avoidance_system);
    }
}

/// Collision detection utilities for preventing object overlap and unit collisions
pub struct CollisionUtils;

impl CollisionUtils {
    /// Check if a position would collide with any existing entities
    pub fn check_position_collision(
        position: Vec3,
        radius: f32,
        existing_entities: &Query<(&Transform, &CollisionRadius), With<GameEntity>>,
        _exclude_entity: Option<Entity>,
    ) -> bool {
        for (transform, collision_radius) in existing_entities.iter() {
            // Note: Entity filtering would require access to the entity ID in the query
            // This is a simplified version that checks all entities

            let distance = position.distance(transform.translation);
            let min_distance = radius + collision_radius.radius;

            if distance < min_distance {
                return true; // Collision detected
            }
        }
        false
    }
    
    /// Find a safe position within a given area that doesn't collide with existing entities
    pub fn find_safe_position(
        min_pos: Vec3,
        max_pos: Vec3,
        radius: f32,
        existing_entities: &Query<(&Transform, &CollisionRadius), With<GameEntity>>,
        max_attempts: u32,
    ) -> Option<Vec3> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        for _ in 0..max_attempts {
            let x = rng.gen_range(min_pos.x..max_pos.x);
            let z = rng.gen_range(min_pos.z..max_pos.z);
            let y = (min_pos.y + max_pos.y) * 0.5; // Use average height
            
            let test_position = Vec3::new(x, y, z);
            
            if !Self::check_position_collision(test_position, radius, existing_entities, None) {
                return Some(test_position);
            }
        }
        
        None // Couldn't find a safe position
    }
    
    /// Get all entities within a certain radius of a position
    pub fn get_nearby_entities(
        position: Vec3,
        search_radius: f32,
        entities: &Query<(Entity, &Transform, &CollisionRadius), With<GameEntity>>,
    ) -> Vec<Entity> {
        entities
            .iter()
            .filter_map(|(entity, transform, _)| {
                let distance = position.distance(transform.translation);
                if distance <= search_radius {
                    Some(entity)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Calculate separation force to avoid collision with nearby entities
    pub fn calculate_separation_force(
        entity: Entity,
        position: Vec3,
        radius: f32,
        entities: &Query<(Entity, &Transform, &CollisionRadius), With<GameEntity>>,
    ) -> Vec3 {
        let mut separation_force = Vec3::ZERO;
        let mut neighbor_count = 0;
        
        for (other_entity, transform, collision_radius) in entities.iter() {
            if entity == other_entity {
                continue;
            }
            
            let distance = position.distance(transform.translation);
            let min_distance = radius + collision_radius.radius;
            
            if distance < min_distance && distance > 0.001 {
                // Calculate repulsion force
                let direction = (position - transform.translation).normalize();
                let force_magnitude = (min_distance - distance) / min_distance;
                separation_force += direction * force_magnitude;
                neighbor_count += 1;
            }
        }
        
        if neighbor_count > 0 {
            separation_force / neighbor_count as f32
        } else {
            Vec3::ZERO
        }
    }
}

/// System to prevent units from overlapping with buildings, environment objects, and other units
pub fn unit_collision_avoidance_system(
    mut units: Query<(Entity, &mut Transform, &mut Movement, &CollisionRadius), With<RTSUnit>>,
    obstacles: Query<(&Transform, &CollisionRadius), (With<GameEntity>, Without<RTSUnit>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs().min(0.1); // Cap delta time to prevent large jumps

    // Store unit transformations BEFORE mutating (immutable borrow, then release)
    let unit_positions: Vec<(Entity, Vec3, f32, Vec3)> = units.iter()
        .map(|(e, t, m, r)| {
            (e, t.translation, r.radius, m.current_velocity)
        })
        .collect();

    // Now do mutable iteration
    for (unit_entity, mut unit_transform, mut movement, unit_radius) in units.iter_mut() {
        let mut avoidance_force = Vec3::ZERO;
        let velocity_magnitude = movement.current_velocity.length();
        let is_moving = velocity_magnitude > 0.5; // Higher threshold to reduce sensitivity

        // Check collision with all static obstacles (buildings, environment objects)
        for (obstacle_transform, obstacle_radius) in obstacles.iter() {
            let distance = unit_transform.translation.distance(obstacle_transform.translation);
            let min_distance = unit_radius.radius + obstacle_radius.radius + 1.5; // Larger buffer

            if distance < min_distance && distance > 0.001 {
                // Calculate avoidance direction
                let direction = (unit_transform.translation - obstacle_transform.translation).normalize();
                let overlap = min_distance - distance;

                // Only apply force if significantly overlapping
                if overlap > 0.3 {
                    let force_magnitude = (overlap / min_distance).min(1.0);
                    avoidance_force += direction * force_magnitude * 40.0; // Reduced from 50.0

                    // If seriously overlapping, push directly away
                    if distance < min_distance * 0.85 {
                        unit_transform.translation += direction * overlap * 0.3;
                    }
                }
            }
        }

        // Check collision with other units for separation
        for (other_entity, other_pos, other_radius, other_velocity) in &unit_positions {
            if unit_entity == *other_entity {
                continue;
            }

            let distance = unit_transform.translation.distance(*other_pos);
            let min_distance = unit_radius.radius + other_radius + 1.2; // Larger spacing buffer

            // Only process if within collision range
            if distance < min_distance && distance > 0.001 {
                let direction = (unit_transform.translation - other_pos).normalize();
                let overlap = min_distance - distance;

                // Add dead zone - don't apply forces for minor overlaps
                if overlap > 0.4 {
                    // Use relative velocity to determine force strength
                    let relative_velocity = (movement.current_velocity - other_velocity).length();
                    let is_converging = relative_velocity > 0.5;

                    // Reduced forces to prevent oscillation
                    let force_multiplier = if is_converging {
                        5.0 // Apply force when units are moving toward each other
                    } else if is_moving {
                        2.0 // Gentle force when moving
                    } else {
                        1.0 // Minimal force when stationary
                    };

                    let force_magnitude = (overlap / min_distance).min(1.0);
                    avoidance_force += direction * force_magnitude * force_multiplier;

                    // Only push apart if seriously overlapping
                    if overlap > min_distance * 0.6 {
                        let push_distance = overlap * 0.05; // Very gentle push
                        unit_transform.translation += direction * push_distance;
                    }
                }
            }
        }

        // Apply avoidance force to movement with strong damping
        let avoidance_magnitude = avoidance_force.length();
        if avoidance_magnitude > 0.1 {
            if is_moving {
                // Smooth velocity adjustment with clamping
                let adjustment = avoidance_force * dt * 0.3; // Reduced from 0.5
                let new_velocity = movement.current_velocity + adjustment;

                // Clamp velocity to prevent excessive speeds
                let max_avoidance_speed = movement.max_speed * 1.2;
                if new_velocity.length() > max_avoidance_speed {
                    movement.current_velocity = new_velocity.normalize() * max_avoidance_speed;
                } else {
                    movement.current_velocity = new_velocity;
                }
            } else {
                // For stationary units, minimal adjustment with exponential smoothing
                let adjustment = avoidance_force.normalize() * dt * 0.5; // Reduced from 2.0
                unit_transform.translation += adjustment;
            }

            // Apply velocity damping to reduce oscillation
            movement.current_velocity *= 0.98; // Add slight damping
        }
    }
}

/// System to prevent environment objects from spawning too close to each other
pub fn validate_environment_object_position(
    position: Vec3,
    radius: f32,
    existing_objects: &Query<(&Transform, &CollisionRadius), With<EnvironmentObject>>,
    buildings: &Query<(&Transform, &CollisionRadius), With<Building>>,
    units: &Query<(&Transform, &CollisionRadius), With<RTSUnit>>,
) -> bool {
    // Check against environment objects
    for (transform, collision_radius) in existing_objects.iter() {
        let distance = position.distance(transform.translation);
        let min_distance = radius + collision_radius.radius + 5.0; // Extra spacing

        if distance < min_distance {
            return false;
        }
    }

    // Check against buildings
    for (transform, collision_radius) in buildings.iter() {
        let distance = position.distance(transform.translation);
        let min_distance = radius + collision_radius.radius + 10.0; // Extra spacing from buildings

        if distance < min_distance {
            return false;
        }
    }

    // Check against units (ensure objects don't spawn on top of units)
    for (transform, collision_radius) in units.iter() {
        let distance = position.distance(transform.translation);
        let min_distance = radius + collision_radius.radius + 3.0; // Small spacing from units

        if distance < min_distance {
            return false;
        }
    }

    true // Position is safe
}

/// Validate that a building can be placed at the given position without overlapping
/// Returns true if the position is valid (no collisions), false otherwise
pub fn validate_building_placement<BF, UF, EF>(
    position: Vec3,
    building_radius: f32,
    existing_buildings: &Query<(&Transform, &CollisionRadius), BF>,
    units: &Query<(&Transform, &CollisionRadius), UF>,
    environment_objects: &Query<(&Transform, &CollisionRadius), EF>,
) -> bool
where
    BF: bevy::ecs::query::QueryFilter,
    UF: bevy::ecs::query::QueryFilter,
    EF: bevy::ecs::query::QueryFilter,
{
    use crate::constants::building_placement::*;

    // Check against existing buildings - prevent overlap
    for (transform, collision_radius) in existing_buildings.iter() {
        let distance = position.distance(transform.translation);
        let min_distance = building_radius + collision_radius.radius + MIN_SPACING_BETWEEN_BUILDINGS;

        if distance < min_distance {
            return false; // Too close to another building
        }
    }

    // Check against units - buildings shouldn't be placed on top of units
    for (transform, collision_radius) in units.iter() {
        let distance = position.distance(transform.translation);
        let min_distance = building_radius + collision_radius.radius + MIN_SPACING_FROM_UNITS;

        if distance < min_distance {
            return false; // Too close to a unit
        }
    }

    // Check against environment objects - avoid placing buildings on obstacles
    for (transform, collision_radius) in environment_objects.iter() {
        let distance = position.distance(transform.translation);
        let min_distance = building_radius + collision_radius.radius + MIN_SPACING_FROM_ENVIRONMENT;

        if distance < min_distance {
            return false; // Too close to environment object
        }
    }

    true // Position is valid
}