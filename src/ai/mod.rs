// AI module containing all AI-related functionality
// Enhanced RTS AI with scouting, tactics, intelligence gathering, and adaptive strategies

pub mod decision_making;
pub mod unit_management;
pub mod resource_management;
pub mod combat_ai;
pub mod player_state;
pub mod strategy;
pub mod intelligence;
pub mod scouting;
pub mod tactics;
pub mod economy;

pub use decision_making::*;
pub use unit_management::*;
pub use resource_management::*;
pub use combat_ai::*;
// pub use player_state::*;  // Unused - remove to fix warning
pub use strategy::*;
pub use intelligence::*;
pub use scouting::*;
pub use tactics::*;
pub use economy::*;

use bevy::prelude::*;

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize AI resources
            .init_resource::<AIStrategy>()
            .init_resource::<IntelligenceSystem>()
            .init_resource::<TacticalManager>()
            .init_resource::<EconomyManager>()
            // Startup systems
            .add_systems(Startup, (
                ai_resource_initialization_system,
                setup_ai_intelligence,
            ))
            // Core AI systems - split into smaller groups to avoid parameter limits
            .add_systems(Update, (
                // Intelligence and reconnaissance phase
                intelligence_update_system,
                scouting_system,
                scout_survival_system,
            ).chain())
            .add_systems(Update, (
                // Tactical planning phase
                tactical_decision_system,
                economy_optimization_system,
                worker_idle_detection_system,
            ).chain())
            .add_systems(Update, ai_decision_system)
            .add_systems(Update, ai_strategy_system)
            .add_systems(Update, (
                // Execution phase
                ai_unit_management_system,
                ai_resource_management_system,
                army_coordination_system,
                ai_combat_system,
                combat_to_resource_transition_system,  // Handle transition from combat to resource gathering
                ai_worker_initialization_system,
                ai_worker_dropoff_system,
            ).chain());
    }
}

/// Setup intelligence system for AI players
fn setup_ai_intelligence(
    mut intelligence: ResMut<IntelligenceSystem>,
) {
    // Initialize intelligence tracking for AI player 2 (monitoring player 1)
    intelligence.initialize_player(2, 1);
    info!("AI Intelligence System initialized - AI Player 2 will scout Player 1");
}