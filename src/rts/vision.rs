use crate::core::components::*;
use crate::core::resources::SpatialGrids;
use bevy::prelude::*;

pub fn vision_system(
    units: Query<(Entity, &Position, &Vision, &RTSUnit)>,
    spatial_grids: Res<SpatialGrids>,
    mut visible_units: Local<std::collections::HashMap<Entity, Vec<Entity>>>,
) {
    visible_units.clear();

    for (entity, position, vision, unit) in units.iter() {
        let visible = find_visible_units_spatial(entity, position, vision, unit, &spatial_grids);
        visible_units.insert(entity, visible);
    }
}

// New optimized version using spatial grid
fn find_visible_units_spatial(
    observer_entity: Entity,
    observer_position: &Position,
    vision: &Vision,
    observer_unit: &RTSUnit,
    spatial_grids: &SpatialGrids,
) -> Vec<Entity> {
    let mut visible = Vec::new();
    
    // Query only nearby entities within vision range using spatial grid
    let nearby_entities = spatial_grids.entity_grid.query_nearby_entities(
        observer_position.translation,
        vision.sight_range,
        Some(observer_entity), // Exclude self
    );

    for (entity, position, _radius) in nearby_entities {
        // We need to check if this is actually a unit and get its RTSUnit component
        // For spatial grid optimization, we can only do basic distance/visibility checks here
        let distance = observer_position.translation.distance(position);
        
        if distance <= vision.sight_range {
            // Basic visibility check - can be enhanced with line-of-sight later
            if observer_unit.player_id != 1 || entity != observer_entity {
                // Simple visibility: if it's in range and not self, it's visible
                // More complex line-of-sight could be added here
                visible.push(entity);
            }
        }
    }

    visible
}

// Keep old implementation as fallback
fn find_visible_units(
    observer_entity: Entity,
    observer_position: &Position,
    vision: &Vision,
    observer_unit: &RTSUnit,
    all_units: &Query<(Entity, &Position, &Vision, &RTSUnit)>,
) -> Vec<Entity> {
    let mut visible = Vec::new();

    for (entity, position, _, unit) in all_units.iter() {
        if should_be_visible(
            observer_entity,
            observer_position,
            observer_unit,
            entity,
            position,
            unit,
            vision,
        ) {
            visible.push(entity);
        }
    }

    visible
}

fn should_be_visible(
    observer_entity: Entity,
    observer_position: &Position,
    observer_unit: &RTSUnit,
    target_entity: Entity,
    target_position: &Position,
    target_unit: &RTSUnit,
    vision: &Vision,
) -> bool {
    if observer_entity == target_entity {
        return false;
    }

    if observer_unit.player_id == target_unit.player_id {
        return false;
    }

    let distance = observer_position
        .translation
        .distance(target_position.translation);
    distance <= vision.sight_range
}
