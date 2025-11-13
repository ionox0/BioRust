use crate::core::components::*;
use crate::core::spatial_grid::GridCoord;
use crate::core::resources::SpatialGrids;
use bevy::prelude::*;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            spatial_grid_update_system.before(unit_collision_avoidance_system),
            unit_collision_avoidance_system,
        ));
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

// Collision constants
mod collision_constants {
    pub const MAX_DELTA_TIME: f32 = 0.1;
    pub const MOVING_VELOCITY_THRESHOLD: f32 = 2.0;
    pub const BUILDING_BUFFER: f32 = 1.5;
    pub const UNIT_BUFFER: f32 = 0.8;
    pub const OVERLAP_THRESHOLD: f32 = 0.3;
    pub const AVOIDANCE_FORCE_BASE: f32 = 40.0;
    pub const SERIOUS_OVERLAP_RATIO: f32 = 0.85;
    pub const PUSH_MULTIPLIER: f32 = 0.3;
    pub const UNIT_OVERLAP_THRESHOLD: f32 = 0.8;
    pub const CONVERGENCE_THRESHOLD: f32 = 2.0;
    pub const CONVERGING_FORCE: f32 = 3.0;
    pub const MOVING_FORCE: f32 = 1.0;
    pub const STATIONARY_FORCE: f32 = 0.5;
    pub const SERIOUS_UNIT_OVERLAP_RATIO: f32 = 0.6;
    pub const GENTLE_PUSH_MULTIPLIER: f32 = 0.05;
    pub const MIN_AVOIDANCE_MAGNITUDE: f32 = 0.1;
    pub const VELOCITY_ADJUSTMENT_FACTOR: f32 = 0.3;
    pub const MAX_SPEED_MULTIPLIER: f32 = 1.2;
    pub const STATIONARY_ADJUSTMENT_FACTOR: f32 = 0.5;
    pub const VELOCITY_DAMPING: f32 = 0.98;
}

/// System to incrementally update the spatial grid for entities
pub fn spatial_grid_update_system(
    mut spatial_grids: ResMut<SpatialGrids>,
    mut units: Query<(Entity, &Transform, &CollisionRadius, &mut SpatialGridPosition), With<RTSUnit>>,
) {

    // Track entities that need to be removed (no longer exist in query)
    let current_entities: std::collections::HashSet<Entity> = units.iter().map(|(e, _, _, _)| e).collect();
    let grid_entities: std::collections::HashSet<Entity> = spatial_grids.entity_grid.entity_positions.keys().copied().collect();
    
    // Remove entities that no longer exist
    for entity in grid_entities.difference(&current_entities) {
        spatial_grids.entity_grid.remove_item(*entity);
    }

    // Update entities that are dirty or have changed position
    for (entity, transform, collision_radius, mut grid_pos) in units.iter_mut() {
        let current_coord = GridCoord::from_world_pos(transform.translation, spatial_grids.entity_grid.cell_size);
        
        // Check if position changed significantly or is dirty
        let needs_update = grid_pos.dirty || 
            grid_pos.last_grid_coord.map_or(true, |last_coord| last_coord != current_coord);
        
        if needs_update {
            let changed_cells = spatial_grids.entity_grid.update_entity(
                entity, 
                transform.translation, 
                collision_radius.radius
            );
            
            grid_pos.last_grid_coord = Some(current_coord);
            grid_pos.dirty = false;
            
            if changed_cells {
                debug!("Entity {:?} moved to new grid cell {:?}", entity, current_coord);
            }
        }
    }
}

/// System to prevent units from overlapping with buildings, environment objects, and other units
///
/// This system handles collision avoidance for all moving units by:
/// 1. Checking collisions with static obstacles (buildings, environment objects, resources)
/// 2. Checking collisions with other moving units using persistent spatial grid
/// 3. Applying appropriate avoidance forces
pub fn unit_collision_avoidance_system(
    mut units: Query<(Entity, &mut Transform, &mut Movement, &CollisionRadius), With<RTSUnit>>,
    buildings: Query<(&Position, &CollisionRadius), (With<Building>, Without<Movement>)>,
    environment_objects: Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<Movement>),
    >,
    spatial_grids: Res<SpatialGrids>,
    time: Res<Time>,
) {
    use collision_constants::*;

    let dt = time.delta_secs().min(MAX_DELTA_TIME);

    // Get unit velocities for collision calculations
    let unit_velocities: std::collections::HashMap<Entity, Vec3> = units
        .iter()
        .map(|(entity, _, movement, _)| (entity, movement.current_velocity))
        .collect();

    // Process each unit
    for (unit_entity, mut unit_transform, mut movement, unit_radius) in units.iter_mut() {
        let mut avoidance_force = Vec3::ZERO;
        let is_moving = movement.current_velocity.length() > MOVING_VELOCITY_THRESHOLD;

        // Check collisions with static obstacles
        avoidance_force += process_static_collisions(
            unit_transform.translation,
            unit_radius.radius,
            &buildings,
            BUILDING_BUFFER,
            &mut unit_transform,
        );

        avoidance_force += process_environment_collisions(
            unit_transform.translation,
            unit_radius.radius,
            &environment_objects,
            BUILDING_BUFFER,
            &mut unit_transform,
        );

        // Check collisions with nearby units using persistent spatial grid
        let nearby_units = spatial_grids.entity_grid.query_nearby_entities(
            unit_transform.translation,
            unit_radius.radius * 5.0,
            Some(unit_entity),
        );
        for (other_entity, other_pos, other_radius) in nearby_units {

            // Find velocity of other unit
            let other_velocity = unit_velocities
                .get(&other_entity)
                .copied()
                .unwrap_or(Vec3::ZERO);

            if let Some(collision) = check_collision(
                unit_transform.translation,
                unit_radius.radius,
                other_pos,
                other_radius,
                UNIT_BUFFER,
            ) {
                if collision.overlap > UNIT_OVERLAP_THRESHOLD {
                    let relative_velocity = (movement.current_velocity - other_velocity).length();
                    let is_converging = relative_velocity > CONVERGENCE_THRESHOLD;

                    let force_multiplier = if is_converging {
                        CONVERGING_FORCE
                    } else if is_moving {
                        MOVING_FORCE
                    } else {
                        STATIONARY_FORCE
                    };

                    let force_magnitude = (collision.overlap / collision.min_distance).min(1.0);
                    avoidance_force += collision.direction * force_magnitude * force_multiplier;

                    if collision.overlap > collision.min_distance * SERIOUS_UNIT_OVERLAP_RATIO {
                        unit_transform.translation +=
                            collision.direction * collision.overlap * GENTLE_PUSH_MULTIPLIER;
                    }
                }
            }
        }

        // Apply avoidance force
        if avoidance_force.length() > MIN_AVOIDANCE_MAGNITUDE {
            if is_moving {
                let adjustment = avoidance_force * dt * VELOCITY_ADJUSTMENT_FACTOR;
                let new_velocity = movement.current_velocity + adjustment;
                let max_avoidance_speed = movement.max_speed * MAX_SPEED_MULTIPLIER;

                movement.current_velocity = if new_velocity.length() > max_avoidance_speed {
                    new_velocity.normalize() * max_avoidance_speed
                } else {
                    new_velocity
                };
            } else {
                unit_transform.translation +=
                    avoidance_force.normalize() * dt * STATIONARY_ADJUSTMENT_FACTOR;
            }

            movement.current_velocity *= VELOCITY_DAMPING;
        }
    }
}

fn process_static_collisions(
    position: Vec3,
    radius: f32,
    buildings: &Query<(&Position, &CollisionRadius), (With<Building>, Without<Movement>)>,
    buffer: f32,
    transform: &mut Transform,
) -> Vec3 {
    use collision_constants::*;

    let mut force = Vec3::ZERO;
    for (building_position, building_radius) in buildings.iter() {
        if let Some(collision) = check_collision(
            position,
            radius,
            building_position.translation,
            building_radius.radius,
            buffer,
        ) {
            if collision.overlap > OVERLAP_THRESHOLD {
                let force_magnitude = (collision.overlap / collision.min_distance).min(1.0);
                force += collision.direction * force_magnitude * AVOIDANCE_FORCE_BASE;

                if collision.distance < collision.min_distance * SERIOUS_OVERLAP_RATIO {
                    transform.translation +=
                        collision.direction * collision.overlap * PUSH_MULTIPLIER;
                }
            }
        }
    }
    force
}

fn process_environment_collisions(
    position: Vec3,
    radius: f32,
    environment_objects: &Query<
        (&Transform, &CollisionRadius),
        (With<EnvironmentObject>, Without<Movement>),
    >,
    buffer: f32,
    transform: &mut Transform,
) -> Vec3 {
    use collision_constants::*;

    let mut force = Vec3::ZERO;
    for (env_transform, env_radius) in environment_objects.iter() {
        if let Some(collision) = check_collision(
            position,
            radius,
            env_transform.translation,
            env_radius.radius,
            buffer,
        ) {
            if collision.overlap > OVERLAP_THRESHOLD {
                let force_magnitude = (collision.overlap / collision.min_distance).min(1.0);
                force += collision.direction * force_magnitude * AVOIDANCE_FORCE_BASE;

                if collision.distance < collision.min_distance * SERIOUS_OVERLAP_RATIO {
                    transform.translation +=
                        collision.direction * collision.overlap * PUSH_MULTIPLIER;
                }
            }
        }
    }
    force
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
    check_building_collisions(position, building_radius, existing_buildings)
        && check_unit_collisions(position, building_radius, units)
        && check_environment_collisions(position, building_radius, environment_objects)
}

fn check_building_collisions<BF>(
    position: Vec3,
    building_radius: f32,
    existing_buildings: &Query<(&Transform, &CollisionRadius), BF>,
) -> bool
where
    BF: bevy::ecs::query::QueryFilter,
{
    use crate::constants::building_placement::MIN_SPACING_BETWEEN_BUILDINGS;

    for (transform, collision_radius) in existing_buildings.iter() {
        let distance = position.distance(transform.translation);
        let min_distance =
            building_radius + collision_radius.radius + MIN_SPACING_BETWEEN_BUILDINGS;

        if distance < min_distance {
            return false; // Too close to another building
        }
    }
    true
}

fn check_unit_collisions<UF>(
    position: Vec3,
    building_radius: f32,
    units: &Query<(&Transform, &CollisionRadius), UF>,
) -> bool
where
    UF: bevy::ecs::query::QueryFilter,
{
    use crate::constants::building_placement::MIN_SPACING_FROM_UNITS;

    for (transform, collision_radius) in units.iter() {
        let distance = position.distance(transform.translation);
        let min_distance = building_radius + collision_radius.radius + MIN_SPACING_FROM_UNITS;

        if distance < min_distance {
            return false; // Too close to a unit
        }
    }
    true
}

fn check_environment_collisions<EF>(
    position: Vec3,
    building_radius: f32,
    environment_objects: &Query<(&Transform, &CollisionRadius, &EnvironmentObject), EF>,
) -> bool
where
    EF: bevy::ecs::query::QueryFilter,
{
    for (transform, collision_radius, env_object) in environment_objects.iter() {
        let distance = position.distance(transform.translation);
        let extra_spacing = calculate_environment_spacing(&env_object.object_type);
        let min_distance = building_radius + collision_radius.radius + extra_spacing;

        if distance < min_distance {
            return false; // Too close to environment object
        }
    }
    true
}

fn calculate_environment_spacing(object_type: &EnvironmentObjectType) -> f32 {
    match object_type {
        EnvironmentObjectType::Mushrooms => 80.0, // Much wider radius for mushrooms
        _ => 3.0,                                 // Normal spacing for other objects
    }
}
