use crate::core::components::*;
use crate::core::resources::PlayerResources;
use bevy::prelude::*;
use crate::core::game::GameState;
// Terrain asset no longer needed

// === RTS SETUP CONSTANTS ===
const INITIAL_CAMERA_HEIGHT: f32 = 400.0; // Increased from 200.0 for more zoomed out view
const INITIAL_CAMERA_DISTANCE: f32 = 200.0; // Increased from 100.0 for more zoomed out view
const LIGHT_ROTATION_X: f32 = -0.8;
const LIGHT_ROTATION_Y: f32 = -0.3;

#[allow(dead_code)]
const RESOURCE_SPAWN_AREAS: usize = 6;


pub fn setup_menu(mut commands: Commands) {
    // Spawn UI camera for menu
    commands.spawn((
        Camera2d,
        UI,
    ));

    commands.spawn((
        Text::new("BioRust - Press SPACE to Start"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(50.0),
            ..default()
        },
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
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

        // Transition to team selection state
        next_state.set(GameState::TeamSelection);
        info!("Transitioning to Team Selection state");
    }
}


/// Spawn minimal environment objects near center area only
fn spawn_minimal_environment_objects(
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    terrain_heights: &Res<crate::world::static_terrain::StaticTerrainHeights>,
    model_assets: Option<&crate::rendering::model_loader::ModelAssets>,
) {
    use crate::core::components::*;
    use crate::rendering::model_loader::*;

    // Helper function to get terrain-aware position
    let get_terrain_position = |x: f32, z: f32, height_offset: f32| -> Vec3 {
        let terrain_height = terrain_heights.get_height(x, z);
        Vec3::new(x, terrain_height + height_offset, z)
    };

    // Only spawn a few objects in the center area between player bases
    if let Some(models) = model_assets {
        if models.models_loaded {
            info!("Spawning distributed environment objects across the map");

            // All resources placed far from player bases - forces territorial expansion
            let object_positions = [
                // Mid-range contested resources (moved out from player base areas)
                (1500.0, 1500.0),  // Northeast quadrant
                (-1500.0, 1500.0), // Northwest quadrant  
                (1500.0, -1500.0), // Southeast quadrant
                (-1500.0, -1500.0), // Southwest quadrant
                (0.0, 1800.0),     // North contested
                (1800.0, 0.0),     // East contested  
                (0.0, -1800.0),    // South contested
                (-1800.0, 0.0),    // West contested
                // Intermediate distance resources
                (2000.0, 1000.0),  // Northeast intermediate
                (-2000.0, 1000.0), // Northwest intermediate
                (2000.0, -1000.0), // Southeast intermediate  
                (-2000.0, -1000.0), // Southwest intermediate
                (1000.0, 2000.0),  // North intermediate
                (-1000.0, 2000.0), // North intermediate
                (1000.0, -2000.0), // South intermediate
                (-1000.0, -2000.0), // South intermediate
                // Edge resources - far from player bases at map edges
                (-2300.0, 2300.0), // Far northwest edge
                (2300.0, 2300.0),  // Far northeast edge
                (-2300.0, -2300.0), // Far southwest edge
                (2300.0, -2300.0), // Far southeast edge
                // Additional edge resources at cardinal directions
                (0.0, 2400.0),   // Far north edge
                (0.0, -2400.0),  // Far south edge
                (-2400.0, 0.0),  // Far west edge
                (2400.0, 0.0),   // Far east edge
            ];

            for (i, &(x, z)) in object_positions.iter().enumerate() {
                let position = get_terrain_position(x, z, 1.0);
                let rotation = Quat::from_rotation_y(i as f32 * 1.2); // Slight rotation variation

                // More balanced resource distribution for testing
                let (insect_model_type, env_obj_type, model_handle, object_name) = match i % 4 {
                    // 25% Nectar sources (mushrooms)
                    0 => (
                        InsectModelType::Mushrooms,
                        EnvironmentObjectType::Mushrooms,
                        &models.mushrooms,
                        "Mushroom Cluster",
                    ),
                    1 => (
                        InsectModelType::RiverRock,
                        EnvironmentObjectType::Rocks,
                        &models.river_rock,
                        "Rock Formation",
                    ),
                    2 => (
                        InsectModelType::WoodStick,
                        EnvironmentObjectType::WoodStick,
                        &models.wood_stick,
                        "Wood Debris",
                    ),
                    _ => (
                        InsectModelType::PineCone,
                        EnvironmentObjectType::PineCone,
                        &models.pine_cone,
                        "Pine Cone Resource",
                    ),
                };

                // Skip if model handle is invalid
                if model_handle == &Handle::default() {
                    warn!(
                        "Invalid model handle for {:?}, skipping spawn",
                        insect_model_type
                    );
                    continue;
                }

                // Get base scale for this object type
                let base_scale =
                    crate::rendering::model_loader::get_model_scale(&insect_model_type);

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
                        radius: crate::constants::resource_interaction::RESOURCE_COLLISION_RADIUS,
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
                            amount: 800.0, // Increased from 300 to 800 for better economy
                            max_gatherers: 4, // Allow more gatherers for faster collection
                            current_gatherers: 0,
                        });
                        entity_commands.insert(Selectable {
                            is_selected: false,
                            selection_radius: (base_scale * 2.0).max(15.0), // Minimum 15.0 for easy clicking
                        });
                    }
                    EnvironmentObjectType::Rocks => {
                        entity_commands.insert(ResourceSource {
                            resource_type: ResourceType::Minerals,
                            amount: 500.0,
                            max_gatherers: 4,
                            current_gatherers: 0,
                        });
                        entity_commands.insert(Selectable {
                            is_selected: false,
                            selection_radius: (base_scale * 2.0).max(15.0), // Minimum 15.0 for easy clicking
                        });
                    }
                    EnvironmentObjectType::WoodStick => {
                        entity_commands.insert(ResourceSource {
                            resource_type: ResourceType::Chitin,
                            amount: 400.0,
                            max_gatherers: 3,
                            current_gatherers: 0,
                        });
                        entity_commands.insert(Selectable {
                            is_selected: false,
                            selection_radius: (base_scale * 2.0).max(15.0), // Minimum 15.0 for easy clicking
                        });
                    }
                    EnvironmentObjectType::PineCone => {
                        entity_commands.insert(ResourceSource {
                            resource_type: ResourceType::Pheromones,
                            amount: 350.0,
                            max_gatherers: 3,
                            current_gatherers: 0,
                        });
                        entity_commands.insert(Selectable {
                            is_selected: false,
                            selection_radius: (base_scale * 2.0).max(15.0), // Minimum 15.0 for easy clicking
                        });
                    }
                    EnvironmentObjectType::Hive => {
                        // Legacy hive case - no longer spawned but kept for compatibility
                        entity_commands.insert(ResourceSource {
                            resource_type: ResourceType::Pheromones,
                            amount: 350.0,
                            max_gatherers: 3,
                            current_gatherers: 0,
                        });
                        entity_commands.insert(Selectable {
                            is_selected: false,
                            selection_radius: (base_scale * 2.0).max(15.0), // Minimum 15.0 for easy clicking
                        });
                    }
                }

                info!(
                    "Spawned {} at {:?} (scale: {:.1})",
                    object_name.to_lowercase(),
                    position,
                    base_scale
                );
            }
        } else {
            info!("Models not yet loaded, skipping environment objects");
        }
    } else {
        info!("No model assets available, skipping environment objects");
    }

    info!("Minimal environment objects spawning complete");
}

/// Team-based RTS element spawning system
/// Spawns units based on selected teams instead of hardcoded units
pub fn spawn_rts_elements_with_teams(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_heights: Res<crate::world::static_terrain::StaticTerrainHeights>,
    model_assets: Option<Res<crate::rendering::model_loader::ModelAssets>>,
    game_setup: Option<Res<GameSetup>>,
    mut ai_resources: ResMut<crate::core::resources::AIResources>,
    mut economy_manager: ResMut<crate::ai::economy::EconomyManager>,
    mut intelligence: ResMut<crate::ai::intelligence::IntelligenceSystem>,
    mut ai_strategy: ResMut<crate::ai::strategy::AIStrategy>,
) {
    info!("🎮 SPAWN SYSTEM TRIGGERED: Starting team-based RTS element spawning");
    
    if game_setup.is_none() {
        error!("❌ No GameSetup found! spawn_rts_elements_with_teams called without team selection");
        return;
    }
    
    // Spawn 3D camera for gameplay
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, INITIAL_CAMERA_HEIGHT, INITIAL_CAMERA_DISTANCE)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        MainCamera,
        RTSCamera {
            move_speed: crate::constants::camera::CAMERA_MOVE_SPEED,
        },
    ));

    // Spawn directional light for 3D scene
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.8),
            illuminance: 32000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, LIGHT_ROTATION_X, LIGHT_ROTATION_Y, 0.0)),
    ));

    // Add ambient light for better terrain visibility
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.5, 0.5, 0.7),
        brightness: 500.0,
    });
    
    // Get game setup or use defaults
    let setup = game_setup.as_deref().cloned().unwrap_or_else(|| GameSetup {
        player_team: TeamType::BalancedColony,
        // Default to fewer AI teams for better performance
        ai_teams: vec![
            TeamType::Predators,
            TeamType::ShadowCrawlers,
        ],
        player_count: 3, // Updated to reflect actual player count (1 human + 2 AI)
    });

    info!("🎯 SPAWN CONFIG: Player team: {:?}, AI teams: {:?}, Total players: {}", 
          setup.player_team, setup.ai_teams, setup.ai_teams.len() + 1);

    use crate::entities::entity_factory::{EntityFactory, EntityType, SpawnConfig};

    // Helper function to get terrain-aware position
    let get_terrain_position = |x: f32, z: f32, height_offset: f32| -> Vec3 {
        let terrain_height = terrain_heights.get_height(x, z);
        Vec3::new(x, terrain_height + height_offset, z)
    };

    // === PLAYER 1 (Human) - Left side of map ===
    let player1_base_2d = Vec3::new(-800.0, 0.0, 0.0);
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

    // Spawn player team units with debug mode for model visibility  
    let player_roster = setup.player_team.get_unit_roster();
    let unit_spacing = 15.0;
    let mut spawn_index = 0;
    
    // If this is 8-player mode (7 AI + 1 human), spawn more units for model debugging
    let units_to_spawn = if setup.ai_teams.len() >= 7 { 
        player_roster.len().min(12) // Spawn all units (up to 12) for full model showcase
    } else {
        player_roster.len().min(6) // Normal mode - spawn up to 6 units
    };
    
    for (_i, unit_type) in player_roster.iter().enumerate().take(units_to_spawn) {
        let offset_x = (spawn_index % 4) as f32 * unit_spacing; // 4 units per row for better layout
        let offset_z = (spawn_index / 4) as f32 * unit_spacing + 30.0;
        
        let unit_config = SpawnConfig::unit(
            EntityType::from_unit(unit_type.clone()),
            get_terrain_position(player1_base.x + offset_x, player1_base.z + offset_z, 2.0),
            1,
        );
        EntityFactory::spawn(
            &mut commands,
            &mut meshes,
            &mut materials,
            unit_config,
            model_assets.as_deref(),
        );
        
        info!("🐛 DEBUG: Spawned {} (team: {:?}) for player 1", 
              format!("{:?}", unit_type), setup.player_team);
        spawn_index += 1;
    }
    
    // Add extra resource gatherers for player 1 (start with at least 5 total)
    let primary_gatherer = get_primary_gatherer(&setup.player_team);
    let gatherers_in_roster = player_roster.iter()
        .filter(|unit| is_gatherer(unit))
        .count();
    let extra_gatherers_needed = 5_usize.saturating_sub(gatherers_in_roster);
    
    for _ in 0..extra_gatherers_needed {
        let offset_x = (spawn_index % 4) as f32 * unit_spacing;
        let offset_z = (spawn_index / 4) as f32 * unit_spacing + 30.0;
        
        let gatherer_config = SpawnConfig::unit(
            EntityType::from_unit(primary_gatherer.clone()),
            get_terrain_position(player1_base.x + offset_x, player1_base.z + offset_z, 2.0),
            1,
        );
        EntityFactory::spawn(
            &mut commands,
            &mut meshes,
            &mut materials,
            gatherer_config,
            model_assets.as_deref(),
        );
        
        info!("🐛 EXTRA: Spawned additional {} gatherer for player 1", format!("{:?}", primary_gatherer));
        spawn_index += 1;
    }

    // Spawn player buildings
    let building_spacing = 30.0;

    // Nursery (house equivalent)
    let nursery_config = SpawnConfig::building(
        EntityType::Building(crate::core::components::BuildingType::Nursery),
        get_terrain_position(
            player1_base.x - building_spacing,
            player1_base.z - building_spacing,
            0.0,
        ),
        1,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        nursery_config,
        model_assets.as_deref(),
    );

    // Warrior Chamber (barracks equivalent)
    let warrior_config = SpawnConfig::building(
        EntityType::Building(crate::core::components::BuildingType::WarriorChamber),
        get_terrain_position(
            player1_base.x + building_spacing,
            player1_base.z - building_spacing,
            0.0,
        ),
        1,
    );
    EntityFactory::spawn(
        &mut commands,
        &mut meshes,
        &mut materials,
        warrior_config,
        model_assets.as_deref(),
    );

    // Store player team info
    commands.spawn(PlayerTeam {
        team_type: setup.player_team.clone(),
        player_id: 1,
    });

    // === AI PLAYERS ===
    let base_positions = vec![
        Vec3::new(800.0, 0.0, 0.0),      // Player 2 (right)
        Vec3::new(0.0, 0.0, 800.0),      // Player 3 (top)
        Vec3::new(0.0, 0.0, -800.0),     // Player 4 (bottom)
        Vec3::new(-600.0, 0.0, 600.0),   // Player 5 (top-left)
        Vec3::new(600.0, 0.0, 600.0),    // Player 6 (top-right)
        Vec3::new(-600.0, 0.0, -600.0),  // Player 7 (bottom-left)
        Vec3::new(600.0, 0.0, -600.0),   // Player 8 (bottom-right)
    ];

    for (ai_index, ai_team) in setup.ai_teams.iter().enumerate() {
        let player_id = (ai_index + 2) as u8; // AI players start from ID 2
        
        // Add AI player to resource system
        ai_resources.add_ai_player(player_id);
        info!("🏧 Added AI player {} to resources system", player_id);
        
        // Add AI player to economy management system 
        economy_manager.add_ai_player(player_id);
        
        // Initialize intelligence system for this AI player (monitoring player 1)
        intelligence.initialize_player(player_id, 1);
        info!("🧠 Added AI player {} to intelligence system", player_id);
        
        // Initialize strategy system for this AI player
        ai_strategy.add_ai_player(player_id);
        info!("🎯 Added AI player {} to strategy system", player_id);
        
        if let Some(&base_pos_2d) = base_positions.get(ai_index) {
            let ai_base = get_terrain_position(base_pos_2d.x, base_pos_2d.z, 0.0);

            // Spawn Queen Chamber for AI
            let ai_queen_config = SpawnConfig::building(
                EntityType::Building(crate::core::components::BuildingType::Queen),
                ai_base,
                player_id,
            );
            EntityFactory::spawn(
                &mut commands,
                &mut meshes,
                &mut materials,
                ai_queen_config,
                model_assets.as_deref(),
            );

            // Spawn AI team units with debug mode for model visibility
            let ai_roster = ai_team.get_unit_roster();
            let mut ai_spawn_index = 0;
            
            // If this is 8-player mode (7 AI + 1 human), spawn more units for model debugging
            let units_to_spawn = if setup.ai_teams.len() >= 7 { 
                ai_roster.len().min(12) // Spawn all units (up to 12) for full model showcase
            } else {
                ai_roster.len().min(6) // Normal mode - spawn up to 6 units
            };
            
            for (_i, unit_type) in ai_roster.iter().enumerate().take(units_to_spawn) {
                let offset_x = (ai_spawn_index % 4) as f32 * unit_spacing; // 4 units per row for better layout
                let offset_z = (ai_spawn_index / 4) as f32 * unit_spacing - 30.0; // Opposite side from player
                
                let ai_unit_config = SpawnConfig::unit(
                    EntityType::from_unit(unit_type.clone()),
                    get_terrain_position(ai_base.x + offset_x, ai_base.z + offset_z, 2.0),
                    player_id,
                );
                EntityFactory::spawn(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    ai_unit_config,
                    model_assets.as_deref(),
                );
                
                info!("🐛 DEBUG: Spawned {} (team: {:?}) for player {}", 
                      format!("{:?}", unit_type), ai_team, player_id);
                ai_spawn_index += 1;
            }
            
            // Add extra resource gatherers for AI teams
            let ai_primary_gatherer = get_primary_gatherer(ai_team);
            let ai_gatherers_in_roster = ai_roster.iter()
                .filter(|unit| is_gatherer(unit))
                .count();
            
            // Give Tiny Legion 10 gatherers, others get 5
            let min_gatherers: usize = if matches!(ai_team, TeamType::TinyLegion) { 10 } else { 5 };
            let ai_extra_gatherers_needed = min_gatherers.saturating_sub(ai_gatherers_in_roster);
            
            for _ in 0..ai_extra_gatherers_needed {
                let offset_x = (ai_spawn_index % 4) as f32 * unit_spacing;
                let offset_z = (ai_spawn_index / 4) as f32 * unit_spacing - 30.0;
                
                let ai_gatherer_config = SpawnConfig::unit(
                    EntityType::from_unit(ai_primary_gatherer.clone()),
                    get_terrain_position(ai_base.x + offset_x, ai_base.z + offset_z, 2.0),
                    player_id,
                );
                EntityFactory::spawn(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    ai_gatherer_config,
                    model_assets.as_deref(),
                );
                
                info!("🐛 EXTRA: Spawned additional {} gatherer for AI player {} ({})", 
                      format!("{:?}", ai_primary_gatherer), player_id, 
                      if matches!(ai_team, TeamType::TinyLegion) { "TinyLegion" } else { "Standard" });
                ai_spawn_index += 1;
            }

            // Spawn AI buildings
            let ai_nursery_config = SpawnConfig::building(
                EntityType::Building(crate::core::components::BuildingType::Nursery),
                get_terrain_position(
                    ai_base.x + building_spacing,
                    ai_base.z + building_spacing,
                    0.0,
                ),
                player_id,
            );
            EntityFactory::spawn(
                &mut commands,
                &mut meshes,
                &mut materials,
                ai_nursery_config,
                model_assets.as_deref(),
            );

            let ai_warrior_config = SpawnConfig::building(
                EntityType::Building(crate::core::components::BuildingType::WarriorChamber),
                get_terrain_position(
                    ai_base.x - building_spacing,
                    ai_base.z + building_spacing,
                    0.0,
                ),
                player_id,
            );
            EntityFactory::spawn(
                &mut commands,
                &mut meshes,
                &mut materials,
                ai_warrior_config,
                model_assets.as_deref(),
            );

            // Store AI team info
            commands.spawn(PlayerTeam {
                team_type: ai_team.clone(),
                player_id,
            });

            info!("Spawned AI player {} with team: {:?}", player_id, ai_team);
        }
    }

    // === NEUTRAL RESOURCES ===
    // Resources are now spawned as environment objects (mushrooms, rocks) with ResourceSource components
    // This provides better visual integration and uses GLB models instead of primitive shapes

    // === ENVIRONMENT OBJECTS ===
    spawn_minimal_environment_objects(
        &mut commands,
        &mut meshes,
        &mut materials,
        &terrain_heights,
        model_assets.as_ref().map(|v| &**v),
    );

    info!("Team-based RTS elements spawned");
}

/// Setup game UI when entering Playing state
pub fn setup_game_ui(
    mut commands: Commands,
    ui_icons: Res<crate::ui::icons::UIIcons>,
    _game_costs: Res<crate::core::resources::GameCosts>,
) {
    use crate::ui::building_panel::setup_building_panel;
    use crate::ui::resource_display::setup_resource_display;
    use crate::constants::resources::*;

    // Setup basic building UI
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            setup_resource_display(parent, &ui_icons);
            setup_building_panel(parent, &ui_icons, &_game_costs);
        });

    // Initialize player resources (will sync with main PlayerResources)
    commands.insert_resource(PlayerResources {
        nectar: STARTING_NECTAR,
        chitin: STARTING_CHITIN,
        minerals: STARTING_MINERALS,
        pheromones: STARTING_PHEROMONES,
        current_population: 0,
        max_population: STARTING_POPULATION_LIMIT,
    });

    info!("Game UI setup complete");
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
    terrain_heights: Res<crate::world::static_terrain::StaticTerrainHeights>,
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
            movement += Vec3::new(0.0, 0.0, -1.0); // Forward/North (negative Z)
        }
        if keyboard.pressed(KeyCode::KeyS) {
            movement += Vec3::new(0.0, 0.0, 1.0); // Backward/South (positive Z)
        }
        if keyboard.pressed(KeyCode::KeyA) {
            movement += Vec3::new(-1.0, 0.0, 0.0); // West (negative X)
        }
        if keyboard.pressed(KeyCode::KeyD) {
            movement += Vec3::new(1.0, 0.0, 0.0); // East (positive X)
        }

        // Apply movement with multiple speed modes
        if movement.length() > 0.0 {
            let speed_multiplier = if keyboard.pressed(KeyCode::ControlLeft) {
                0.1
            }
            // Very slow
            else if keyboard.pressed(KeyCode::ShiftLeft) {
                10.0
            }
            // Very fast
            else if keyboard.pressed(KeyCode::AltLeft) {
                50.0
            }
            // Hyper speed
            else {
                1.0
            }; // Normal
            movement = movement.normalize() * rts_camera.move_speed * speed_multiplier * dt;
            camera_transform.translation += movement;

            // Apply camera boundary limits
            use crate::constants::movement::CAMERA_BOUNDARY;
            camera_transform.translation.x = camera_transform
                .translation
                .x
                .clamp(-CAMERA_BOUNDARY, CAMERA_BOUNDARY);
            camera_transform.translation.z = camera_transform
                .translation
                .z
                .clamp(-CAMERA_BOUNDARY, CAMERA_BOUNDARY);
        }

        // Apply terrain following if enabled
        if terrain_following {
            // Sample terrain height at camera position and nearby points for slope calculation
            let current_pos = camera_transform.translation;
            let terrain_height = terrain_heights.get_height(current_pos.x, current_pos.z);

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
                    crate::constants::camera::MAX_HEIGHT_ABOVE_TERRAIN,
                );
            }
        } else if !keyboard.pressed(KeyCode::KeyF) {
            // Standard height clamping when not terrain following
            camera_transform.translation.y = camera_transform.translation.y.clamp(
                crate::constants::camera::MIN_HEIGHT_ABOVE_TERRAIN,
                crate::constants::camera::MAX_HEIGHT_ABOVE_TERRAIN,
            );
        }

        // True zoom in/out using vertical camera movement with terrain constraints
        for wheel_event in mouse_wheel.read() {
            use crate::constants::camera::*;
            let zoom_speed_multiplier = if keyboard.pressed(KeyCode::ShiftLeft) {
                ZOOM_SPEED_FAST_MULTIPLIER
            } else if keyboard.pressed(KeyCode::AltLeft) {
                ZOOM_SPEED_HYPER_MULTIPLIER
            } else {
                1.0
            };

            // More responsive zoom - don't use dt for scroll wheel as it's event-based
            let zoom_delta = -wheel_event.y * SCROLL_ZOOM_SENSITIVITY * zoom_speed_multiplier;

            // Calculate new camera height
            let current_position = camera_transform.translation;
            let new_height = current_position.y + zoom_delta;

            // Sample terrain height at current camera position
            let terrain_height = terrain_heights.get_height(current_position.x, current_position.z);

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

// Helper functions for gatherer spawning

/// Get the primary gatherer unit type for a team
fn get_primary_gatherer(team: &TeamType) -> UnitType {
    use crate::core::components::*;
    match team {
        TeamType::BeetleSwarm => UnitType::DungBeetle,
        TeamType::Predators => UnitType::Silverfish,
        TeamType::SkyDominion => UnitType::Honeybees,
        TeamType::TinyLegion => UnitType::Aphids,
        TeamType::BalancedColony => UnitType::WorkerAnt,
        TeamType::AntEmpire => UnitType::WorkerAnt,
        TeamType::ShadowCrawlers => UnitType::Silverfish,
        TeamType::HiveMind => UnitType::Honeybees,
    }
}

/// Check if a unit type is a resource gatherer
fn is_gatherer(unit_type: &UnitType) -> bool {
    use crate::core::components::*;
    matches!(unit_type, 
        UnitType::WorkerAnt | UnitType::WorkerFourmi | UnitType::TermiteWorker |
        UnitType::DungBeetle | UnitType::Silverfish | UnitType::Honeybees |
        UnitType::HoneyBee | UnitType::Aphids | UnitType::Mites
    )
}

// Removed old unused systems (update_enemies, handle_collisions, update_ui)
// These were replaced by the RTS systems
