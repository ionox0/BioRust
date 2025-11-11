// UI module containing all UI-related functionality

pub mod building_panel;
pub mod resource_display;
pub mod placement;
pub mod button_styles;
pub mod icons;
pub mod tooltip;

pub use building_panel::*;
pub use resource_display::*;
pub use placement::*;
// pub use button_styles::*; // Not currently used
pub use icons::*;
pub use tooltip::*;

use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerResources>()
            .init_resource::<BuildingPlacement>()
            .init_resource::<UIIcons>()
            .init_resource::<HoveredUnit>()
            .add_systems(Startup, (load_ui_icons, setup_building_ui, setup_tooltip, setup_ai_resource_display).chain())
            .add_systems(Update, sync_player_resources)
            .add_systems(Update, update_resource_display)
            .add_systems(Update, update_ai_resource_display)
            .add_systems(Update, handle_building_panel_interactions)
            .add_systems(Update, handle_building_placement)
            .add_systems(Update, update_production_queue_display)
            .add_systems(Update, update_placement_status)
            .add_systems(Update, building_hotkeys_system)
            .add_systems(Update, unit_hover_detection_system)
            .add_systems(Update, update_tooltip_system);
    }
}