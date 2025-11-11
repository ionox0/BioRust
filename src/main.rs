use bevy::prelude::*;

mod core;
mod world;
mod entities;
mod rts;
mod rendering;
mod ui;
mod ai;
mod health_ui;
mod combat_systems;
mod collision;

use core::game::*;
use core::constants;
use world::terrain_v2::TerrainPluginV2;
use combat_systems::CombatPlugin;
use health_ui::HealthUIPlugin;
use ui::UIPlugin;
use ai::AIPlugin;
// use resource_ui::ResourceUIPlugin;  // REMOVED: Duplicate of ui::resource_display
use rendering::model_loader::ModelLoaderPlugin;
use rendering::animation_systems::AnimationPlugin;
use entities::entity_state_systems::EntityStatePlugin;
use collision::CollisionPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: constants::WINDOW_TITLE.into(),
                    resolution: (constants::WINDOW_WIDTH, constants::WINDOW_HEIGHT).into(),
                    ..default()
                }),
                ..default()
            }),
            GamePlugin,
            TerrainPluginV2,
            rts::RTSSystemsPlugin,
            CombatPlugin,
            HealthUIPlugin,
            UIPlugin,  // Contains comprehensive resource display system
            AIPlugin,
            // ResourceUIPlugin,  // REMOVED: Duplicate overlapping resource display
            ModelLoaderPlugin,
            AnimationPlugin,
            EntityStatePlugin,
            CollisionPlugin,
        ))
        .run();
}
