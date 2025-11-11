use bevy::prelude::*;

pub mod movement;
pub mod resource_gathering;
pub mod selection;
pub mod production;
pub mod construction;
pub mod formation;
pub mod vision;
pub mod unit_commands;
pub mod unstuck;

use movement::movement_system;
use resource_gathering::resource_gathering_system;
use selection::{click_selection_system, drag_selection_system, selection_indicator_system, create_selection_indicators};
use production::{production_system, building_completion_system, population_management_system};
use construction::{construction_system, ai_construction_workflow_system};
use formation::formation_system;
use vision::vision_system;
use unit_commands::{unit_command_system, spawn_test_units_system};
use unstuck::{unstuck_system, add_stuck_detection_system};

pub struct RTSSystemsPlugin;

impl Plugin for RTSSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
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
        ));
    }
}