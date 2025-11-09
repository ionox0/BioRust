use bevy::prelude::*;
use crate::resources::PlayerResources;
use crate::game::GameState;

pub struct ResourceUIPlugin;

impl Plugin for ResourceUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_resource_ui)
           .add_systems(Update, update_resource_ui.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct ResourceText;

#[derive(Component)]
pub struct PopulationText;

pub fn setup_resource_ui(mut commands: Commands) {
    // Create UI root
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    )).with_children(|parent| {
        // Resource display
        parent.spawn((
            Text::new("Resources: "),
            TextFont::from_font_size(20.0),
            TextColor(Color::WHITE),
            ResourceText,
        ));
        
        // Population display
        parent.spawn((
            Text::new("Population: "),
            TextFont::from_font_size(18.0),
            TextColor(Color::WHITE),
            PopulationText,
        ));
        
        // Instructions
        parent.spawn((
            Text::new("Controls:\n• Click to select units\n• Right-click to move/attack\n• M: Spawn Militia\n• A: Spawn Archer\n• E: Spawn Enemy\n• T: Toggle wireframe"),
            TextFont::from_font_size(14.0),
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });
    
    info!("Resource UI setup complete");
}

pub fn update_resource_ui(
    player_resources: Res<PlayerResources>,
    mut resource_query: Query<&mut Text, (With<ResourceText>, Without<PopulationText>)>,
    mut population_query: Query<&mut Text, (With<PopulationText>, Without<ResourceText>)>,
) {
    // Update resource text
    if let Ok(mut text) = resource_query.get_single_mut() {
        **text = format!(
            "Resources: 🍯{:.0} 🦎{:.0} ⚡{:.0} 🌸{:.0}",
            player_resources.nectar,
            player_resources.chitin,
            player_resources.minerals,
            player_resources.pheromones
        );
    }
    
    // Update population text
    if let Ok(mut text) = population_query.get_single_mut() {
        **text = format!(
            "Population: {}/{} (🐜{})",
            player_resources.current_population,
            player_resources.max_population,
            player_resources.idle_workers
        );
    }
}