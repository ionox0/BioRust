use crate::core::components::*;
use bevy::prelude::*;

/// Resource to track which unit is currently being hovered
#[derive(Resource, Default)]
pub struct HoveredUnit {
    pub entity: Option<Entity>,
    pub last_update: f32,
}

/// Component marking the tooltip UI element
#[derive(Component)]
pub struct UnitTooltip;

/// Component for the tooltip text
#[derive(Component)]
pub struct TooltipText;

/// Setup the tooltip UI (invisible by default)
pub fn setup_tooltip(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                display: Display::None, // Hidden by default
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
            BorderColor(Color::srgb(0.6, 0.6, 0.6)),
            UnitTooltip,
            ZIndex(1000),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TooltipText,
            ));
        });
}

/// System to detect which unit is under the cursor
pub fn unit_hover_detection_system(
    mut hovered_unit: ResMut<HoveredUnit>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    units: Query<(Entity, &Transform, &RTSUnit, &Selectable)>,
    time: Res<Time>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        hovered_unit.entity = None;
        return;
    };

    // Convert cursor position to world ray
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        hovered_unit.entity = None;
        return;
    };

    // Find the closest unit to the cursor ray
    let mut closest_distance = f32::INFINITY;
    let mut closest_entity = None;

    for (entity, transform, _unit, selectable) in units.iter() {
        // Show tooltips for all units (both player and AI)

        // Calculate distance from ray to unit
        let to_entity = transform.translation - ray.origin;
        let projected_distance = to_entity.dot(*ray.direction);

        if projected_distance <= 0.0 {
            continue;
        }

        let closest_point = ray.origin + *ray.direction * projected_distance;
        let distance_to_ray = closest_point.distance(transform.translation);

        // Check if cursor is within selection radius
        if distance_to_ray < selectable.selection_radius && projected_distance < closest_distance {
            closest_distance = projected_distance;
            closest_entity = Some(entity);
        }
    }

    // Update hovered unit
    if hovered_unit.entity != closest_entity {
        hovered_unit.entity = closest_entity;
        hovered_unit.last_update = time.elapsed_secs();
    }
}

/// Determine the current task of a unit
fn get_unit_task(
    entity: Entity,
    gatherer_query: &Query<&ResourceGatherer>,
    combat_query: &Query<&Combat>,
    movement_query: &Query<&Movement>,
    construction_query: &Query<&ConstructionTask>,
    building_query: &Query<&Building>,
    gathering_state_query: &Query<&GatheringState>,
    combat_state_query: &Query<&CombatState>,
    health_query: &Query<&RTSHealth>,
) -> String {
    // Check if it's a building first
    if let Ok(building) = building_query.get(entity) {
        let completion_percent =
            (building.construction_progress / building.max_construction * 100.0) as i32;
        return format!(
            "{:?} ({}% complete)",
            building.building_type, completion_percent
        );
    }

    // Check if constructing
    if let Ok(construction) = construction_query.get(entity) {
        if construction.is_moving_to_site {
            return format!("Moving to construction site");
        } else {
            return format!(
                "Constructing {:?} ({}%)",
                construction.building_type,
                (construction.construction_progress * 100.0) as i32
            );
        }
    }

    // Check if gathering resources
    if let Ok(gatherer) = gatherer_query.get(entity) {
        if let Some(ref resource_type) = gatherer.resource_type {
            if gatherer.carried_amount > 0.0 {
                return format!(
                    "Returning {:?} ({:.0})",
                    resource_type, gatherer.carried_amount
                );
            } else if gatherer.target_resource.is_some() {
                return format!("Gathering {:?}", resource_type);
            }
        }
    }

    // Check specialized states for more accurate state information
    
    // Check for death first
    if let Ok(health) = health_query.get(entity) {
        if health.current <= 0.0 {
            return "Dead".to_string();
        }
    }
    
    // Check gathering state
    if let Ok(gathering_state) = gathering_state_query.get(entity) {
        match gathering_state.state {
            GatheringStateType::MovingToResource => return "Moving to Resource".to_string(),
            GatheringStateType::Gathering => return "Gathering Resources".to_string(),
            GatheringStateType::ReturningToBase => return "Returning to Base".to_string(),
            GatheringStateType::DeliveringResources => return "Delivering Resources".to_string(),
        }
    }
    
    // Check combat state
    if let Ok(combat_state) = combat_state_query.get(entity) {
        match combat_state.state {
            CombatStateType::InCombat => return "In Combat".to_string(),
            CombatStateType::MovingToAttack => return "Moving to Attack".to_string(),
            CombatStateType::MovingToCombat => return "Engaging Enemy".to_string(),
            _ => {} // Continue to other checks
        }
    }

    // Check if in combat (only for units that actually auto-attack AND have a target)
    if let Ok(combat) = combat_query.get(entity) {
        if combat.auto_attack && combat.target.is_some() {
            if let Ok(movement) = movement_query.get(entity) {
                if movement.target_position.is_some() {
                    return "Moving to attack".to_string();
                }
            }
            return "In combat".to_string();
        }
    }

    // Check if just moving
    if let Ok(movement) = movement_query.get(entity) {
        if movement.target_position.is_some() {
            return "Moving".to_string();
        }
    }

    // Default to idle
    "Idle".to_string()
}

/// System to update tooltip content and position
pub fn update_tooltip_system(
    hovered_unit: Res<HoveredUnit>,
    units: Query<(&RTSUnit, &RTSHealth, &Transform)>,
    gatherer_query: Query<&ResourceGatherer>,
    combat_query: Query<&Combat>,
    movement_query: Query<&Movement>,
    construction_query: Query<&ConstructionTask>,
    building_query: Query<&Building>,
    gathering_state_query: Query<&GatheringState>,
    combat_state_query: Query<&CombatState>,
    health_query: Query<&RTSHealth>,
    mut tooltip_query: Query<(&mut Node, &mut BackgroundColor), With<UnitTooltip>>,
    mut text_query: Query<&mut Text, With<TooltipText>>,
    windows: Query<&Window>,
) {
    let Ok((mut tooltip_style, mut tooltip_bg)) = tooltip_query.get_single_mut() else {
        return;
    };
    let Ok(mut tooltip_text) = text_query.get_single_mut() else {
        return;
    };

    if let Some(hovered_entity) = hovered_unit.entity {
        // Get unit info
        if let Ok((unit, health, _transform)) = units.get(hovered_entity) {
            let task = get_unit_task(
                hovered_entity,
                &gatherer_query,
                &combat_query,
                &movement_query,
                &construction_query,
                &building_query,
                &gathering_state_query,
                &combat_state_query,
                &health_query,
            );

            let unit_type = match unit.unit_type {
                Some(UnitType::WorkerAnt) => "Worker Ant",
                Some(UnitType::SoldierAnt) => "Soldier Ant",
                Some(UnitType::HunterWasp) => "Hunter Wasp",
                Some(UnitType::BeetleKnight) => "Beetle Knight",
                Some(UnitType::SpearMantis) => "Spear Mantis",
                Some(UnitType::ScoutAnt) => "Scout Ant",
                Some(UnitType::DragonFly) => "DragonFly",
                _ => "Unit",
            };

            let player_name = if unit.player_id == 1 {
                "Player".to_string()
            } else {
                format!("AI Player {}", unit.player_id)
            };

            // Update tooltip text
            **tooltip_text = format!(
                "{} ({})\nHealth: {:.0}/{:.0}\nTask: {}",
                unit_type, player_name, health.current, health.max, task
            );

            // Position tooltip near cursor
            if let Ok(window) = windows.get_single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    tooltip_style.left = Val::Px(cursor_pos.x + 15.0);
                    tooltip_style.top = Val::Px(cursor_pos.y + 15.0);
                }
            }

            // Show tooltip with color based on player
            tooltip_style.display = Display::Flex;
            if unit.player_id == 1 {
                // Player units: friendly blue-green tint
                *tooltip_bg = BackgroundColor(Color::srgba(0.05, 0.15, 0.15, 0.95));
            } else {
                // AI units: neutral or enemy red tint
                *tooltip_bg = BackgroundColor(Color::srgba(0.15, 0.05, 0.05, 0.95));
            }
        }
    } else {
        // Hide tooltip
        tooltip_style.display = Display::None;
    }
}
