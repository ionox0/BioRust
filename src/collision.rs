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
        exclude_entity: Option<Entity>,
    ) -> bool {
        for (transform, collision_radius) in existing_entities.iter() {
            if let Some(exclude) = exclude_entity {
                // Skip the entity we're checking against itself
                continue;
            }
            
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
    let dt = time.delta_secs();

    // Store unit transformations BEFORE mutating (immutable borrow, then release)
    let unit_positions: Vec<(Entity, Vec3, f32, bool)> = units.iter()
        .map(|(e, t, m, r)| {
            // Track if unit is stationary (at destination or not moving)
            let is_stationary = m.target_position.is_none() || m.current_velocity.length() < 0.1;
            (e, t.translation, r.radius, is_stationary)
        })
        .collect();

    // Now do mutable iteration
    for (unit_entity, mut unit_transform, mut movement, unit_radius) in units.iter_mut() {
        let mut avoidance_force = Vec3::ZERO;
        let is_moving = movement.target_position.is_some() && movement.current_velocity.length() > 0.1;

        // Check collision with all static obstacles (buildings, environment objects)
        for (obstacle_transform, obstacle_radius) in obstacles.iter() {
            let distance = unit_transform.translation.distance(obstacle_transform.translation);
            let min_distance = unit_radius.radius + obstacle_radius.radius + 1.0; // Add buffer

            if distance < min_distance && distance > 0.001 {
                // Calculate avoidance direction
                let direction = (unit_transform.translation - obstacle_transform.translation).normalize();
                let force_magnitude = (min_distance - distance) / min_distance;
                avoidance_force += direction * force_magnitude * 50.0; // Strong avoidance force

                // If overlapping, push directly away immediately
                if distance < min_distance * 0.9 {
                    unit_transform.translation += direction * (min_distance - distance) * 0.5;
                }
            }
        }

        // Check collision with other units for separation
        for (other_entity, other_pos, other_radius, other_stationary) in &unit_positions {
            if unit_entity == *other_entity {
                continue;
            }

            let distance = unit_transform.translation.distance(*other_pos);
            let min_distance = unit_radius.radius + other_radius + 0.5; // Always maintain minimum spacing

            // Apply separation whenever units are too close
            if distance < min_distance && distance > 0.001 {
                // Calculate separation direction
                let direction = (unit_transform.translation - other_pos).normalize();
                let force_magnitude = (min_distance - distance) / min_distance;

                // Use stronger force when both units are stationary (at destination)
                let force_multiplier = if !is_moving && *other_stationary {
                    20.0 // Strong push when both stationary
                } else if !is_moving || *other_stationary {
                    15.0 // Medium push when one is stationary
                } else {
                    10.0 // Gentle push when both moving
                };

                avoidance_force += direction * force_magnitude * force_multiplier;

                // If units are overlapping significantly, directly push them apart
                if distance < min_distance * 0.7 {
                    let push_distance = (min_distance - distance) * 0.3;
                    unit_transform.translation += direction * push_distance;
                }
            }
        }

        // Apply avoidance force to movement
        if avoidance_force.length() > 0.001 {
            // Always apply some force, even if not moving
            if is_moving {
                movement.current_velocity += avoidance_force * dt;
            } else {
                // For stationary units, directly adjust position
                unit_transform.translation += avoidance_force.normalize() * dt * 3.0;
            }

            // If we're very close to an obstacle or unit, push directly away
            if avoidance_force.length() > 15.0 {
                unit_transform.translation += avoidance_force.normalize() * dt * 5.0;
            }
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