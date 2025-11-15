use bevy::prelude::*;
use crate::core::game::GameState;
use crate::core::ai_intervals::{should_run_unit_management, should_run_resources};

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
use movement::{movement_system, terrain_collision_correction_system};
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
            // Real-time systems that need to run every frame for responsiveness
            .add_systems(
                Update,
                (
                    movement_system,              // Units need smooth movement
                    click_selection_system,       // Player input must be responsive
                    drag_selection_system,        // Player input must be responsive
                    unit_command_system,          // Player commands must be responsive
                    cursor_management_system,     // UI must be responsive
                    vision_system,               // Vision updates for real-time gameplay
                    selection_indicator_system,   // Selection rings must track movement in real-time
                ).run_if(in_state(GameState::Playing)),
            )
            // Unit management systems - run every 0.3 seconds
            .add_systems(
                Update,
                (
                    add_stuck_detection_system,
                    unstuck_system,
                    formation_system,
                    create_selection_indicators,
                    spawn_test_units_system,
                    terrain_collision_correction_system, // Periodically correct units below terrain
                ).run_if(in_state(GameState::Playing))
                .run_if(should_run_unit_management),
            )
            // Resource and production systems - run every 1 second
            .add_systems(
                Update,
                (
                    resource_gathering_system,
                    production_system,
                    construction_system,
                    ai_construction_workflow_system,
                    building_completion_system,
                    population_management_system,
                ).run_if(in_state(GameState::Playing))
                .run_if(should_run_resources),
            );
    }
}
