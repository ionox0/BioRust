use bevy::prelude::*;
use crate::core::components::*;

pub fn production_system(
    mut buildings: Query<(&mut ProductionQueue, &Building, &RTSUnit, &Transform), With<Building>>,
    mut player_resources: ResMut<crate::core::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::core::resources::AIResources>,
    game_costs: Res<crate::core::resources::GameCosts>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (mut queue, building, unit, transform) in buildings.iter_mut() {
        if building.is_complete && !queue.queue.is_empty() {
            process_production_queue(
                &mut queue,
                building,
                unit,
                transform,
                &mut player_resources,
                &mut ai_resources,
                &game_costs,
                &mut commands,
                &mut meshes,
                &mut materials,
                delta_time
            );
        }
    }
}

fn process_production_queue(
    queue: &mut ProductionQueue,
    building: &Building,
    unit: &RTSUnit,
    transform: &Transform,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
    game_costs: &Res<crate::core::resources::GameCosts>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    delta_time: f32,
) {
    queue.current_progress += delta_time;

    if queue.current_progress < queue.production_time {
        return;
    }

    let unit_type = queue.queue[0].clone();

    if can_afford_production(unit.player_id, &unit_type, game_costs, player_resources, ai_resources) {
        complete_production(queue, building, unit, transform, unit_type, player_resources, ai_resources, game_costs, commands, meshes, materials);
    } else {
        handle_production_failure(unit.player_id);
    }
}

fn can_afford_production(
    player_id: u8, 
    unit_type: &UnitType, 
    game_costs: &Res<crate::core::resources::GameCosts>,
    player_resources: &ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &ResMut<crate::core::resources::AIResources>,
) -> bool {
    let Some(cost) = game_costs.unit_costs.get(unit_type) else {
        return true;
    };
    
    if player_id == 1 {
        player_resources.can_afford(cost) && player_resources.has_population_space()
    } else {
        ai_resources.resources.get(&player_id)
            .map(|resources| resources.can_afford(cost) && resources.has_population_space())
            .unwrap_or(true)
    }
}

fn complete_production(
    queue: &mut ProductionQueue,
    building: &Building,
    unit: &RTSUnit,
    transform: &Transform,
    unit_type: UnitType,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
    game_costs: &Res<crate::core::resources::GameCosts>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    queue.queue.remove(0);
    queue.current_progress = 0.0;

    pay_production_cost(unit.player_id, &unit_type, player_resources, ai_resources, game_costs);
    spawn_produced_unit(&unit_type, building, transform, unit.player_id, commands, meshes, materials);

    info!("Player {} produced {:?} unit", unit.player_id, unit_type);
}

fn pay_production_cost(
    player_id: u8, 
    unit_type: &UnitType, 
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
    game_costs: &Res<crate::core::resources::GameCosts>,
) {
    let Some(cost) = game_costs.unit_costs.get(unit_type) else {
        return;
    };
    
    if player_id == 1 {
        player_resources.spend_resources(cost);
        player_resources.add_population(1);
    } else if let Some(ai_player_resources) = ai_resources.resources.get_mut(&player_id) {
        ai_player_resources.spend_resources(cost);
        ai_player_resources.add_population(1);
    }
}

fn spawn_produced_unit(
    unit_type: &UnitType,
    building: &Building,
    building_transform: &Transform,
    player_id: u8,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Spawn units next to the building, offset slightly to the side
    let default_offset = Vec3::new(8.0, 0.0, 8.0); // Spawn 8 units away from building
    let spawn_position = building.rally_point.unwrap_or(building_transform.translation + default_offset);
    let unit_id = rand::random();
    
    use crate::entities::rts_entities::RTSEntityFactory;
    
    match unit_type {
        UnitType::WorkerAnt => {
            RTSEntityFactory::spawn_worker_ant(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        UnitType::SoldierAnt => {
            RTSEntityFactory::spawn_soldier_ant(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        UnitType::HunterWasp => {
            RTSEntityFactory::spawn_hunter_wasp(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        UnitType::BeetleKnight => {
            RTSEntityFactory::spawn_beetle_knight(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        UnitType::SpearMantis => {
            RTSEntityFactory::spawn_spear_mantis(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        UnitType::ScoutAnt => {
            RTSEntityFactory::spawn_scout_ant(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        UnitType::DragonFly => {
            RTSEntityFactory::spawn_dragonfly(
                commands,
                meshes,
                materials,
                spawn_position,
                player_id,
                unit_id,
            );
        },
        _ => {},
    }
}

fn handle_production_failure(player_id: u8) {
    if player_id == 1 {
        info!("Cannot produce unit: insufficient resources or population space");
    }
}

pub fn building_completion_system(
    mut buildings: Query<(&mut Building, &mut crate::core::components::RTSHealth), With<Building>>,
    time: Res<Time>,
) {
    for (mut building, mut health) in buildings.iter_mut() {
        if !building.is_complete {
            building.construction_progress += time.delta_secs();
            
            let completion_ratio = (building.construction_progress / building.max_construction).min(1.0);
            health.current = (health.max * completion_ratio);
            
            if building.construction_progress >= building.max_construction {
                building.is_complete = true;
                health.current = health.max;
                info!("Building completed!");
            }
        }
    }
}

pub fn population_management_system(
    mut player_resources: ResMut<crate::core::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::core::resources::AIResources>,
    units: Query<&RTSUnit, With<RTSUnit>>,
) {
    reset_population_counts(&mut player_resources, &mut ai_resources);
    count_active_units(&units, &mut player_resources, &mut ai_resources);
}

fn reset_population_counts(
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    player_resources.current_population = 0;
    
    for resources in ai_resources.resources.values_mut() {
        resources.current_population = 0;
    }
}

fn count_active_units(
    units: &Query<&RTSUnit, With<RTSUnit>>,
    player_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    ai_resources: &mut ResMut<crate::core::resources::AIResources>,
) {
    for unit in units.iter() {
        if unit.player_id == 1 {
            player_resources.current_population += 1;
        } else if let Some(ai_player_resources) = ai_resources.resources.get_mut(&unit.player_id) {
            ai_player_resources.current_population += 1;
        }
    }
}