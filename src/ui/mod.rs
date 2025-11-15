// UI module containing all UI-related functionality

pub mod building_panel;
pub mod button_styles;
pub mod icons;
pub mod placement;
pub mod resource_display;
pub mod team_selection;
pub mod tooltip;

pub use building_panel::*;
pub use placement::*;
pub use resource_display::*;
// pub use button_styles::*; // Not currently used
pub use icons::*;
// pub use team_building_panel::*; // Unused
// pub use team_selection::*; // Unused
pub use tooltip::*;

use bevy::prelude::*;
use crate::core::game::GameState;
use crate::core::ai_intervals::should_run_resources;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerResources>()
            .init_resource::<BuildingPlacement>()
            .init_resource::<UIIcons>()
            .init_resource::<HoveredUnit>()
            .add_systems(
                Startup,
                (
                    load_ui_icons,
                    setup_tooltip,
                    setup_resource_tooltip,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                setup_ai_resource_display,
            )
            // Real-time UI that must be responsive
            .add_systems(
                Update,
                (
                    handle_building_panel_interactions,  // Player input must be responsive  
                    handle_building_placement,           // Player input must be responsive
                    building_hotkeys_system,            // Player input must be responsive
                    unit_hover_detection_system,        // Real-time hover feedback
                    update_tooltip_system,              // Real-time tooltip updates
                    resource_hover_system,              // Real-time hover feedback
                    resource_hover_effects_system,      // Real-time hover effects
                    update_placement_status,            // Real-time placement feedback
                ).run_if(in_state(GameState::Playing)),
            )
            // Resource and status displays - update every 1 second (less critical)
            .add_systems(
                Update,
                (
                    sync_player_resources,
                    update_resource_display,
                    manage_ai_resource_display,
                    update_ai_resource_display,
                    update_production_queue_display,
                ).run_if(in_state(GameState::Playing))
                .run_if(should_run_resources),
            );
    }
}
