use bevy::prelude::*;
use crate::core::components::*;

// Helper functions for selection system

/// System for raycast-based single-click selection
pub fn click_selection_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut selectables: Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>,
) {
    // Only trigger on left mouse button release (not drag)
    if !mouse_button.just_released(MouseButton::Left) {
        return;
    }

    let window = windows.single();
    let Some(cursor_position) = window.cursor_position() else { return };
    let (camera, camera_transform) = camera_q.single();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else { return };

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Find closest unit to raycast
    let mut closest_entity = None;
    let mut closest_distance = f32::INFINITY;

    for (entity, selectable, transform, unit) in selectables.iter() {
        // Only select player 1 units
        if unit.player_id != 1 {
            continue;
        }

        let to_unit = transform.translation - ray.origin;
        let projected_distance = to_unit.dot(ray.direction.normalize());

        if projected_distance <= 0.0 {
            continue;
        }

        let closest_point = ray.origin + ray.direction.normalize() * projected_distance;
        let distance_to_ray = closest_point.distance(transform.translation);

        // Use selection radius
        if distance_to_ray < selectable.selection_radius && projected_distance < closest_distance {
            closest_distance = projected_distance;
            closest_entity = Some(entity);
        }
    }

    // Apply selection
    if let Some(selected_entity) = closest_entity {
        for (entity, mut selectable, _, unit) in selectables.iter_mut() {
            if unit.player_id != 1 {
                continue;
            }

            if entity == selected_entity {
                if shift_held {
                    selectable.is_selected = !selectable.is_selected;
                } else {
                    selectable.is_selected = true;
                }
                info!("✅ Selected unit {:?}", unit.unit_id);
            } else if !shift_held {
                selectable.is_selected = false;
            }
        }
    } else if !shift_held {
        // Clicked on empty space without shift - clear all selections
        for (_, mut selectable, _, unit) in selectables.iter_mut() {
            if unit.player_id == 1 && selectable.is_selected {
                selectable.is_selected = false;
            }
        }
    }
}

pub fn drag_selection_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut selectables: Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drag_selection_query: Query<&mut DragSelection>,
    selection_box_query: Query<Entity, With<SelectionBox>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    let Some(cursor_position) = window.cursor_position() else { return };

    if mouse_button.just_pressed(MouseButton::Left) {
        start_drag_selection(&mut drag_selection_query, cursor_position, &mut commands);
    }

    if mouse_button.pressed(MouseButton::Left) {
        update_drag_selection(&mut drag_selection_query, cursor_position, &selection_box_query, &mut commands, &mut meshes, &mut materials);
    }

    if mouse_button.just_released(MouseButton::Left) {
        finalize_selection(&mut drag_selection_query, &mut selectables, &keyboard, &selection_box_query, cursor_position, &mut commands, camera, camera_transform);
    }
}

fn start_drag_selection(
    drag_selection_query: &mut Query<&mut DragSelection>,
    cursor_position: Vec2,
    commands: &mut Commands,
) {
    if drag_selection_query.is_empty() {
        commands.spawn(DragSelection {
            start_position: cursor_position,
            current_position: cursor_position,
            is_active: true,
        });
    } else if let Ok(mut drag_selection) = drag_selection_query.get_single_mut() {
        drag_selection.start_position = cursor_position;
        drag_selection.current_position = cursor_position;
        drag_selection.is_active = true;
    }
}

fn update_drag_selection(
    drag_selection_query: &mut Query<&mut DragSelection>,
    cursor_position: Vec2,
    selection_box_query: &Query<Entity, With<SelectionBox>>,
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let Ok(mut drag_selection) = drag_selection_query.get_single_mut() else { return };
    
    if !drag_selection.is_active {
        return;
    }
    
    drag_selection.current_position = cursor_position;
    
    let bounds = calculate_selection_bounds(&drag_selection);
    
    cleanup_old_selection_box(selection_box_query, commands);
    
    if is_significant_drag(&bounds) {
        create_visual_selection_box(&bounds, commands);
    }
}

struct SelectionBounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

fn calculate_selection_bounds(drag_selection: &DragSelection) -> SelectionBounds {
    SelectionBounds {
        min_x: drag_selection.start_position.x.min(drag_selection.current_position.x),
        max_x: drag_selection.start_position.x.max(drag_selection.current_position.x),
        min_y: drag_selection.start_position.y.min(drag_selection.current_position.y),
        max_y: drag_selection.start_position.y.max(drag_selection.current_position.y),
    }
}

fn is_significant_drag(bounds: &SelectionBounds) -> bool {
    (bounds.max_x - bounds.min_x > 5.0) && (bounds.max_y - bounds.min_y > 5.0)
}

fn cleanup_old_selection_box(
    selection_box_query: &Query<Entity, With<SelectionBox>>,
    commands: &mut Commands,
) {
    for entity in selection_box_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn create_visual_selection_box(bounds: &SelectionBounds, commands: &mut Commands) {
    let width = bounds.max_x - bounds.min_x;
    let height = bounds.max_y - bounds.min_y;

    // Spawn a UI node that shows the selection box
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(bounds.min_x),
            top: Val::Px(bounds.min_y),
            width: Val::Px(width),
            height: Val::Px(height),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor(Color::srgba(0.3, 0.8, 1.0, 0.8)), // Bright blue border
        BackgroundColor(Color::srgba(0.3, 0.8, 1.0, 0.15)), // Semi-transparent blue fill
        SelectionBox,
    ));
}

fn finalize_selection(
    drag_selection_query: &mut Query<&mut DragSelection>,
    selectables: &mut Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>,
    keyboard: &Res<ButtonInput<KeyCode>>,
    selection_box_query: &Query<Entity, With<SelectionBox>>,
    _cursor_position: Vec2,
    commands: &mut Commands,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    let Ok(mut drag_selection) = drag_selection_query.get_single_mut() else { return };

    if !drag_selection.is_active {
        return;
    }

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let bounds = calculate_selection_bounds(&drag_selection);
    let is_drag = is_significant_drag(&bounds);

    if is_drag {
        if !shift_held {
            clear_all_selections(selectables);
        }
        perform_box_selection(&bounds, selectables, shift_held, camera, camera_transform);
    }
    // For single clicks, don't clear selections here - let click_selection_system handle it

    drag_selection.is_active = false;
    cleanup_old_selection_box(selection_box_query, commands);
}

fn clear_all_selections(selectables: &mut Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>) {
    for (_, mut selectable, _, _) in selectables.iter_mut() {
        selectable.is_selected = false;
    }
}

fn perform_box_selection(
    bounds: &SelectionBounds,
    selectables: &mut Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>,
    shift_held: bool,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) {
    let mut selected_count = 0;

    for (_entity, mut selectable, transform, unit) in selectables.iter_mut() {
        if unit.player_id != 1 {
            continue;
        }

        // Convert unit world position to screen space
        if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, transform.translation) {
            // Check if the unit's screen position is within the selection bounds
            if screen_pos.x >= bounds.min_x
                && screen_pos.x <= bounds.max_x
                && screen_pos.y >= bounds.min_y
                && screen_pos.y <= bounds.max_y
            {
                update_selection_state(&mut selectable, shift_held);
                selected_count += 1;
            }
        }
    }

    info!("📦 Box selected {} units", selected_count);
}

fn perform_click_selection(
    _cursor_position: Vec2,
    selectables: &mut Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>,
    shift_held: bool,
    
) {
    // TODO: Implement click selection with proper camera/viewport conversion
    // For now, just select the first selectable unit for player 1
    for (entity, _selectable, _, unit) in selectables.iter_mut() {
        if unit.player_id == 1 {
            apply_single_selection(entity, selectables, shift_held);
            break;
        }
    }
}

fn find_closest_selectable(
    ray: Ray3d,
    selectables: &Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>,
) -> Option<Entity> {
    let mut closest_distance = f32::INFINITY;
    let mut closest_entity = None;
    
    for (entity, selectable, transform, unit) in selectables.iter() {
        if unit.player_id != 1 {
            continue;
        }
        
        let to_entity = transform.translation - ray.origin;
        let projected_distance = to_entity.dot(ray.direction.normalize());
        
        if projected_distance <= 0.0 {
            continue;
        }
        
        let closest_point = ray.origin + ray.direction.normalize() * projected_distance;
        let distance_to_ray = closest_point.distance(transform.translation);
        
        if distance_to_ray < selectable.selection_radius && projected_distance < closest_distance {
            closest_distance = projected_distance;
            closest_entity = Some(entity);
        }
    }
    
    closest_entity
}

fn apply_single_selection(
    selected_entity: Entity,
    selectables: &mut Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>,
    shift_held: bool,
    
) {
    for (entity, mut selectable, transform, unit) in selectables.iter_mut() {
        if entity == selected_entity {
            update_selection_state(&mut selectable, shift_held);
            
            info!("✅ Selected unit {} at position {:?}", unit.unit_id, transform.translation);
            
            if selectable.is_selected {
                // TODO: spawn_selection_indicator(entity, transform, &selectable);
            }
            break;
        }
    }
}

fn update_selection_state(selectable: &mut Selectable, shift_held: bool) {
    if shift_held {
        selectable.is_selected = !selectable.is_selected;
    } else {
        selectable.is_selected = true;
    }
}

fn spawn_selection_indicator(
    entity: Entity,
    transform: &Transform,
    selectable: &Selectable,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Create a blue ring using a torus mesh
    let ring_radius = selectable.selection_radius;
    let ring_thickness = 0.1;

    commands.spawn((
        Mesh3d(meshes.add(Torus::new(ring_thickness, ring_radius))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.6, 1.0), // Bright blue
            emissive: Color::srgb(0.2, 0.4, 0.8).into(), // Blue glow
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(Vec3::new(
            transform.translation.x,
            0.1, // Slightly above ground
            transform.translation.z
        ))
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)), // Rotate to lay flat
        SelectionIndicator { target: entity },
    ));
}

/// System to create selection indicators for newly selected units
pub fn create_selection_indicators(
    mut commands: Commands,
    selectables: Query<(Entity, &Selectable, &Transform), (With<RTSUnit>, Changed<Selectable>)>,
    existing_indicators: Query<&SelectionIndicator>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, selectable, transform) in selectables.iter() {
        let has_indicator = existing_indicators.iter().any(|ind| ind.target == entity);

        if selectable.is_selected && !has_indicator {
            // Unit is selected but doesn't have an indicator yet - create one
            spawn_selection_indicator(entity, transform, selectable, &mut commands, &mut meshes, &mut materials);
        }
    }
}

/// System to update selection indicator positions and remove indicators for deselected units
pub fn selection_indicator_system(
    mut selection_indicators: Query<(Entity, &mut Transform, &SelectionIndicator), With<SelectionIndicator>>,
    selectables: Query<(&Selectable, &Transform), (With<RTSUnit>, Without<SelectionIndicator>)>,
    mut commands: Commands,
) {
    for (indicator_entity, mut indicator_transform, selection_indicator) in selection_indicators.iter_mut() {
        if let Ok((selectable, unit_transform)) = selectables.get(selection_indicator.target) {
            if selectable.is_selected {
                // Update indicator position to follow the unit
                indicator_transform.translation.x = unit_transform.translation.x;
                indicator_transform.translation.z = unit_transform.translation.z;
            } else {
                // Unit is no longer selected - remove the indicator
                commands.entity(indicator_entity).despawn();
            }
        } else {
            // Target unit no longer exists - remove the indicator
            commands.entity(indicator_entity).despawn();
        }
    }
}