use bevy::prelude::*;
use crate::core::game::GameState;

pub mod construction;
pub mod cursor_manager;
pub mod formation;
pub mod movement;
pub mod production;
pub mod resource_gathering;
pub mod selection;
pub mod unit_commands;
pub mod unstuck;
pub mod vision;

use construction::{ai_construction_workflow_system, construction_system};
use cursor_manager::{cursor_management_system, CursorState};
use formation::formation_system;
use movement::movement_system;
use production::{building_completion_system, population_management_system, production_system};
use resource_gathering::resource_gathering_system;
use selection::{
    click_selection_system, create_selection_indicators, drag_selection_system,
    selection_indicator_system,
};
use unit_commands::{spawn_test_units_system, unit_command_system, FormationSettings};
use unstuck::{add_stuck_detection_system, unstuck_system};
use vision::vision_system;

pub struct RTSSystemsPlugin;

impl Plugin for RTSSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FormationSettings>()
            .init_resource::<CursorState>()
            .add_systems(
            Update,
            (
                add_stuck_detection_system,
                movement_system,
                unstuck_system,
                resource_gathering_system,
                click_selection_system,
                drag_selection_system,
                production_system,
                construction_system,
                ai_construction_workflow_system,
                formation_system,
                vision_system,
                unit_command_system,
                create_selection_indicators,
                selection_indicator_system,
                spawn_test_units_system,
                building_completion_system,
                population_management_system,
                cursor_management_system,
            ).run_if(in_state(GameState::Playing)),
        );
    }
}
