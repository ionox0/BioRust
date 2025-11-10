use bevy::prelude::*;
use crate::components::*;

pub struct RTSSystemsPlugin;

impl Plugin for RTSSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            movement_system,
            resource_gathering_system,
            drag_selection_system,
            production_system,
            construction_system,
            formation_system,
            vision_system,
            unit_command_system,
            selection_indicator_system,
            spawn_test_units_system,
            building_completion_system,
            population_management_system,
        ));
    }
}

pub fn movement_system(
    mut units_query: Query<(Entity, &mut Transform, &mut Movement, &CollisionRadius), With<RTSUnit>>,
    terrain_manager: Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::terrain_v2::TerrainSettings>,
    time: Res<Time>,
) {
    use crate::constants::movement::*;
    
    // Collect all unit positions and collision radii for collision detection
    let unit_positions: Vec<(Entity, Vec3, f32)> = units_query
        .iter()
        .map(|(entity, transform, _, collision_radius)| (entity, transform.translation, collision_radius.radius))
        .collect();

    for (entity, mut transform, mut movement, collision_radius) in units_query.iter_mut() {
        let current_pos = transform.translation;
        
        // Safety check: Reset units that have gone to astronomical positions
        if current_pos.x.abs() > MAX_POSITION || current_pos.z.abs() > MAX_POSITION || 
           !current_pos.x.is_finite() || !current_pos.z.is_finite() {
            warn!("Unit at extreme position {:?}, resetting to origin", current_pos);
            transform.translation = Vec3::new(0.0, DEFAULT_SPAWN_HEIGHT, 0.0);
            movement.current_velocity = Vec3::ZERO;
            movement.target_position = None;
            continue;
        }
        
        if let Some(target) = movement.target_position {
            // Validate target position
            if target.x.abs() > MAX_POSITION || target.z.abs() > MAX_POSITION || 
               !target.x.is_finite() || !target.z.is_finite() {
                warn!("Invalid target position {:?}, clearing target", target);
                movement.target_position = None;
                continue;
            }
            
            let direction = (target - current_pos).normalize_or_zero();
            let distance = current_pos.distance(target);
            
            // Validate distance calculation
            if !distance.is_finite() || distance > MAX_DISTANCE {
                warn!("Invalid distance {:.1} from {:?} to {:?}, clearing target", distance, current_pos, target);
                movement.target_position = None;
                continue;
            }
            
            if distance > DECELERATION_FACTOR {
                let target_velocity = direction * movement.max_speed;
                
                // Clamp velocity to prevent runaway units
                let clamped_velocity = Vec3::new(
                    target_velocity.x.clamp(-MAX_VELOCITY, MAX_VELOCITY),
                    target_velocity.y.clamp(-MAX_VELOCITY, MAX_VELOCITY),
                    target_velocity.z.clamp(-MAX_VELOCITY, MAX_VELOCITY),
                );
                
                // Smooth acceleration with reduced jerkiness
                let acceleration_factor = (movement.acceleration * 0.1 * time.delta_secs()).min(0.3); // Cap acceleration rate
                movement.current_velocity = movement.current_velocity.lerp(
                    clamped_velocity,
                    acceleration_factor
                );
                
                // Clamp current velocity as additional safety
                movement.current_velocity = Vec3::new(
                    movement.current_velocity.x.clamp(-MAX_VELOCITY, MAX_VELOCITY),
                    movement.current_velocity.y.clamp(-MAX_VELOCITY, MAX_VELOCITY),
                    movement.current_velocity.z.clamp(-MAX_VELOCITY, MAX_VELOCITY),
                );
                
                // Debug movement occasionally
                if rand::random::<f32>() < 0.01 { // 1% chance per frame
                    info!("Unit moving: pos={:.1},{:.1} target={:.1},{:.1} dist={:.1} vel={:.1},{:.1}", 
                          current_pos.x, current_pos.z, target.x, target.z, distance, 
                          movement.current_velocity.x, movement.current_velocity.z);
                }
                
                let delta_time = time.delta_secs().min(0.1); // Cap delta time
                let mut new_position = current_pos + movement.current_velocity * delta_time;
                
                // Validate new position before applying
                if new_position.x.abs() > MAX_POSITION || new_position.z.abs() > MAX_POSITION || 
                   !new_position.x.is_finite() || !new_position.z.is_finite() {
                    warn!("New position {:?} would be invalid, stopping movement", new_position);
                    movement.current_velocity = Vec3::ZERO;
                    movement.target_position = None;
                    continue;
                }
                
                // Gentle collision detection and separation
                let mut separation_force = Vec3::ZERO;
                let separation_radius = collision_radius.radius * SEPARATION_MULTIPLIER;
                let mut has_collision = false;
                
                for &(other_entity, other_position, other_radius) in &unit_positions {
                    if entity != other_entity {
                        let to_other = other_position - new_position;
                        let distance = to_other.length();
                        let min_distance = collision_radius.radius + other_radius + 0.2; // Smaller buffer to prevent over-separation
                        
                        if distance < separation_radius && distance > 0.001 {
                            // Gentle separation force - only push away gradually
                            let separation_strength = ((separation_radius - distance) / separation_radius).powf(2.0); // Quadratic falloff for smoother transition
                            let separation_direction = -to_other.normalize_or_zero();
                            separation_force += separation_direction * separation_strength * SEPARATION_FORCE_STRENGTH;
                        }
                        
                        // Soft collision prevention - reduce velocity instead of hard repositioning
                        if distance < min_distance && distance > 0.001 {
                            has_collision = true;
                            // Reduce velocity towards collision direction
                            let collision_direction = to_other.normalize_or_zero();
                            let velocity_towards_collision = movement.current_velocity.dot(collision_direction);
                            if velocity_towards_collision > 0.0 {
                                movement.current_velocity -= collision_direction * velocity_towards_collision * 0.8;
                            }
                        }
                    }
                }
                
                // Apply gentle separation force to velocity
                if separation_force.length() > 0.001 {
                    // Scale separation force based on current velocity to prevent jiggling
                    let velocity_factor = (1.0 - movement.current_velocity.length() / movement.max_speed).max(0.2);
                    movement.current_velocity += separation_force * delta_time * velocity_factor;
                }
                
                // Dampen velocity if there are collisions to prevent jiggling
                if has_collision {
                    movement.current_velocity *= 0.7;
                }
                
                movement.current_velocity = Vec3::new(
                    movement.current_velocity.x.clamp(-MAX_VELOCITY, MAX_VELOCITY),
                    movement.current_velocity.y.clamp(-MAX_VELOCITY, MAX_VELOCITY),
                    movement.current_velocity.z.clamp(-MAX_VELOCITY, MAX_VELOCITY),
                );
                
                // Sample terrain height safely
                let terrain_height = if new_position.x.abs() < TERRAIN_SAMPLE_LIMIT && new_position.z.abs() < TERRAIN_SAMPLE_LIMIT {
                    crate::terrain_v2::sample_terrain_height(
                        new_position.x,
                        new_position.z,
                        &terrain_manager.noise_generator,
                        &terrain_settings,
                    )
                } else {
                    0.0 // Use ground level for positions outside normal terrain bounds
                };
                
                // Smoothly adjust height to terrain with interpolation to prevent jerky movement
                let target_height = terrain_height + 2.0;
                new_position.y = transform.translation.y.lerp(target_height, 3.0 * time.delta_secs());
                transform.translation = new_position;
                
                if direction.length() > 0.1 {
                    // Calculate proper rotation towards target using atan2 with correct axes
                    // Add π to account for models facing backwards by default
                    let target_rotation = Quat::from_rotation_y(-direction.x.atan2(-direction.z) + std::f32::consts::PI);
                    
                    // More gradual rotation to prevent jittery turning
                    let turn_speed = (movement.turning_speed * time.delta_secs()).min(0.1); // Cap rotation speed
                    transform.rotation = transform.rotation.slerp(target_rotation, turn_speed);
                }
            } else {
                // Close to target - gradual deceleration
                let deceleration_factor = (distance / DECELERATION_FACTOR).max(0.1); // Slow down as we get closer
                let target_velocity = direction * movement.max_speed * deceleration_factor;
                
                movement.current_velocity = movement.current_velocity.lerp(
                    target_velocity,
                    5.0 * time.delta_secs() // Faster response when close to target
                );
                
                // Stop when very close
                if distance < 1.0 {
                    movement.target_position = None;
                    movement.current_velocity *= 0.8; // Gradual stop
                }
            }
        } else {
            movement.current_velocity *= 0.9;
        }
    }
}


pub fn resource_gathering_system(
    mut gatherers: Query<(Entity, &mut ResourceGatherer, &Transform, &RTSUnit), With<RTSUnit>>,
    mut resources: Query<(Entity, &mut ResourceSource, &Transform), Without<RTSUnit>>,
    mut player_resources: ResMut<crate::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::resources::AIResources>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (_gatherer_entity, mut gatherer, gatherer_transform, unit) in gatherers.iter_mut() {
        if let Some(resource_entity) = gatherer.target_resource {
            if let Ok((_, mut resource, resource_transform)) = resources.get_mut(resource_entity) {
                let distance = gatherer_transform.translation.distance(resource_transform.translation);
                
                if distance <= 5.0 {
                    if gatherer.carried_amount < gatherer.capacity {
                        let gather_amount = gatherer.gather_rate * time.delta_secs();
                        let actual_gathered = gather_amount.min(
                            gatherer.capacity - gatherer.carried_amount
                        ).min(resource.amount);
                        
                        gatherer.carried_amount += actual_gathered;
                        resource.amount -= actual_gathered;
                        gatherer.resource_type = Some(resource.resource_type.clone());
                        
                        if resource.amount <= 0.0 {
                            commands.entity(resource_entity).despawn();
                            gatherer.target_resource = None;
                        }
                    } else {
                        gatherer.target_resource = None;
                    }
                }
            } else {
                gatherer.target_resource = None;
            }
        }
        
        if gatherer.carried_amount >= gatherer.capacity {
            if let Some(dropoff_entity) = gatherer.drop_off_building {
                if let Ok(dropoff_query) = resources.get(dropoff_entity) {
                    let distance = gatherer_transform.translation.distance(dropoff_query.2.translation);
                    if distance <= 10.0 {
                        // Deliver resources to player stockpile
                        if let Some(resource_type) = gatherer.resource_type.clone() {
                            if unit.player_id == 1 {
                                player_resources.add_resource(resource_type, gatherer.carried_amount);
                                info!("Player delivered {:.1} {:?}", gatherer.carried_amount, gatherer.resource_type);
                            } else if let Some(ai_player_resources) = ai_resources.resources.get_mut(&unit.player_id) {
                                ai_player_resources.add_resource(resource_type, gatherer.carried_amount);
                                info!("AI Player {} delivered {:.1} {:?}", unit.player_id, gatherer.carried_amount, gatherer.resource_type);
                            }
                        }
                        gatherer.carried_amount = 0.0;
                        gatherer.resource_type = None;
                    }
                }
            }
        }
    }
}


pub fn drag_selection_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut selectables: Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drag_selection_query: Query<&mut DragSelection>,
    selection_box_query: Query<Entity, With<SelectionBox>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    
    if let Some(cursor_position) = window.cursor_position() {
        // Initialize or update drag selection
        if mouse_button.just_pressed(MouseButton::Left) {
            // Start drag selection
            if drag_selection_query.is_empty() {
                commands.spawn(DragSelection {
                    start_position: cursor_position,
                    current_position: cursor_position,
                    is_active: true,
                });
            } else if let Ok(mut drag_selection) = drag_selection_query.get_single_mut() {
                drag_selection.start_position = cursor_position;
                drag_selection.current_position = cursor_position;
                drag_selection.is_active = true;
            }
        }
        
        // Update drag selection if active
        if mouse_button.pressed(MouseButton::Left) {
            if let Ok(mut drag_selection) = drag_selection_query.get_single_mut() {
                if drag_selection.is_active {
                    drag_selection.current_position = cursor_position;
                    
                    // Create or update visual selection box
                    let min_x = drag_selection.start_position.x.min(drag_selection.current_position.x);
                    let max_x = drag_selection.start_position.x.max(drag_selection.current_position.x);
                    let min_y = drag_selection.start_position.y.min(drag_selection.current_position.y);
                    let max_y = drag_selection.start_position.y.max(drag_selection.current_position.y);
                    
                    // Remove old selection box
                    for entity in selection_box_query.iter() {
                        commands.entity(entity).despawn();
                    }
                    
                    // Only create visual box if dragging more than a few pixels
                    if (max_x - min_x > 5.0) && (max_y - min_y > 5.0) {
                        // Create 2D selection box (simplified - in a real game you'd use UI overlay)
                        let center_screen = Vec2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
                        let size = Vec2::new(max_x - min_x, max_y - min_y);
                        
                        // Convert screen position to world position for visualization
                        if let Ok(ray) = camera.viewport_to_world(camera_transform, center_screen) {
                            let ground_y = 1.0;
                            let t = (ground_y - ray.origin.y) / ray.direction.y.max(0.001);
                            if t > 0.0 {
                                let world_pos = ray.origin + ray.direction * t;
                                
                                // Create a visual selection rectangle on the ground
                                commands.spawn((
                                    Mesh3d(meshes.add(Rectangle::new(size.x * 0.1, size.y * 0.1))),
                                    MeshMaterial3d(materials.add(StandardMaterial {
                                        base_color: Color::srgba(0.0, 1.0, 0.0, 0.3),
                                        alpha_mode: AlphaMode::Blend,
                                        ..default()
                                    })),
                                    Transform::from_translation(world_pos)
                                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                                    SelectionBox,
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        // Finalize selection on release
        if mouse_button.just_released(MouseButton::Left) {
            if let Ok(mut drag_selection) = drag_selection_query.get_single_mut() {
                if drag_selection.is_active {
                    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
                    
                    let min_x = drag_selection.start_position.x.min(drag_selection.current_position.x);
                    let max_x = drag_selection.start_position.x.max(drag_selection.current_position.x);
                    let min_y = drag_selection.start_position.y.min(drag_selection.current_position.y);
                    let max_y = drag_selection.start_position.y.max(drag_selection.current_position.y);
                    
                    let is_drag = (max_x - min_x > 5.0) || (max_y - min_y > 5.0);
                    
                    if !shift_held && !is_drag {
                        // Clear all selections for single click
                        for (_, mut selectable, _, _) in selectables.iter_mut() {
                            selectable.is_selected = false;
                        }
                    }
                    
                    let mut selected_count = 0;
                    
                    if is_drag {
                        // Box selection
                        for (entity, mut selectable, transform, unit) in selectables.iter_mut() {
                            if unit.player_id == 1 { // Only player 1 units
                                // Project unit world position to screen
                                if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, transform.translation) {
                                    if screen_pos.x >= min_x && screen_pos.x <= max_x &&
                                       screen_pos.y >= min_y && screen_pos.y <= max_y {
                                        if !shift_held {
                                            selectable.is_selected = true;
                                        } else {
                                            selectable.is_selected = !selectable.is_selected; // Toggle
                                        }
                                        selected_count += 1;
                                        
                                        // Spawn selection indicator if selected
                                        if selectable.is_selected {
                                            commands.spawn((
                                                Mesh3d(meshes.add(Torus::new(selectable.selection_radius * 0.8, 0.5))),
                                                MeshMaterial3d(materials.add(StandardMaterial {
                                                    base_color: Color::srgb(0.0, 1.0, 0.0),
                                                    emissive: Color::srgb(0.0, 0.5, 0.0).into(),
                                                    alpha_mode: AlphaMode::Blend,
                                                    ..default()
                                                })),
                                                Transform::from_translation(Vec3::new(transform.translation.x, 1.0, transform.translation.z)),
                                                SelectionIndicator { target: entity },
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        info!("📦 Box selected {} units", selected_count);
                    } else {
                        // Single click selection
                        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                            let mut closest_distance = f32::INFINITY;
                            let mut closest_entity = None;
                            
                            for (entity, selectable, transform, unit) in selectables.iter() {
                                if unit.player_id == 1 {
                                    let to_entity = transform.translation - ray.origin;
                                    let projected_distance = to_entity.dot(ray.direction.normalize());
                                    
                                    if projected_distance > 0.0 {
                                        let closest_point = ray.origin + ray.direction.normalize() * projected_distance;
                                        let distance_to_ray = closest_point.distance(transform.translation);
                                        
                                        if distance_to_ray < selectable.selection_radius && projected_distance < closest_distance {
                                            closest_distance = projected_distance;
                                            closest_entity = Some(entity);
                                        }
                                    }
                                }
                            }
                            
                            if let Some(selected_entity) = closest_entity {
                                for (entity, mut selectable, transform, unit) in selectables.iter_mut() {
                                    if entity == selected_entity {
                                        if shift_held {
                                            selectable.is_selected = !selectable.is_selected; // Toggle
                                        } else {
                                            selectable.is_selected = true;
                                        }
                                        
                                        info!("✅ Selected unit {} at position {:?}", unit.unit_id, transform.translation);
                                        
                                        if selectable.is_selected {
                                            commands.spawn((
                                                Mesh3d(meshes.add(Torus::new(selectable.selection_radius * 0.8, 0.5))),
                                                MeshMaterial3d(materials.add(StandardMaterial {
                                                    base_color: Color::srgb(0.0, 1.0, 0.0),
                                                    emissive: Color::srgb(0.0, 0.5, 0.0).into(),
                                                    alpha_mode: AlphaMode::Blend,
                                                    ..default()
                                                })),
                                                Transform::from_translation(Vec3::new(transform.translation.x, 1.0, transform.translation.z)),
                                                SelectionIndicator { target: entity },
                                            ));
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    
                    drag_selection.is_active = false;
                    
                    // Clean up selection box
                    for entity in selection_box_query.iter() {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

pub fn production_system(
    mut buildings: Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    mut player_resources: ResMut<crate::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::resources::AIResources>,
    game_costs: Res<crate::resources::GameCosts>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (mut queue, building, unit) in buildings.iter_mut() {
        if !queue.queue.is_empty() && building.is_complete {
            queue.current_progress += time.delta_secs();
            
            if queue.current_progress >= queue.production_time {
                let unit_type = queue.queue[0].clone();
                
                // Check if player can afford the unit and has population space
                let can_produce = if unit.player_id == 1 {
                    if let Some(cost) = game_costs.unit_costs.get(&unit_type) {
                        player_resources.can_afford(cost) && player_resources.has_population_space()
                    } else {
                        true
                    }
                } else {
                    if let Some(ai_player_resources) = ai_resources.resources.get(&unit.player_id) {
                        if let Some(cost) = game_costs.unit_costs.get(&unit_type) {
                            ai_player_resources.can_afford(cost) && ai_player_resources.has_population_space()
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                };
                
                if can_produce {
                    queue.queue.remove(0);
                    queue.current_progress = 0.0;
                    
                    // Pay the cost
                    if unit.player_id == 1 {
                        if let Some(cost) = game_costs.unit_costs.get(&unit_type) {
                            player_resources.spend_resources(cost);
                            player_resources.add_population(1);
                        }
                    } else if let Some(ai_player_resources) = ai_resources.resources.get_mut(&unit.player_id) {
                        if let Some(cost) = game_costs.unit_costs.get(&unit_type) {
                            ai_player_resources.spend_resources(cost);
                            ai_player_resources.add_population(1);
                        }
                    }
                    
                    let spawn_position = building.rally_point.unwrap_or(Vec3::ZERO);
                    
                    match unit_type {
                        UnitType::WorkerAnt => {
                            crate::rts_entities::RTSEntityFactory::spawn_worker_ant(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                spawn_position,
                                unit.player_id,
                                rand::random(),
                            );
                        },
                        UnitType::SoldierAnt => {
                            crate::rts_entities::RTSEntityFactory::spawn_soldier_ant(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                spawn_position,
                                unit.player_id,
                                rand::random(),
                            );
                        },
                        UnitType::HunterWasp => {
                            crate::rts_entities::RTSEntityFactory::spawn_hunter_wasp(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                spawn_position,
                                unit.player_id,
                                rand::random(),
                            );
                        },
                        UnitType::BeetleKnight => {
                            crate::rts_entities::RTSEntityFactory::spawn_beetle_knight(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                spawn_position,
                                unit.player_id,
                                rand::random(),
                            );
                        },
                        _ => {},
                    }
                    
                    info!("Player {} produced {:?} unit", unit.player_id, unit_type);
                } else {
                    // Can't afford or no population space - pause production
                    if unit.player_id == 1 {
                        info!("Cannot produce unit: insufficient resources or population space");
                    }
                }
            }
        }
    }
}

pub fn construction_system(
    mut constructors: Query<(&mut Constructor, &Position), With<RTSUnit>>,
    mut buildings: Query<(Entity, &mut Building)>,
    time: Res<Time>,
) {
    for (mut constructor, _constructor_pos) in constructors.iter_mut() {
        if let Some(target_entity) = constructor.current_target {
            if let Ok((_, mut building)) = buildings.get_mut(target_entity) {
                if !building.is_complete {
                    building.construction_progress += constructor.build_speed * time.delta_secs();
                    
                    if building.construction_progress >= building.max_construction {
                        building.is_complete = true;
                        constructor.current_target = None;
                    }
                }
            } else {
                constructor.current_target = None;
            }
        }
    }
}

pub fn formation_system(
    mut units: Query<(&mut Movement, &Formation), With<RTSUnit>>,
    leaders: Query<&Position, (With<RTSUnit>, Without<Formation>)>,
) {
    for (mut movement, formation) in units.iter_mut() {
        if let Some(leader_entity) = formation.leader {
            if let Ok(leader_position) = leaders.get(leader_entity) {
                let formation_offset = match formation.formation_type {
                    FormationType::Line => {
                        Vec3::new(formation.position_in_formation.x * 10.0, 0.0, 0.0)
                    },
                    FormationType::Box => {
                        Vec3::new(
                            formation.position_in_formation.x * 8.0,
                            0.0,
                            formation.position_in_formation.y * 8.0
                        )
                    },
                    FormationType::Wedge => {
                        Vec3::new(
                            formation.position_in_formation.x * 6.0,
                            0.0,
                            formation.position_in_formation.y * 12.0
                        )
                    },
                    FormationType::Circle => {
                        let angle = formation.position_in_formation.x;
                        let radius = formation.position_in_formation.y;
                        Vec3::new(
                            angle.cos() * radius,
                            0.0,
                            angle.sin() * radius
                        )
                    },
                };
                
                movement.target_position = Some(leader_position.translation + formation_offset);
            }
        }
    }
}

pub fn vision_system(
    units: Query<(Entity, &Position, &Vision, &RTSUnit)>,
    mut visible_units: Local<std::collections::HashMap<Entity, Vec<Entity>>>,
) {
    visible_units.clear();
    
    for (entity, position, vision, unit) in units.iter() {
        let mut visible = Vec::new();
        
        for (other_entity, other_position, _, other_unit) in units.iter() {
            if entity != other_entity && unit.player_id != other_unit.player_id {
                let distance = position.translation.distance(other_position.translation);
                if distance <= vision.sight_range {
                    visible.push(other_entity);
                }
            }
        }
        
        visible_units.insert(entity, visible);
    }
}

pub fn unit_command_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut units: Query<(&mut Movement, &mut Combat, &Selectable, &RTSUnit), With<RTSUnit>>,
    all_units: Query<(&Transform, &RTSUnit), With<RTSUnit>>,
    resources: Query<&Transform, With<ResourceSource>>,
    _gatherers: Query<&mut ResourceGatherer>,
    terrain_manager: Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::terrain_v2::TerrainSettings>,
) {
    if mouse_button.just_pressed(MouseButton::Right) {
        let window = windows.single();
        if let Some(cursor_position) = window.cursor_position() {
            let (camera, camera_transform) = camera_q.single();
            
            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                // Check if clicking on an enemy unit (attack command)
                let mut target_enemy = None;
                let mut closest_distance = f32::INFINITY;
                
                for (target_transform, target_unit) in all_units.iter() {
                    let to_target = target_transform.translation - ray.origin;
                    let projected_distance = to_target.dot(ray.direction.normalize());
                    
                    if projected_distance > 0.0 {
                        let closest_point = ray.origin + ray.direction.normalize() * projected_distance;
                        let distance_to_ray = closest_point.distance(target_transform.translation);
                        
                        if distance_to_ray < 8.0 && projected_distance < closest_distance {
                            // Check if it's an enemy (different player)
                            let mut is_enemy = false;
                            for (_, _, selectable, unit) in units.iter() {
                                if selectable.is_selected && unit.player_id != target_unit.player_id {
                                    is_enemy = true;
                                    break;
                                }
                            }
                            
                            if is_enemy {
                                closest_distance = projected_distance;
                                // Find entity by comparing positions (simplified approach)
                                for (entity, (transform_comp, unit_comp)) in all_units.iter().enumerate() {
                                    if transform_comp.translation == target_transform.translation && 
                                       unit_comp.player_id == target_unit.player_id {
                                        // Convert to Entity (this is hacky, but works for demo)
                                        target_enemy = Some(Entity::from_raw(entity as u32));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Check if clicking on a resource (gather command)
                let mut target_resource = None;
                if target_enemy.is_none() {
                    for (_resource_entity, resource_transform) in resources.iter().enumerate() {
                        let to_resource = resource_transform.translation - ray.origin;
                        let projected_distance = to_resource.dot(ray.direction.normalize());
                        
                        if projected_distance > 0.0 {
                            let closest_point = ray.origin + ray.direction.normalize() * projected_distance;
                            let distance_to_ray = closest_point.distance(resource_transform.translation);
                            
                            if distance_to_ray < 6.0 {
                                // Get the actual entity (this is a hack - we'd need better entity tracking)
                                target_resource = Some(resource_transform.translation);
                                break;
                            }
                        }
                    }
                }
                
                // Calculate target position using terrain height sampling
                let horizontal_intersection = if ray.direction.y.abs() > 0.001 {
                    // Ray intersects with a ground plane - use approximate ground level for calculation
                    let ground_y = 0.0; // Use terrain base level for intersection calculation
                    let t = (ground_y - ray.origin.y) / ray.direction.y;
                    if t > 0.0 && t < 1000.0 { // Limit ray distance to prevent extreme positions
                        let intersection = ray.origin + ray.direction * t;
                        // Clamp intersection to reasonable bounds
                        Vec3::new(
                            intersection.x.clamp(-5000.0, 5000.0),
                            intersection.y,
                            intersection.z.clamp(-5000.0, 5000.0),
                        )
                    } else {
                        let horizontal_dir = Vec3::new(ray.direction.x, 0.0, ray.direction.z).normalize_or_zero();
                        let offset = ray.origin + horizontal_dir * 50.0;
                        Vec3::new(
                            offset.x.clamp(-5000.0, 5000.0),
                            offset.y,
                            offset.z.clamp(-5000.0, 5000.0),
                        )
                    }
                } else {
                    let horizontal_dir = Vec3::new(ray.direction.x, 0.0, ray.direction.z).normalize_or_zero();
                    let offset = ray.origin + horizontal_dir * 50.0;
                    Vec3::new(
                        offset.x.clamp(-5000.0, 5000.0),
                        offset.y,
                        offset.z.clamp(-5000.0, 5000.0),
                    )
                };
                
                // Validate horizontal intersection before proceeding
                if !horizontal_intersection.x.is_finite() || !horizontal_intersection.z.is_finite() ||
                   horizontal_intersection.x.abs() > 10000.0 || horizontal_intersection.z.abs() > 10000.0 {
                    warn!("Invalid target position calculated: {:?}, ignoring command", horizontal_intersection);
                    return;
                }
                
                // Sample the actual terrain height at the target location
                let terrain_height = if horizontal_intersection.x.abs() < 10000.0 && horizontal_intersection.z.abs() < 10000.0 {
                    crate::terrain_v2::sample_terrain_height(
                        horizontal_intersection.x,
                        horizontal_intersection.z,
                        &terrain_manager.noise_generator,
                        &terrain_settings,
                    )
                } else {
                    0.0 // Use ground level for positions outside normal terrain bounds
                };
                
                // Set target position with proper terrain height
                let target_point = Vec3::new(
                    horizontal_intersection.x,
                    terrain_height + 2.0, // Keep units above ground
                    horizontal_intersection.z,
                );
                
                // Issue commands to selected units
                for (mut movement, mut combat, selectable, unit) in units.iter_mut() {
                    if selectable.is_selected && unit.player_id == 1 { // Only player 1 units
                        if let Some(enemy_entity) = target_enemy {
                            // Attack command
                            combat.target = Some(enemy_entity);
                            movement.target_position = None; // Stop moving, engage target
                            info!("🗡️ Unit {:?} attacking target {:?}!", unit.unit_id, enemy_entity);
                        } else if let Some(_resource_pos) = target_resource {
                            // Gather command (for villagers) - simplified for demo
                            movement.target_position = Some(target_point);
                            // Reduced resource gathering movement logging
                        } else {
                            // Validate target point before setting
                            if target_point.x.is_finite() && target_point.z.is_finite() &&
                               target_point.x.abs() < 10000.0 && target_point.z.abs() < 10000.0 {
                                // Move command
                                movement.target_position = Some(target_point);
                                combat.target = None; // Cancel any attack
                                // Reduced movement command logging
                            } else {
                                warn!("Invalid target point {:?}, ignoring move command", target_point);
                            }
                        }
                    }
                }
            }
        }
    }
}


pub fn selection_indicator_system(
    mut commands: Commands,
    indicators: Query<(Entity, &SelectionIndicator), With<SelectionIndicator>>,
    selectables: Query<(&Selectable, &Transform), With<RTSUnit>>,
    mut indicator_transforms: Query<&mut Transform, (With<SelectionIndicator>, Without<RTSUnit>)>,
) {
    // Update existing indicators
    for (indicator_entity, indicator) in indicators.iter() {
        if let Ok((selectable, unit_transform)) = selectables.get(indicator.target) {
            if selectable.is_selected {
                // Update indicator position to follow unit
                if let Ok(mut indicator_transform) = indicator_transforms.get_mut(indicator_entity) {
                    indicator_transform.translation = Vec3::new(
                        unit_transform.translation.x,
                        1.0,
                        unit_transform.translation.z
                    );
                }
            } else {
                // Unit is no longer selected, remove indicator
                commands.entity(indicator_entity).despawn();
            }
        } else {
            // Target unit no longer exists, remove indicator
            commands.entity(indicator_entity).despawn();
        }
    }
}

// Test system to spawn combat units for demonstration
pub fn spawn_test_units_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_q: Query<&Transform, With<crate::components::RTSCamera>>,
    mut units_q: Query<(&mut Movement, &Transform), (With<RTSUnit>, Without<crate::components::RTSCamera>)>,
) {
    if keyboard.just_pressed(KeyCode::KeyM) {
        // Spawn militia unit at ground level near camera
        if let Ok(camera_transform) = camera_q.get_single() {
            let camera_ground_pos = Vec3::new(camera_transform.translation.x, 0.0, camera_transform.translation.z);
            let spawn_pos = camera_ground_pos + Vec3::new(crate::constants::combat::UNIT_SPAWN_OFFSET, 1.0, 0.0);
            crate::combat_systems::create_combat_unit(
                &mut commands,
                &mut meshes,
                &mut materials,
                spawn_pos,
                1, // Player 1
                UnitType::SoldierAnt,
            );
            info!("Spawned SoldierAnt at {:?}", spawn_pos);
        }
    }
    
    if keyboard.just_pressed(KeyCode::KeyA) {
        // Spawn archer unit at ground level near camera
        if let Ok(camera_transform) = camera_q.get_single() {
            let camera_ground_pos = Vec3::new(camera_transform.translation.x, 0.0, camera_transform.translation.z);
            let spawn_pos = camera_ground_pos + Vec3::new(-crate::constants::combat::UNIT_SPAWN_OFFSET, 1.0, 0.0);
            crate::combat_systems::create_combat_unit(
                &mut commands,
                &mut meshes,
                &mut materials,
                spawn_pos,
                1, // Player 1
                UnitType::HunterWasp,
            );
            info!("Spawned HunterWasp at {:?}", spawn_pos);
        }
    }
    
    if keyboard.just_pressed(KeyCode::KeyE) {
        // Spawn enemy unit at ground level near camera
        if let Ok(camera_transform) = camera_q.get_single() {
            let camera_ground_pos = Vec3::new(camera_transform.translation.x, 0.0, camera_transform.translation.z);
            let spawn_pos = camera_ground_pos + Vec3::new(0.0, 1.0, crate::constants::combat::UNIT_SPAWN_RANGE);
            crate::combat_systems::create_combat_unit(
                &mut commands,
                &mut meshes,
                &mut materials,
                spawn_pos,
                2, // Player 2 (enemy)
                UnitType::SoldierAnt,
            );
            info!("Spawned Enemy at {:?}", spawn_pos);
        }
    }
    
    // Test movement commands - press N to give all units a move command  
    if keyboard.just_pressed(KeyCode::KeyN) {
        if let Ok(camera_transform) = camera_q.get_single() {
            let target_pos = Vec3::new(
                camera_transform.translation.x + 50.0,
                0.0,
                camera_transform.translation.z + 50.0
            );
            
            let mut count = 0;
            for (mut movement, _transform) in units_q.iter_mut() {
                movement.target_position = Some(target_pos + Vec3::new(count as f32 * 5.0, 0.0, 0.0));
                count += 1;
            }
            
            // Reduced group movement logging
        }
    }
}

pub fn building_completion_system(
    mut buildings: Query<(Entity, &mut Building, &RTSUnit), Changed<Building>>,
    mut player_resources: ResMut<crate::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::resources::AIResources>,
) {
    for (_, mut building, unit) in buildings.iter_mut() {
        if building.is_complete && building.construction_progress >= building.max_construction {
            // Building just completed, grant benefits
            match building.building_type {
                BuildingType::Nursery => {
                    if unit.player_id == 1 {
                        player_resources.add_housing(crate::constants::resources::NURSERY_POPULATION_CAPACITY);
                        info!("Player 1 house completed! Population limit increased by {}", crate::constants::resources::NURSERY_POPULATION_CAPACITY);
                    } else if let Some(ai_player_resources) = ai_resources.resources.get_mut(&unit.player_id) {
                        ai_player_resources.add_housing(crate::constants::resources::NURSERY_POPULATION_CAPACITY);
                        info!("Player {} house completed! Population limit increased by {}", unit.player_id, crate::constants::resources::NURSERY_POPULATION_CAPACITY);
                    }
                },
                BuildingType::Queen => {
                    if unit.player_id == 1 {
                        player_resources.add_housing(crate::constants::resources::QUEEN_POPULATION_CAPACITY);
                        info!("Player 1 town center completed! Population limit increased by {}", crate::constants::resources::QUEEN_POPULATION_CAPACITY);
                    } else if let Some(ai_player_resources) = ai_resources.resources.get_mut(&unit.player_id) {
                        ai_player_resources.add_housing(crate::constants::resources::QUEEN_POPULATION_CAPACITY);
                        info!("Player {} town center completed! Population limit increased by {}", unit.player_id, crate::constants::resources::QUEEN_POPULATION_CAPACITY);
                    }
                },
                _ => {}
            }
            
            // Mark that we've already processed this completion
            building.construction_progress = building.max_construction + 1.0;
        }
    }
}

pub fn population_management_system(
    units: Query<&RTSUnit, With<RTSUnit>>,
    mut player_resources: ResMut<crate::resources::PlayerResources>,
    mut ai_resources: ResMut<crate::resources::AIResources>,
    time: Res<Time>,
    mut update_timer: Local<Timer>,
) {
    // Update population counts
    update_timer.set_duration(crate::constants::population::UPDATE_INTERVAL);
    update_timer.tick(time.delta());
    
    if update_timer.just_finished() {
        // Count current population for each player
        let mut population_counts: std::collections::HashMap<u8, u32> = std::collections::HashMap::new();
        let mut villager_counts: std::collections::HashMap<u8, u32> = std::collections::HashMap::new();
        
        for unit in units.iter() {
            *population_counts.entry(unit.player_id).or_insert(0) += 1;
            // For now, assume all units are villagers for idle count
            // In a real game, you'd track unit types
            *villager_counts.entry(unit.player_id).or_insert(0) += 1;
        }
        
        // Update player resources
        if let Some(&current_pop) = population_counts.get(&1) {
            player_resources.current_population = current_pop;
            // Note: idle_villagers field removed - this data can be calculated from existing unit queries
        }
        
        // Update AI resources
        for (&player_id, resources) in ai_resources.resources.iter_mut() {
            if let Some(&current_pop) = population_counts.get(&player_id) {
                resources.current_population = current_pop;
                // Note: idle_villagers field removed - this data can be calculated from existing unit queries
            }
        }
    }
}