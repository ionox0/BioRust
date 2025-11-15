//! Team Selection UI for Multi-Team RTS System

use crate::core::components::*;
use crate::core::game::GameState;
// Removed unused import - using inline styles instead
use bevy::prelude::*;
use bevy::ecs::system::ParamSet;

/// Marker component for team selection UI
#[derive(Component)]
pub struct TeamSelectionUI;

/// Marker component for team selection buttons
#[derive(Component, Debug, Clone)]
pub struct TeamSelectionButton {
    pub team_type: TeamType,
}

/// Marker component for AI count buttons
#[derive(Component, Debug, Clone)]
pub struct AICountButton {
    pub count: u8,
}

/// Marker component for start game button
#[derive(Component)]
pub struct StartGameButton;

/// Current selection state
#[derive(Resource, Debug, Clone)]
pub struct TeamSelectionState {
    pub player_team: TeamType,
    pub ai_count: u8,
    pub ai_teams: Vec<TeamType>,
}

impl Default for TeamSelectionState {
    fn default() -> Self {
        Self {
            player_team: TeamType::BalancedColony,
            ai_count: 1,
            ai_teams: vec![TeamType::Predators], // Default AI team
        }
    }
}

/// Setup team selection UI
pub fn setup_team_selection_ui(mut commands: Commands) {
    info!("Setting up team selection UI");
    
    // Spawn UI camera for team selection
    commands.spawn((
        Camera2d,
        TeamSelectionUI,
    ));
    
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
        TeamSelectionUI,
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Select Your Team"),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));

        // Team selection grid
        parent.spawn((
            Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
                column_gap: Val::Px(15.0),
                row_gap: Val::Px(15.0),
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        )).with_children(|parent| {
            // Create team selection buttons
            for team in TeamType::all_teams() {
                create_team_button(parent, team);
            }
        });

        // AI Configuration
        parent.spawn((
            Text::new("AI Players"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        // AI count buttons
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        )).with_children(|parent| {
            for i in 0..=7 {
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(50.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if i == 1 { Color::srgb(0.3, 0.5, 0.7) } else { Color::srgb(0.2, 0.2, 0.3) }),
                    AICountButton { count: i },
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new(i.to_string()),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });

        // Start game button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.7, 0.3)),
            StartGameButton,
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Start Game"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn create_team_button(parent: &mut ChildBuilder, team: TeamType) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(300.0),
            height: Val::Px(120.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(
            if matches!(team, TeamType::BalancedColony) {
                Color::srgb(0.3, 0.5, 0.7) // Selected color
            } else {
                Color::srgb(0.2, 0.2, 0.3) // Default color
            }
        ),
        TeamSelectionButton { team_type: team.clone() },
    )).with_children(|parent| {
        // Team name
        parent.spawn((
            Text::new(team.display_name()),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
        ));

        // Team description
        parent.spawn((
            Text::new(team.description()),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                // Text wrapping enabled by default in Bevy 0.15
                ..default()
            },
        ));
    });
}

/// Handle team selection button interactions
pub fn handle_team_selection(
    mut params: ParamSet<(
        Query<
            (&Interaction, &TeamSelectionButton, &mut BackgroundColor),
            (Changed<Interaction>, With<Button>),
        >,
        Query<(&TeamSelectionButton, &mut BackgroundColor)>,
    )>,
    mut selection_state: ResMut<TeamSelectionState>,
) {
    // First, collect information from the interaction query
    let mut pressed_team: Option<TeamType> = None;
    let mut hovered_teams = Vec::new();
    let mut none_teams = Vec::new();
    
    // Process interactions
    for (interaction, team_button, mut color) in params.p0().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Update selection state
                selection_state.player_team = team_button.team_type.clone();
                pressed_team = Some(team_button.team_type.clone());
            }
            Interaction::Hovered => {
                if selection_state.player_team != team_button.team_type {
                    *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.4)); // Hover
                    hovered_teams.push(team_button.team_type.clone());
                }
            }
            Interaction::None => {
                if selection_state.player_team != team_button.team_type {
                    *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3)); // Default
                    none_teams.push(team_button.team_type.clone());
                }
            }
        }
    }
    
    // If a team was pressed, update all button colors
    if let Some(selected_team) = pressed_team {
        for (button, mut bg_color) in params.p1().iter_mut() {
            if button.team_type == selected_team {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.5, 0.7)); // Selected
            } else {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3)); // Default
            }
        }
    }
}

/// Handle AI count selection
pub fn handle_ai_count_selection(
    mut params: ParamSet<(
        Query<
            (&Interaction, &AICountButton, &mut BackgroundColor),
            (Changed<Interaction>, With<Button>),
        >,
        Query<(&AICountButton, &mut BackgroundColor)>,
    )>,
    mut selection_state: ResMut<TeamSelectionState>,
) {
    // First, collect information from the interaction query
    let mut pressed_count: Option<u8> = None;
    
    // Process interactions
    for (interaction, ai_button, mut color) in params.p0().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Update selection state
                selection_state.ai_count = ai_button.count;
                
                // Generate random AI teams
                let available_teams = TeamType::all_teams();
                selection_state.ai_teams.clear();
                for i in 0..ai_button.count {
                    let team_index = (i as usize) % available_teams.len();
                    selection_state.ai_teams.push(available_teams[team_index].clone());
                }
                
                pressed_count = Some(ai_button.count);
            }
            Interaction::Hovered => {
                if selection_state.ai_count != ai_button.count {
                    *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.4)); // Hover
                }
            }
            Interaction::None => {
                if selection_state.ai_count != ai_button.count {
                    *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3)); // Default
                }
            }
        }
    }
    
    // If a count was pressed, update all button colors
    if let Some(selected_count) = pressed_count {
        for (button, mut bg_color) in params.p1().iter_mut() {
            if button.count == selected_count {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.5, 0.7)); // Selected
            } else {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3)); // Default
            }
        }
    }
}

/// Handle start game button
pub fn handle_start_game(
    interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<StartGameButton>),
    >,
    selection_state: Res<TeamSelectionState>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    ui_query: Query<Entity, With<TeamSelectionUI>>,
) {
    for interaction in interaction_query.iter() {
        if matches!(*interaction, Interaction::Pressed) {
            // Create game setup resource
            let game_setup = GameSetup {
                player_team: selection_state.player_team.clone(),
                ai_teams: selection_state.ai_teams.clone(),
                player_count: 1 + selection_state.ai_count,
            };
            commands.insert_resource(game_setup);
            
            // Clean up team selection UI
            for entity in ui_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
            
            // Transition to game
            next_state.set(GameState::Playing);
            info!("Starting game with team: {:?}, AI teams: {:?}", 
                  selection_state.player_team, selection_state.ai_teams);
        }
    }
}

/// Cleanup team selection UI when leaving state
pub fn cleanup_team_selection_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<TeamSelectionUI>>,
) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}