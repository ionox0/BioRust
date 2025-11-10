use bevy::prelude::*;
use crate::core::components::*;

pub fn vision_system(
    units: Query<(Entity, &Position, &Vision, &RTSUnit)>,
    mut visible_units: Local<std::collections::HashMap<Entity, Vec<Entity>>>,
) {
    visible_units.clear();
    
    for (entity, position, vision, unit) in units.iter() {
        let visible = find_visible_units(entity, position, vision, unit, &units);
        visible_units.insert(entity, visible);
    }
}

fn find_visible_units(
    observer_entity: Entity,
    observer_position: &Position,
    vision: &Vision,
    observer_unit: &RTSUnit,
    all_units: &Query<(Entity, &Position, &Vision, &RTSUnit)>,
) -> Vec<Entity> {
    let mut visible = Vec::new();
    
    for (entity, position, _, unit) in all_units.iter() {
        if should_be_visible(observer_entity, observer_position, observer_unit, entity, position, unit, vision) {
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
    
    let distance = observer_position.translation.distance(target_position.translation);
    distance <= vision.sight_range
}