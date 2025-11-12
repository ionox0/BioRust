use crate::core::components::*;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;

/// Resource to track current formation preference
#[derive(Resource, Default, Debug)]
pub struct FormationSettings {
    pub formation_type: FormationType,
    pub force_spread_formation: bool, // Force spread formation regardless of unit count
}

impl Default for FormationType {
    fn default() -> Self {
        FormationType::Circle // Default to circle formation
    }
}

pub struct CommandContext<'a> {
    pub terrain_manager: &'a crate::world::terrain_v2::TerrainChunkManager,
    pub terrain_settings: &'a crate::world::terrain_v2::TerrainSettings,
}

pub fn unit_command_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Movement,
            &mut Combat,
            &Selectable,
            &RTSUnit,
            Option<&mut ResourceGatherer>,
        ),
        With<RTSUnit>,
    >,
    all_units: Query<(Entity, &Transform, &RTSUnit), With<RTSUnit>>,
    resources: Query<(Entity, &Transform, &Selectable, &ResourceSource), With<ResourceSource>>,
    buildings: Query<(Entity, &Transform), With<Building>>,
    mut building_sites: ParamSet<(
        Query<(Entity, &Transform, &BuildingSite, &Selectable), With<BuildingSite>>,
        Query<&mut BuildingSite, With<BuildingSite>>,
    )>,
    terrain_manager: Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::world::terrain_v2::TerrainSettings>,
    mut commands: Commands,
    mut formation_settings: ResMut<FormationSettings>,
) {
    // Handle formation hotkeys (check before mouse button check so they work anytime)
    if keyboard.just_pressed(KeyCode::KeyJ) {
        formation_settings.force_spread_formation = !formation_settings.force_spread_formation;
        info!(
            "Formation mode: {}",
            if formation_settings.force_spread_formation {
                "Spread Formation (Wide Spacing)"
            } else {
                "Auto Formation (Standard Spacing)"
            }
        );
    }

    // Handle formation type cycling with K key
    if keyboard.just_pressed(KeyCode::KeyK) {
        formation_settings.formation_type = match formation_settings.formation_type {
            FormationType::Circle => FormationType::Box,
            FormationType::Box => FormationType::Line,
            FormationType::Line => FormationType::Wedge,
            FormationType::Wedge => FormationType::Spread,
            FormationType::Spread => FormationType::Circle,
        };
        info!(
            "Formation type changed to: {:?}",
            formation_settings.formation_type
        );
    }

    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    // Debug resource availability
    let resource_count = resources.iter().count();
    info!(
        "🖱️ Right-click detected! {} resources available in world",
        resource_count
    );

    let window = windows.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = camera_q.single();

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let context = CommandContext {
        terrain_manager: &terrain_manager,
        terrain_settings: &terrain_settings,
    };

    let target_enemy = find_enemy_target(ray, &units, &all_units);
    let target_resource = find_resource_target(ray, &resources);
    let target_building_site = find_building_site_target(ray, &building_sites.p0());

    info!(
        "🎯 Targets found - Enemy: {:?}, Resource: {:?}, BuildingSite: {:?}",
        target_enemy.is_some(),
        target_resource.is_some(),
        target_building_site.is_some()
    );

    let target_point = if let Some((_, resource_pos)) = target_resource {
        resource_pos
    } else if let Some((_, building_site_pos)) = target_building_site {
        building_site_pos
    } else {
        calculate_target_position(ray, &context)
    };

    debug!("🎯 Final target point: {:?}", target_point);
    debug!(
        "🎯 Ray origin: {:?}, direction: {:?}",
        ray.origin, ray.direction
    );

    if !is_valid_target_point(target_point) {
        warn!(
            "Invalid target position calculated: {:?}, ignoring command",
            target_point
        );
        return;
    }

    execute_commands_for_selected_units(
        &mut units,
        &buildings,
        &mut building_sites,
        &resources,
        &mut commands,
        &keyboard,
        target_enemy,
        target_resource,
        target_building_site,
        target_point,
    );
}

fn find_enemy_target(
    ray: Ray3d,
    units: &Query<
        (
            Entity,
            &Transform,
            &mut Movement,
            &mut Combat,
            &Selectable,
            &RTSUnit,
            Option<&mut ResourceGatherer>,
        ),
        With<RTSUnit>,
    >,
    all_units: &Query<(Entity, &Transform, &RTSUnit), With<RTSUnit>>,
) -> Option<Entity> {
    let mut closest_distance = f32::INFINITY;
    let mut target_enemy = None;

    for (entity, target_transform, target_unit) in all_units.iter() {
        let projected_distance = calculate_projected_distance(ray, target_transform.translation)?;

        if projected_distance >= closest_distance {
            continue;
        }

        let distance_to_ray =
            calculate_distance_to_ray(ray, target_transform.translation, projected_distance);

        if distance_to_ray < 8.0 && is_enemy_unit(target_unit, units) {
            closest_distance = projected_distance;
            target_enemy = Some(entity); // Use entity directly instead of searching
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
    units: &Query<
        (
            Entity,
            &Transform,
            &mut Movement,
            &mut Combat,
            &Selectable,
            &RTSUnit,
            Option<&mut ResourceGatherer>,
        ),
        With<RTSUnit>,
    >,
) -> bool {
    for (_, _, _, _, selectable, unit, _) in units.iter() {
        if selectable.is_selected && unit.player_id != target_unit.player_id {
            return true;
        }
    }
    false
}

fn find_resource_target(
    ray: Ray3d,
    resources: &Query<(Entity, &Transform, &Selectable, &ResourceSource), With<ResourceSource>>,
) -> Option<(Entity, Vec3)> {
    let resource_count = resources.iter().count();
    info!(
        "🎯 Checking {} resources for click detection",
        resource_count
    );

    for (resource_entity, resource_transform, selectable, _resource_source) in resources.iter() {
        let projected_distance_opt =
            calculate_projected_distance(ray, resource_transform.translation);
        if let Some(projected_distance) = projected_distance_opt {
            let distance_to_ray =
                calculate_distance_to_ray(ray, resource_transform.translation, projected_distance);
            info!(
                "🎯 Resource {:?} at {:?} - distance_to_ray: {:.1}, selection_radius: {:.1}",
                resource_entity,
                resource_transform.translation,
                distance_to_ray,
                selectable.selection_radius
            );

            if distance_to_ray < selectable.selection_radius {
                info!(
                    "✅ Found resource target! Entity {:?} at distance {:.1}",
                    resource_entity, distance_to_ray
                );
                return Some((resource_entity, resource_transform.translation));
            }
        } else {
            info!(
                "🎯 Resource {:?} - projected distance is None (behind camera)",
                resource_entity
            );
        }
    }
    info!("❌ No resource found within selection radius");
    None
}

fn find_building_site_target(
    ray: Ray3d,
    building_sites: &Query<(Entity, &Transform, &BuildingSite, &Selectable), With<BuildingSite>>,
) -> Option<(Entity, Vec3)> {
    for (entity, transform, site, selectable) in building_sites.iter() {
        // Only allow player 1 to target player 1 building sites
        if site.player_id != 1 {
            continue;
        }

        let projected_distance = calculate_projected_distance(ray, transform.translation)?;
        let distance_to_ray =
            calculate_distance_to_ray(ray, transform.translation, projected_distance);

        if distance_to_ray < selectable.selection_radius {
            return Some((entity, transform.translation));
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
    // More robust ground intersection calculation
    if ray.direction.y.abs() > 0.0001 {
        // Smaller threshold for better precision
        let ground_y = 0.0;
        let t = (ground_y - ray.origin.y) / ray.direction.y;

        // Remove the upper limit check that was causing issues with zoomed out cameras
        if t > 0.0 {
            let intersection = ray.origin + ray.direction * t;
            debug!(
                "🎯 Ground intersection calculated: {:?} (t={:.2})",
                intersection, t
            );
            return clamp_position(intersection);
        }
    }

    // Fallback: project ray horizontally from camera position
    // Use a much larger projection distance for zoomed out cameras
    let horizontal_dir = Vec3::new(ray.direction.x, 0.0, ray.direction.z).normalize_or_zero();
    let projection_distance = ray.origin.y.abs() * 2.0; // Scale with camera height
    let offset = ray.origin + horizontal_dir * projection_distance;
    debug!(
        "🎯 Fallback projection used: {:?} (distance={:.2})",
        offset, projection_distance
    );
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
    target_point.x.is_finite()
        && target_point.z.is_finite()
        && target_point.x.abs() < 10000.0
        && target_point.z.abs() < 10000.0
}

fn execute_commands_for_selected_units(
    units: &mut Query<
        (
            Entity,
            &Transform,
            &mut Movement,
            &mut Combat,
            &Selectable,
            &RTSUnit,
            Option<&mut ResourceGatherer>,
        ),
        With<RTSUnit>,
    >,
    buildings: &Query<(Entity, &Transform), With<Building>>,
    building_sites: &mut ParamSet<(
        Query<(Entity, &Transform, &BuildingSite, &Selectable), With<BuildingSite>>,
        Query<&mut BuildingSite, With<BuildingSite>>,
    )>,
    resources: &Query<(Entity, &Transform, &Selectable, &ResourceSource), With<ResourceSource>>,
    commands: &mut Commands,
    keyboard: &Res<ButtonInput<KeyCode>>,
    target_enemy: Option<Entity>,
    target_resource: Option<(Entity, Vec3)>,
    target_building_site: Option<(Entity, Vec3)>,
    target_point: Vec3,
) {
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Special handling for resource gathering with multiple workers
    if let Some((clicked_resource_entity, _)) = target_resource {
        // Check if we have multiple workers selected
        let selected_workers: Vec<_> = units
            .iter()
            .filter(|(_, _, _, _, selectable, unit, gatherer)| {
                selectable.is_selected 
                && unit.player_id == 1 
                && (gatherer.is_some() || matches!(unit.unit_type, Some(UnitType::WorkerAnt)))
            })
            .map(|(entity, transform, _, _, _, unit, _)| (entity, transform.translation, unit.unit_id))
            .collect();

        if selected_workers.len() > 1 {
            // Multiple workers - distribute them across available resources
            distribute_workers_across_resources(
                selected_workers,
                clicked_resource_entity,
                target_point,
                units,
                buildings,
                resources,
                commands,
            );
            return;
        }
    }

    // Count selected units first (immutable borrow)
    let selected_count = units
        .iter()
        .filter(|(_, _, _, _, selectable, unit, _)| selectable.is_selected && unit.player_id == 1)
        .count();

    // Now do mutable iteration
    let mut index = 0;
    for (unit_entity, transform, mut movement, mut combat, selectable, unit, gatherer) in
        units.iter_mut()
    {
        if !selectable.is_selected || unit.player_id != 1 {
            continue;
        }

        // Calculate distributed position for all move commands to prevent bunching
        let adjusted_target = if selected_count > 1 {
            // Spread units in a formation when multiple units are selected
            calculate_formation_position(target_point, index, selected_count)
        } else {
            target_point
        };

        // Handle building site command separately to avoid ParamSet borrowing conflicts
        if let Some((site_entity, _site_pos)) = target_building_site {
            handle_building_site_command(
                unit_entity,
                unit,
                gatherer,
                &mut movement,
                &mut combat,
                building_sites,
                commands,
                site_entity,
                adjusted_target,
            );
        } else {
            execute_unit_command(
                unit_entity,
                transform.translation,
                &mut movement,
                &mut combat,
                gatherer,
                unit,
                buildings,
                commands,
                target_enemy,
                target_resource,
                adjusted_target,
                shift_held,
            );
        }
        index += 1;
    }
}

/// Calculate a formation position for units to spread out and avoid bunching
fn calculate_formation_position(target_pos: Vec3, unit_index: usize, total_units: usize) -> Vec3 {
    if total_units == 1 {
        return target_pos;
    }

    // Use spread formation for larger groups (automatically activates for 6+ units)
    if total_units >= 6 {
        calculate_spread_formation_position(target_pos, unit_index, total_units)
    } else {
        calculate_compact_formation_position(target_pos, unit_index, total_units)
    }
}

/// Calculate a compact formation position for small groups
fn calculate_compact_formation_position(
    target_pos: Vec3,
    unit_index: usize,
    _total_units: usize,
) -> Vec3 {
    // Spread units in concentric circles with standard spacing
    let units_per_ring = 8;
    let ring = unit_index / units_per_ring;
    let pos_in_ring = unit_index % units_per_ring;

    let formation_radius = 3.0 + (ring as f32 * 4.0); // Each ring is 4 units further out
    let angle = (pos_in_ring as f32 / units_per_ring as f32) * std::f32::consts::TAU;

    Vec3::new(
        target_pos.x + angle.cos() * formation_radius,
        target_pos.y,
        target_pos.z + angle.sin() * formation_radius,
    )
}

/// Calculate a spread formation position with much greater spacing for large groups
fn calculate_spread_formation_position(
    target_pos: Vec3,
    unit_index: usize,
    total_units: usize,
) -> Vec3 {
    // Use fewer units per ring but much larger spacing for better tactical positioning
    let units_per_ring = 6; // Fewer units per ring for better spacing
    let ring = unit_index / units_per_ring;
    let pos_in_ring = unit_index % units_per_ring;

    // Much larger base radius and ring spacing for spread formation
    let base_radius = 8.0; // Start further from center
    let ring_spacing = 12.0; // Much larger gaps between rings
    let formation_radius = base_radius + (ring as f32 * ring_spacing);

    // Add slight randomization to break up perfect geometric patterns
    let angle_variation = (unit_index as f32 * 0.73).sin() * 0.2; // Small variation for natural look
    let angle =
        (pos_in_ring as f32 / units_per_ring as f32) * std::f32::consts::TAU + angle_variation;

    // Apply additional spacing based on total unit count for very large groups
    let size_scaling = if total_units > 15 { 1.5 } else { 1.0 };

    Vec3::new(
        target_pos.x + angle.cos() * formation_radius * size_scaling,
        target_pos.y,
        target_pos.z + angle.sin() * formation_radius * size_scaling,
    )
}

fn execute_unit_command(
    unit_entity: Entity,
    unit_pos: Vec3,
    movement: &mut Movement,
    combat: &mut Combat,
    gatherer: Option<Mut<ResourceGatherer>>,
    unit: &RTSUnit,
    buildings: &Query<(Entity, &Transform), With<Building>>,
    commands: &mut Commands,
    target_enemy: Option<Entity>,
    target_resource: Option<(Entity, Vec3)>,
    target_point: Vec3,
    _shift_held: bool,
) {
    if let Some(enemy_entity) = target_enemy {
        // Attack command - enable auto_attack and set target
        combat.target = Some(enemy_entity);
        combat.auto_attack = true; // Enable auto_attack when given attack command
        movement.target_position = None;
        info!(
            "🗡️ Unit {:?} attacking target {:?}!",
            unit.unit_id, enemy_entity
        );
    } else if let Some((resource_entity, _resource_transform)) = target_resource {
        // Resource gathering command
        if let Some(mut resource_gatherer) = gatherer {
            // Unit already has ResourceGatherer component
            let nearest_building = find_nearest_building(unit.player_id, unit_pos, buildings);

            resource_gatherer.target_resource = Some(resource_entity);
            resource_gatherer.drop_off_building = nearest_building;
            movement.target_position = Some(target_point);
            combat.target = None;
            combat.auto_attack = false; // Disable auto_attack for non-combat commands
            combat.is_attacking = false;
            combat.last_attack_time = 0.0;

            info!(
                "⛏️ Worker {:?} assigned to gather from resource {:?}!",
                unit.unit_id, resource_entity
            );
        } else if matches!(unit.unit_type, Some(UnitType::WorkerAnt)) {
            // WorkerAnt without ResourceGatherer - add the component
            let nearest_building = find_nearest_building(unit.player_id, unit_pos, buildings);

            commands.entity(unit_entity).insert(ResourceGatherer {
                gather_rate: 10.0,
                capacity: 5.0,
                carried_amount: 0.0,
                resource_type: None,
                target_resource: Some(resource_entity),
                drop_off_building: nearest_building,
            });

            movement.target_position = Some(target_point);
            combat.target = None;
            combat.auto_attack = false; // Disable auto_attack for non-combat commands
            combat.is_attacking = false;
            combat.last_attack_time = 0.0;

            info!("⛏️ Added ResourceGatherer to Worker {:?} and assigned to gather from resource {:?}!", unit.unit_id, resource_entity);
        } else {
            // Not a worker, just move to location
            movement.target_position = Some(target_point);
            combat.target = None;
            combat.auto_attack = false; // Disable auto_attack for non-combat commands
            combat.is_attacking = false;
            combat.last_attack_time = 0.0;
            info!(
                "🚶 Unit {:?} moving to resource location: {:?}",
                unit.unit_id, target_point
            );
        }
    } else {
        // Simple move command - force clear combat state for player units
        movement.target_position = Some(target_point);
        combat.target = None;
        combat.auto_attack = false; // Disable auto_attack for move commands
        combat.is_attacking = false; // Also clear attacking state
        combat.last_attack_time = 0.0; // Reset attack timing
        info!(
            "🚶 Unit {:?} moving to position: {:?}",
            unit.unit_id, target_point
        );
    }
}

/// Handle building site construction command
fn handle_building_site_command(
    unit_entity: Entity,
    unit: &RTSUnit,
    gatherer: Option<Mut<ResourceGatherer>>,
    movement: &mut Movement,
    combat: &mut Combat,
    building_sites: &mut ParamSet<(
        Query<(Entity, &Transform, &BuildingSite, &Selectable), With<BuildingSite>>,
        Query<&mut BuildingSite, With<BuildingSite>>,
    )>,
    commands: &mut Commands,
    site_entity: Entity,
    target_point: Vec3,
) {
    if !can_construct_building(unit, gatherer.is_some()) {
        handle_non_constructor_unit(movement, combat, unit, target_point);
        return;
    }

    let Some(site_info) = get_building_site_info(building_sites, site_entity) else {
        handle_missing_building_site(movement, combat, target_point, site_entity);
        return;
    };

    if try_assign_worker_to_site(building_sites, site_entity, unit_entity) {
        assign_construction_task(
            commands,
            unit_entity,
            site_entity,
            site_info,
            movement,
            combat,
            unit,
        );
    } else {
        handle_site_already_assigned(movement, combat, target_point);
    }
}

/// Distribute multiple workers across available resources near the clicked resource
fn distribute_workers_across_resources(
    selected_workers: Vec<(Entity, Vec3, u32)>,
    clicked_resource_entity: Entity,
    clicked_position: Vec3,
    units: &mut Query<
        (
            Entity,
            &Transform,
            &mut Movement,
            &mut Combat,
            &Selectable,
            &RTSUnit,
            Option<&mut ResourceGatherer>,
        ),
        With<RTSUnit>,
    >,
    buildings: &Query<(Entity, &Transform), With<Building>>,
    resources: &Query<(Entity, &Transform, &Selectable, &ResourceSource), With<ResourceSource>>,
    commands: &mut Commands,
) {
    // Find available resources near the clicked one
    let search_radius = 200.0; // Search within 200 units for resources to distribute workers
    let mut available_resources = Vec::new();

    // First add the clicked resource
    if let Ok((clicked_entity, clicked_transform, _, clicked_source)) = resources.get(clicked_resource_entity) {
        available_resources.push((clicked_entity, clicked_transform.translation, clicked_source.current_gatherers, clicked_source.max_gatherers));
    }

    // Find nearby resources
    for (resource_entity, resource_transform, _, resource_source) in resources.iter() {
        if resource_entity != clicked_resource_entity {
            let distance = clicked_position.distance(resource_transform.translation);
            if distance <= search_radius && resource_source.amount > 0.0 {
                available_resources.push((resource_entity, resource_transform.translation, resource_source.current_gatherers, resource_source.max_gatherers));
            }
        }
    }

    // Sort resources by distance from clicked position
    available_resources.sort_by(|a, b| {
        let dist_a = clicked_position.distance(a.1);
        let dist_b = clicked_position.distance(b.1);
        dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    let worker_count = selected_workers.len();
    info!("🔀 Distributing {} workers across {} available resources", worker_count, available_resources.len());

    // Assign workers to resources in round-robin fashion, respecting max_gatherers
    let mut resource_index = 0;
    let mut assigned_count = 0;

    for (worker_entity, worker_pos, worker_id) in selected_workers {
        // Find next available resource
        let mut attempts = 0;
        while attempts < available_resources.len() {
            if let Some((resource_entity, resource_pos, current_gatherers, max_gatherers)) = available_resources.get_mut(resource_index) {
                if *current_gatherers < *max_gatherers {
                    // Assign this worker to this resource
                    assign_worker_to_resource(
                        worker_entity,
                        worker_pos,
                        worker_id,
                        *resource_entity,
                        *resource_pos,
                        units,
                        buildings,
                        commands,
                    );
                    
                    *current_gatherers += 1; // Track assignment locally
                    assigned_count += 1;
                    break;
                }
            }
            
            resource_index = (resource_index + 1) % available_resources.len();
            attempts += 1;
        }
        
        resource_index = (resource_index + 1) % available_resources.len();
    }

    info!("✅ Successfully assigned {} out of {} workers to resources", assigned_count, worker_count);
}

/// Assign a specific worker to a specific resource
fn assign_worker_to_resource(
    worker_entity: Entity,
    worker_pos: Vec3,
    worker_id: u32,
    resource_entity: Entity,
    resource_pos: Vec3,
    units: &mut Query<
        (
            Entity,
            &Transform,
            &mut Movement,
            &mut Combat,
            &Selectable,
            &RTSUnit,
            Option<&mut ResourceGatherer>,
        ),
        With<RTSUnit>,
    >,
    buildings: &Query<(Entity, &Transform), With<Building>>,
    commands: &mut Commands,
) {
    // Find the worker in the units query and assign it
    if let Ok((_, _, mut movement, mut combat, _, unit, gatherer)) = units.get_mut(worker_entity) {
        let nearest_building = find_nearest_building(unit.player_id, worker_pos, buildings);

        if let Some(mut resource_gatherer) = gatherer {
            // Unit already has ResourceGatherer component
            resource_gatherer.target_resource = Some(resource_entity);
            resource_gatherer.drop_off_building = nearest_building;
            movement.target_position = Some(resource_pos);
            combat.target = None;
            combat.auto_attack = false; // Disable auto_attack for non-combat commands
            combat.is_attacking = false;
            combat.last_attack_time = 0.0;

            info!("⛏️ Worker {} assigned to resource {:?}", worker_id, resource_entity);
        } else if matches!(unit.unit_type, Some(UnitType::WorkerAnt)) {
            // WorkerAnt without ResourceGatherer - add the component
            commands.entity(worker_entity).insert(ResourceGatherer {
                gather_rate: 10.0,
                capacity: 5.0,
                carried_amount: 0.0,
                resource_type: None,
                target_resource: Some(resource_entity),
                drop_off_building: nearest_building,
            });

            movement.target_position = Some(resource_pos);
            combat.target = None;
            combat.auto_attack = false; // Disable auto_attack for non-combat commands
            combat.is_attacking = false;
            combat.last_attack_time = 0.0;

            info!("⛏️ Added ResourceGatherer to Worker {} and assigned to resource {:?}", worker_id, resource_entity);
        }
    }
}

/// Find the nearest building for a given player
fn find_nearest_building(
    _player_id: u8,
    unit_pos: Vec3,
    buildings: &Query<(Entity, &Transform), With<Building>>,
) -> Option<Entity> {
    let mut nearest_building = None;
    let mut nearest_distance = f32::INFINITY;

    for (building_entity, building_transform) in buildings.iter() {
        // TODO: check if building belongs to player
        let distance = unit_pos.distance(building_transform.translation);
        if distance < nearest_distance {
            nearest_distance = distance;
            nearest_building = Some(building_entity);
        }
    }

    nearest_building
}

struct BuildingSiteInfo {
    building_type: BuildingType,
    position: Vec3,
}

fn can_construct_building(unit: &RTSUnit, has_gatherer: bool) -> bool {
    unit.player_id == 1 && has_gatherer
}

fn handle_non_constructor_unit(
    movement: &mut Movement,
    combat: &mut Combat,
    unit: &RTSUnit,
    target_point: Vec3,
) {
    movement.target_position = Some(target_point);
    combat.target = None;
    info!(
        "🚶 Unit {:?} moving to construction site: {:?}",
        unit.unit_id, target_point
    );
}

fn get_building_site_info(
    building_sites: &mut ParamSet<(
        Query<(Entity, &Transform, &BuildingSite, &Selectable), With<BuildingSite>>,
        Query<&mut BuildingSite, With<BuildingSite>>,
    )>,
    site_entity: Entity,
) -> Option<BuildingSiteInfo> {
    if let Ok((_, _, site, _)) = building_sites.p0().get(site_entity) {
        Some(BuildingSiteInfo {
            building_type: site.building_type.clone(),
            position: site.position,
        })
    } else {
        None
    }
}

fn handle_missing_building_site(
    movement: &mut Movement,
    combat: &mut Combat,
    target_point: Vec3,
    site_entity: Entity,
) {
    movement.target_position = Some(target_point);
    combat.target = None;
    warn!("Building site not found for entity {:?}", site_entity);
}

fn try_assign_worker_to_site(
    building_sites: &mut ParamSet<(
        Query<(Entity, &Transform, &BuildingSite, &Selectable), With<BuildingSite>>,
        Query<&mut BuildingSite, With<BuildingSite>>,
    )>,
    site_entity: Entity,
    unit_entity: Entity,
) -> bool {
    if let Ok(mut site_mut) = building_sites.p1().get_mut(site_entity) {
        if site_mut.assigned_worker.is_none() {
            site_mut.assigned_worker = Some(unit_entity);
            site_mut.site_reserved = true;
            return true;
        }
    }
    false
}

fn assign_construction_task(
    commands: &mut Commands,
    unit_entity: Entity,
    site_entity: Entity,
    site_info: BuildingSiteInfo,
    movement: &mut Movement,
    combat: &mut Combat,
    unit: &RTSUnit,
) {
    let total_build_time = get_construction_time_for_building(&site_info.building_type);

    commands.entity(unit_entity).insert(ConstructionTask {
        building_site: site_entity,
        building_type: site_info.building_type.clone(),
        target_position: site_info.position,
        is_moving_to_site: true,
        construction_progress: 0.0,
        total_build_time,
    });

    movement.target_position = Some(site_info.position);
    combat.target = None;

    info!(
        "🔨 Player 1 worker {:?} assigned to construct {:?} at {:?}!",
        unit.unit_id, site_info.building_type, site_info.position
    );
}

fn get_construction_time_for_building(building_type: &BuildingType) -> f32 {
    match building_type {
        BuildingType::Queen => 120.0,
        BuildingType::Nursery => 80.0,
        BuildingType::WarriorChamber => 100.0,
        BuildingType::HunterChamber => 100.0,
        BuildingType::FungalGarden => 90.0,
        BuildingType::StorageChamber => 60.0,
        _ => 100.0,
    }
}

fn handle_site_already_assigned(movement: &mut Movement, combat: &mut Combat, target_point: Vec3) {
    movement.target_position = Some(target_point);
    combat.target = None;
    info!("⚠️ Building site already has worker assigned, moving to location instead");
}

pub fn spawn_test_units_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_q: Query<&Transform, With<crate::core::components::RTSCamera>>,
) {
    let Ok(camera_transform) = camera_q.get_single() else {
        return;
    };

    let camera_ground_pos = Vec3::new(
        camera_transform.translation.x,
        0.0,
        camera_transform.translation.z,
    );

    // Removed KeyCode::KeyM and KeyCode::KeyA spawn shortcuts to prevent random unit spawning

    if keyboard.just_pressed(KeyCode::KeyE) {
        let spawn_pos =
            camera_ground_pos + Vec3::new(0.0, 1.0, crate::constants::combat::UNIT_SPAWN_RANGE);

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

        info!("Spawned Enemy SoldierAnt at {:?}", spawn_pos);
    }

    // Test DragonFly speed with KeyCode::KeyD
    if keyboard.just_pressed(KeyCode::KeyT) {
        let spawn_pos = camera_ground_pos + Vec3::new(15.0, 1.0, 0.0);

        let config = crate::entities::entity_factory::SpawnConfig::unit(
            crate::entities::entity_factory::EntityType::from_unit(UnitType::DragonFly),
            spawn_pos,
            1, // Player 1 (friendly)
        );

        crate::entities::entity_factory::EntityFactory::spawn(
            &mut commands,
            &mut meshes,
            &mut materials,
            config,
            None,
        );

        info!(
            "Spawned Player 1 DragonFly at {:?} - max_speed should be 400.0",
            spawn_pos
        );
    }

    // Test production queue overflow - spawn a second Queen building for AI
    if keyboard.just_pressed(KeyCode::KeyQ) {
        let spawn_pos = Vec3::new(700.0, 0.0, 50.0); // Near AI base

        let config = crate::entities::entity_factory::SpawnConfig::building(
            crate::entities::entity_factory::EntityType::Building(
                crate::core::components::BuildingType::Queen,
            ),
            spawn_pos,
            2, // Player 2 (AI)
        );

        let entity = crate::entities::entity_factory::EntityFactory::spawn(
            &mut commands,
            &mut meshes,
            &mut materials,
            config,
            None,
        );

        // Make the building immediately complete for testing
        commands
            .entity(entity)
            .insert(crate::core::components::Building {
                building_type: crate::core::components::BuildingType::Queen,
                is_complete: true,
                construction_progress: 100.0,
                max_construction: 100.0,
                rally_point: None,
            });

        info!(
            "Spawned second Queen building for AI Player 2 at {:?} to test overflow (completed)",
            spawn_pos
        );
    }
}
