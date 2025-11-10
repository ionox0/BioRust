use bevy::prelude::*;
use crate::core::components::*;

pub struct HealthUIPlugin;

impl Plugin for HealthUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            create_health_bars_for_new_units,
            update_health_bars,
            cleanup_orphaned_health_bars,
            health_status_indicator_system,
        ));
    }
}

#[derive(Component)]
pub struct HealthBarUI {
    pub target_entity: Entity,
    pub background: Entity,
    pub foreground: Entity,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,   // 80-100% health
    Wounded,   // 40-79% health
    Critical,  // 1-39% health
}

impl HealthStatus {
    pub fn from_health_ratio(ratio: f32) -> Self {
        if ratio >= 0.8 {
            Self::Healthy
        } else if ratio >= 0.4 {
            Self::Wounded
        } else {
            Self::Critical
        }
    }
    
    pub fn color(&self) -> Color {
        match self {
            Self::Healthy => Color::srgb(0.2, 0.8, 0.2),   // Green
            Self::Wounded => Color::srgb(1.0, 0.8, 0.0),   // Yellow
            Self::Critical => Color::srgb(1.0, 0.2, 0.2),  // Red
        }
    }
}

#[derive(Component)]
pub struct HealthStatusIndicator {
    pub target_entity: Entity,
}

// System to create health bars for new units that don't have them
pub fn create_health_bars_for_new_units(
    mut commands: Commands,
    units_without_health_bars: Query<(Entity, &Transform, &RTSHealth), (With<RTSUnit>, Without<HealthBarUI>)>,
    existing_health_bars: Query<&HealthBarUI>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, transform, _health) in units_without_health_bars.iter() {
        // Check if this unit already has a health bar
        let has_health_bar = existing_health_bars.iter().any(|hb| hb.target_entity == entity);
        
        if !has_health_bar {
            // Create health bar background (red)
            let background = commands.spawn((
                Mesh3d(meshes.add(Rectangle::new(2.0, 0.3))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.8, 0.2, 0.2),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(transform.translation + Vec3::new(0.0, 3.0, 0.0))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            )).id();

            // Create health bar foreground (green)
            let foreground = commands.spawn((
                Mesh3d(meshes.add(Rectangle::new(2.0, 0.3))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.8, 0.2),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(transform.translation + Vec3::new(0.0, 3.1, 0.0))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            )).id();

            // Add health bar component to the unit
            commands.entity(entity).insert((
                HealthBarUI {
                    target_entity: entity,
                    background,
                    foreground,
                },
                HealthStatus::Healthy, // Start with healthy status
            ));
        }
    }
}

// System to update health bar positions and health display
pub fn update_health_bars(
    mut unit_query: Query<(Entity, &Transform, &RTSHealth, &HealthBarUI, &mut HealthStatus), With<RTSUnit>>,
    mut health_bar_transforms: Query<&mut Transform, (Without<RTSUnit>, Without<Camera3d>)>,
    health_bar_materials: Query<&MeshMaterial3d<StandardMaterial>, (Without<RTSUnit>, Without<Camera3d>)>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<RTSUnit>, Without<HealthBarUI>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get camera transform
    let camera_transform = if let Ok(camera_transform) = camera_query.get_single() {
        *camera_transform
    } else {
        return;
    };

    for (_entity, unit_transform, health, health_bar, mut health_status) in unit_query.iter_mut() {
        let health_ratio = health.current / health.max;
        
        // Update health status
        let new_status = HealthStatus::from_health_ratio(health_ratio);
        let status_changed = *health_status != new_status;
        *health_status = new_status;
        
        // Always show health bars for units (changed to always be visible)
        let should_show = true;
        
        if should_show {
            let bar_position = unit_transform.translation + Vec3::new(0.0, 3.0, 0.0);
            
            // Calculate camera facing rotation
            let to_camera = (camera_transform.translation - bar_position).normalize();
            let up = Vec3::Y;
            let right = to_camera.cross(up).normalize();
            let forward = up.cross(right);
            let rotation = Quat::from_mat3(&Mat3::from_cols(right, up, forward));
            
            // Update background position and rotation
            if let Ok(mut bg_transform) = health_bar_transforms.get_mut(health_bar.background) {
                bg_transform.translation = bar_position;
                bg_transform.rotation = rotation;
                bg_transform.scale = Vec3::ONE;
            }
            
            // Update foreground position, rotation, and scale
            if let Ok(mut fg_transform) = health_bar_transforms.get_mut(health_bar.foreground) {
                let fg_position = bar_position + Vec3::new(0.0, 0.01, 0.0); // Slightly above background
                let offset_x = (1.0 - health_ratio) * -1.0; // Adjust for scaling from left
                let final_fg_pos = fg_position + right * offset_x;
                
                fg_transform.translation = final_fg_pos;
                fg_transform.rotation = rotation;
                fg_transform.scale = Vec3::new(health_ratio, 1.0, 1.0);
            }
            
            // Update foreground color based on health status if status changed
            if status_changed {
                if let Ok(fg_material_handle) = health_bar_materials.get(health_bar.foreground) {
                    if let Some(material) = materials.get_mut(&fg_material_handle.0) {
                        material.base_color = health_status.color();
                    }
                }
            }
        } else {
            // Hide health bars for healthy units
            if let Ok(mut bg_transform) = health_bar_transforms.get_mut(health_bar.background) {
                bg_transform.scale = Vec3::ZERO; // Hide by scaling to zero
            }
            if let Ok(mut fg_transform) = health_bar_transforms.get_mut(health_bar.foreground) {
                fg_transform.scale = Vec3::ZERO; // Hide by scaling to zero
            }
        }
    }
}

// System to clean up health bars for units that no longer exist
pub fn cleanup_orphaned_health_bars(
    mut commands: Commands,
    health_bars: Query<(Entity, &HealthBarUI)>,
    units: Query<Entity, With<RTSUnit>>,
) {
    for (health_bar_entity, health_bar) in health_bars.iter() {
        // Check if the target unit still exists
        if units.get(health_bar.target_entity).is_err() {
            // Unit doesn't exist, clean up health bar
            commands.entity(health_bar.background).despawn();
            commands.entity(health_bar.foreground).despawn();
            commands.entity(health_bar_entity).remove::<HealthBarUI>();
        }
    }
}

// System to add visual indicators for different health statuses
pub fn health_status_indicator_system(
    mut commands: Commands,
    units_query: Query<(Entity, &Transform, &RTSHealth, &HealthStatus, &Selectable), (With<RTSUnit>, Changed<HealthStatus>)>,
    existing_indicators: Query<(Entity, &HealthStatusIndicator)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, transform, _health, status, _selectable) in units_query.iter() {
        // Remove existing status indicator if any
        for (indicator_entity, indicator) in existing_indicators.iter() {
            if indicator.target_entity == entity {
                commands.entity(indicator_entity).despawn();
            }
        }
        
        // Add status indicator for wounded or critical units
        match status {
            HealthStatus::Wounded | HealthStatus::Critical => {
                let indicator_color = status.color();
                let indicator_size = match status {
                    HealthStatus::Wounded => 0.3,
                    HealthStatus::Critical => 0.5,
                    _ => 0.0,
                };
                
                // Create a small sphere above the unit to indicate health status
                let _indicator_entity = commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(indicator_size))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: indicator_color,
                        emissive: Color::srgb(0.0, 0.5, 0.0).into(),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    Transform::from_translation(
                        transform.translation + Vec3::new(0.0, 4.0, 0.0)
                    ),
                    HealthStatusIndicator {
                        target_entity: entity,
                    },
                )).id();
                
                // Reduced critical condition logging (only on status change)
                if matches!(status, HealthStatus::Critical) {
                    warn!("⚠️ Unit {:?} is now in critical condition!", entity);
                }
            }
            HealthStatus::Healthy => {
                // No indicator needed for healthy units
            }
        }
    }
    
    // Update indicator positions for moving units
    let mut indicator_updates = Vec::new();
    for (indicator_entity, indicator) in existing_indicators.iter() {
        if let Ok((_, transform, _, _, _)) = units_query.get(indicator.target_entity) {
            indicator_updates.push((
                indicator_entity, 
                transform.translation + Vec3::new(0.0, 4.0, 0.0)
            ));
        }
    }
    
    // Apply position updates - disabled for now
    // for (indicator_entity, new_position) in indicator_updates {
    //     // Position update logic would go here
    // }
}