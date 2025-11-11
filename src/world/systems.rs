use bevy::prelude::*;
use crate::core::components::*;
// Removed unused import: use crate::core::resources::*;
use crate::core::game::GameState;
// Terrain asset no longer needed

// === RTS SETUP CONSTANTS ===
const INITIAL_CAMERA_HEIGHT: f32 = 200.0;
const INITIAL_CAMERA_DISTANCE: f32 = 100.0;
const CAMERA_PITCH_ANGLE: f32 = -0.8;
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
            zoom_speed: crate::constants::camera::ZOOM_SPEED_BASE,
            min_height: crate::constants::camera::MIN_HEIGHT_ABOVE_TERRAIN,
            max_height: crate::constants::camera::MAX_HEIGHT_ABOVE_TERRAIN,
            look_sensitivity: crate::constants::camera::LOOK_SENSITIVITY,
            pitch: CAMERA_PITCH_ANGLE,
            yaw: 0.0,
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

    // Helper for terrain-aware positioning
    let get_terrain_position = |x: f32, z: f32, height_offset: f32| -> Vec3 {
        let terrain_height = crate::world::terrain_v2::sample_terrain_height(
            x, z, &terrain_manager.noise_generator, &terrain_settings
        );
        Vec3::new(x, terrain_height + height_offset, z)
    };

    // Spawn player 1 (human) base on left side
    spawn_player_base(
        &mut commands,
        &mut meshes,
        &mut materials,
        model_assets.as_deref(),
        1,
        Vec3::new(-200.0, 0.0, 0.0),
        &get_terrain_position,
    );

    // Spawn player 2 (AI) base on right side
    spawn_player_base(
        &mut commands,
        &mut meshes,
        &mut materials,
        model_assets.as_deref(),
        2,
        Vec3::new(200.0, 0.0, 0.0),
        &get_terrain_position,
    );

    // Spawn environment objects (resources, decorations)
    spawn_minimal_environment_objects(
        &mut commands,
        &mut meshes,
        &mut materials,
        &terrain_manager,
        &terrain_settings,
        model_assets.as_ref().map(|v| &**v),
    );

    info!("RTS elements spawned");
}

/// Spawn a player's base with buildings and units
fn spawn_player_base(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    model_assets: Option<&crate::rendering::model_loader::ModelAssets>,
    player_id: u8,
    base_position_2d: Vec3,
    get_terrain_position: &dyn Fn(f32, f32, f32) -> Vec3,
) {
    use crate::entities::entity_factory::{EntityFactory, SpawnConfig, EntityType};
    use crate::core::components::{BuildingType, UnitType};

    let base_pos = get_terrain_position(base_position_2d.x, base_position_2d.z, 0.0);

    // Spawn main building (Queen Chamber)
    spawn_building(
        commands,
        meshes,
        materials,
        model_assets,
        BuildingType::Queen,
        base_pos,
        player_id,
    );

    // Spawn units
    spawn_player_units(
        commands,
        meshes,
        materials,
        model_assets,
        player_id,
        base_pos,
        get_terrain_position,
    );

    // Spawn additional buildings
    spawn_player_buildings(
        commands,
        meshes,
        materials,
        model_assets,
        player_id,
        base_pos,
        get_terrain_position,
    );
}

/// Spawn units for a player
fn spawn_player_units(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    model_assets: Option<&crate::rendering::model_loader::ModelAssets>,
    player_id: u8,
    base_pos: Vec3,
    get_terrain_position: &dyn Fn(f32, f32, f32) -> Vec3,
) {
    use crate::entities::entity_factory::{EntityFactory, SpawnConfig, EntityType};
    use crate::core::components::UnitType;

    let unit_spacing = 15.0;
    let unit_offset = if player_id == 1 { 30.0 } else { -30.0 };

    // For player 2 (AI), spawn multiple workers
    if player_id == 2 {
        for i in 0..5 {
            let worker_offset_x = (i as f32 - 2.0) * 8.0;
            spawn_unit(
                commands,
                meshes,
                materials,
                model_assets,
                UnitType::WorkerAnt,
                get_terrain_position(base_pos.x + worker_offset_x, base_pos.z + unit_offset, 2.0),
                player_id,
            );
        }
    } else {
        // For player 1, spawn one worker
        spawn_unit(
            commands,
            meshes,
            materials,
            model_assets,
            UnitType::WorkerAnt,
            get_terrain_position(base_pos.x, base_pos.z + unit_offset, 2.0),
            player_id,
        );
    }

    // Spawn combat units
    let unit_x_offset = if player_id == 1 { 1.0 } else { -1.0 };
    let combat_units = [
        UnitType::SoldierAnt,
        UnitType::HunterWasp,
        UnitType::BeetleKnight,
    ];

    for (i, unit_type) in combat_units.iter().enumerate() {
        let x_pos = base_pos.x + (unit_x_offset * unit_spacing * (i as f32 + 1.0));
        spawn_unit(
            commands,
            meshes,
            materials,
            model_assets,
            unit_type.clone(),
            get_terrain_position(x_pos, base_pos.z + unit_offset, 2.0),
            player_id,
        );
    }
}

/// Spawn additional buildings for a player
fn spawn_player_buildings(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    model_assets: Option<&crate::rendering::model_loader::ModelAssets>,
    player_id: u8,
    base_pos: Vec3,
    get_terrain_position: &dyn Fn(f32, f32, f32) -> Vec3,
) {
    use crate::core::components::BuildingType;

    let building_spacing = 30.0;
    let x_sign = if player_id == 1 { 1.0 } else { -1.0 };
    let z_sign = if player_id == 1 { -1.0 } else { 1.0 };

    // Nursery
    spawn_building(
        commands,
        meshes,
        materials,
        model_assets,
        BuildingType::Nursery,
        get_terrain_position(
            base_pos.x - x_sign * building_spacing,
            base_pos.z + z_sign * building_spacing,
            0.0,
        ),
        player_id,
    );

    // Warrior Chamber
    spawn_building(
        commands,
        meshes,
        materials,
        model_assets,
        BuildingType::WarriorChamber,
        get_terrain_position(
            base_pos.x + x_sign * building_spacing,
            base_pos.z + z_sign * building_spacing,
            0.0,
        ),
        player_id,
    );
}

/// Helper to spawn a single unit
fn spawn_unit(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    model_assets: Option<&crate::rendering::model_loader::ModelAssets>,
    unit_type: crate::core::components::UnitType,
    position: Vec3,
    player_id: u8,
) {
    use crate::entities::entity_factory::{EntityFactory, SpawnConfig, EntityType};

    let config = SpawnConfig::unit(EntityType::from_unit(unit_type), position, player_id);
    EntityFactory::spawn(commands, meshes, materials, config, model_assets);
}

/// Helper to spawn a single building
fn spawn_building(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    model_assets: Option<&crate::rendering::model_loader::ModelAssets>,
    building_type: crate::core::components::BuildingType,
    position: Vec3,
    player_id: u8,
) {
    use crate::entities::entity_factory::{EntityFactory, SpawnConfig, EntityType};

    let config = SpawnConfig::building(EntityType::Building(building_type), position, player_id);
    EntityFactory::spawn(commands, meshes, materials, config, model_assets);
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
            
            // Spread out environment objects far across the map with large spacing
            let object_positions = [
                (800.0, 1200.0),    // Far northeast
                (-1200.0, 800.0),   // Far northwest  
                (0.0, -1000.0),     // Far south
                (1500.0, -500.0),   // Far southeast
                (-1500.0, -300.0),  // Far southwest
                (600.0, -600.0),    // Southeast quadrant
                (-800.0, 600.0),    // Northwest quadrant
                (200.0, 800.0),     // North-center
            ];
            
            for (i, &(x, z)) in object_positions.iter().enumerate() {
                let position = get_terrain_position(x, z, 1.0);
                let rotation = Quat::from_rotation_y(i as f32 * 1.2); // Slight rotation variation
                
                // Alternate between different resource types (cycle through all 4)
                let (insect_model_type, env_obj_type, model_handle, object_name) = match i % 4 {
                    0 => (InsectModelType::Mushrooms, EnvironmentObjectType::Mushrooms, &models.mushrooms, "Mushroom Cluster"),
                    1 => (InsectModelType::RiverRock, EnvironmentObjectType::Rocks, &models.river_rock, "Rock Formation"),
                    2 => (InsectModelType::WoodStick, EnvironmentObjectType::WoodStick, &models.wood_stick, "Wood Debris"),
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
                            amount: 300.0,
                            max_gatherers: 3,
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
                    _ => {} // Other objects (Grass, etc.) are not harvestable
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