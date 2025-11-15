// UI module containing all UI-related functionality

pub mod building_panel;
pub mod button_styles;
pub mod icons;
pub mod placement;
pub mod resource_display;
pub mod team_building_panel;
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
            .add_systems(
                Update,
                (
                    sync_player_resources,
                    update_resource_display,
                    manage_ai_resource_display,
                    update_ai_resource_display,
                    handle_building_panel_interactions,
                    handle_building_placement,
                    update_production_queue_display,
                    update_placement_status,
                    building_hotkeys_system,
                    unit_hover_detection_system,
                    update_tooltip_system,
                    resource_hover_system,
                    resource_hover_effects_system,
                ).run_if(in_state(GameState::Playing)),
            );
    }
}
