use bevy::prelude::*;
use crate::core::components::{RTSUnit, Selectable};

/// Resource to track current cursor state
#[derive(Resource, Debug, Default)]
pub struct CursorState {
    pub hovered_type: HoveredType,
    pub cursor_visible: bool,
    #[allow(dead_code)] // Placeholder for cursor lock functionality
    pub cursor_locked: bool,
}

/// System to manage cursor appearance based on game context
pub fn cursor_management_system(
    mut windows: Query<&mut Window>,
    mut cursor_state: ResMut<CursorState>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    units_query: Query<(&Transform, &RTSUnit, &Selectable), With<crate::core::components::RTSUnit>>,
    buildings_query: Query<(&Transform, &crate::core::components::Building, &crate::core::components::RTSUnit)>,
    resources_query: Query<(&Transform, &crate::core::components::ResourceSource)>,
    placement: Option<Res<crate::ui::placement::BuildingPlacement>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    let mut window = windows.single_mut();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let mut cursor_visible = true;
    let mut cursor_hit_test = true;
    let mut hovered_type = HoveredType::None;

    // Check if we're in building placement mode
    if let Some(placement) = placement {
        if placement.active_building.is_some() {
            // Building placement mode
            cursor_visible = true;
            cursor_hit_test = false; // Don't let UI elements interfere with placement
            hovered_type = HoveredType::BuildingPlacement;
        }
    }
    // Check if holding Alt (for special actions)
    else if keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight) {
        cursor_visible = true;
        hovered_type = HoveredType::SpecialAction;
    }
    // Check if dragging (selection box)
    else if mouse_button.pressed(MouseButton::Left) {
        cursor_visible = true;
        cursor_hit_test = false; // Make selection more responsive
        hovered_type = HoveredType::Dragging;
    }
    // Check what we're hovering over
    else {
        let mut closest_distance = f32::MAX;

        // Check for units
        for (transform, unit, selectable) in units_query.iter() {
            let distance_to_ray = ray.get_point(1000.0).distance(transform.translation);
            if distance_to_ray < selectable.selection_radius && distance_to_ray < closest_distance {
                closest_distance = distance_to_ray;
                hovered_type = if unit.player_id == 1 {
                    HoveredType::FriendlyUnit
                } else {
                    HoveredType::EnemyUnit
                };
            }
        }

        // Check for buildings
        for (transform, _building, building_unit) in buildings_query.iter() {
            let distance_to_ray = ray.get_point(1000.0).distance(transform.translation);
            if distance_to_ray < 10.0 && distance_to_ray < closest_distance {
                closest_distance = distance_to_ray;
                hovered_type = if building_unit.player_id == 1 {
                    HoveredType::FriendlyBuilding
                } else {
                    HoveredType::EnemyBuilding
                };
            }
        }

        // Check for resources
        for (transform, _resource) in resources_query.iter() {
            let distance_to_ray = ray.get_point(1000.0).distance(transform.translation);
            if distance_to_ray < 8.0 && distance_to_ray < closest_distance {
                closest_distance = distance_to_ray;
                hovered_type = HoveredType::Resource;
            }
        }

        // Set cursor behavior based on what we're hovering
        match hovered_type {
            HoveredType::FriendlyUnit | HoveredType::FriendlyBuilding => {
                cursor_visible = true;
                cursor_hit_test = true;
            }
            HoveredType::EnemyUnit | HoveredType::EnemyBuilding => {
                cursor_visible = true; 
                cursor_hit_test = false; // Make cursor more responsive for combat
            }
            HoveredType::Resource => {
                cursor_visible = true;
                cursor_hit_test = true;
            }
            HoveredType::None => {
                cursor_visible = true;
                cursor_hit_test = true;
            }
            _ => {
                cursor_visible = true;
                cursor_hit_test = true;
            }
        };
    }

    // Update cursor state resource for other systems to use
    cursor_state.hovered_type = hovered_type;
    cursor_state.cursor_visible = cursor_visible;

    // Apply cursor settings
    window.cursor_options.visible = cursor_visible;
    window.cursor_options.hit_test = cursor_hit_test;
}

/// Helper enum to track what type of object we're hovering over
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HoveredType {
    #[default]
    None,
    FriendlyUnit,
    EnemyUnit,
    FriendlyBuilding,
    EnemyBuilding,
    Resource,
    BuildingPlacement,
    SpecialAction,
    Dragging,
}

