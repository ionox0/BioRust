use bevy::prelude::*;
use crate::components::*;

pub fn formation_system(
    mut units: Query<(&mut Movement, &Formation), With<RTSUnit>>,
    leaders: Query<&Position, (With<RTSUnit>, Without<Formation>)>,
) {
    for (mut movement, formation) in units.iter_mut() {
        update_formation_position(&mut movement, formation, &leaders);
    }
}

fn update_formation_position(
    movement: &mut Movement,
    formation: &Formation,
    leaders: &Query<&Position, (With<RTSUnit>, Without<Formation>)>,
) {
    let Some(leader_entity) = formation.leader else { return };
    
    let Ok(leader_position) = leaders.get(leader_entity) else { return };
    
    let formation_offset = calculate_formation_offset(formation);
    movement.target_position = Some(leader_position.translation + formation_offset);
}

fn calculate_formation_offset(formation: &Formation) -> Vec3 {
    match formation.formation_type {
        FormationType::Line => {
            Vec3::new(formation.position_in_formation.x * 10.0, 0.0, 0.0)
        },
        FormationType::Box => {
            Vec3::new(
                formation.position_in_formation.x * 8.0,
                0.0,
                formation.position_in_formation.y * 8.0
            )
        },
        FormationType::Wedge => {
            Vec3::new(
                formation.position_in_formation.x * 6.0,
                0.0,
                formation.position_in_formation.y * 12.0
            )
        },
        FormationType::Circle => {
            let angle = formation.position_in_formation.x;
            let radius = formation.position_in_formation.y;
            Vec3::new(
                angle.cos() * radius,
                0.0,
                angle.sin() * radius
            )
        },
    }
}