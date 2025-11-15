use bevy::prelude::*;

mod ai;
mod collision;
mod combat_systems;
mod core;
mod entities;
mod health_ui;
mod rendering;
mod rts;
mod ui;
mod world;

use ai::AIPlugin;
use combat_systems::CombatPlugin;
use core::constants;
use core::game::*;
use core::query_cache::QueryCachePlugin;
use core::time_controls::TimeControlPlugin;
use health_ui::HealthUIPlugin;
use ui::UIPlugin;
use world::{StaticTerrainPlugin, GridPlugin};
// use resource_ui::ResourceUIPlugin;  // REMOVED: Duplicate of ui::resource_display
use collision::CollisionPlugin;
use entities::entity_state_systems::EntityStatePlugin;
use rendering::animation_systems::AnimationPlugin;
use rendering::hover_effects::HoverEffectsPlugin;
use rendering::model_loader::ModelLoaderPlugin;


fn main() {
    // Set log level to info to see debug messages, but filter out verbose asset warnings
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,bevy_gltf=warn,bevy_ui::layout=error");
    }

    // Initialize console logging
    tracing_subscriber::fmt::init();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: constants::WINDOW_TITLE.into(),
                resolution: (constants::WINDOW_WIDTH, constants::WINDOW_HEIGHT).into(),
                mode: bevy::window::WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }))
        // Core game plugins
        .add_plugins((
            GamePlugin,
            QueryCachePlugin, // High-performance query cache system - must be early in the pipeline
            StaticTerrainPlugin, // Fast preloaded terrain system for RTS gameplay
            GridPlugin,
        ))
        // Gameplay plugins
        .add_plugins((
            rts::RTSSystemsPlugin,
            CombatPlugin,
            HealthUIPlugin,
            UIPlugin, // Contains comprehensive resource display system
        ))
        // AI and rendering plugins
        .add_plugins((
            AIPlugin,
            ModelLoaderPlugin,
            HoverEffectsPlugin,
            AnimationPlugin,
        ))
        // Additional systems
        .add_plugins((
            EntityStatePlugin,
            CollisionPlugin,
            TimeControlPlugin, // Fast-forward and time control system
        ))
        .run();
}
