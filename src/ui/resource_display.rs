use bevy::prelude::*;
use crate::core::components::*;
use crate::ui::icons::UIIcons;

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
}

#[derive(Component)]
pub struct AIPopulationCounter;

pub fn setup_resource_display(parent: &mut ChildBuilder, ui_icons: &UIIcons) {
    use crate::constants::ui::*;
    
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
}

fn create_population_counter(parent: &mut ChildBuilder, ui_icons: &UIIcons) {
    use crate::constants::ui::*;
    
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

// Setup AI resources display (bottom left corner)
pub fn setup_ai_resource_display(mut commands: Commands, ui_icons: Res<UIIcons>) {
    use crate::constants::ui::*;

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(170.0), // Position above the 150px bottom HUD + 20px margin
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(5.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        BorderColor(Color::srgb(0.8, 0.3, 0.3)), // Red border for AI
        ZIndex(1000), // High z-index to appear above other UI
        AIResourceDisplay,
    )).with_children(|parent| {
        // AI Player label
        parent.spawn((
            Text::new("AI Player 2"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.3, 0.3)),
        ));

        // AI Resource counters
        for (resource, icon_handle) in [
            (ResourceType::Nectar, ui_icons.nectar_icon.clone()),
            (ResourceType::Chitin, ui_icons.chitin_icon.clone()),
            (ResourceType::Minerals, ui_icons.minerals_icon.clone()),
            (ResourceType::Pheromones, ui_icons.pheromones_icon.clone()),
        ] {
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(5.0),
                    ..default()
                },
            )).with_children(|parent| {
                parent.spawn((
                    ImageNode::new(icon_handle),
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                ));
                parent.spawn((
                    Text::new("0"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    AIResourceCounter { resource_type: resource },
                ));
            });
        }

        // AI Population counter
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                ..default()
            },
        )).with_children(|parent| {
            parent.spawn((
                ImageNode::new(ui_icons.population_icon.clone()),
                Node {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("0/0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                AIPopulationCounter,
            ));
        });
    });
}

// Update AI resource display
pub fn update_ai_resource_display(
    ai_resources: Res<crate::core::resources::AIResources>,
    mut ai_resource_query: Query<(&AIResourceCounter, &mut Text)>,
    mut ai_population_query: Query<&mut Text, (With<AIPopulationCounter>, Without<AIResourceCounter>)>,
) {
    // Get AI Player 2's resources
    let Some(ai_player_2) = ai_resources.resources.get(&2) else { return };

    for (counter, mut text) in ai_resource_query.iter_mut() {
        let amount = match counter.resource_type {
            ResourceType::Nectar => ai_player_2.nectar,
            ResourceType::Chitin => ai_player_2.chitin,
            ResourceType::Minerals => ai_player_2.minerals,
            ResourceType::Pheromones => ai_player_2.pheromones,
        };
        **text = format!("{:.0}", amount);
    }

    for mut text in ai_population_query.iter_mut() {
        **text = format!("{}/{}", ai_player_2.current_population, ai_player_2.max_population);
    }
}