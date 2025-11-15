use crate::core::resources::*;
use crate::world::systems::*;
use crate::ui::team_selection::*;
use bevy::prelude::*;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    TeamSelection,
    Playing,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    Input,
    Logic,
    Physics,
    Rendering,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .configure_sets(
                Update,
                (
                    GameSet::Input,
                    GameSet::Logic,
                    GameSet::Physics,
                    GameSet::Rendering,
                )
                    .chain(),
            )
            .init_resource::<GameConfig>()
            .init_resource::<Score>()
            .init_resource::<GameTimer>()
            .init_resource::<PlayerResources>()
            .init_resource::<AIResources>()
            .init_resource::<SpatialGrids>()
            .init_resource::<TeamSelectionState>()
            .insert_resource(GameCosts::initialize())
            .add_systems(
                Update,
                (
                    handle_rts_camera_input.in_set(GameSet::Input),
                    // Removed references to dead systems
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::MainMenu), setup_menu)
            .add_systems(
                Update,
                handle_menu_input.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnEnter(GameState::TeamSelection), setup_team_selection_ui)
            .add_systems(
                Update,
                (
                    handle_team_selection,
                    handle_ai_count_selection,
                    handle_start_game,
                ).run_if(in_state(GameState::TeamSelection)),
            )
            .add_systems(OnExit(GameState::TeamSelection), cleanup_team_selection_ui)
            .add_systems(OnEnter(GameState::Playing), (spawn_rts_elements_with_teams, setup_game_ui));
            // Team building panel updates handled in UI plugin
    }
}
