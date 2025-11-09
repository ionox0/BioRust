use bevy::prelude::*;
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};

mod game;
mod systems;
mod components;
mod resources;
mod terrain;
mod terrain_v2;
mod rts_entities;
mod rts_systems;
mod seamless_texture;
mod combat_systems;
mod health_ui;
mod ui;
mod ai;
mod resource_ui;
mod model_loader;
mod debug_health;
mod constants;
mod animation_systems;
mod entity_state_systems;
mod model_showcase;

use game::*;
use terrain_v2::TerrainPluginV2;
use rts_systems::RTSSystemsPlugin;
use combat_systems::CombatPlugin;
use health_ui::HealthUIPlugin;
use ui::UIPlugin;
use ai::AIPlugin;
use resource_ui::ResourceUIPlugin;
use model_loader::ModelLoaderPlugin;
use animation_systems::AnimationPlugin;
use entity_state_systems::EntityStatePlugin;
use model_showcase::ModelShowcasePlugin;

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
            WireframePlugin,
            GamePlugin,
            TerrainPluginV2,
            RTSSystemsPlugin,
            CombatPlugin,
            HealthUIPlugin,
            UIPlugin,
            AIPlugin,
            ResourceUIPlugin,
            ModelLoaderPlugin,
            AnimationPlugin,
            EntityStatePlugin,
            ModelShowcasePlugin,
        ))
        .add_systems(Update, toggle_wireframe)
        .run();
}

// Debug function to toggle wireframe rendering
fn toggle_wireframe(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut wireframe_query: Query<(Entity, Option<&Wireframe>)>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(constants::hotkeys::WIREFRAME_TOGGLE) {
        for (entity, wireframe) in wireframe_query.iter_mut() {
            if wireframe.is_some() {
                commands.entity(entity).remove::<Wireframe>();
                info!("Removed wireframe from entity {:?}", entity);
            } else {
                commands.entity(entity).insert(Wireframe);
                info!("Added wireframe to entity {:?}", entity);
            }
        }
    }
}
