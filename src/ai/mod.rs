// AI module containing all AI-related functionality

pub mod decision_making;
pub mod unit_management;
pub mod resource_management;
pub mod combat_ai;
pub mod player_state;

pub use decision_making::*;
pub use unit_management::*;
pub use resource_management::*;
pub use combat_ai::*;
pub use player_state::*;

use bevy::prelude::*;

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            ai_decision_system,
            ai_unit_management_system,
            ai_resource_management_system,
            ai_combat_system,
        ));
    }
}