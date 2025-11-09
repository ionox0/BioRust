use bevy::prelude::*;
use crate::components::*;

// Debug system to test health tracking (press H to damage units, J to heal)
pub fn debug_health_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut units_query: Query<(Entity, &mut RTSHealth, &Selectable), With<RTSUnit>>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        // Damage selected units
        for (entity, mut health, selectable) in units_query.iter_mut() {
            if selectable.is_selected {
                health.current = (health.current - 20.0).max(1.0); // Damage but don't kill
                info!("💥 Damaged unit {:?} - Health: {:.1}/{:.1}", entity, health.current, health.max);
            }
        }
    }
    
    if keyboard.just_pressed(KeyCode::KeyJ) {
        // Heal selected units
        for (entity, mut health, selectable) in units_query.iter_mut() {
            if selectable.is_selected {
                health.current = (health.current + 25.0).min(health.max);
                info!("💚 Healed unit {:?} - Health: {:.1}/{:.1}", entity, health.current, health.max);
            }
        }
    }
}