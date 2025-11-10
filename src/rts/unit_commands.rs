use bevy::prelude::*;
use crate::core::components::*;

pub struct CommandContext<'a> {
    pub camera: &'a Camera,
    pub camera_transform: &'a GlobalTransform,
    pub terrain_manager: &'a crate::world::terrain_v2::TerrainChunkManager,
    pub terrain_settings: &'a crate::world::terrain_v2::TerrainSettings,
}

pub fn unit_command_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut units: Query<(&mut Movement, &mut Combat, &Selectable, &RTSUnit), With<RTSUnit>>,
    all_units: Query<(&Transform, &RTSUnit), With<RTSUnit>>,
    resources: Query<&Transform, With<ResourceSource>>,
    terrain_manager: Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::world::terrain_v2::TerrainSettings>,
) {
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }
    
    let window = windows.single();
    let Some(cursor_position) = window.cursor_position() else { return };
    let (camera, camera_transform) = camera_q.single();
    
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else { return };
    
    let context = CommandContext {
        camera,
        camera_transform,
        terrain_manager: &terrain_manager,
        terrain_settings: &terrain_settings,
    };
    
    let target_enemy = find_enemy_target(ray, &units, &all_units);
    let target_resource = find_resource_target(ray, &resources);
    let target_point = calculate_target_position(ray, &context);
    
    if !is_valid_target_point(target_point) {
        warn!("Invalid target position calculated: {:?}, ignoring command", target_point);
        return;
    }
    
    execute_commands_for_selected_units(&mut units, &keyboard, target_enemy, target_resource, target_point);
}

fn find_enemy_target(
    ray: Ray3d,
    units: &Query<(&mut Movement, &mut Combat, &Selectable, &RTSUnit), With<RTSUnit>>,
    all_units: &Query<(&Transform, &RTSUnit), With<RTSUnit>>,
) -> Option<Entity> {
    let mut closest_distance = f32::INFINITY;
    let mut target_enemy = None;
    
    for (target_transform, target_unit) in all_units.iter() {
        let projected_distance = calculate_projected_distance(ray, target_transform.translation)?;
        
        if projected_distance >= closest_distance {
            continue;
        }
        
        let distance_to_ray = calculate_distance_to_ray(ray, target_transform.translation, projected_distance);
        
        if distance_to_ray < 8.0 && is_enemy_unit(target_unit, units) {
            closest_distance = projected_distance;
            target_enemy = find_entity_by_transform(target_transform, target_unit, all_units);
        }
    }
    
    target_enemy
}

fn calculate_projected_distance(ray: Ray3d, target_position: Vec3) -> Option<f32> {
    let to_target = target_position - ray.origin;
    let projected_distance = to_target.dot(ray.direction.normalize());
    
    if projected_distance > 0.0 {
        Some(projected_distance)
    } else {
        None
    }
}

fn calculate_distance_to_ray(ray: Ray3d, target_position: Vec3, projected_distance: f32) -> f32 {
    let closest_point = ray.origin + ray.direction.normalize() * projected_distance;
    closest_point.distance(target_position)
}

fn is_enemy_unit(
    target_unit: &RTSUnit,
    units: &Query<(&mut Movement, &mut Combat, &Selectable, &RTSUnit), With<RTSUnit>>,
) -> bool {
    for (_, _, selectable, unit) in units.iter() {
        if selectable.is_selected && unit.player_id != target_unit.player_id {
            return true;
        }
    }
    false
}

fn find_entity_by_transform(
    target_transform: &Transform,
    target_unit: &RTSUnit,
    all_units: &Query<(&Transform, &RTSUnit), With<RTSUnit>>,
) -> Option<Entity> {
    for (entity, (transform_comp, unit_comp)) in all_units.iter().enumerate() {
        if transform_comp.translation == target_transform.translation && 
           unit_comp.player_id == target_unit.player_id {
            return Some(Entity::from_raw(entity as u32));
        }
    }
    None
}

fn find_resource_target(ray: Ray3d, resources: &Query<&Transform, With<ResourceSource>>) -> Option<Vec3> {
    for resource_transform in resources.iter() {
        let projected_distance = calculate_projected_distance(ray, resource_transform.translation)?;
        let distance_to_ray = calculate_distance_to_ray(ray, resource_transform.translation, projected_distance);
        
        if distance_to_ray < 6.0 {
            return Some(resource_transform.translation);
        }
    }
    None
}

fn calculate_target_position(ray: Ray3d, context: &CommandContext) -> Vec3 {
    let horizontal_intersection = calculate_horizontal_intersection(ray);
    let terrain_height = sample_terrain_height_safe(horizontal_intersection, context);
    
    Vec3::new(
        horizontal_intersection.x,
        terrain_height + 2.0,
        horizontal_intersection.z,
    )
}

fn calculate_horizontal_intersection(ray: Ray3d) -> Vec3 {
    if ray.direction.y.abs() > 0.001 {
        let ground_y = 0.0;
        let t = (ground_y - ray.origin.y) / ray.direction.y;
        
        if t > 0.0 && t < 1000.0 {
            let intersection = ray.origin + ray.direction * t;
            return clamp_position(intersection);
        }
    }
    
    let horizontal_dir = Vec3::new(ray.direction.x, 0.0, ray.direction.z).normalize_or_zero();
    let offset = ray.origin + horizontal_dir * 50.0;
    clamp_position(offset)
}

fn clamp_position(position: Vec3) -> Vec3 {
    Vec3::new(
        position.x.clamp(-5000.0, 5000.0),
        position.y,
        position.z.clamp(-5000.0, 5000.0),
    )
}

fn sample_terrain_height_safe(position: Vec3, context: &CommandContext) -> f32 {
    if position.x.abs() < 10000.0 && position.z.abs() < 10000.0 {
        crate::world::terrain_v2::sample_terrain_height(
            position.x,
            position.z,
            &context.terrain_manager.noise_generator,
            context.terrain_settings,
        )
    } else {
        0.0
    }
}

fn is_valid_target_point(target_point: Vec3) -> bool {
    target_point.x.is_finite() && 
    target_point.z.is_finite() &&
    target_point.x.abs() < 10000.0 && 
    target_point.z.abs() < 10000.0
}

fn execute_commands_for_selected_units(
    units: &mut Query<(&mut Movement, &mut Combat, &Selectable, &RTSUnit), With<RTSUnit>>,
    keyboard: &Res<ButtonInput<KeyCode>>,
    target_enemy: Option<Entity>,
    target_resource: Option<Vec3>,
    target_point: Vec3,
) {
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Count selected units first (immutable borrow)
    let selected_count = units.iter()
        .filter(|(_, _, selectable, unit)| selectable.is_selected && unit.player_id == 1)
        .count();

    // Now do mutable iteration
    let mut index = 0;
    for (mut movement, mut combat, selectable, unit) in units.iter_mut() {
        if !selectable.is_selected || unit.player_id != 1 {
            continue;
        }

        // Calculate distributed position for resource gathering
        let adjusted_target = if target_resource.is_some() {
            calculate_gathering_position(target_point, index, selected_count)
        } else {
            target_point
        };

        execute_unit_command(&mut movement, &mut combat, unit, target_enemy, target_resource, adjusted_target, shift_held);
        index += 1;
    }
}

/// Calculate a position in a circle around the resource for better distribution
fn calculate_gathering_position(resource_pos: Vec3, unit_index: usize, total_units: usize) -> Vec3 {
    if total_units == 1 {
        return resource_pos;
    }

    let gather_radius = 4.0 + (total_units as f32 * 0.5); // Radius increases with more units
    let angle = (unit_index as f32 / total_units as f32) * std::f32::consts::TAU;

    Vec3::new(
        resource_pos.x + angle.cos() * gather_radius,
        resource_pos.y,
        resource_pos.z + angle.sin() * gather_radius,
    )
}

fn execute_unit_command(
    movement: &mut Movement,
    combat: &mut Combat,
    unit: &RTSUnit,
    target_enemy: Option<Entity>,
    target_resource: Option<Vec3>,
    target_point: Vec3,
    shift_held: bool,
) {
    if let Some(enemy_entity) = target_enemy {
        combat.target = Some(enemy_entity);
        movement.target_position = None;
        info!("🗡️ Unit {:?} attacking target {:?}!", unit.unit_id, enemy_entity);
    } else if target_resource.is_some() {
        movement.target_position = Some(target_point);
        info!("⛏️ Unit {:?} moving to gather resources at {:?}!", unit.unit_id, target_point);
    } else {
        movement.target_position = Some(target_point);
        combat.target = None;
        
        if shift_held {
            info!("📋 Unit {:?} queuing move command to {:?}!", unit.unit_id, target_point);
        } else {
            info!("🚶 Unit {:?} moving to position: {:?}", unit.unit_id, target_point);
        }
    }
}

pub fn spawn_test_units_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_q: Query<&Transform, With<crate::core::components::RTSCamera>>,
) {
    let Ok(camera_transform) = camera_q.get_single() else { return };
    
    let camera_ground_pos = Vec3::new(camera_transform.translation.x, 0.0, camera_transform.translation.z);
    
    if keyboard.just_pressed(KeyCode::KeyM) {
        spawn_test_unit(&mut commands, &mut meshes, &mut materials, camera_ground_pos, UnitType::SoldierAnt, 1, crate::constants::combat::UNIT_SPAWN_OFFSET);
    }
    
    if keyboard.just_pressed(KeyCode::KeyA) {
        spawn_test_unit(&mut commands, &mut meshes, &mut materials, camera_ground_pos, UnitType::HunterWasp, 1, -crate::constants::combat::UNIT_SPAWN_OFFSET);
    }
    
    if keyboard.just_pressed(KeyCode::KeyE) {
        let spawn_pos = camera_ground_pos + Vec3::new(0.0, 1.0, crate::constants::combat::UNIT_SPAWN_RANGE);
        
        // Use the new EntityFactory for enemy spawning
        let config = crate::entities::entity_factory::SpawnConfig::unit(
            crate::entities::entity_factory::EntityType::from_unit(UnitType::SoldierAnt),
            spawn_pos,
            2, // Player 2 (enemy)
        );
        
        crate::entities::entity_factory::EntityFactory::spawn(
            &mut commands,
            &mut meshes,
            &mut materials,
            config,
            None, // No model assets for now - will use primitives that upgrade to GLB
        );
        
        info!("Spawned Enemy with animation controller at {:?}", spawn_pos);
    }
}

fn spawn_test_unit(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    camera_ground_pos: Vec3,
    unit_type: UnitType,
    player_id: u8,
    offset_x: f32,
) {
    let spawn_pos = camera_ground_pos + Vec3::new(offset_x, 1.0, 0.0);
    
    // Use the new EntityFactory instead of the old create_combat_unit
    let config = crate::entities::entity_factory::SpawnConfig::unit(
        crate::entities::entity_factory::EntityType::from_unit(unit_type.clone()),
        spawn_pos,
        player_id,
    );
    
    crate::entities::entity_factory::EntityFactory::spawn(
        commands,
        meshes,
        materials,
        config,
        None, // No model assets for now - will use primitives that upgrade to GLB
    );
    
    info!("Spawned {:?} with animation controller at {:?}", unit_type, spawn_pos);
}