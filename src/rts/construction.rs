use bevy::prelude::*;
use crate::components::*;

pub fn construction_system(
    mut constructors: Query<(&mut Constructor, &Position), With<RTSUnit>>,
    mut buildings: Query<(Entity, &mut Building)>,
    time: Res<Time>,
) {
    for (mut constructor, _constructor_pos) in constructors.iter_mut() {
        process_construction_work(&mut constructor, &mut buildings, time.delta_secs());
    }
}

fn process_construction_work(
    constructor: &mut Constructor,
    buildings: &mut Query<(Entity, &mut Building)>,
    delta_time: f32,
) {
    let Some(target_entity) = constructor.current_target else { return };
    
    let Ok((_, mut building)) = buildings.get_mut(target_entity) else {
        constructor.current_target = None;
        return;
    };
    
    if building.is_complete {
        constructor.current_target = None;
        return;
    }
    
    building.construction_progress += constructor.build_speed * delta_time;
    
    if building.construction_progress >= building.max_construction {
        complete_building(&mut building, constructor);
    }
}

fn complete_building(building: &mut Building, constructor: &mut Constructor) {
    building.is_complete = true;
    constructor.current_target = None;
    info!("Construction completed!");
}