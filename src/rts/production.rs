use bevy::prelude::*;
use crate::core::components::*;

/// Context for production operations
pub struct ProductionContext<'a> {
    pub player_resources: &'a mut ResMut<'a, crate::core::resources::PlayerResources>,
    pub ai_resources: &'a mut ResMut<'a, crate::core::resources::AIResources>,
    pub game_costs: &'a Res<'a, crate::core::resources::GameCosts>,
    pub commands: &'a mut Commands,
    pub meshes: &'a mut ResMut<'a, Assets<Mesh>>,
    pub materials: &'a mut ResMut<'a, Assets<StandardMaterial>>,
}

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

    // Create production context
    let mut context = ProductionContext {
        player_resources: &mut player_resources,
        ai_resources: &mut ai_resources,
        game_costs: &game_costs,
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
    };

    for (mut queue, building, unit, transform) in buildings.iter_mut() {
        if building.is_complete && !queue.queue.is_empty() {
            process_production_queue(
                &mut queue,
                building,
                unit,
                transform,
                &mut context,
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
    context: &mut ProductionContext,
    delta_time: f32,
) {
    queue.current_progress += delta_time;

    if queue.current_progress < queue.production_time {
        return;
    }

    let unit_type = queue.queue[0].clone();

    if can_afford_production(unit.player_id, &unit_type, context) {
        complete_production(queue, building, unit, transform, unit_type, context);
    } else {
        handle_production_failure(unit.player_id);
    }
}

fn can_afford_production(
    player_id: u8,
    unit_type: &UnitType,
    context: &ProductionContext,
) -> bool {
    let Some(cost) = context.game_costs.unit_costs.get(unit_type) else {
        return true;
    };

    // Use direct access for read-only check (no mutation needed)
    if player_id == 1 {
        context.player_resources.can_afford(cost) && context.player_resources.has_population_space()
    } else {
        context.ai_resources.resources.get(&player_id)
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
    context: &mut ProductionContext,
) {
    queue.queue.remove(0);
    queue.current_progress = 0.0;

    pay_production_cost(unit.player_id, &unit_type, context);
    spawn_produced_unit(&unit_type, building, transform, unit.player_id, context);

    info!("Player {} produced {:?} unit", unit.player_id, unit_type);
}

fn pay_production_cost(
    player_id: u8,
    unit_type: &UnitType,
    context: &mut ProductionContext,
) {
    let Some(cost) = context.game_costs.unit_costs.get(unit_type) else {
        return;
    };

    let mut manager = crate::core::resources::ResourceManager::new(context.player_resources, context.ai_resources);
    manager.spend_resources(player_id, cost);
    manager.add_population(player_id, 1);
}

fn spawn_produced_unit(
    unit_type: &UnitType,
    building: &Building,
    building_transform: &Transform,
    player_id: u8,
    context: &mut ProductionContext,
) {
    use crate::entities::entity_factory::{EntityFactory, SpawnConfig, EntityType};

    // Spawn units next to the building, offset slightly to the side
    let default_offset = Vec3::new(8.0, 0.0, 8.0); // Spawn 8 units away from building
    let spawn_position = building.rally_point.unwrap_or(building_transform.translation + default_offset);

    let config = SpawnConfig::unit(EntityType::Unit(unit_type.clone()), spawn_position, player_id);
    EntityFactory::spawn(context.commands, context.meshes, context.materials, config, None);
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