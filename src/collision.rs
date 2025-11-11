use bevy::prelude::*;
use crate::core::components::*;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unit_collision_avoidance_system);
    }
}



/// Helper to check if two entities are colliding
fn check_collision(
    pos1: Vec3,
    radius1: f32,
    pos2: Vec3,
    radius2: f32,
    buffer: f32,
) -> Option<CollisionInfo> {
    let distance = pos1.distance(pos2);
    let min_distance = radius1 + radius2 + buffer;

    if distance < min_distance && distance > 0.001 {
        let direction = (pos1 - pos2).normalize();
        let overlap = min_distance - distance;
        Some(CollisionInfo {
            direction,
            overlap,
            distance,
            min_distance,
        })
    } else {
        None
    }
}

/// Information about a collision between two entities
struct CollisionInfo {
    direction: Vec3,
    overlap: f32,
    distance: f32,
    min_distance: f32,
}

/// System to prevent units from overlapping with buildings, environment objects, and other units
///
/// This system handles collision avoidance for all moving units by:
/// 1. Checking collisions with static obstacles (buildings, environment objects, resources)
/// 2. Checking collisions with other moving units
/// 3. Applying appropriate avoidance forces
pub fn unit_collision_avoidance_system(
    mut units: Query<(Entity, &mut Transform, &mut Movement, &CollisionRadius), With<RTSUnit>>,
    // Query for static obstacles: buildings use Position+GlobalTransform for GLB model compatibility
    buildings: Query<(&Position, &CollisionRadius), (With<Building>, Without<Movement>)>,
    // Environment objects use Transform directly
    environment_objects: Query<(&Transform, &CollisionRadius), (With<EnvironmentObject>, Without<Movement>)>,
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

        // Check collision with buildings (static obstacles)
        // Buildings use Position component for their world location (GLB models)
        for (building_position, building_radius) in buildings.iter() {
            if let Some(collision) = check_collision(
                unit_transform.translation,
                unit_radius.radius,
                building_position.translation,
                building_radius.radius,
                1.5, // Larger buffer for buildings
            ) {
                // Only apply force if significantly overlapping
                if collision.overlap > 0.3 {
                    let force_magnitude = (collision.overlap / collision.min_distance).min(1.0);
                    avoidance_force += collision.direction * force_magnitude * 40.0;

                    // If seriously overlapping, push directly away
                    if collision.distance < collision.min_distance * 0.85 {
                        unit_transform.translation += collision.direction * collision.overlap * 0.3;
                    }
                }
            }
        }

        // Check collision with environment objects and resources (static obstacles)
        for (env_transform, env_radius) in environment_objects.iter() {
            if let Some(collision) = check_collision(
                unit_transform.translation,
                unit_radius.radius,
                env_transform.translation,
                env_radius.radius,
                1.5, // Larger buffer
            ) {
                // Only apply force if significantly overlapping
                if collision.overlap > 0.3 {
                    let force_magnitude = (collision.overlap / collision.min_distance).min(1.0);
                    avoidance_force += collision.direction * force_magnitude * 40.0;

                    // If seriously overlapping, push directly away
                    if collision.distance < collision.min_distance * 0.85 {
                        unit_transform.translation += collision.direction * collision.overlap * 0.3;
                    }
                }
            }
        }

        // Check collision with other units for separation
        for (other_entity, other_pos, other_radius, other_velocity) in &unit_positions {
            if unit_entity == *other_entity {
                continue;
            }

            if let Some(collision) = check_collision(
                unit_transform.translation,
                unit_radius.radius,
                *other_pos,
                *other_radius,
                1.2, // Larger spacing buffer for units
            ) {
                // Add dead zone - don't apply forces for minor overlaps
                if collision.overlap > 0.4 {
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

                    let force_magnitude = (collision.overlap / collision.min_distance).min(1.0);
                    avoidance_force += collision.direction * force_magnitude * force_multiplier;

                    // Only push apart if seriously overlapping
                    if collision.overlap > collision.min_distance * 0.6 {
                        let push_distance = collision.overlap * 0.05; // Very gentle push
                        unit_transform.translation += collision.direction * push_distance;
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


/// Validate that a building can be placed at the given position without overlapping
/// Returns true if the position is valid (no collisions), false otherwise
pub fn validate_building_placement<BF, UF, EF>(
    position: Vec3,
    building_radius: f32,
    existing_buildings: &Query<(&Transform, &CollisionRadius), BF>,
    units: &Query<(&Transform, &CollisionRadius), UF>,
    environment_objects: &Query<(&Transform, &CollisionRadius, &EnvironmentObject), EF>,
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
    for (transform, collision_radius, env_object) in environment_objects.iter() {
        let distance = position.distance(transform.translation);
        
        // Apply larger spacing for mushrooms to prevent buildings from being placed too close
        let extra_spacing = match env_object.object_type {
            EnvironmentObjectType::Mushrooms => 80.0, // Much wider radius for mushrooms (10x increased)
            _ => MIN_SPACING_FROM_ENVIRONMENT, // Normal spacing for other objects
        };
        let min_distance = building_radius + collision_radius.radius + extra_spacing;

        if distance < min_distance {
            return false; // Too close to environment object
        }
    }

    true // Position is valid
}