use bevy::prelude::*;
use crate::systems::*;
use crate::resources::*;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    MainMenu,
    #[default]
    Playing, // Start directly in Playing state to test animations
    // Removed unused states: Loading, Paused, GameOver
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
        app
            .init_state::<GameState>()
            .configure_sets(
                Update,
                (
                    GameSet::Input,
                    GameSet::Logic,
                    GameSet::Physics,
                    GameSet::Rendering,
                ).chain()
            )
            .init_resource::<GameConfig>()
            .init_resource::<Score>()
            .init_resource::<GameTimer>()
            .init_resource::<PlayerResources>()
            .init_resource::<AIResources>()
            .insert_resource(GameCosts::initialize())
            .add_systems(Startup, (setup_game, spawn_rts_elements))
            .add_systems(
                Update,
                (
                    handle_rts_camera_input.in_set(GameSet::Input),
                    // Removed references to dead systems
                ).run_if(in_state(GameState::Playing))
            )
            .add_systems(
                OnEnter(GameState::MainMenu),
                setup_menu
            )
            .add_systems(
                Update,
                handle_menu_input.run_if(in_state(GameState::MainMenu))
            );
    }
}