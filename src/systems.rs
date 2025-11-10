use bevy::prelude::*;
use crate::components::*;
// Removed unused import: use crate::resources::*;
use crate::game::GameState;
// Terrain asset no longer needed

// === RTS SETUP CONSTANTS ===
const INITIAL_CAMERA_HEIGHT: f32 = 200.0;
const INITIAL_CAMERA_DISTANCE: f32 = 100.0;
const CAMERA_PITCH_ANGLE: f32 = -0.8;
const LIGHT_ROTATION_X: f32 = -0.8;
const LIGHT_ROTATION_Y: f32 = -0.3;

// === SPAWNING CONSTANTS ===
const PLAYER_1_BASE_POS: Vec3 = Vec3::new(-80.0, 10.0, -80.0);
const PLAYER_2_BASE_POS: Vec3 = Vec3::new(80.0, 10.0, 80.0);
const WORKER_ANT_COUNT_PER_PLAYER: usize = 8;
const WORKER_ANT_SPAWN_RADIUS_BASE: f32 = 30.0;
const WORKER_ANT_SPAWN_RADIUS_INCREMENT: f32 = 5.0;
const BUILDING_SPAWN_HEIGHT: f32 = 10.0;
#[allow(dead_code)]
const RESOURCE_SPAWN_AREAS: usize = 6;
const MILITARY_UNIT_COUNT: usize = 6;
const ARCHER_COUNT: usize = 4;
const WOOD_RESOURCE_COUNT: usize = 12;
const STONE_RESOURCE_COUNT: usize = 8;
const GOLD_RESOURCE_COUNT: usize = 6;

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
    terrain_manager: Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::terrain_v2::TerrainSettings>,
    model_assets: Option<Res<crate::model_loader::ModelAssets>>,
) {
    info!("=== SPAWNING RTS ELEMENTS ===");
    // Spawn RTS units and buildings - one of each type for testing
    use crate::entity_factory::{EntityFactory, SpawnConfig, EntityType};
    use crate::rts_entities::RTSEntityFactory;
    
    // Helper function to get terrain-aware position
    let get_terrain_position = |x: f32, z: f32, height_offset: f32| -> Vec3 {
        let terrain_height = crate::terrain_v2::sample_terrain_height(
            x, z, &terrain_manager.noise_generator, &terrain_settings
        );
        Vec3::new(x, terrain_height + height_offset, z)
    };
    
    // === PLAYER 1 (Human) - Left side of map ===
    let player1_base_2d = Vec3::new(-200.0, 0.0, 0.0);
    let player1_base = get_terrain_position(player1_base_2d.x, player1_base_2d.z, 0.0);
    
    // Spawn Queen Chamber (main building)
    RTSEntityFactory::spawn_queen_chamber(
        &mut commands,
        &mut meshes,
        &mut materials,
        player1_base,
        1,
    );
    
    // Spawn one of each unit type in a grid formation
    let unit_spacing = 15.0;
    let _units_per_row = 4;
    
    // Worker units using new EntityFactory with animations
    let worker_config = SpawnConfig::unit(
        EntityType::from_unit(crate::components::UnitType::WorkerAnt),
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
        EntityType::from_unit(crate::components::UnitType::SoldierAnt),
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
        EntityType::from_unit(crate::components::UnitType::HunterWasp),
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
        EntityType::from_unit(crate::components::UnitType::BeetleKnight),
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
    RTSEntityFactory::spawn_nursery(
        &mut commands, &mut meshes, &mut materials,
        get_terrain_position(player1_base.x - building_spacing, player1_base.z - building_spacing, 0.0),
        1,
    );
    
    // Warrior Chamber (barracks equivalent)
    RTSEntityFactory::spawn_warrior_chamber(
        &mut commands, &mut meshes, &mut materials,
        get_terrain_position(player1_base.x + building_spacing, player1_base.z - building_spacing, 0.0),
        1,
    );
    
    // Note: Additional building types would need spawn functions created
    
    // === PLAYER 2 (Enemy) - Right side of map ===
    let player2_base_2d = Vec3::new(200.0, 0.0, 0.0);
    let player2_base = get_terrain_position(player2_base_2d.x, player2_base_2d.z, 0.0);
    
    // Spawn Queen Chamber for enemy
    RTSEntityFactory::spawn_queen_chamber(
        &mut commands,
        &mut meshes,
        &mut materials,
        player2_base,
        2,
    );
    
    // Enemy units - same types but different positions using new EntityFactory
    let enemy_worker_config = SpawnConfig::unit(
        EntityType::from_unit(crate::components::UnitType::WorkerAnt),
        get_terrain_position(player2_base.x, player2_base.z - 30.0, 2.0),
        2,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        enemy_worker_config,
        model_assets.as_deref(),
    );
    
    let enemy_soldier_config = SpawnConfig::unit(
        EntityType::from_unit(crate::components::UnitType::SoldierAnt),
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
        EntityType::from_unit(crate::components::UnitType::HunterWasp),
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
        EntityType::from_unit(crate::components::UnitType::BeetleKnight),
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
    RTSEntityFactory::spawn_nursery(
        &mut commands, &mut meshes, &mut materials,
        get_terrain_position(player2_base.x + building_spacing, player2_base.z + building_spacing, 0.0),
        2,
    );
    
    RTSEntityFactory::spawn_warrior_chamber(
        &mut commands, &mut meshes, &mut materials,
        get_terrain_position(player2_base.x - building_spacing, player2_base.z + building_spacing, 0.0),
        2,
    );
    
    // === NEUTRAL RESOURCES ===
    // Organized resource spawning for better map layout
    
    // Chitin sources (north forest area)
    let chitin_positions_2d = [
        (-100.0, 100.0),
        (-60.0, 120.0),
        (-140.0, 80.0),
        (100.0, 100.0),
        (60.0, 120.0),
        (140.0, 80.0),
    ];
    
    for (x, z) in chitin_positions_2d.iter() {
        RTSEntityFactory::spawn_chitin_source(
            &mut commands,
            &mut meshes,
            &mut materials,
            get_terrain_position(*x, *z, 1.0),
        );
    }
    
    // Mineral deposits (center area)
    let mineral_positions_2d = [
        (0.0, 150.0),
        (-50.0, -150.0),
        (50.0, -150.0),
        (0.0, -100.0),
    ];
    
    for (x, z) in mineral_positions_2d.iter() {
        RTSEntityFactory::spawn_mineral_deposit(
            &mut commands,
            &mut meshes,
            &mut materials,
            get_terrain_position(*x, *z, 0.5),
        );
    }
    
    // Pheromone caches (rare, scattered)
    let pheromone_positions_2d = [
        (0.0, 0.0),   // Center of map
        (-150.0, -50.0),
        (150.0, 50.0),
    ];
    
    for (x, z) in pheromone_positions_2d.iter() {
        RTSEntityFactory::spawn_pheromone_cache(
            &mut commands,
            &mut meshes,
            &mut materials,
            get_terrain_position(*x, *z, 1.5),
        );
    }
    
    // === ENVIRONMENT OBJECTS ===
    spawn_environment_objects(&mut commands, &mut meshes, &mut materials, &terrain_manager, &terrain_settings, model_assets.as_ref().map(|v| &**v));
    
    info!("RTS elements spawned");
}

/// Spawn environment objects randomly throughout the scene
fn spawn_environment_objects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    terrain_manager: &Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: &Res<crate::terrain_v2::TerrainSettings>,
    model_assets: Option<&crate::model_loader::ModelAssets>,
) {
    use crate::model_loader::*;
    use crate::components::*;
    use rand::{Rng, thread_rng};
    
    // Helper function to get terrain-aware position
    let get_terrain_position = |x: f32, z: f32, height_offset: f32| -> Vec3 {
        let terrain_height = crate::terrain_v2::sample_terrain_height(
            x, z, &terrain_manager.noise_generator, &terrain_settings
        );
        Vec3::new(x, terrain_height + height_offset, z)
    };
    
    let mut rng = thread_rng();
    
    // Define spawn zones to avoid player bases and resources
    let spawn_zones = [
        // Northwestern area
        (-600.0, -500.0, 500.0, 600.0),   // (min_x, max_x, min_z, max_z)
        // Northeastern area  
        (500.0, 600.0, 500.0, 600.0),
        // Southwestern area
        (-600.0, -500.0, -600.0, -500.0),
        // Southeastern area
        (500.0, 600.0, -600.0, -500.0),
        // Central northern area
        (-100.0, 100.0, 400.0, 600.0),
        // Central southern area
        (-100.0, 100.0, -600.0, -400.0),
    ];
    
    // Spawn objects if models are available
    if let Some(models) = model_assets {
        if models.models_loaded {
            info!("Spawning environment objects with GLB models (models_loaded = true)");
            
            // Spawn diverse environment objects throughout the map
            for &(min_x, max_x, min_z, max_z) in &spawn_zones {
                let objects_per_zone = rng.gen_range(2..5); // 2-4 objects per zone for more variety
                
                for _ in 0..objects_per_zone {
                    let x = rng.gen_range(min_x..max_x);
                    let z = rng.gen_range(min_z..max_z);
                    let position = get_terrain_position(x, z, 1.0); // Lift objects slightly above terrain
                    
                    // Random rotation for natural look
                    let rotation = Quat::from_rotation_y(rng.gen_range(0.0..std::f32::consts::TAU));
                    
                    // Choose random object type with weighted distribution
                    let object_type = match rng.gen_range(0..100) {
                        // Original objects (60% total)
                        0..=15 => (InsectModelType::Mushrooms, EnvironmentObjectType::Mushrooms, &models.mushrooms, "Mushroom Cluster"),
                        16..=25 => (InsectModelType::Grass, EnvironmentObjectType::Grass, &models.grass, "Grass Patch"),
                        26..=35 => (InsectModelType::Grass2, EnvironmentObjectType::Grass, &models.grass_2, "Grass Variant"),
                        // 36..=50 => (InsectModelType::StickShelter, EnvironmentObjectType::StickShelter, &models.stick_shelter, "Stick Shelter"),
                        51..=60 => (InsectModelType::SimpleGrassChunks, EnvironmentObjectType::Grass, &models.simple_grass_chunks, "Grass Chunks"),
                        
                        // New objects (40% total)
                        // 61..=65 => (InsectModelType::CherryBlossomTree, EnvironmentObjectType::Trees, &models.cherry_blossom_tree, "Cherry Blossom Tree"),
                        // 66..=70 => (InsectModelType::TreesPack, EnvironmentObjectType::Trees, &models.trees_pack, "Realistic Tree"),
                        71..=75 => (InsectModelType::BeechFern, EnvironmentObjectType::Plants, &models.beech_fern, "Beech Fern"),
                        76..=80 => (InsectModelType::PlantsAssetSet, EnvironmentObjectType::Plants, &models.plants_asset_set, "Plant Collection"),
                        81..=85 => (InsectModelType::RiverRock, EnvironmentObjectType::Rocks, &models.river_rock, "River Rock"),
                        86..=90 => (InsectModelType::SmallRocks, EnvironmentObjectType::Rocks, &models.small_rocks, "Small Rocks"),
                        91..=95 => (InsectModelType::PineCone, EnvironmentObjectType::ForestDebris, &models.pine_cone, "Pine Cone"),
                        96..=99 => (InsectModelType::WoodStick, EnvironmentObjectType::WoodStick, &models.wood_stick, "Wood Stick"),
                        
                        _ => (InsectModelType::Mushrooms, EnvironmentObjectType::Mushrooms, &models.mushrooms, "Mushroom Cluster"),
                    };
                    
                    // Check if the model handle is valid before using it
                    if object_type.2 == &Handle::default() {
                        warn!("Invalid model handle for {:?}, skipping spawn", object_type.0);
                        continue;
                    }
                    
                    // Store values before they're moved
                    let (insect_model_type, env_obj_type, model_handle, object_name) = object_type;
                    
                    // Get scale for this object type
                    let base_scale = crate::model_loader::get_model_scale(&insect_model_type);
                    let scale_variation = rng.gen_range(0.8..1.3);
                    let final_scale = base_scale * scale_variation;
                    
                    let _entity = commands.spawn((
                        SceneRoot(model_handle.clone()),
                        Transform::from_translation(position)
                            .with_rotation(rotation)
                            .with_scale(Vec3::splat(final_scale)),
                        InsectModel {
                            model_type: insect_model_type.clone(),
                            scale: final_scale,
                        },
                        UseGLBModel,
                        EnvironmentObject {
                            object_type: env_obj_type,
                        },
                        CollisionRadius { 
                            radius: final_scale * 1.2 // Smaller collision for navigation
                        },
                        Position {
                            translation: position,
                            rotation,
                        },
                        GameEntity,
                        Name::new(object_name),
                    )).id();
                    
                    info!("Spawned {} at {:?} (scale: {:.1}, model_type: {:?})", object_name.to_lowercase(), position, final_scale, insect_model_type);
                }
            }
        } else {
            info!("Models not yet loaded, skipping environment objects (models_loaded = false)");
        }
    } else {
        info!("No model assets available, spawning basic environment objects");
        
        // Fallback: spawn at least some primitive environment objects to test spawning logic
        
        // Fallback: spawn simple primitive shapes as placeholders
        for &(min_x, max_x, min_z, max_z) in &spawn_zones {
            let objects_per_zone = rng.gen_range(5..10); // Fewer fallback objects
            
            for _ in 0..objects_per_zone {
                let x = rng.gen_range(min_x..max_x);
                let z = rng.gen_range(min_z..max_z);
                let position = get_terrain_position(x, z, 0.5);
                
                // Create a simple mushroom-like shape with cylinder + sphere
                let _stem_entity = commands.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.5, 2.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.8, 0.7, 0.6), // Light brown stem
                        ..default()
                    })),
                    Transform::from_translation(position),
                    EnvironmentObject {
                        object_type: EnvironmentObjectType::Mushrooms,
                    },
                    GameEntity,
                    Name::new("Mushroom Stem (Primitive)"),
                )).id();
                
                let _cap_entity = commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(1.5))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.7, 0.4, 0.3), // Reddish brown cap
                        ..default()
                    })),
                    Transform::from_translation(position + Vec3::new(0.0, 1.5, 0.0)),
                    EnvironmentObject {
                        object_type: EnvironmentObjectType::Mushrooms,
                    },
                    GameEntity,
                    Name::new("Mushroom Cap (Primitive)"),
                )).id();
                
                info!("Spawned primitive mushroom at {:?}", position);
            }
        }
    }
    
    info!("Environment objects spawning complete");
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
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut camera_query: Query<(&mut Transform, &mut RTSCamera), With<MainCamera>>,
    terrain_manager: Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::terrain_v2::TerrainSettings>,
    time: Res<Time>,
) {
    if let Ok((mut camera_transform, mut rts_camera)) = camera_query.get_single_mut() {
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
            let terrain_height = crate::terrain_v2::sample_terrain_height(
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
                    rts_camera.min_height, 
                    rts_camera.max_height
                );
            }
        } else if !keyboard.pressed(KeyCode::KeyF) {
            // Standard height clamping when not terrain following
            camera_transform.translation.y = camera_transform.translation.y.clamp(
                rts_camera.min_height, 
                rts_camera.max_height
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
            let terrain_height = crate::terrain_v2::sample_terrain_height(
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