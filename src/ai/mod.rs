// AI module containing all AI-related functionality
// Enhanced RTS AI with scouting, tactics, intelligence gathering, and adaptive strategies

pub mod combat_ai;
pub mod decision_making;
pub mod economy;
pub mod intelligence;
pub mod player_state;
pub mod resource_management;
pub mod scouting;
pub mod strategy;
pub mod tactics;
pub mod unit_management;

pub use combat_ai::*;
pub use decision_making::*;
pub use resource_management::*;
pub use unit_management::*;
// pub use player_state::*;  // Unused - remove to fix warning
pub use economy::*;
pub use intelligence::*;
pub use scouting::*;
pub use strategy::*;
pub use tactics::*;

use bevy::prelude::*;
use crate::core::ai_intervals::*;

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add AI intervals plugin first
            .add_plugins(AIIntervalsPlugin)
            // Initialize AI resources
            .init_resource::<AIStrategy>()
            .init_resource::<IntelligenceSystem>()
            .init_resource::<TacticalManager>()
            .init_resource::<EconomyManager>()
            .init_resource::<LogRateLimiter>()
            // Startup systems
            .add_systems(
                Startup,
                (ai_resource_initialization_system, setup_ai_intelligence),
            )
            // Intelligence and reconnaissance phase - run every 1 second
            .add_systems(
                Update,
                (
                    intelligence_update_system,
                    scouting_system,
                    scout_survival_system,
                )
                    .chain()
                    .run_if(in_state(crate::core::game::GameState::Playing))
                    .run_if(should_run_intelligence),
            )
            // Strategic decision making - run every 2 seconds
            .add_systems(
                Update, 
                ai_strategy_system
                    .run_if(in_state(crate::core::game::GameState::Playing))
                    .run_if(should_run_strategy)
            )
            // Tactical planning phase - run every 0.5 seconds  
            .add_systems(
                Update,
                (
                    tactical_decision_system,
                    ai_decision_system,
                )
                    .chain()
                    .run_if(in_state(crate::core::game::GameState::Playing))
                    .run_if(should_run_tactics),
            )
            // Economy optimization - run every 1.5 seconds
            .add_systems(
                Update,
                (
                    economy_optimization_system,
                    worker_idle_detection_system,
                )
                    .chain()
                    .run_if(in_state(crate::core::game::GameState::Playing))
                    .run_if(should_run_economy),
            )
            // Combat systems - run every 0.2 seconds (more responsive)
            .add_systems(
                Update,
                (
                    ai_combat_system,
                    army_coordination_system,
                    combat_to_resource_transition_system,
                )
                    .chain()
                    .run_if(in_state(crate::core::game::GameState::Playing))
                    .run_if(should_run_combat),
            )
            // Unit and resource management - run every 0.3 seconds and 1 second respectively
            .add_systems(
                Update,
                ai_unit_management_system
                    .run_if(in_state(crate::core::game::GameState::Playing))
                    .run_if(should_run_unit_management),
            )
            .add_systems(
                Update,
                (
                    ai_resource_management_system,
                    ai_worker_initialization_system,
                    ai_worker_dropoff_system,
                )
                    .chain()
                    .run_if(in_state(crate::core::game::GameState::Playing))
                    .run_if(should_run_resources),
            );
    }
}

/// Setup intelligence system for AI players - now dynamically initialized in world system
fn setup_ai_intelligence(_intelligence: ResMut<IntelligenceSystem>) {
    // Intelligence system is now initialized dynamically when AI players are spawned
    info!("AI Intelligence System ready for dynamic initialization");
}
