use bevy::prelude::*;
use crate::components::*;

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
        update_drag_selection(&mut drag_selection_query, cursor_position, &selection_box_query, &mut commands, &mut meshes, &mut materials, camera, camera_transform);
    }
    
    if mouse_button.just_released(MouseButton::Left) {
        finalize_selection(&mut drag_selection_query, &mut selectables, &keyboard, &selection_box_query, cursor_position, &mut commands, &mut meshes, &mut materials, camera, camera_transform);
    }
}

fn start_drag_selection(
    drag_selection_query: &mut Query<&mut DragSelection>,
    cursor_position: Vec2,
    context: &mut SelectionContext,
) {
    if drag_selection_query.is_empty() {
        context.commands.spawn(DragSelection {
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
    context: &mut SelectionContext,
) {
    let Ok(mut drag_selection) = drag_selection_query.get_single_mut() else { return };
    
    if !drag_selection.is_active {
        return;
    }
    
    drag_selection.current_position = cursor_position;
    
    let bounds = calculate_selection_bounds(&drag_selection);
    
    cleanup_old_selection_box(selection_box_query, context);
    
    if is_significant_drag(&bounds) {
        create_visual_selection_box(&bounds, context);
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
    context: &mut SelectionContext,
) {
    for entity in selection_box_query.iter() {
        context.commands.entity(entity).despawn();
    }
}

fn create_visual_selection_box(bounds: &SelectionBounds, context: &mut SelectionContext) {
    let center_screen = Vec2::new(
        (bounds.min_x + bounds.max_x) * 0.5,
        (bounds.min_y + bounds.max_y) * 0.5
    );
    let size = Vec2::new(bounds.max_x - bounds.min_x, bounds.max_y - bounds.min_y);
    
    if let Ok(ray) = context.camera.viewport_to_world(context.camera_transform, center_screen) {
        let ground_y = 1.0;
        let t = (ground_y - ray.origin.y) / ray.direction.y.max(0.001);
        
        if t > 0.0 {
            let world_pos = ray.origin + ray.direction * t;
            
            context.commands.spawn((
                Mesh3d(context.meshes.add(Rectangle::new(size.x * 0.1, size.y * 0.1))),
                MeshMaterial3d(context.materials.add(StandardMaterial {
                    base_color: Color::srgba(0.0, 1.0, 0.0, 0.3),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(world_pos)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                SelectionBox,
            ));
        }
    }
}

fn finalize_selection(
    drag_selection_query: &mut Query<&mut DragSelection>,
    selectables: &mut Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>,
    keyboard: &Res<ButtonInput<KeyCode>>,
    selection_box_query: &Query<Entity, With<SelectionBox>>,
    cursor_position: Vec2,
    context: &mut SelectionContext,
) {
    let Ok(mut drag_selection) = drag_selection_query.get_single_mut() else { return };
    
    if !drag_selection.is_active {
        return;
    }
    
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let bounds = calculate_selection_bounds(&drag_selection);
    let is_drag = is_significant_drag(&bounds);
    
    if !shift_held && !is_drag {
        clear_all_selections(selectables);
    }
    
    if is_drag {
        perform_box_selection(&bounds, selectables, shift_held, context);
    } else {
        perform_click_selection(cursor_position, selectables, shift_held, context);
    }
    
    drag_selection.is_active = false;
    cleanup_old_selection_box(selection_box_query, context);
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
    context: &mut SelectionContext,
) {
    let mut selected_count = 0;
    
    for (entity, mut selectable, transform, unit) in selectables.iter_mut() {
        if unit.player_id != 1 {
            continue;
        }
        
        if let Ok(screen_pos) = context.camera.world_to_viewport(context.camera_transform, transform.translation) {
            if screen_pos.x >= bounds.min_x && screen_pos.x <= bounds.max_x &&
               screen_pos.y >= bounds.min_y && screen_pos.y <= bounds.max_y {
                
                update_selection_state(&mut selectable, shift_held);
                selected_count += 1;
                
                if selectable.is_selected {
                    spawn_selection_indicator(entity, transform, &selectable, context);
                }
            }
        }
    }
    
    info!("📦 Box selected {} units", selected_count);
}

fn perform_click_selection(
    cursor_position: Vec2,
    selectables: &mut Query<(Entity, &mut Selectable, &Transform, &RTSUnit)>,
    shift_held: bool,
    context: &mut SelectionContext,
) {
    if let Ok(ray) = context.camera.viewport_to_world(context.camera_transform, cursor_position) {
        let closest_entity = find_closest_selectable(ray, selectables);
        
        if let Some(selected_entity) = closest_entity {
            apply_single_selection(selected_entity, selectables, shift_held, context);
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
    context: &mut SelectionContext,
) {
    for (entity, mut selectable, transform, unit) in selectables.iter_mut() {
        if entity == selected_entity {
            update_selection_state(&mut selectable, shift_held);
            
            info!("✅ Selected unit {} at position {:?}", unit.unit_id, transform.translation);
            
            if selectable.is_selected {
                spawn_selection_indicator(entity, transform, &selectable, context);
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
    context: &mut SelectionContext,
) {
    context.commands.spawn((
        Mesh3d(context.meshes.add(Torus::new(selectable.selection_radius * 0.8, 0.5))),
        MeshMaterial3d(context.materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0),
            emissive: Color::srgb(0.0, 0.5, 0.0).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(Vec3::new(transform.translation.x, 1.0, transform.translation.z)),
        SelectionIndicator { target: entity },
    ));
}

pub fn selection_indicator_system(
    mut selection_indicators: Query<(Entity, &mut Transform, &SelectionIndicator), With<SelectionIndicator>>,
    selectables: Query<(&Selectable, &Transform), (With<RTSUnit>, Without<SelectionIndicator>)>,
    mut commands: Commands,
) {
    for (indicator_entity, mut indicator_transform, selection_indicator) in selection_indicators.iter_mut() {
        if let Ok((selectable, unit_transform)) = selectables.get(selection_indicator.target) {
            if selectable.is_selected {
                indicator_transform.translation.x = unit_transform.translation.x;
                indicator_transform.translation.z = unit_transform.translation.z;
            } else {
                commands.entity(indicator_entity).despawn();
            }
        } else {
            commands.entity(indicator_entity).despawn();
        }
    }
}