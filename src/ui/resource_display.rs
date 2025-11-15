use crate::core::components::*;
use crate::ui::icons::UIIcons;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct PlayerResources {
    pub nectar: f32,
    pub chitin: f32,
    pub minerals: f32,
    pub pheromones: f32,
    pub population_used: u32,
    pub population_limit: u32,
}

#[derive(Component)]
pub struct ResourceDisplay;

#[derive(Component)]
pub struct ResourceCounter {
    pub resource_type: ResourceType,
}

#[derive(Component)]
pub struct PopulationCounter;

#[derive(Component)]
pub struct AIResourceDisplay;

#[derive(Component)]
pub struct AIResourceCounter {
    pub resource_type: ResourceType,
    pub player_id: u8,
}

#[derive(Component)]
pub struct AIPopulationCounter {
    pub player_id: u8,
}

#[derive(Component)]
pub struct AIPlayerContainer {
    pub player_id: u8,
}

#[derive(Component)]
pub struct ResourceTooltip;

#[derive(Component)]
pub struct ResourceTooltipText;

pub fn setup_resource_display(parent: &mut ChildBuilder, ui_icons: &UIIcons) {
    use crate::constants::ui::*;

    // Top bar - Resources
    parent
        .spawn((
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
        ))
        .with_children(|parent| {
            // Resource counters with icons
            create_resource_counters(parent, ui_icons);
            create_population_counter(parent, ui_icons);
        });
}

fn create_resource_counters(parent: &mut ChildBuilder, ui_icons: &UIIcons) {
    use crate::constants::ui::*;

    let resource_data = [
        (ResourceType::Nectar, ui_icons.nectar_icon.clone()),
        (ResourceType::Chitin, ui_icons.chitin_icon.clone()),
        (ResourceType::Minerals, ui_icons.minerals_icon.clone()),
        (ResourceType::Pheromones, ui_icons.pheromones_icon.clone()),
    ];

    for (resource, icon_handle) in resource_data {
        parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(5.0),
                    padding: UiRect::all(Val::Px(4.0)), // Add padding for better hover area
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.0)), // Initially transparent border
                Interaction::default(), // Add interaction component for hover detection
                ResourceCounter {
                    resource_type: resource.clone(),
                },
            ))
            .with_children(|parent| {
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
                ));
            });
    }
}

fn create_population_counter(parent: &mut ChildBuilder, ui_icons: &UIIcons) {
    use crate::constants::ui::*;

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                padding: UiRect::all(Val::Px(4.0)), // Add padding for consistency
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.0)), // Initially transparent border
            Interaction::default(), // Add interaction component for consistency
            PopulationCounter,
        ))
        .with_children(|parent| {
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
            ));
        });
}

// Sync the UI resource system with the main game resource system
pub fn sync_player_resources(
    main_resources: Res<crate::core::resources::PlayerResources>,
    mut ui_resources: ResMut<PlayerResources>,
) {
    if main_resources.is_changed() {
        ui_resources.nectar = main_resources.nectar;
        ui_resources.chitin = main_resources.chitin;
        ui_resources.minerals = main_resources.minerals;
        ui_resources.pheromones = main_resources.pheromones;
        ui_resources.population_used = main_resources.current_population;
        ui_resources.population_limit = main_resources.max_population;
    }
}

pub fn update_resource_display(
    player_resources: Res<PlayerResources>,
    resource_query: Query<(&ResourceCounter, &Children)>,
    population_query: Query<&Children, With<PopulationCounter>>,
    mut text_query: Query<&mut Text>,
) {
    if !player_resources.is_changed() {
        return;
    }

    // Update resource texts
    for (counter, children) in resource_query.iter() {
        let amount = match counter.resource_type {
            ResourceType::Nectar => player_resources.nectar,
            ResourceType::Chitin => player_resources.chitin,
            ResourceType::Minerals => player_resources.minerals,
            ResourceType::Pheromones => player_resources.pheromones,
        };

        // Find the text child and update it
        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = format!("{:.0}", amount);
                break; // Only update the first text component found
            }
        }
    }

    // Update population text
    for children in population_query.iter() {
        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = format!(
                    "{}/{}",
                    player_resources.population_used, player_resources.population_limit
                );
                break; // Only update the first text component found
            }
        }
    }
}

// Setup AI resources display (top right corner, below main resource bar)
pub fn setup_ai_resource_display(mut commands: Commands, _ui_icons: Res<UIIcons>) {
    use crate::constants::ui::*;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(RESOURCE_BAR_HEIGHT + 10.0), // Position below main resource bar with margin
                right: Val::Px(10.0), // Right side of screen
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(6.0)), // Even more compact padding
                row_gap: Val::Px(6.0), // Spacing between players
                border: UiRect::all(Val::Px(1.0)),
                max_width: Val::Px(180.0), // Slightly wider to accommodate more info
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.85)),
            BorderColor(Color::srgb(0.5, 0.5, 0.5)), // Neutral border for all AI
            ZIndex(1000),                            // High z-index to appear above other UI
            AIResourceDisplay,
        ))
        .with_children(|parent| {
            // Title label
            parent.spawn((
                Text::new("AI Players"),
                TextFont {
                    font_size: 12.0, // Smaller font
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));
            
            // Container for all AI players - initially empty, populated dynamically
        });
}

// Manage AI player containers dynamically
pub fn manage_ai_resource_display(
    mut commands: Commands,
    ui_icons: Res<UIIcons>,
    ai_resources: Res<crate::core::resources::AIResources>,
    game_setup: Option<Res<GameSetup>>,
    ai_display_query: Query<Entity, With<AIResourceDisplay>>,
    ai_container_query: Query<(Entity, &AIPlayerContainer)>,
) {
    // Get the main AI display container
    let Ok(ai_display_entity) = ai_display_query.get_single() else {
        return;
    };

    // Get existing AI player containers
    let existing_players: std::collections::HashSet<u8> = ai_container_query
        .iter()
        .map(|(_, container)| container.player_id)
        .collect();

    // Get AI players that should exist
    let needed_players: std::collections::HashSet<u8> = ai_resources.resources.keys().copied().collect();

    // Remove containers for players that no longer exist
    for (entity, container) in ai_container_query.iter() {
        if !needed_players.contains(&container.player_id) {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Add containers for new players
    for &player_id in &needed_players {
        if !existing_players.contains(&player_id) {
            // Get player color based on player ID
            let player_color = crate::core::components::TeamColor::get_primitive_color(player_id);

            // Get team name for this player (if available from game setup)
            let display_name = if let Some(ref setup) = game_setup {
                // Player IDs 2+ map to AI team indices 0+
                if player_id >= 2 {
                    let ai_index = (player_id - 2) as usize;
                    if let Some(team_type) = setup.ai_teams.get(ai_index) {
                        team_type.display_name().to_string()
                    } else {
                        format!("AI {}", player_id)
                    }
                } else {
                    format!("Player {}", player_id)
                }
            } else {
                format!("AI {}", player_id)
            };

            // Create container for this AI player
            let player_container = commands
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(4.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        margin: UiRect::bottom(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.6)),
                    BorderColor(player_color.into()),
                    AIPlayerContainer { player_id },
                ))
                .with_children(|parent| {
                    // Player label with team name
                    parent.spawn((
                        Text::new(display_name),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(player_color.into()),
                        Node {
                            margin: UiRect::bottom(Val::Px(2.0)),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                    ));

                    // Resources in a compact row
                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceEvenly,
                            column_gap: Val::Px(4.0),
                            ..default()
                        },
                    ))
                    .with_children(|resource_row| {
                        // Compact resource display - just numbers with tiny icons
                        for (resource, icon_handle) in [
                            (ResourceType::Nectar, ui_icons.nectar_icon.clone()),
                            (ResourceType::Chitin, ui_icons.chitin_icon.clone()),
                            (ResourceType::Minerals, ui_icons.minerals_icon.clone()),
                            (ResourceType::Pheromones, ui_icons.pheromones_icon.clone()),
                        ] {
                            resource_row.spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    row_gap: Val::Px(1.0),
                                    ..default()
                                },
                            ))
                            .with_children(|res_item| {
                                // Tiny icon
                                res_item.spawn((
                                    ImageNode::new(icon_handle),
                                    Node {
                                        width: Val::Px(10.0),
                                        height: Val::Px(10.0),
                                        ..default()
                                    },
                                ));
                                // Resource amount
                                res_item.spawn((
                                    Text::new("0"),
                                    TextFont {
                                        font_size: 9.0, // Very small text
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    AIResourceCounter {
                                        resource_type: resource,
                                        player_id,
                                    },
                                ));
                            });
                        }
                    });

                    // Population display
                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(2.0),
                            margin: UiRect::top(Val::Px(1.0)),
                            ..default()
                        },
                    ))
                    .with_children(|pop_row| {
                        pop_row.spawn((
                            ImageNode::new(ui_icons.population_icon.clone()),
                            Node {
                                width: Val::Px(10.0),
                                height: Val::Px(10.0),
                                ..default()
                            },
                        ));
                        pop_row.spawn((
                            Text::new("0/0"),
                            TextFont {
                                font_size: 9.0, // Very small text
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            AIPopulationCounter { player_id },
                        ));
                    });
                })
                .id();

            // Add the new player container to the main display
            commands.entity(ai_display_entity).add_child(player_container);
        }
    }
}

// Update AI resource display values
pub fn update_ai_resource_display(
    ai_resources: Res<crate::core::resources::AIResources>,
    mut ai_resource_query: Query<(&AIResourceCounter, &mut Text)>,
    mut ai_population_query: Query<
        (&AIPopulationCounter, &mut Text),
        Without<AIResourceCounter>,
    >,
) {
    if !ai_resources.is_changed() {
        return;
    }

    for (counter, mut text) in ai_resource_query.iter_mut() {
        if let Some(player_resources) = ai_resources.resources.get(&counter.player_id) {
            let amount = match counter.resource_type {
                ResourceType::Nectar => player_resources.nectar,
                ResourceType::Chitin => player_resources.chitin,
                ResourceType::Minerals => player_resources.minerals,
                ResourceType::Pheromones => player_resources.pheromones,
            };
            **text = format!("{:.0}", amount);
        }
    }

    for (counter, mut text) in ai_population_query.iter_mut() {
        if let Some(player_resources) = ai_resources.resources.get(&counter.player_id) {
            **text = format!(
                "{}/{}",
                player_resources.current_population, player_resources.max_population
            );
        }
    }
}

// Setup the resource tooltip UI (initially hidden)
pub fn setup_resource_tooltip(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                display: Display::None, // Hidden by default
                max_width: Val::Px(300.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.95)),
            BorderColor(Color::srgb(0.6, 0.8, 0.9)),
            ResourceTooltip,
            ZIndex(2000), // Higher than unit tooltips
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ResourceTooltipText,
            ));
        });
}

// System to handle resource tooltips on hover
pub fn resource_hover_system(
    resource_query: Query<(&ResourceCounter, &Interaction, &GlobalTransform)>,
    mut tooltip_query: Query<(&mut Node, &mut BackgroundColor), With<ResourceTooltip>>,
    mut text_query: Query<&mut Text, With<ResourceTooltipText>>,
    windows: Query<&Window>,
    player_resources: Res<PlayerResources>,
    game_resources: Query<(Entity, &crate::core::components::ResourceSource, &Transform)>,
    gatherers_query: Query<(
        &crate::core::components::ResourceGatherer,
        &crate::core::components::RTSUnit,
    )>,
) {
    let Ok((mut tooltip_style, mut tooltip_bg)) = tooltip_query.get_single_mut() else {
        return;
    };
    let Ok(mut tooltip_text) = text_query.get_single_mut() else {
        return;
    };

    let mut show_tooltip = false;
    let mut tooltip_content = String::new();

    // Check for hovered resource counters
    for (counter, interaction, global_transform) in resource_query.iter() {
        if matches!(*interaction, Interaction::Hovered) {
            show_tooltip = true;

            // Get current amount
            let current_amount = match counter.resource_type {
                ResourceType::Nectar => player_resources.nectar,
                ResourceType::Chitin => player_resources.chitin,
                ResourceType::Minerals => player_resources.minerals,
                ResourceType::Pheromones => player_resources.pheromones,
            };

            // Count resource sources of this type
            let mut total_sources = 0;
            let mut total_remaining = 0.0;
            let mut active_gatherers = 0;

            for (_entity, source, _transform) in game_resources.iter() {
                if source.resource_type == counter.resource_type {
                    total_sources += 1;
                    total_remaining += source.amount;
                }
            }

            // Count gatherers working on this resource type
            for (gatherer, unit) in gatherers_query.iter() {
                if unit.player_id == 1 {
                    // Only count player gatherers
                    if let Some(ref gatherer_resource_type) = gatherer.resource_type {
                        if *gatherer_resource_type == counter.resource_type {
                            active_gatherers += 1;
                        }
                    }
                }
            }

            let resource_name = format!("{:?}", counter.resource_type);

            tooltip_content = format!(
                "{}\n\nCurrent: {:.0}\nSources: {}\nRemaining: {:.0}\nGatherers: {}",
                resource_name, current_amount, total_sources, total_remaining, active_gatherers
            );

            // Position tooltip near the resource counter
            let transform = global_transform.compute_transform();
            if let Ok(window) = windows.get_single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    tooltip_style.left = Val::Px(cursor_pos.x + 10.0);
                    tooltip_style.top = Val::Px(cursor_pos.y + 10.0);
                } else {
                    // Fallback to transform position
                    tooltip_style.left = Val::Px(transform.translation.x + 30.0);
                    tooltip_style.top = Val::Px(transform.translation.y + 30.0);
                }
            }

            break; // Only show tooltip for one resource at a time
        }
    }

    // Update tooltip display
    if show_tooltip {
        **tooltip_text = tooltip_content;
        tooltip_style.display = Display::Flex;
        *tooltip_bg = BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.95));
    } else {
        tooltip_style.display = Display::None;
    }
}

// System to update resource hover effects
pub fn resource_hover_effects_system(
    mut resource_query: Query<
        (&Interaction, &mut BorderColor),
        (With<ResourceCounter>, Changed<Interaction>),
    >,
) {
    for (interaction, mut border_color) in resource_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.9, 1.0));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.0));
            }
            _ => {}
        }
    }
}
