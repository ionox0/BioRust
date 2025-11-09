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
const VILLAGER_COUNT_PER_PLAYER: usize = 8;
const VILLAGER_SPAWN_RADIUS_BASE: f32 = 30.0;
const VILLAGER_SPAWN_RADIUS_INCREMENT: f32 = 5.0;
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
) {
    // Spawn RTS units and buildings
    use crate::rts_entities::RTSEntityFactory;
    
    // PLAYER 1 BASE (Blue - Left side)
    RTSEntityFactory::spawn_town_center(
        &mut commands,
        &mut meshes,
        &mut materials,
        PLAYER_1_BASE_POS,
        1,
    );
    
    // Player 1 villagers scattered around base
    for i in 0..VILLAGER_COUNT_PER_PLAYER {
        let angle = (i as f32 / VILLAGER_COUNT_PER_PLAYER as f32) * std::f32::consts::TAU;
        let radius = VILLAGER_SPAWN_RADIUS_BASE + (i as f32 * VILLAGER_SPAWN_RADIUS_INCREMENT);
        RTSEntityFactory::spawn_villager(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(PLAYER_1_BASE_POS.x + angle.cos() * radius, BUILDING_SPAWN_HEIGHT / 2.0, PLAYER_1_BASE_POS.z + angle.sin() * radius),
            1,
            (i + 1) as u32,
        );
    }
    
    // Player 1 military units in formation
    for i in 0..MILITARY_UNIT_COUNT {
        RTSEntityFactory::spawn_militia(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(-50.0 + (i % 3) as f32 * 8.0, BUILDING_SPAWN_HEIGHT / 2.0, -60.0 + (i / 3) as f32 * 8.0),
            1,
            (i + 20) as u32,
        );
    }
    
    for i in 0..ARCHER_COUNT {
        RTSEntityFactory::spawn_archer(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(-70.0 + i as f32 * 8.0, BUILDING_SPAWN_HEIGHT / 2.0, -40.0),
            1,
            (i + 30) as u32,
        );
    }
    
    // Player 1 buildings
    RTSEntityFactory::spawn_house(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-120.0, BUILDING_SPAWN_HEIGHT / 2.0, -100.0),
        1,
    );
    
    RTSEntityFactory::spawn_house(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-120.0, BUILDING_SPAWN_HEIGHT / 2.0, -60.0),
        1,
    );
    
    RTSEntityFactory::spawn_barracks(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-40.0, BUILDING_SPAWN_HEIGHT / 2.0, -120.0),
        1,
    );
    
    // PLAYER 2 BASE (Red - Right side) - AI Player
    crate::ai::spawn_ai_player(
        &mut commands,
        &mut meshes,
        &mut materials,
        2,
        crate::ai::AIType::Aggressive,
        PLAYER_2_BASE_POS,
    );
    
    // NEUTRAL AREAS AND RESOURCES
    
    // Forest areas (wood resources)
    for i in 0..WOOD_RESOURCE_COUNT {
        let x = -150.0 + (i % 4) as f32 * 25.0;
        let z = -20.0 + (i / 4) as f32 * 15.0;
        RTSEntityFactory::spawn_wood_resource(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(x, BUILDING_SPAWN_HEIGHT / 2.0, z),
        );
    }
    
    for i in 0..STONE_RESOURCE_COUNT {
        let x = 20.0 + (i % 4) as f32 * 20.0;
        let z = 120.0 + (i / 4) as f32 * 15.0;
        RTSEntityFactory::spawn_wood_resource(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(x, BUILDING_SPAWN_HEIGHT / 2.0, z),
        );
    }
    
    // Stone quarries
    for i in 0..4 {
        RTSEntityFactory::spawn_stone_resource(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(100.0 + i as f32 * 30.0, BUILDING_SPAWN_HEIGHT / 2.0, -100.0 + i as f32 * 20.0),
        );
    }
    
    RTSEntityFactory::spawn_stone_resource(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-120.0, BUILDING_SPAWN_HEIGHT / 2.0, 120.0),
    );
    
    // Gold mines (valuable but limited)
    for i in 0..GOLD_RESOURCE_COUNT {
        RTSEntityFactory::spawn_gold_resource(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(i as f32 * 40.0 - 100.0, BUILDING_SPAWN_HEIGHT / 2.0, i as f32 * 30.0 - 60.0),
        );
    }
    
    // SCATTERED NEUTRAL UNITS FOR EXPLORATION
    // Some neutral villagers that could be recruited
    for i in 0..3 {
        RTSEntityFactory::spawn_villager(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(-20.0 + i as f32 * 40.0, 5.0, -20.0 + i as f32 * 40.0),
            0, // Neutral player
            200 + i,
        );
    }
    
    // Add some clouds for atmosphere
    for i in 0..5 {
        let x = (i as f32 - 2.0) * 100.0;
        let z = (i as f32 - 2.0) * 80.0;
        
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(8.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, 0.7),
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(Vec3::new(x, 40.0, z)),
            // Cloud marker removed
        ));
    }
    
    info!("RTS elements spawned");
}

/// Enhanced RTS camera system with terrain-aware scroll wheel zoom
/// 
/// Controls:
/// - WASD: Move camera relative to view direction
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
        
        // Get camera's forward and right vectors
        let forward = camera_transform.forward();
        let right = camera_transform.right();
        let up = Vec3::Y;
        
        // WASD movement relative to camera direction
        if keyboard.pressed(KeyCode::KeyW) {
            movement += *forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            movement -= *forward;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            movement -= *right;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            movement += *right;
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
                
                // Helpful logging for zoom behavior
                info!("Zoom: {:.1} -> {:.1} (Terrain: {:.1}, Limits: {:.1}-{:.1})", 
                      current_position.y, constrained_height, terrain_height, 
                      absolute_min_height, absolute_max_height);
            }
        }
    }
}


// Removed old unused systems (update_enemies, handle_collisions, update_ui)
// These were replaced by the RTS systems