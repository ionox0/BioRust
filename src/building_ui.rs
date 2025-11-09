use bevy::prelude::*;
use crate::components::*;

pub struct BuildingUIPlugin;

impl Plugin for BuildingUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerResources>()
            .init_resource::<BuildingPlacement>()
            .init_resource::<UIIcons>()
            .add_systems(Startup, (load_ui_icons, setup_building_ui).chain())
            .add_systems(Update, (
                sync_player_resources,
                update_resource_display,
                handle_building_panel_interactions,
                handle_building_placement,
                update_production_queue_display,
                building_hotkeys_system,
            ));
    }
}

#[derive(Resource, Default)]
pub struct PlayerResources {
    pub nectar: f32,
    pub chitin: f32,
    pub minerals: f32,
    pub pheromones: f32,
    pub population_used: u32,
    pub population_limit: u32,
}

#[derive(Resource, Default)]
pub struct BuildingPlacement {
    pub active_building: Option<BuildingType>,
    pub placement_preview: Option<Entity>,
    pub is_valid_placement: bool,
}

#[derive(Component)]
pub struct BuildingPanel;

#[derive(Component)]
pub struct ResourceDisplay;

#[derive(Component)]
pub struct ProductionQueueDisplay;

#[derive(Component)]
pub struct BuildingButton {
    pub building_type: BuildingType,
    pub cost: Vec<(ResourceType, f32)>,
}

#[derive(Component)]
pub struct UnitButton {
    pub unit_type: UnitType,
    pub cost: Vec<(ResourceType, f32)>,
    pub building_type: BuildingType,
}

#[derive(Component)]
pub struct PlacementPreview;

#[derive(Component)]
pub struct PlacementStatusText;

#[derive(Resource, Default)]
pub struct UIIcons {
    // Building icons
    pub nursery_icon: Handle<Image>,
    pub warrior_chamber_icon: Handle<Image>,
    pub hunter_chamber_icon: Handle<Image>,
    pub fungal_garden_icon: Handle<Image>,
    pub wood_processor_icon: Handle<Image>,
    pub stone_crusher_icon: Handle<Image>,
    
    // Unit icons
    pub worker_icon: Handle<Image>,
    pub soldier_icon: Handle<Image>,
    pub hunter_icon: Handle<Image>,
    
    // Resource icons
    pub nectar_icon: Handle<Image>,
    pub chitin_icon: Handle<Image>,
    pub minerals_icon: Handle<Image>,
    pub pheromones_icon: Handle<Image>,
    pub population_icon: Handle<Image>,
    
    pub icons_loaded: bool,
}

pub fn load_ui_icons(
    mut ui_icons: ResMut<UIIcons>,
    asset_server: Res<AssetServer>,
) {
    info!("Loading UI icons from game-icons collection...");
    
    // Building icons using appropriate themed icons
    ui_icons.nursery_icon = asset_server.load("icons/ffffff/000000/1x1/sbed/hive.svg");
    ui_icons.warrior_chamber_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/artificial-hive.svg");
    ui_icons.hunter_chamber_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/spider-web.svg");
    ui_icons.fungal_garden_icon = asset_server.load("icons/ffffff/000000/1x1/caro-asercion/water-mill.svg");
    ui_icons.wood_processor_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/gear-hammer.svg");
    ui_icons.stone_crusher_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/stone-sphere.svg");
    
    // Unit icons using insect-themed icons
    ui_icons.worker_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/bee.svg");
    ui_icons.soldier_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/centipede.svg");
    ui_icons.hunter_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/dragonfly.svg");
    
    // Resource icons using thematic representations
    ui_icons.nectar_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/honeycomb.svg");
    ui_icons.chitin_icon = asset_server.load("icons/ffffff/000000/1x1/sbed/claw.svg");
    ui_icons.minerals_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/stone-block.svg");
    ui_icons.pheromones_icon = asset_server.load("icons/ffffff/000000/1x1/willdabeast/gold-bar.svg");
    ui_icons.population_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/all-for-one.svg");
    
    ui_icons.icons_loaded = true;
    info!("UI icons loaded successfully");
}

pub fn setup_building_ui(
    mut commands: Commands,
    ui_icons: Res<UIIcons>,
) {
    use crate::constants::{ui::*, resources::*};
    // Main UI root
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        GlobalZIndex(100),
    )).with_children(|parent| {
        // Top bar - Resources
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(RESOURCE_BAR_HEIGHT),
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(RESOURCE_BAR_PADDING)),
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            BorderColor(BORDER_COLOR),
            ResourceDisplay,
        )).with_children(|parent| {
            // Resource counters with icons
            let resource_data = [
                (ResourceType::Nectar, ui_icons.nectar_icon.clone()),
                (ResourceType::Chitin, ui_icons.chitin_icon.clone()),
                (ResourceType::Minerals, ui_icons.minerals_icon.clone()),
                (ResourceType::Pheromones, ui_icons.pheromones_icon.clone()),
            ];
            
            for (resource, icon_handle) in resource_data {
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(5.0),
                        ..default()
                    },
                )).with_children(|parent| {
                    // Resource icon
                    parent.spawn((
                        ImageNode::new(icon_handle),
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            ..default()
                        },
                    ));
                    // Resource text
                    parent.spawn((
                        Text::new("1000"),
                        TextFont {
                            font_size: RESOURCE_TEXT_SIZE,
                            ..default()
                        },
                        TextColor(TEXT_COLOR),
                        ResourceCounter { resource_type: resource },
                    ));
                });
            }
            
            // Population counter
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(5.0),
                    ..default()
                },
            )).with_children(|parent| {
                // Population icon
                parent.spawn((
                    ImageNode::new(ui_icons.population_icon.clone()),
                    Node {
                        width: Val::Px(24.0),
                        height: Val::Px(24.0),
                        ..default()
                    },
                ));
                // Population text
                parent.spawn((
                    Text::new("0/200"),
                    TextFont {
                        font_size: RESOURCE_TEXT_SIZE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    PopulationCounter,
                ));
            });
        });

        // Bottom panel - Building interface
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(BUILDING_PANEL_HEIGHT),
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(RESOURCE_BAR_PADDING)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
            BorderColor(PANEL_BORDER_COLOR),
            BuildingPanel,
        )).with_children(|parent| {
            // Buildings section
            parent.spawn((
                Node {
                    width: Val::Percent(BUILDING_PANEL_BUILDINGS_WIDTH),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("Buildings"),
                    TextFont {
                        font_size: PANEL_TITLE_SIZE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                
                // Instructions for building placement
                parent.spawn((
                    Text::new("Click building, then click terrain to place"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                ));
                
                // Placement status indicator
                parent.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.2, 0.8, 0.2)),
                    Node {
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                    PlacementStatusText,
                ));
                
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(80.0),
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(BUTTON_GAP),
                        row_gap: Val::Px(BUTTON_GAP),
                        ..default()
                    },
                )).with_children(|parent| {
                    // Building buttons with proper icons
                    create_building_button_with_icon(parent, BuildingType::Nursery, ui_icons.nursery_icon.clone(), "Nursery", vec![(ResourceType::Chitin, HOUSE_WOOD_COST)]);
                    create_building_button_with_icon(parent, BuildingType::WarriorChamber, ui_icons.warrior_chamber_icon.clone(), "Warriors", vec![(ResourceType::Chitin, BARRACKS_WOOD_COST), (ResourceType::Minerals, BARRACKS_STONE_COST)]);
                    create_building_button_with_icon(parent, BuildingType::HunterChamber, ui_icons.hunter_chamber_icon.clone(), "Hunters", vec![(ResourceType::Chitin, ARCHERY_WOOD_COST)]);
                    create_building_button_with_icon(parent, BuildingType::WoodProcessor, ui_icons.wood_processor_icon.clone(), "Processor", vec![(ResourceType::Chitin, LUMBER_CAMP_WOOD_COST)]);
                    create_building_button_with_icon(parent, BuildingType::MineralProcessor, ui_icons.mineral_processor_icon.clone(), "Mineral Processor", vec![(ResourceType::Chitin, MINERAL_PROCESSOR_WOOD_COST)]);
                    create_building_button_with_icon(parent, BuildingType::FungalGarden, ui_icons.fungal_garden_icon.clone(), "Garden", vec![(ResourceType::Chitin, FARM_WOOD_COST)]);
                });
            });

            // Units/Production section
            parent.spawn((
                Node {
                    width: Val::Percent(BUILDING_PANEL_UNITS_WIDTH),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("Units"),
                    TextFont {
                        font_size: PANEL_TITLE_SIZE,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(80.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(BUTTON_GAP),
                        ..default()
                    },
                    ProductionQueueDisplay,
                )).with_children(|parent| {
                    // Unit buttons with proper icons
                    create_unit_button_with_icon(parent, UnitType::WorkerAnt, ui_icons.worker_icon.clone(), "Worker", vec![(ResourceType::Nectar, VILLAGER_FOOD_COST)], BuildingType::Queen);
                    create_unit_button_with_icon(parent, UnitType::SoldierAnt, ui_icons.soldier_icon.clone(), "Soldier", vec![(ResourceType::Nectar, MILITIA_FOOD_COST), (ResourceType::Pheromones, MILITIA_GOLD_COST)], BuildingType::WarriorChamber);
                    create_unit_button_with_icon(parent, UnitType::HunterWasp, ui_icons.hunter_icon.clone(), "Hunter", vec![(ResourceType::Chitin, ARCHER_WOOD_COST), (ResourceType::Pheromones, ARCHER_GOLD_COST)], BuildingType::HunterChamber);
                });
            });
        });
    });

    // Initialize player resources (will sync with main PlayerResources)
    commands.insert_resource(PlayerResources {
        nectar: STARTING_NECTAR,
        chitin: STARTING_CHITIN,
        minerals: STARTING_MINERALS,
        pheromones: STARTING_PHEROMONES,
        population_used: 0,
        population_limit: STARTING_POPULATION_LIMIT,
    });
}

fn create_building_button(
    parent: &mut ChildBuilder,
    building_type: BuildingType,
    icon: &str,
    name: &str,
    cost: Vec<(ResourceType, f32)>,
) {
    use crate::constants::ui::*;
    parent.spawn((
        Button,
        Node {
            width: Val::Px(BUILDING_BUTTON_SIZE),
            height: Val::Px(BUILDING_BUTTON_SIZE),
            border: UiRect::all(Val::Px(BUILDING_BUTTON_BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        BuildingButton { building_type, cost },
    )).with_children(|parent| {
        parent.spawn((
            Text::new(icon),
            TextFont {
                font_size: BUILDING_BUTTON_ICON_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Text::new(name),
            TextFont {
                font_size: BUILDING_BUTTON_TEXT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

fn create_unit_button(
    parent: &mut ChildBuilder,
    unit_type: UnitType,
    icon: &str,
    name: &str,
    cost: Vec<(ResourceType, f32)>,
    building_type: BuildingType,
) {
    use crate::constants::ui::*;
    parent.spawn((
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(UNIT_BUTTON_HEIGHT),
            border: UiRect::all(Val::Px(UNIT_BUTTON_BORDER)),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(UNIT_BUTTON_PADDING)),
            ..default()
        },
        BackgroundColor(UNIT_BUTTON_COLOR),
        BorderColor(UNIT_BORDER_COLOR),
        UnitButton { unit_type, cost, building_type },
    )).with_children(|parent| {
        parent.spawn((
            Text::new(format!("{} {}", icon, name)),
            TextFont {
                font_size: UNIT_BUTTON_TEXT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

fn create_building_button_with_icon(
    parent: &mut ChildBuilder,
    building_type: BuildingType,
    icon_handle: Handle<Image>,
    name: &str,
    cost: Vec<(ResourceType, f32)>,
) {
    use crate::constants::ui::*;
    parent.spawn((
        Button,
        Node {
            width: Val::Px(BUILDING_BUTTON_SIZE),
            height: Val::Px(BUILDING_BUTTON_SIZE),
            border: UiRect::all(Val::Px(BUILDING_BUTTON_BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        BuildingButton { building_type, cost },
    )).with_children(|parent| {
        // Icon image
        parent.spawn((
            ImageNode::new(icon_handle),
            Node {
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                ..default()
            },
        ));
        // Building name
        parent.spawn((
            Text::new(name),
            TextFont {
                font_size: BUILDING_BUTTON_TEXT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

fn create_unit_button_with_icon(
    parent: &mut ChildBuilder,
    unit_type: UnitType,
    icon_handle: Handle<Image>,
    name: &str,
    cost: Vec<(ResourceType, f32)>,
    building_type: BuildingType,
) {
    use crate::constants::ui::*;
    parent.spawn((
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(UNIT_BUTTON_HEIGHT),
            border: UiRect::all(Val::Px(UNIT_BUTTON_BORDER)),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(UNIT_BUTTON_PADDING)),
            column_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(UNIT_BUTTON_COLOR),
        BorderColor(UNIT_BORDER_COLOR),
        UnitButton { unit_type, cost, building_type },
    )).with_children(|parent| {
        // Unit icon
        parent.spawn((
            ImageNode::new(icon_handle),
            Node {
                width: Val::Px(24.0),
                height: Val::Px(24.0),
                ..default()
            },
        ));
        // Unit name
        parent.spawn((
            Text::new(name),
            TextFont {
                font_size: UNIT_BUTTON_TEXT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

#[derive(Component)]
pub struct ResourceCounter {
    pub resource_type: ResourceType,
}

#[derive(Component)]
pub struct PopulationCounter;

// Sync the UI resource system with the main game resource system
pub fn sync_player_resources(
    main_resources: Res<crate::resources::PlayerResources>,
    mut ui_resources: ResMut<PlayerResources>,
) {
    if main_resources.is_changed() {
        ui_resources.nectar = main_resources.food;
        ui_resources.chitin = main_resources.wood;
        ui_resources.minerals = main_resources.stone;
        ui_resources.pheromones = main_resources.gold;
        ui_resources.population_used = main_resources.current_population;
        ui_resources.population_limit = main_resources.max_population;
    }
}

pub fn update_resource_display(
    player_resources: Res<PlayerResources>,
    mut resource_query: Query<(&ResourceCounter, &mut Text)>,
    mut population_query: Query<&mut Text, (With<PopulationCounter>, Without<ResourceCounter>)>,
) {
    if !player_resources.is_changed() {
        return;
    }

    for (counter, mut text) in resource_query.iter_mut() {
        let amount = match counter.resource_type {
            ResourceType::Nectar => player_resources.nectar,
            ResourceType::Chitin => player_resources.chitin,
            ResourceType::Minerals => player_resources.minerals,
            ResourceType::Pheromones => player_resources.pheromones,
        };
        **text = format!("{:.0}", amount);
    }

    for mut text in population_query.iter_mut() {
        **text = format!("{}/{}", player_resources.population_used, player_resources.population_limit);
    }
}

pub fn handle_building_panel_interactions(
    mut interaction_query: Query<(&Interaction, &BuildingButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut unit_interaction_query: Query<(&Interaction, &UnitButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>, Without<BuildingButton>)>,
    mut placement: ResMut<BuildingPlacement>,
    player_resources: Res<PlayerResources>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Handle building button interactions
    for (interaction, building_button, mut background_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if can_afford_building(&building_button.cost, &player_resources) {
                    placement.active_building = Some(building_button.building_type.clone());
                    info!("Selected building: {:?} - Click on terrain to place, ESC to cancel", building_button.building_type);
                    *background_color = Color::srgba(0.2, 0.8, 0.2, 0.8).into(); // Bright green when selected
                } else {
                    info!("Cannot afford building: {:?}", building_button.building_type);
                    *background_color = Color::srgba(0.8, 0.2, 0.2, 0.8).into(); // Red when can't afford
                }
            }
            Interaction::Hovered => {
                if can_afford_building(&building_button.cost, &player_resources) {
                    *background_color = Color::srgba(0.3, 0.7, 0.3, 0.8).into(); // Green hover when affordable
                } else {
                    *background_color = Color::srgba(0.7, 0.3, 0.3, 0.8).into(); // Red hover when can't afford
                }
            }
            Interaction::None => {
                // Check if this building is currently selected for placement
                if let Some(active_building) = &placement.active_building {
                    if *active_building == building_button.building_type {
                        *background_color = Color::srgba(0.2, 0.6, 0.2, 0.8).into(); // Keep selected buildings highlighted
                        continue;
                    }
                }
                if can_afford_building(&building_button.cost, &player_resources) {
                    *background_color = Color::srgba(0.2, 0.2, 0.2, 0.8).into(); // Normal when affordable
                } else {
                    *background_color = Color::srgba(0.4, 0.2, 0.2, 0.8).into(); // Darker red when can't afford
                }
            }
        }
    }

    // Handle unit button interactions
    for (interaction, unit_button, mut background_color) in unit_interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if can_afford_unit(&unit_button.cost, &player_resources) {
                    // Find a building of the correct type to produce this unit
                    spawn_unit_from_building(&mut commands, &mut meshes, &mut materials, unit_button.unit_type.clone());
                    info!("Producing unit: {:?}", unit_button.unit_type);
                } else {
                    info!("Cannot afford unit: {:?}", unit_button.unit_type);
                }
            }
            Interaction::Hovered => {
                *background_color = Color::srgba(0.4, 0.4, 0.4, 0.8).into();
            }
            Interaction::None => {
                *background_color = Color::srgba(0.3, 0.3, 0.3, 0.8).into();
            }
        }
    }
}

pub fn handle_building_placement(
    mut placement: ResMut<BuildingPlacement>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_manager: Res<crate::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::terrain_v2::TerrainSettings>,
    mut player_resources: ResMut<PlayerResources>,
    mut main_resources: ResMut<crate::resources::PlayerResources>,
    mut preview_transforms: Query<&mut Transform, With<PlacementPreview>>,
) {
    // Cancel building placement with ESC
    if keyboard.just_pressed(crate::constants::hotkeys::CANCEL_BUILD) {
        if placement.active_building.is_some() {
            placement.active_building = None;
            if let Some(preview_entity) = placement.placement_preview.take() {
                commands.entity(preview_entity).despawn();
            }
        }
    }

    if let Some(building_type) = placement.active_building.clone() {
        let window = windows.single();
        if let Some(cursor_position) = window.cursor_position() {
            let (camera, camera_transform) = camera_q.single();

            if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
                // Calculate placement position
                let ground_y = 0.0;
                let t = (ground_y - ray.origin.y) / ray.direction.y.max(0.001);
                let placement_pos = if t > 0.0 {
                    let intersection = ray.origin + ray.direction * t;
                    let terrain_height = crate::terrain_v2::sample_terrain_height(
                        intersection.x,
                        intersection.z,
                        &terrain_manager.noise_generator,
                        &terrain_settings,
                    );
                    Vec3::new(intersection.x, terrain_height, intersection.z)
                } else {
                    Vec3::new(0.0, 0.0, 0.0)
                };

                // Update or create placement preview
                if placement.placement_preview.is_none() {
                    let preview_entity = create_building_preview(&mut commands, &mut meshes, &mut materials, building_type.clone(), placement_pos);
                    placement.placement_preview = Some(preview_entity);
                } else if let Some(preview_entity) = placement.placement_preview {
                    // Update preview position
                    if let Ok(mut transform) = preview_transforms.get_mut(preview_entity) {
                        transform.translation = placement_pos;
                    }
                }

                // Place building on left click
                if mouse_button.just_pressed(MouseButton::Left) {
                    let building_cost = get_building_cost(&building_type);
                    if can_afford_building(&building_cost, &player_resources) {
                        // Deduct resources from both systems
                        for (resource_type, cost) in &building_cost {
                            match resource_type {
                                ResourceType::Nectar => {
                                    player_resources.nectar -= cost;
                                    main_resources.food -= cost;
                                },
                                ResourceType::Chitin => {
                                    player_resources.chitin -= cost;
                                    main_resources.wood -= cost;
                                },
                                ResourceType::Minerals => {
                                    player_resources.minerals -= cost;
                                    main_resources.stone -= cost;
                                },
                                ResourceType::Pheromones => {
                                    player_resources.pheromones -= cost;
                                    main_resources.gold -= cost;
                                },
                            }
                        }

                        // Place building
                        place_building(&mut commands, &mut meshes, &mut materials, building_type.clone(), placement_pos);
                        
                        // Clear placement
                        placement.active_building = None;
                        if let Some(preview_entity) = placement.placement_preview.take() {
                            commands.entity(preview_entity).despawn();
                        }

                        info!("Placed building: {:?} at {:?}", building_type, placement_pos);
                    }
                }
            }
        }
    }
}

pub fn update_production_queue_display(
    // TODO: Update production queue display based on selected buildings
) {
    // Placeholder for production queue visualization
}

pub fn update_placement_status(
    placement: Res<BuildingPlacement>,
    mut status_query: Query<&mut Text, With<PlacementStatusText>>,
) {
    if let Ok(mut text) = status_query.get_single_mut() {
        match &placement.active_building {
            Some(building_type) => {
                **text = format!("Placing {:?} - Click terrain or ESC to cancel", building_type);
            }
            None => {
                **text = "".to_string();
            }
        }
    }
}

pub fn building_hotkeys_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut placement: ResMut<BuildingPlacement>,
    player_resources: Res<PlayerResources>,
) {
    use crate::constants::{hotkeys::*, resources::*};
    
    if keyboard.just_pressed(BUILD_BARRACKS) {
        // B for Warrior Chamber
        let cost = vec![(ResourceType::Chitin, BARRACKS_WOOD_COST), (ResourceType::Minerals, BARRACKS_STONE_COST)];
        if can_afford_building(&cost, &player_resources) {
            placement.active_building = Some(BuildingType::WarriorChamber);
        }
    } else if keyboard.just_pressed(BUILD_HOUSE) {
        // H for Nursery
        let cost = vec![(ResourceType::Chitin, HOUSE_WOOD_COST)];
        if can_afford_building(&cost, &player_resources) {
            placement.active_building = Some(BuildingType::Nursery);
        }
    } else if keyboard.just_pressed(BUILD_FARM) {
        // F for Fungal Garden
        let cost = vec![(ResourceType::Chitin, FARM_WOOD_COST)];
        if can_afford_building(&cost, &player_resources) {
            placement.active_building = Some(BuildingType::FungalGarden);
        }
    }
}

// Helper functions
fn can_afford_building(cost: &[(ResourceType, f32)], resources: &PlayerResources) -> bool {
    for (resource_type, amount) in cost {
        let available = match resource_type {
            ResourceType::Nectar => resources.nectar,
            ResourceType::Chitin => resources.chitin,
            ResourceType::Minerals => resources.minerals,
            ResourceType::Pheromones => resources.pheromones,
        };
        if available < *amount {
            return false;
        }
    }
    true
}

fn can_afford_unit(cost: &[(ResourceType, f32)], resources: &PlayerResources) -> bool {
    can_afford_building(cost, resources) && resources.population_used < resources.population_limit
}

fn get_building_cost(building_type: &BuildingType) -> Vec<(ResourceType, f32)> {
    use crate::constants::resources::*;
    match building_type {
        BuildingType::Nursery => vec![(ResourceType::Chitin, HOUSE_WOOD_COST)],
        BuildingType::WarriorChamber => vec![(ResourceType::Chitin, BARRACKS_WOOD_COST), (ResourceType::Minerals, BARRACKS_STONE_COST)],
        BuildingType::HunterChamber => vec![(ResourceType::Chitin, ARCHERY_WOOD_COST)],
        BuildingType::FungalGarden => vec![(ResourceType::Chitin, FARM_WOOD_COST)],
        BuildingType::WoodProcessor => vec![(ResourceType::Chitin, LUMBER_CAMP_WOOD_COST)],
        BuildingType::MineralProcessor => vec![(ResourceType::Chitin, MINERAL_PROCESSOR_WOOD_COST)],
        _ => vec![(ResourceType::Chitin, DEFAULT_BUILDING_WOOD_COST)],
    }
}

fn create_building_preview(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    building_type: BuildingType,
    position: Vec3,
) -> Entity {
    use crate::constants::buildings::*;
    let (mesh, size) = match building_type {
        BuildingType::Nursery => (meshes.add(Cuboid::from_size(HOUSE_SIZE)), HOUSE_SIZE),
        BuildingType::WarriorChamber => (meshes.add(Cuboid::from_size(BARRACKS_SIZE)), BARRACKS_SIZE),
        BuildingType::HunterChamber => (meshes.add(Cuboid::from_size(ARCHERY_SIZE)), ARCHERY_SIZE),
        BuildingType::FungalGarden => (meshes.add(Cuboid::from_size(FARM_SIZE)), FARM_SIZE),
        _ => (meshes.add(Cuboid::from_size(DEFAULT_BUILDING_SIZE)), DEFAULT_BUILDING_SIZE),
    };

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: PREVIEW_COLOR,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(position + Vec3::new(0.0, size.y / 2.0, 0.0)),
        PlacementPreview,
    )).id()
}

fn place_building(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    building_type: BuildingType,
    position: Vec3,
) {
    use crate::rts_entities::RTSEntityFactory;
    
    // Use proper building spawn functions with all required components
    let _building_entity = match building_type {
        BuildingType::Queen => RTSEntityFactory::spawn_town_center(commands, meshes, materials, position, 1),
        BuildingType::Nursery => RTSEntityFactory::spawn_house(commands, meshes, materials, position, 1),
        BuildingType::WarriorChamber => RTSEntityFactory::spawn_barracks(commands, meshes, materials, position, 1),
        // Add more building types as needed
        _ => {
            // Fallback for undefined building types
            use crate::constants::buildings::*;
            let (mesh, material, size) = (
                meshes.add(Cuboid::from_size(DEFAULT_BUILDING_SIZE)),
                materials.add(StandardMaterial {
                    base_color: DEFAULT_BUILDING_COLOR,
                    ..default()
                }),
                DEFAULT_BUILDING_SIZE,
            );
            
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(position + Vec3::new(0.0, size.y / 2.0, 0.0)),
                Building {
                    building_type: building_type.clone(),
                    construction_progress: crate::constants::buildings::CONSTRUCTION_PROGRESS_MAX,
                    max_construction: crate::constants::buildings::CONSTRUCTION_PROGRESS_MAX,
                    is_complete: true,
                    rally_point: None,
                },
                RTSHealth {
                    current: crate::constants::buildings::DEFAULT_BUILDING_HEALTH,
                    max: crate::constants::buildings::DEFAULT_BUILDING_HEALTH,
                    armor: crate::constants::buildings::DEFAULT_BUILDING_ARMOR,
                    regeneration_rate: 0.0,
                    last_damage_time: 0.0,
                },
                Selectable::default(),
                GameEntity,
            )).id()
        }
    };
    
    info!("Placed building: {:?} at position: {:?}", building_type, position);
}

fn spawn_unit_from_building(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    unit_type: UnitType,
) {
    // Spawn near origin for now - in a real game, this would spawn near the producing building
    use crate::constants::combat::*;
    let x = rand::random::<f32>() * UNIT_SPAWN_RANGE - UNIT_SPAWN_OFFSET;
    let z = rand::random::<f32>() * UNIT_SPAWN_RANGE - UNIT_SPAWN_OFFSET;
    
    // For now, spawn at a reasonable ground level height (will be fixed by movement system)
    let spawn_position = Vec3::new(x, 1.0, z); // Just 1 unit above ground level

    match unit_type {
        UnitType::WorkerAnt => {
            crate::rts_entities::RTSEntityFactory::spawn_villager(
                commands, meshes, materials, spawn_position, 1, rand::random()
            );
        }
        UnitType::SoldierAnt => {
            crate::combat_systems::create_combat_unit(
                commands, meshes, materials, spawn_position, 1, UnitType::SoldierAnt
            );
        }
        UnitType::HunterWasp => {
            crate::combat_systems::create_combat_unit(
                commands, meshes, materials, spawn_position, 1, UnitType::HunterWasp
            );
        }
        _ => {}
    }
}