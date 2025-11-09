// UI module containing all UI-related functionality

pub mod building_panel;
pub mod resource_display;
pub mod placement;
pub mod button_styles;
pub mod icons;

pub use building_panel::*;
pub use resource_display::*;
pub use placement::*;
// pub use button_styles::*; // Not currently used
pub use icons::*;

use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerResources>()
            .init_resource::<BuildingPlacement>()
            .init_resource::<UIIcons>()
            .add_systems(Startup, (load_ui_icons, setup_building_ui).chain())
            .add_systems(Update, (
                sync_player_resources,
                update_resource_display,
                handle_building_panel_interactions,
                handle_building_placement,
                update_production_queue_display,
                update_placement_status,
                building_hotkeys_system,
            ));
    }
}