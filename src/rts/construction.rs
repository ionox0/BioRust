use bevy::prelude::*;
use crate::core::components::*;

pub fn construction_system(
    mut constructors: Query<(&mut Constructor, &Position), With<RTSUnit>>,
    mut buildings: Query<(Entity, &mut Building)>,
    time: Res<Time>,
) {
    for (mut constructor, _constructor_pos) in constructors.iter_mut() {
        process_construction_work(&mut constructor, &mut buildings, time.delta_secs());
    }
}

/// System to handle AI construction workflow with worker movement
pub fn ai_construction_workflow_system(
    mut commands: Commands,
    mut workers: Query<(Entity, &mut Movement, &mut ConstructionTask, &Transform, &RTSUnit), With<ConstructionTask>>,
    mut building_sites: Query<(Entity, &mut BuildingSite), With<BuildingSite>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    
    for (worker_entity, _movement, mut task, worker_transform, unit) in workers.iter_mut() {
        if task.is_moving_to_site {
            // Check if worker has reached the construction site
            let distance = worker_transform.translation.distance(task.target_position);
            
            if distance <= 5.0 { // Close enough to start construction
                task.is_moving_to_site = false;
                
                // Mark the building site as construction started
                if let Ok((_, mut site)) = building_sites.get_mut(task.building_site) {
                    site.construction_started = true;
                    
                    info!("AI Worker {} reached construction site for {:?}", unit.unit_id, task.building_type);
                }
            }
        } else {
            // Worker is at site, perform construction work
            task.construction_progress += 50.0 * delta_time; // Construction speed
            
            // Check if construction is complete
            if task.construction_progress >= task.total_build_time {
                // Construction finished - spawn the actual building
                if let Ok((site_entity, site)) = building_sites.get(task.building_site) {
                    spawn_completed_building(
                        &mut commands, 
                        &mut meshes, 
                        &mut materials, 
                        &site, 
                        unit.player_id
                    );
                    
                    // Clean up the building site and task
                    commands.entity(site_entity).despawn();
                    commands.entity(worker_entity).remove::<ConstructionTask>();
                    
                    info!("AI Player {} completed construction of {:?}", unit.player_id, task.building_type);
                }
            }
        }
    }
}

/// Spawn the completed building at the construction site
fn spawn_completed_building(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    site: &BuildingSite,
    player_id: u8,
) {
    use crate::entities::entity_factory::{EntityFactory, SpawnConfig, EntityType};
    
    let building_config = SpawnConfig::building(
        EntityType::from_building(site.building_type.clone()),
        site.position,
        player_id,
    );
    
    EntityFactory::spawn(
        commands,
        meshes,
        materials,
        building_config,
        None, // Model assets will be found automatically
    );
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