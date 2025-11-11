use bevy::prelude::*;
use crate::core::components::*;
// Removed unused import: use crate::core::resources::*;
use crate::core::game::GameState;
// Terrain asset no longer needed

// === RTS SETUP CONSTANTS ===
const INITIAL_CAMERA_HEIGHT: f32 = 200.0;
const INITIAL_CAMERA_DISTANCE: f32 = 100.0;
const LIGHT_ROTATION_X: f32 = -0.8;
const LIGHT_ROTATION_Y: f32 = -0.3;

#[allow(dead_code)]
const RESOURCE_SPAWN_AREAS: usize = 6;

pub fn setup_game(
    mut commands: Commands,
) {
    // Terrain-following Camera - moderate altitude starting position
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, INITIAL_CAMERA_HEIGHT, INITIAL_CAMERA_DISTANCE)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        MainCamera,
        RTSCamera {
            move_speed: crate::constants::camera::CAMERA_MOVE_SPEED,
        },
    ));
    
    // Lighting - positioned for terrain visibility
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 1.0, 0.9),
            illuminance: 8000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, LIGHT_ROTATION_X, LIGHT_ROTATION_Y, 0.0)),
    ));
    
    // Ambient light for better terrain visibility
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.5, 0.5, 0.7),
        brightness: 500.0,
    });
    
    info!("RTS Game setup complete");
}

pub fn setup_menu(
    mut commands: Commands,
) {
    commands.spawn((
        Text::new("Press SPACE to Start RTS Mode"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(50.0),
            ..default()
        },
        UI,
    ));
}

pub fn handle_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    ui_query: Query<Entity, With<UI>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // Despawn menu UI
        for entity in ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        
        // Transition to playing state
        next_state.set(GameState::Playing);
        info!("Transitioning to RTS Playing state");
    }
}

pub fn spawn_rts_elements(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_manager: Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::world::terrain_v2::TerrainSettings>,
    model_assets: Option<Res<crate::rendering::model_loader::ModelAssets>>,
) {
    info!("=== SPAWNING RTS ELEMENTS ===");
    // Spawn RTS units and buildings - one of each type for testing
    use crate::entities::entity_factory::{EntityFactory, SpawnConfig, EntityType};
    
    // Helper function to get terrain-aware position
    let get_terrain_position = |x: f32, z: f32, height_offset: f32| -> Vec3 {
        let terrain_height = crate::world::terrain_v2::sample_terrain_height(
            x, z, &terrain_manager.noise_generator, &terrain_settings
        );
        Vec3::new(x, terrain_height + height_offset, z)
    };
    
    // === PLAYER 1 (Human) - Left side of map ===
    let player1_base_2d = Vec3::new(-200.0, 0.0, 0.0);
    let player1_base = get_terrain_position(player1_base_2d.x, player1_base_2d.z, 0.0);
    
    // Spawn Queen Chamber (main building)
    let queen_config = SpawnConfig::building(
        EntityType::Building(crate::core::components::BuildingType::Queen),
        player1_base,
        1,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        queen_config,
        model_assets.as_deref(),
    );
    
    // Spawn one of each unit type in a grid formation
    let unit_spacing = 15.0;
    let _units_per_row = 4;
    
    // Worker units using new EntityFactory with animations
    let worker_config = SpawnConfig::unit(
        EntityType::from_unit(crate::core::components::UnitType::WorkerAnt),
        get_terrain_position(player1_base.x, player1_base.z + 30.0, 2.0),
        1,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        worker_config,
        model_assets.as_deref(),
    );
    
    // Combat units using new EntityFactory with animations
    let soldier_config = SpawnConfig::unit(
        EntityType::from_unit(crate::core::components::UnitType::SoldierAnt),
        get_terrain_position(player1_base.x + unit_spacing, player1_base.z + 30.0, 2.0),
        1,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        soldier_config,
        model_assets.as_deref(),
    );
    
    let wasp_config = SpawnConfig::unit(
        EntityType::from_unit(crate::core::components::UnitType::HunterWasp),
        get_terrain_position(player1_base.x + unit_spacing * 2.0, player1_base.z + 30.0, 2.0),
        1,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        wasp_config,
        model_assets.as_deref(),
    );
    
    let beetle_config = SpawnConfig::unit(
        EntityType::from_unit(crate::core::components::UnitType::BeetleKnight),
        get_terrain_position(player1_base.x + unit_spacing * 3.0, player1_base.z + 30.0, 2.0),
        1,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        beetle_config,
        model_assets.as_deref(),
    );
    
    // Note: Additional unit types (SpearMantis, ScoutAnt, etc.) would need spawn functions created
    // For now we have the 4 main unit types that have spawn functions
    
    // Spawn one of each building type
    let building_spacing = 30.0;
    
    // Nursery (house equivalent)
    let nursery_config = SpawnConfig::building(
        EntityType::Building(crate::core::components::BuildingType::Nursery),
        get_terrain_position(player1_base.x - building_spacing, player1_base.z - building_spacing, 0.0),
        1,
    );
    EntityFactory::spawn(&mut commands, &mut meshes, &mut materials, nursery_config, model_assets.as_deref());

    // Warrior Chamber (barracks equivalent)
    let warrior_config = SpawnConfig::building(
        EntityType::Building(crate::core::components::BuildingType::WarriorChamber),
        get_terrain_position(player1_base.x + building_spacing, player1_base.z - building_spacing, 0.0),
        1,
    );
    EntityFactory::spawn(&mut commands, &mut meshes, &mut materials, warrior_config, model_assets.as_deref());
    
    // Note: Additional building types would need spawn functions created
    
    // === PLAYER 2 (Enemy) - Right side of map ===
    let player2_base_2d = Vec3::new(200.0, 0.0, 0.0);
    let player2_base = get_terrain_position(player2_base_2d.x, player2_base_2d.z, 0.0);
    
    // Spawn Queen Chamber for enemy
    let queen2_config = SpawnConfig::building(
        EntityType::Building(crate::core::components::BuildingType::Queen),
        player2_base,
        2,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        queen2_config,
        model_assets.as_deref(),
    );
    
    // Enemy units - spawn multiple workers for better economy
    for i in 0..5 {
        let worker_offset_x = (i as f32 - 2.0) * 8.0;
        let enemy_worker_config = SpawnConfig::unit(
            EntityType::from_unit(crate::core::components::UnitType::WorkerAnt),
            get_terrain_position(player2_base.x + worker_offset_x, player2_base.z - 30.0, 2.0),
            2,
        );
        EntityFactory::spawn(
            &mut commands,
            &mut meshes,
            &mut materials,
            enemy_worker_config,
            model_assets.as_deref(),
        );
    }
    
    let enemy_soldier_config = SpawnConfig::unit(
        EntityType::from_unit(crate::core::components::UnitType::SoldierAnt),
        get_terrain_position(player2_base.x - unit_spacing, player2_base.z - 30.0, 2.0),
        2,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        enemy_soldier_config,
        model_assets.as_deref(),
    );
    
    let enemy_wasp_config = SpawnConfig::unit(
        EntityType::from_unit(crate::core::components::UnitType::HunterWasp),
        get_terrain_position(player2_base.x - unit_spacing * 2.0, player2_base.z - 30.0, 2.0),
        2,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        enemy_wasp_config,
        model_assets.as_deref(),
    );
    
    let enemy_beetle_config = SpawnConfig::unit(
        EntityType::from_unit(crate::core::components::UnitType::BeetleKnight),
        get_terrain_position(player2_base.x - unit_spacing * 3.0, player2_base.z - 30.0, 2.0),
        2,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        enemy_beetle_config,
        model_assets.as_deref(),
    );
    
    // Enemy buildings
    let enemy_nursery_config = SpawnConfig::building(
        EntityType::Building(crate::core::components::BuildingType::Nursery),
        get_terrain_position(player2_base.x + building_spacing, player2_base.z + building_spacing, 0.0),
        2,
    );
    EntityFactory::spawn(&mut commands, &mut meshes, &mut materials, enemy_nursery_config, model_assets.as_deref());

    let enemy_warrior_config = SpawnConfig::building(
        EntityType::Building(crate::core::components::BuildingType::WarriorChamber),
        get_terrain_position(player2_base.x - building_spacing, player2_base.z + building_spacing, 0.0),
        2,
    );
    EntityFactory::spawn(&mut commands, &mut meshes, &mut materials, enemy_warrior_config, model_assets.as_deref());
    
    // === NEUTRAL RESOURCES ===
    // Resources are now spawned as environment objects (mushrooms, rocks) with ResourceSource components
    // This provides better visual integration and uses GLB models instead of primitive shapes
    
    // === ENVIRONMENT OBJECTS ===
    spawn_minimal_environment_objects(&mut commands, &mut meshes, &mut materials, &terrain_manager, &terrain_settings, model_assets.as_ref().map(|v| &**v));
    
    info!("RTS elements spawned");
}

/// Spawn minimal environment objects near center area only
fn spawn_minimal_environment_objects(
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    terrain_manager: &Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: &Res<crate::world::terrain_v2::TerrainSettings>,
    model_assets: Option<&crate::rendering::model_loader::ModelAssets>,
) {
    use crate::rendering::model_loader::*;
    use crate::core::components::*;
    
    // Helper function to get terrain-aware position
    let get_terrain_position = |x: f32, z: f32, height_offset: f32| -> Vec3 {
        let terrain_height = crate::world::terrain_v2::sample_terrain_height(
            x, z, &terrain_manager.noise_generator, &terrain_settings
        );
        Vec3::new(x, terrain_height + height_offset, z)
    };
    
    // Only spawn a few objects in the center area between player bases
    if let Some(models) = model_assets {
        if models.models_loaded {
            info!("Spawning distributed environment objects across the map");
            
            // Add many more resource nodes with better distribution for AI access
            let object_positions = [
                // Original positions
                (800.0, 1200.0),    // Far northeast
                (-1200.0, 800.0),   // Far northwest  
                (0.0, -1000.0),     // Far south
                (1500.0, -500.0),   // Far southeast
                (-1500.0, -300.0),  // Far southwest
                (600.0, -600.0),    // Southeast quadrant
                (-800.0, 600.0),    // Northwest quadrant
                (200.0, 800.0),     // North-center
                
                // NEW: More resources closer to player bases for better economy
                // Near Player 1 base (-200, 0)
                (-150.0, 80.0),     // Northeast of Player 1 base
                (-250.0, 100.0),    // Northwest of Player 1 base  
                (-100.0, -120.0),   // Southeast of Player 1 base
                (-300.0, -80.0),    // Southwest of Player 1 base
                
                // Near Player 2 base (200, 0) - Critical for AI economy
                (150.0, 90.0),      // Northeast of Player 2 base
                (250.0, 110.0),     // Northwest of Player 2 base
                (120.0, -100.0),    // Southeast of Player 2 base
                (280.0, -90.0),     // Southwest of Player 2 base
                
                // Central contested area for strategic resources
                (-50.0, 50.0),      // Northwest center
                (50.0, 60.0),       // Northeast center
                (-30.0, -70.0),     // Southwest center
                (40.0, -60.0),      // Southeast center
                
                // Additional mid-range resources
                (0.0, 200.0),       // North center
                (0.0, -200.0),      // South center
                (-400.0, 0.0),      // West center
                (400.0, 0.0),       // East center
            ];
            
            for (i, &(x, z)) in object_positions.iter().enumerate() {
                let position = get_terrain_position(x, z, 1.0);
                let rotation = Quat::from_rotation_y(i as f32 * 1.2); // Slight rotation variation
                
                // Prioritize Nectar (food) resources for better unit production, especially near AI base
                let (insect_model_type, env_obj_type, model_handle, object_name) = match i % 6 {
                    // 50% Nectar sources (mushrooms) for better economy
                    0 | 1 | 2 => (InsectModelType::Mushrooms, EnvironmentObjectType::Mushrooms, &models.mushrooms, "Mushroom Cluster"),
                    3 => (InsectModelType::RiverRock, EnvironmentObjectType::Rocks, &models.river_rock, "Rock Formation"),
                    4 => (InsectModelType::WoodStick, EnvironmentObjectType::WoodStick, &models.wood_stick, "Wood Debris"),
                    _ => (InsectModelType::Hive, EnvironmentObjectType::Hive, &models.hive, "Hive Structure"),
                };
                
                // Skip if model handle is invalid
                if model_handle == &Handle::default() {
                    warn!("Invalid model handle for {:?}, skipping spawn", insect_model_type);
                    continue;
                }
                
                // Get base scale for this object type
                let base_scale = crate::rendering::model_loader::get_model_scale(&insect_model_type);
                
                let mut entity_commands = commands.spawn((
                    SceneRoot(model_handle.clone()),
                    Transform::from_translation(position)
                        .with_rotation(rotation)
                        .with_scale(Vec3::splat(base_scale)),
                    InsectModel {
                        model_type: insect_model_type.clone(),
                        scale: base_scale,
                    },
                    UseGLBModel,
                    EnvironmentObject {
                        object_type: env_obj_type.clone(),
                    },
                    CollisionRadius {
                        radius: 3.0 // Small fixed radius so workers can get within GATHERING_DISTANCE (5.0)
                    },
                    Position {
                        translation: position,
                        rotation,
                    },
                    GameEntity,
                    Name::new(object_name),
                ));
                
                // Add ResourceSource component for harvestable objects
                match env_obj_type {
                    EnvironmentObjectType::Mushrooms => {
                        entity_commands.insert(ResourceSource {
                            resource_type: ResourceType::Nectar,
                            amount: 800.0,  // Increased from 300 to 800 for better economy
                            max_gatherers: 4,  // Allow more gatherers for faster collection
                            current_gatherers: 0,
                        });
                        entity_commands.insert(Selectable {
                            is_selected: false,
                            selection_radius: base_scale * 2.0,
                        });
                    },
                    EnvironmentObjectType::Rocks => {
                        entity_commands.insert(ResourceSource {
                            resource_type: ResourceType::Minerals,
                            amount: 500.0,
                            max_gatherers: 4,
                            current_gatherers: 0,
                        });
                        entity_commands.insert(Selectable {
                            is_selected: false,
                            selection_radius: base_scale * 2.0,
                        });
                    },
                    EnvironmentObjectType::WoodStick => {
                        entity_commands.insert(ResourceSource {
                            resource_type: ResourceType::Chitin,
                            amount: 400.0,
                            max_gatherers: 3,
                            current_gatherers: 0,
                        });
                        entity_commands.insert(Selectable {
                            is_selected: false,
                            selection_radius: base_scale * 2.0,
                        });
                    },
                    EnvironmentObjectType::Hive => {
                        entity_commands.insert(ResourceSource {
                            resource_type: ResourceType::Pheromones,
                            amount: 350.0,
                            max_gatherers: 3,
                            current_gatherers: 0,
                        });
                        entity_commands.insert(Selectable {
                            is_selected: false,
                            selection_radius: base_scale * 2.0,
                        });
                    },
                }
                
                info!("Spawned {} at {:?} (scale: {:.1})", object_name.to_lowercase(), position, base_scale);
            }
        } else {
            info!("Models not yet loaded, skipping environment objects");
        }
    } else {
        info!("No model assets available, skipping environment objects");
    }
    
    info!("Minimal environment objects spawning complete");
}

/// Enhanced RTS camera system with terrain-aware scroll wheel zoom
/// 
/// Controls:
/// - WASD: Pan camera over terrain (W=North, S=South, A=West, D=East)
/// - Mouse wheel: Zoom in/out (constrained to terrain height)
/// - Right mouse + drag: Look around
/// - Shift + controls: Fast mode (10x speed)
/// - Alt + controls: Hyper mode (50x speed) 
/// - G: Disable terrain following
/// - F: Disable height clamping during zoom
pub fn handle_rts_camera_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    _mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut _mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut camera_query: Query<(&mut Transform, &mut RTSCamera), With<MainCamera>>,
    terrain_manager: Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::world::terrain_v2::TerrainSettings>,
    time: Res<Time>,
) {
    if let Ok((mut camera_transform, rts_camera)) = camera_query.get_single_mut() {
        let dt = time.delta_secs();
        
        // Movement calculation based on current camera rotation
        let mut movement = Vec3::ZERO;
        
        // Enhanced terrain following system (always on, Press G to disable)
        let terrain_following = !keyboard.pressed(KeyCode::KeyG);
        
        // RTS-style movement: use world space directions for proper panning
        // W/S moves north/south (Z axis), A/D moves west/east (X axis)
        
        if keyboard.pressed(KeyCode::KeyW) {
            movement += Vec3::new(0.0, 0.0, 1.0);  // North (positive Z)
        }
        if keyboard.pressed(KeyCode::KeyS) {
            movement += Vec3::new(0.0, 0.0, -1.0); // South (negative Z)
        }
        if keyboard.pressed(KeyCode::KeyA) {
            movement += Vec3::new(-1.0, 0.0, 0.0); // West (negative X)
        }
        if keyboard.pressed(KeyCode::KeyD) {
            movement += Vec3::new(1.0, 0.0, 0.0);  // East (positive X)
        }
        
        // Apply movement with multiple speed modes
        if movement.length() > 0.0 {
            let speed_multiplier = if keyboard.pressed(KeyCode::ControlLeft) { 0.1 }      // Very slow
                                 else if keyboard.pressed(KeyCode::ShiftLeft) { 10.0 }   // Very fast
                                 else if keyboard.pressed(KeyCode::AltLeft) { 50.0 }     // Hyper speed
                                 else { 1.0 };                                            // Normal
            movement = movement.normalize() * rts_camera.move_speed * speed_multiplier * dt;
            camera_transform.translation += movement;
        }
        
        // Apply terrain following if enabled
        if terrain_following {
            // Sample terrain height at camera position and nearby points for slope calculation
            let current_pos = camera_transform.translation;
            let terrain_height = crate::world::terrain_v2::sample_terrain_height(
                current_pos.x,
                current_pos.z,
                &terrain_manager.noise_generator,
                &terrain_settings,
            );
            
            // Only adjust height for terrain following if camera is within reasonable bounds
            // This allows scroll wheel zoom to work while still preventing underground camera
            let current_height_above_terrain = camera_transform.translation.y - terrain_height;
            
            // Only force camera up if it's too close to terrain (below minimum height)
            // Otherwise, let manual zoom control the height
            use crate::constants::camera::*;
            if current_height_above_terrain < MIN_HEIGHT_ABOVE_TERRAIN {
                let target_height = terrain_height + MIN_HEIGHT_ABOVE_TERRAIN;
                camera_transform.translation.y = target_height;
                debug!("Terrain collision: Adjusted camera to minimum height above terrain");
            }
            
            // Apply height limits when not in free flight mode
            if !keyboard.pressed(KeyCode::KeyF) {
                camera_transform.translation.y = camera_transform.translation.y.clamp(
                    crate::constants::camera::MIN_HEIGHT_ABOVE_TERRAIN, 
                    crate::constants::camera::MAX_HEIGHT_ABOVE_TERRAIN
                );
            }
        } else if !keyboard.pressed(KeyCode::KeyF) {
            // Standard height clamping when not terrain following
            camera_transform.translation.y = camera_transform.translation.y.clamp(
                crate::constants::camera::MIN_HEIGHT_ABOVE_TERRAIN, 
                crate::constants::camera::MAX_HEIGHT_ABOVE_TERRAIN
            );
        }
        
        // True zoom in/out using vertical camera movement with terrain constraints
        for wheel_event in mouse_wheel.read() {
            use crate::constants::camera::*;
            let zoom_speed_multiplier = if keyboard.pressed(KeyCode::ShiftLeft) { ZOOM_SPEED_FAST_MULTIPLIER } 
                                      else if keyboard.pressed(KeyCode::AltLeft) { ZOOM_SPEED_HYPER_MULTIPLIER }
                                      else { 1.0 };
            
            // More responsive zoom - don't use dt for scroll wheel as it's event-based
            let zoom_delta = -wheel_event.y * SCROLL_ZOOM_SENSITIVITY * zoom_speed_multiplier;
            
            // Calculate new camera height
            let current_position = camera_transform.translation;
            let new_height = current_position.y + zoom_delta;
            
            // Sample terrain height at current camera position
            let terrain_height = crate::world::terrain_v2::sample_terrain_height(
                current_position.x,
                current_position.z,
                &terrain_manager.noise_generator,
                &terrain_settings,
            );
            
            // Define terrain-relative height constraints using constants
            let absolute_min_height = terrain_height + MIN_HEIGHT_ABOVE_TERRAIN;
            let absolute_max_height = terrain_height + MAX_HEIGHT_ABOVE_TERRAIN;
            
            // Clamp the new height to stay within terrain constraints
            let constrained_height = new_height.clamp(absolute_min_height, absolute_max_height);
            
            // Only apply zoom if there was actually a change
            if (constrained_height - current_position.y).abs() > 0.1 {
                camera_transform.translation.y = constrained_height;
            }
        }
    }
}


// Removed old unused systems (update_enemies, handle_collisions, update_ui)
// These were replaced by the RTS systems