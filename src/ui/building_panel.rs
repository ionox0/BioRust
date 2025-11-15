use crate::core::components::*;
use crate::core::resources::PlayerResources;
use crate::ui::{
    button_styles::{
        create_building_button_with_icon, create_unit_button_with_icon, BuildingButton,
        ButtonStyle, UnitButton,
    },
    icons::UIIcons,
    placement::{BuildingPlacement, PlacementStatusText},
};
use bevy::prelude::*;

#[derive(Component)]
pub struct BuildingPanel;

#[derive(Component)]
pub struct ProductionQueueDisplay;


pub fn setup_building_panel(parent: &mut ChildBuilder, ui_icons: &UIIcons, game_costs: &crate::core::resources::GameCosts) {
    // Bottom panel - Building interface (increased height for better unit layout)
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(180.0), // Increased height to accommodate more rows
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(5.0)), // Reduced padding to give more space
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
            BuildingPanel,
        ))
        .with_children(|parent| {
            setup_buildings_section(parent, ui_icons, game_costs);
            setup_units_section(parent, ui_icons, game_costs);
        });
}

fn setup_buildings_section(parent: &mut ChildBuilder, ui_icons: &UIIcons, game_costs: &crate::core::resources::GameCosts) {
    parent
        .spawn((Node {
            width: Val::Percent(50.0), // Reduced from 70% to 50% to give more space to units
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Buildings"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Instructions
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

            // Building buttons grid
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(80.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(10.0),
                    row_gap: Val::Px(10.0),
                    ..default()
                },))
                .with_children(|parent| {
                    create_building_buttons(parent, ui_icons, game_costs);
                });
        });
}

fn create_building_buttons(parent: &mut ChildBuilder, ui_icons: &UIIcons, game_costs: &crate::core::resources::GameCosts) {
    use crate::core::components::BuildingType;

    // Helper function to get cost from GameCosts or fallback to empty vec
    let get_building_cost = |building_type: BuildingType| -> Vec<(ResourceType, f32)> {
        game_costs.building_costs.get(&building_type).cloned().unwrap_or_default()
    };

    let building_data = [
        (
            BuildingType::Nursery,
            ui_icons.nursery_icon.clone(),
            "Nursery",
            get_building_cost(BuildingType::Nursery),
        ),
        (
            BuildingType::WarriorChamber,
            ui_icons.warrior_chamber_icon.clone(),
            "Warriors",
            get_building_cost(BuildingType::WarriorChamber),
        ),
        (
            BuildingType::HunterChamber,
            ui_icons.hunter_chamber_icon.clone(),
            "Hunters",
            get_building_cost(BuildingType::HunterChamber),
        ),
        (
            BuildingType::WoodProcessor,
            ui_icons.wood_processor_icon.clone(),
            "Processor",
            get_building_cost(BuildingType::WoodProcessor),
        ),
        (
            BuildingType::MineralProcessor,
            ui_icons.mineral_processor_icon.clone(),
            "Mineral Processor",
            get_building_cost(BuildingType::MineralProcessor),
        ),
        (
            BuildingType::FungalGarden,
            ui_icons.fungal_garden_icon.clone(),
            "Garden",
            get_building_cost(BuildingType::FungalGarden),
        ),
    ];

    for (building_type, icon, name, cost) in building_data {
        create_building_button_with_icon(parent, building_type, icon, name, cost);
    }
}

fn setup_units_section(parent: &mut ChildBuilder, ui_icons: &UIIcons, game_costs: &crate::core::resources::GameCosts) {
    parent
        .spawn((Node {
            width: Val::Percent(50.0), // Increased from 30% to 50% for more space
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Units"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Three-column grid container for unit buttons
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(80.0),
                    display: Display::Grid,
                    grid_template_columns: vec![
                        GridTrack::fr(1.0),
                        GridTrack::fr(1.0),
                        GridTrack::fr(1.0),
                    ], // Three equal columns
                    grid_auto_rows: vec![GridTrack::px(20.0)], // Fixed row height
                    column_gap: Val::Px(2.0),                  // Small gap between columns
                    row_gap: Val::Px(1.0),                     // Minimal row spacing
                    overflow: Overflow::clip(), // Prevent buttons from flowing outside the container
                    ..default()
                },))
                .with_children(|parent| {
                    create_unit_buttons(parent, ui_icons, game_costs);
                });
        });
}

fn create_unit_buttons(parent: &mut ChildBuilder, ui_icons: &UIIcons, game_costs: &crate::core::resources::GameCosts) {
    use crate::core::components::UnitType;

    // Helper function to get cost from GameCosts or fallback to empty vec
    let get_cost = |unit_type: UnitType| -> Vec<(ResourceType, f32)> {
        game_costs.unit_costs.get(&unit_type).cloned().unwrap_or_default()
    };

    let unit_data = [
        // Ant units from Queen (Ant Hill)
        (
            UnitType::WorkerAnt,
            ui_icons.worker_icon.clone(),
            "Worker",
            get_cost(UnitType::WorkerAnt),
            BuildingType::Queen,
        ),
        (
            UnitType::SoldierAnt,
            ui_icons.soldier_icon.clone(),
            "Soldier",
            get_cost(UnitType::SoldierAnt),
            BuildingType::Queen,
        ),
        (
            UnitType::SpearMantis,
            ui_icons.worker_icon.clone(),
            "Mantis",
            get_cost(UnitType::SpearMantis),
            BuildingType::Queen,
        ),
        (
            UnitType::ScoutAnt,
            ui_icons.soldier_icon.clone(),
            "Scout",
            get_cost(UnitType::ScoutAnt),
            BuildingType::Queen,
        ),
        (
            UnitType::TermiteWorker,
            ui_icons.worker_icon.clone(),
            "Termite",
            get_cost(UnitType::TermiteWorker),
            BuildingType::Queen,
        ),
        // Bee/Flying units from Nursery (Bee Hive)
        (
            UnitType::DragonFly,
            ui_icons.hunter_icon.clone(),
            "DragonFly",
            get_cost(UnitType::DragonFly),
            BuildingType::Nursery,
        ),
        (
            UnitType::AcidSpitter,
            ui_icons.hunter_icon.clone(),
            "Acid",
            get_cost(UnitType::AcidSpitter),
            BuildingType::Nursery,
        ),
        (
            UnitType::HoneyBee,
            ui_icons.hunter_icon.clone(),
            "Bee",
            get_cost(UnitType::HoneyBee),
            BuildingType::Nursery,
        ),
        (
            UnitType::Housefly,
            ui_icons.hunter_icon.clone(),
            "Fly",
            get_cost(UnitType::Housefly),
            BuildingType::Nursery,
        ),
        // Beetle/Heavy units from WarriorChamber (Pine Cone)
        (
            UnitType::BeetleKnight,
            ui_icons.worker_icon.clone(),
            "Beetle",
            get_cost(UnitType::BeetleKnight),
            BuildingType::WarriorChamber,
        ),
        (
            UnitType::BatteringBeetle,
            ui_icons.soldier_icon.clone(),
            "Battering",
            get_cost(UnitType::BatteringBeetle),
            BuildingType::WarriorChamber,
        ),
        (
            UnitType::LegBeetle,
            ui_icons.soldier_icon.clone(),
            "Leg Beetle",
            get_cost(UnitType::LegBeetle),
            BuildingType::WarriorChamber,
        ),
        (
            UnitType::Scorpion,
            ui_icons.soldier_icon.clone(),
            "Scorpion",
            get_cost(UnitType::Scorpion),
            BuildingType::WarriorChamber,
        ),
        (
            UnitType::TermiteWarrior,
            ui_icons.soldier_icon.clone(),
            "T.Warrior",
            get_cost(UnitType::TermiteWarrior),
            BuildingType::WarriorChamber,
        ),
        (
            UnitType::Stinkbug,
            ui_icons.hunter_icon.clone(),
            "Stinkbug",
            get_cost(UnitType::Stinkbug),
            BuildingType::WarriorChamber,
        ),
        // Spider/Predator units from HunterChamber
        (
            UnitType::EliteSpider,
            ui_icons.hunter_icon.clone(),
            "E.Spider",
            get_cost(UnitType::EliteSpider),
            BuildingType::HunterChamber,
        ),
        (
            UnitType::DefenderBug,
            ui_icons.soldier_icon.clone(),
            "Defender",
            get_cost(UnitType::DefenderBug),
            BuildingType::HunterChamber,
        ),
        (
            UnitType::SpiderHunter,
            ui_icons.hunter_icon.clone(),
            "Spider",
            get_cost(UnitType::SpiderHunter),
            BuildingType::HunterChamber,
        ),
        (
            UnitType::WolfSpider,
            ui_icons.hunter_icon.clone(),
            "W.Spider",
            get_cost(UnitType::WolfSpider),
            BuildingType::HunterChamber,
        ),
    ];

    for (unit_type, icon, name, cost, building_type) in unit_data {
        create_unit_button_with_icon(parent, unit_type, icon, name, cost, building_type);
    }
}

pub fn handle_building_panel_interactions(
    mut interaction_query: Query<
        (&Interaction, &BuildingButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut unit_interaction_query: Query<
        (&Interaction, &UnitButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, Without<BuildingButton>),
    >,
    mut buildings: Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    mut placement: ResMut<BuildingPlacement>,
    mut player_resources: ResMut<PlayerResources>,
    game_costs: Res<crate::core::resources::GameCosts>,
    model_assets: Option<Res<crate::rendering::model_loader::ModelAssets>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    handle_building_button_interactions(&mut interaction_query, &mut placement, &player_resources);
    handle_unit_button_interactions(
        &mut unit_interaction_query,
        &mut buildings,
        &mut player_resources,
        &game_costs,
        model_assets,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}

fn handle_building_button_interactions(
    interaction_query: &mut Query<
        (&Interaction, &BuildingButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    placement: &mut BuildingPlacement,
    player_resources: &PlayerResources,
) {
    for (interaction, building_button, mut background_color) in interaction_query.iter_mut() {
        let can_afford = player_resources.can_afford(&building_button.cost);
        let is_selected =
            placement.active_building.as_ref() == Some(&building_button.building_type);

        let style = if can_afford {
            ButtonStyle::BUILDING_AFFORDABLE
        } else {
            ButtonStyle::BUILDING_UNAFFORDABLE
        };

        match *interaction {
            Interaction::Pressed => {
                if can_afford {
                    placement.active_building = Some(building_button.building_type.clone());
                    info!(
                        "Selected building: {:?} - Click on terrain to place, ESC to cancel",
                        building_button.building_type
                    );
                    *background_color = style.pressed.into();
                } else {
                    info!(
                        "Cannot afford building: {:?}",
                        building_button.building_type
                    );
                    *background_color = style.pressed.into();
                }
            }
            Interaction::Hovered => {
                *background_color = style.hover.into();
            }
            Interaction::None => {
                if is_selected {
                    *background_color = Color::srgba(0.2, 0.6, 0.2, 0.8).into();
                // Keep selected buildings highlighted
                } else {
                    *background_color = style.normal.into();
                }
            }
        }
    }
}

fn handle_unit_button_interactions(
    unit_interaction_query: &mut Query<
        (&Interaction, &UnitButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, Without<BuildingButton>),
    >,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_resources: &mut PlayerResources,
    game_costs: &crate::core::resources::GameCosts,
    _model_assets: Option<Res<crate::rendering::model_loader::ModelAssets>>,
    _commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    for (interaction, unit_button, mut background_color) in unit_interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if can_afford_unit(&unit_button.cost, player_resources) {
                    queue_unit_for_production(
                        unit_button.unit_type.clone(),
                        unit_button.building_type.clone(),
                        buildings,
                        player_resources,
                        game_costs,
                    );
                    info!("Queued unit for production: {:?}", unit_button.unit_type);
                } else {
                    info!("Cannot afford unit: {:?}", unit_button.unit_type);
                }
            }
            Interaction::Hovered => {
                *background_color = ButtonStyle::UNIT_BUTTON.hover.into();
            }
            Interaction::None => {
                *background_color = ButtonStyle::UNIT_BUTTON.normal.into();
            }
        }
    }
}

fn can_afford_unit(cost: &[(ResourceType, f32)], resources: &PlayerResources) -> bool {
    resources.can_afford(cost) && resources.has_population_space()
}

/// Queue a unit for production instead of spawning it instantly
fn queue_unit_for_production(
    unit_type: UnitType,
    building_type: BuildingType,
    buildings: &mut Query<(&mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    player_resources: &mut PlayerResources,
    game_costs: &crate::core::resources::GameCosts,
) {
    // Get unit cost from game costs
    let unit_cost = game_costs.unit_costs.get(&unit_type);

    if let Some(cost) = unit_cost {
        // Check if player can afford the unit
        if !player_resources.can_afford(cost) || !player_resources.has_population_space() {
            warn!("Cannot afford unit {:?} or no population space", unit_type);
            return;
        }

        // Deduct resources immediately when queuing (like in real RTS games)
        if !player_resources.spend_resources(cost) {
            warn!("Failed to deduct resources for unit {:?}", unit_type);
            return;
        }

        // Find an appropriate building to queue the unit with overflow support
        if crate::rts::production::try_queue_unit_with_overflow(
            unit_type.clone(),
            building_type.clone(),
            buildings,
            1,
        ) {
            info!(
                "✅ Player 1 queued {:?} for production in {:?} (resources deducted)",
                unit_type, building_type
            );
            return;
        }

        // If no suitable building found, refund the resources
        for (resource_type, amount) in cost {
            player_resources.add_resource(resource_type.clone(), *amount);
        }
        warn!(
            "No suitable {:?} building found for unit production, resources refunded",
            building_type
        );
    }
}

pub fn update_production_queue_display(
    mut commands: Commands,
    buildings_with_queues: Query<
        (&ProductionQueue, &Building, &RTSUnit, &Selectable),
        With<Building>,
    >,
    existing_queue_displays: Query<Entity, With<ProductionQueueDisplay>>,
    ui_icons: Option<Res<UIIcons>>,
) {
    // Find the currently selected player building with a production queue
    let selected_building = buildings_with_queues
        .iter()
        .find(|(_, _, unit, selectable)| unit.player_id == 1 && selectable.is_selected);

    match selected_building {
        Some((queue, building, _, _)) => {
            // Always remove existing display first
            for entity in existing_queue_displays.iter() {
                commands.entity(entity).despawn_recursive();
            }
            
            // Check if we have UI icons available
            if let Some(icons) = ui_icons.as_ref() {
                if icons.icons_loaded {
                    create_visual_queue_display(&mut commands, queue, building, icons);
                } else {
                    // Create display without icons if they're not loaded yet
                    create_visual_queue_display_no_icons(&mut commands, queue, building);
                }
            } else {
                // Create display without icons if UIIcons resource doesn't exist
                create_visual_queue_display_no_icons(&mut commands, queue, building);
            }
        }
        None => {
            // No building selected or no player buildings selected - remove queue display
            for entity in existing_queue_displays.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn create_visual_queue_display(
    commands: &mut Commands,
    queue: &ProductionQueue,
    building: &Building,
    icons: &UIIcons,
) {
    let building_name = format_building_name(&building.building_type);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(320.0), // Just above the building panel
                left: Val::Px(20.0),    // Left side of screen
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(280.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderColor(Color::srgba(0.4, 0.4, 0.6, 1.0)),
            ProductionQueueDisplay,
            Name::new("Production Queue Display"),
        ))
        .with_children(|parent| {
            // Header
            parent.spawn((
                Text::new(format!(
                    "🏭 {} Production ({}/8)",
                    building_name,
                    queue.queue.len()
                )),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            if !queue.queue.is_empty() {
                // Current unit being produced
                let progress_percent = if queue.production_time > 0.0 {
                    ((queue.current_progress / queue.production_time * 100.0).round() as u32).min(100)
                } else {
                    0
                };
                let current_unit = &queue.queue[0];
                let remaining_time = (queue.production_time - queue.current_progress).max(0.0);

                // Current production row
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.3, 0.2, 0.6)), // Greenish tint for active
                    ))
                    .with_children(|parent| {
                        // Unit icon
                        parent.spawn((
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                ..default()
                            },
                            ImageNode::new(get_unit_icon(current_unit, icons)),
                        ));

                        // Progress info
                        parent
                            .spawn((Node {
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.0,
                                ..default()
                            },))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(format!("🔧 {}", format_unit_name(current_unit))),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));

                                // Progress bar - clamp values to prevent overflow
                                let progress_bars = (progress_percent / 10).min(10) as usize;
                                let empty_bars = (10 - progress_bars).max(0);
                                let progress_bar = "█".repeat(progress_bars);
                                let empty_bar = "░".repeat(empty_bars);
                                parent.spawn((
                                    Text::new(format!(
                                        "[{}{}] {}% | {:.1}s",
                                        progress_bar, empty_bar, progress_percent, remaining_time
                                    )),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                ));
                            });
                    });

                // Queue items
                if queue.queue.len() > 1 {
                    parent.spawn((
                        Text::new("📋 Queued:"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));

                    for (i, unit_type) in queue.queue.iter().enumerate().skip(1) {
                        if i > 3 {
                            // Show max 3 queued units
                            if queue.queue.len() > 4 {
                                parent.spawn((
                                    Text::new(format!("... and {} more", queue.queue.len() - 4)),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                ));
                            }
                            break;
                        }

                        parent
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(6.0),
                                    padding: UiRect::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(format!("{}.", i + 1)),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                                ));

                                parent.spawn((
                                    Node {
                                        width: Val::Px(20.0),
                                        height: Val::Px(20.0),
                                        ..default()
                                    },
                                    ImageNode::new(get_unit_icon(unit_type, icons)),
                                ));

                                parent.spawn((
                                    Text::new(format_unit_name(unit_type)),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                }
            } else {
                // Empty queue
                parent.spawn((
                    Text::new("📋 No units queued"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            }
        });
}

fn create_visual_queue_display_no_icons(
    commands: &mut Commands,
    queue: &ProductionQueue,
    building: &Building,
) {
    let building_name = format_building_name(&building.building_type);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(320.0), // Just above the building panel
                left: Val::Px(20.0),    // Left side of screen
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(280.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderColor(Color::srgba(0.4, 0.4, 0.6, 1.0)),
            ProductionQueueDisplay,
            Name::new("Production Queue Display (No Icons)"),
        ))
        .with_children(|parent| {
            // Header
            parent.spawn((
                Text::new(format!(
                    "🏭 {} Production ({}/8)",
                    building_name,
                    queue.queue.len()
                )),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            if !queue.queue.is_empty() {
                // Current unit being produced
                let progress_percent = if queue.production_time > 0.0 {
                    ((queue.current_progress / queue.production_time * 100.0).round() as u32).min(100)
                } else {
                    0
                };
                let current_unit = &queue.queue[0];
                let remaining_time = (queue.production_time - queue.current_progress).max(0.0);

                // Current production row
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.3, 0.2, 0.6)), // Greenish tint for active
                    ))
                    .with_children(|parent| {
                        // Progress info without icon
                        parent
                            .spawn((Node {
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.0,
                                ..default()
                            },))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(format!("🔧 {}", format_unit_name(current_unit))),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));

                                // Progress bar - clamp values to prevent overflow
                                let progress_bars = (progress_percent / 10).min(10) as usize;
                                let empty_bars = (10 - progress_bars).max(0);
                                let progress_bar = "█".repeat(progress_bars);
                                let empty_bar = "░".repeat(empty_bars);
                                parent.spawn((
                                    Text::new(format!(
                                        "[{}{}] {}% | {:.1}s",
                                        progress_bar, empty_bar, progress_percent, remaining_time
                                    )),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                ));
                            });
                    });

                // Queue items
                if queue.queue.len() > 1 {
                    parent.spawn((
                        Text::new("📋 Queued:"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));

                    for (i, unit_type) in queue.queue.iter().enumerate().skip(1) {
                        if i > 3 {
                            // Show max 3 queued units
                            if queue.queue.len() > 4 {
                                parent.spawn((
                                    Text::new(format!("... and {} more", queue.queue.len() - 4)),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                ));
                            }
                            break;
                        }

                        parent
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(6.0),
                                    padding: UiRect::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(format!("{}. {}", i + 1, format_unit_name(unit_type))),
                                    TextFont {
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                }
            } else {
                // Empty queue
                parent.spawn((
                    Text::new("📋 No units queued"),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            }
        });
}

fn get_unit_icon(unit_type: &UnitType, icons: &UIIcons) -> Handle<Image> {
    match unit_type {
        UnitType::WorkerAnt | UnitType::TermiteWorker => icons.worker_icon.clone(),
        UnitType::SoldierAnt
        | UnitType::ScoutAnt
        | UnitType::BeetleKnight
        | UnitType::SpearMantis
        | UnitType::BatteringBeetle => icons.soldier_icon.clone(),
        | UnitType::DragonFly
        | UnitType::HoneyBee
        | UnitType::Housefly
        | UnitType::AcidSpitter => icons.hunter_icon.clone(),
        _ => icons.worker_icon.clone(), // Default fallback
    }
}

/// Format building names to be more user-friendly
fn format_building_name(building_type: &BuildingType) -> &'static str {
    match building_type {
        BuildingType::Queen => "Queen Chamber",
        BuildingType::Nursery => "Nursery",
        BuildingType::WarriorChamber => "Warrior Chamber",
        BuildingType::HunterChamber => "Hunter Chamber",
        BuildingType::FungalGarden => "Fungal Garden",
        _ => "Building",
    }
}

/// Format unit names to be more user-friendly
fn format_unit_name(unit_type: &UnitType) -> &'static str {
    match unit_type {
        UnitType::WorkerAnt => "Worker Ant",
        UnitType::SoldierAnt => "Soldier Ant",
        UnitType::BeetleKnight => "Beetle Knight",
        UnitType::SpearMantis => "Spear Mantis",
        UnitType::ScoutAnt => "Scout Ant",
        UnitType::DragonFly => "Dragon Fly",
        UnitType::BatteringBeetle => "Battering Beetle",
        _ => "Unit",
    }
}

pub fn building_hotkeys_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut placement: ResMut<BuildingPlacement>,
    player_resources: Res<PlayerResources>,
) {
    use crate::constants::{hotkeys::*, resources::*};

    let hotkey_buildings = [
        (
            BUILD_WARRIOR_CHAMBER,
            BuildingType::WarriorChamber,
            vec![
                (ResourceType::Chitin, WARRIOR_CHAMBER_CHITIN_COST),
                (ResourceType::Minerals, WARRIOR_CHAMBER_MINERALS_COST),
            ],
        ),
        (
            BUILD_NURSERY,
            BuildingType::Nursery,
            vec![(ResourceType::Chitin, NURSERY_CHITIN_COST)],
        ),
        (
            BUILD_FUNGAL_GARDEN,
            BuildingType::FungalGarden,
            vec![(ResourceType::Chitin, FUNGAL_GARDEN_CHITIN_COST)],
        ),
    ];

    for (key, building_type, cost) in hotkey_buildings {
        if keyboard.just_pressed(key) && player_resources.can_afford(&cost) {
            placement.active_building = Some(building_type);
            break;
        }
    }
}
