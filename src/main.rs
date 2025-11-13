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
use tracing_subscriber::layer::SubscriberExt;
use core::constants;
use core::game::*;
use core::time_controls::TimeControlPlugin;
use health_ui::HealthUIPlugin;
use ui::UIPlugin;
use world::terrain_v2::TerrainPluginV2;
use world::GridPlugin;
// use resource_ui::ResourceUIPlugin;  // REMOVED: Duplicate of ui::resource_display
use collision::CollisionPlugin;
use entities::entity_state_systems::EntityStatePlugin;
use rendering::animation_systems::AnimationPlugin;
use rendering::hover_effects::HoverEffectsPlugin;
use rendering::model_loader::ModelLoaderPlugin;

use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::util::SubscriberInitExt; // <-- important
use tracing_flame::FlameLayer;
use tracing_subscriber::Registry;


fn main() {

    let file = std::fs::File::create("flamegraph.folded").unwrap();
    let flame_layer = FlameLayer::new(file);

    // Attach the flame layer to the registry
    let subscriber = Registry::default().with(flame_layer);

    // .init() is provided by SubscriberInitExt
    subscriber.init();

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: constants::WINDOW_TITLE.into(),
                    resolution: (constants::WINDOW_WIDTH, constants::WINDOW_HEIGHT).into(),
                    mode: bevy::window::WindowMode::Windowed,
                    ..default()
                }),
                ..default()
            }),
            GamePlugin,
            TerrainPluginV2,
            GridPlugin,
            rts::RTSSystemsPlugin,
            CombatPlugin,
            HealthUIPlugin,
            UIPlugin, // Contains comprehensive resource display system
            AIPlugin,
            // ResourceUIPlugin,  // REMOVED: Duplicate overlapping resource display
            ModelLoaderPlugin,
            HoverEffectsPlugin,
            AnimationPlugin,
            EntityStatePlugin,
            CollisionPlugin,
            TimeControlPlugin, // Fast-forward and time control system
        ))
        .run();
}
