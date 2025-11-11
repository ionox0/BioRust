use bevy::prelude::*;
use crate::core::components::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                target_acquisition_system,
                attack_system,
                damage_resolution_system,
                health_regeneration_system,
                death_system,
                health_bar_system,
                combat_movement_system,
            ).chain())
            .add_event::<DamageEvent>()
            .add_event::<DeathEvent>();
    }
}

// System to find and acquire targets for units with combat capability
pub fn target_acquisition_system(
    mut combat_query: Query<(Entity, &mut Combat, &Transform, &RTSUnit, &Movement), With<Vision>>,
    potential_targets: Query<(Entity, &Transform, &RTSUnit), (With<RTSHealth>, Without<DeathEvent>)>,
    vision_query: Query<&Vision>,
    _time: Res<Time>,
) {
    for (entity, mut combat, transform, unit, movement) in combat_query.iter_mut() {
        // Skip player units (player_id == 1) if they have a movement target (player gave move command)
        // This allows player move commands to override auto-attacking
        if unit.player_id == 1 && movement.target_position.is_some() && combat.target.is_none() {
            continue;
        }

        // Skip if already has a valid target and is not auto-attacking
        if combat.target.is_some() && !combat.auto_attack {
            continue;
        }

        // Get vision component - give enemies enhanced vision
        let default_vision = Vision::default();
        let vision = vision_query.get(entity).unwrap_or(&default_vision);
        
        // Use standard vision range for balanced gameplay
        let effective_vision_range = vision.sight_range;

        let mut closest_enemy: Option<(Entity, f32)> = None;

        // Find the closest enemy within range
        for (target_entity, target_transform, target_unit) in potential_targets.iter() {
            // Skip same player units (no friendly fire)
            if target_unit.player_id == unit.player_id {
                continue;
            }

            let distance = transform.translation.distance(target_transform.translation);
            
            // Check if target is within vision range
            if distance <= effective_vision_range {
                // For enemies, prioritize player units even at longer distances
                let is_enemy_seeking_player = unit.player_id != 1 && target_unit.player_id == 1;
                
                if distance <= combat.attack_range {
                    // Target within attack range - prefer closer targets
                    match closest_enemy {
                        Some((_, closest_dist)) if distance < closest_dist => {
                            closest_enemy = Some((target_entity, distance));
                        }
                        None => {
                            closest_enemy = Some((target_entity, distance));
                        }
                        _ => {}
                    }
                } else if is_enemy_seeking_player || closest_enemy.is_none() {
                    // Enemies will actively seek player units at longer range
                    closest_enemy = Some((target_entity, distance));
                }
            }
        }

        // Set target if found
        if let Some((target_entity, _)) = closest_enemy {
            combat.target = Some(target_entity);
        } else if combat.auto_attack {
            // Clear target if auto-attacking and no enemies found
            combat.target = None;
        }
    }
}

// System to handle attacks and combat timing
pub fn attack_system(
    mut combat_query: Query<(Entity, &mut Combat, &Transform, &RTSUnit, &CollisionRadius)>,
    target_query: Query<(&Transform, &CollisionRadius), (With<RTSHealth>, Without<DeathEvent>)>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (attacker_entity, mut combat, attacker_transform, _unit, attacker_collision) in combat_query.iter_mut() {
        // Check if we have a target
        let target_entity = match combat.target {
            Some(target) => target,
            None => {
                combat.is_attacking = false;
                continue;
            }
        };

        // Check if target still exists
        let (target_transform, target_collision) = match target_query.get(target_entity) {
            Ok((transform, collision)) => (transform, collision),
            Err(_) => {
                // Target no longer exists, clear it
                combat.target = None;
                combat.is_attacking = false;
                continue;
            }
        };

        // Calculate distance between centers
        let center_distance = attacker_transform.translation.distance(target_transform.translation);
        
        // Calculate edge-to-edge distance by subtracting collision radii
        let edge_distance = center_distance - attacker_collision.radius - target_collision.radius;
        
        // Check if target is in range (using edge distance for more realistic combat)
        if edge_distance <= combat.attack_range {
            // Check if enough time has passed since last attack
            if current_time - combat.last_attack_time >= combat.attack_cooldown {
                // Execute attack
                combat.last_attack_time = current_time;
                combat.is_attacking = true;

                // Determine damage type based on attack type
                let damage_type = match combat.attack_type {
                    AttackType::Melee => DamageType::Physical,
                    AttackType::Ranged => DamageType::Pierce,
                    AttackType::Siege => DamageType::Siege,
                };

                // Send damage event
                damage_events.send(DamageEvent {
                    damage: combat.attack_damage,
                    attacker: attacker_entity,
                    target: target_entity,
                    damage_type,
                });

                info!("⚔️ Unit {:?} attacks {:?} for {} damage (edge_dist: {:.1}, attack_range: {:.1})", 
                      attacker_entity, target_entity, combat.attack_damage, edge_distance, combat.attack_range);
            }
        } else {
            combat.is_attacking = false;
            // Debug occasional range issues  
            if edge_distance <= combat.attack_range + 2.0 { // Only log when close to range
                debug!("🎯 Unit {:?} out of range: edge_dist {:.1} > attack_range {:.1} (center_dist: {:.1}, att_radius: {:.1}, tgt_radius: {:.1})", 
                       attacker_entity, edge_distance, combat.attack_range, center_distance, 
                       attacker_collision.radius, target_collision.radius);
            }
        }
    }
}

// System to handle movement towards combat targets
pub fn combat_movement_system(
    mut unit_query: Query<(&mut Movement, &Combat, &Transform, &RTSUnit)>,
    target_query: Query<&Transform, With<RTSHealth>>,
) {
    for (mut movement, combat, unit_transform, unit) in unit_query.iter_mut() {
        // Only apply combat movement if unit has a combat target
        if let Some(target_entity) = combat.target {
            if let Ok(target_transform) = target_query.get(target_entity) {
                let distance = unit_transform.translation.distance(target_transform.translation);

                // Move towards target if out of attack range
                if distance > combat.attack_range * 0.8 { // Use 80% of range to avoid jittering
                    // Calculate direction towards target
                    let direction = (target_transform.translation - unit_transform.translation).normalize();
                    let target_position = target_transform.translation - direction * (combat.attack_range * 0.7);

                    // Only override movement for AI units or if player unit has no other movement target
                    // This prevents overriding explicit player move commands
                    if unit.player_id != 1 || movement.target_position.is_none() {
                        movement.target_position = Some(target_position);
                    }
                    // DO NOT override max_speed or acceleration - use unit's configured stats
                } else {
                    // Stop moving when in range (only for AI units, player units retain manual control)
                    if unit.player_id != 1 {
                        movement.target_position = None;
                        movement.current_velocity = Vec3::ZERO;
                    }
                }
            }
        }
    }
}

// System to process damage events and apply damage
pub fn damage_resolution_system(
    mut commands: Commands,
    mut damage_events: EventReader<DamageEvent>,
    mut health_query: Query<(Entity, &mut RTSHealth), Without<Dying>>,
    mut combat_stats_query: Query<&mut CombatStats>,
    mut death_events: EventWriter<DeathEvent>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for damage_event in damage_events.read() {
        if let Ok((target_entity, mut health)) = health_query.get_mut(damage_event.target) {
            // Calculate actual damage based on armor and damage type
            let actual_damage = calculate_damage(damage_event.damage, health.armor, &damage_event.damage_type);
            
            // Apply damage
            health.current = (health.current - actual_damage).max(0.0);
            health.last_damage_time = current_time;

            // Update combat stats for attacker
            if let Ok(mut stats) = combat_stats_query.get_mut(damage_event.attacker) {
                stats.damage_dealt += actual_damage;
            }

            // Update combat stats for target
            if let Ok(mut stats) = combat_stats_query.get_mut(damage_event.target) {
                stats.damage_taken += actual_damage;
            }

            info!("Entity {:?} takes {} damage (current health: {})", 
                  target_entity, actual_damage, health.current);

            // Check if unit died
            if health.current <= 0.0 {
                // Mark entity as dying to prevent duplicate death processing
                commands.entity(target_entity).insert(Dying);
                
                // Update killer's stats
                if let Ok(mut stats) = combat_stats_query.get_mut(damage_event.attacker) {
                    stats.kills += 1;
                    const BASE_XP_FOR_KILL: f32 = 10.0;
                    stats.experience += BASE_XP_FOR_KILL; // Base XP for kill
                }

                // Send death event
                death_events.send(DeathEvent {
                    entity: target_entity,
                    killer: Some(damage_event.attacker),
                });
            }
        }
    }
}

// Calculate actual damage based on armor and damage type
fn calculate_damage(base_damage: f32, armor: f32, damage_type: &DamageType) -> f32 {
    match damage_type {
        DamageType::Physical => {
            // Physical damage reduced by armor
            let reduction = armor / (armor + 100.0);
            base_damage * (1.0 - reduction)
        }
        DamageType::Pierce => {
            // Pierce damage partially ignores armor
            let reduction = (armor * 0.5) / (armor * 0.5 + 100.0);
            base_damage * (1.0 - reduction)
        }
        DamageType::Siege => {
            // Siege damage ignores most armor
            let reduction = (armor * 0.1) / (armor * 0.1 + 100.0);
            base_damage * (1.0 - reduction)
        }
        DamageType::True => {
            // True damage ignores all armor
            base_damage
        }
    }
}

// System to handle health regeneration
pub fn health_regeneration_system(
    mut health_query: Query<&mut RTSHealth>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    const REGEN_DELAY: f32 = 5.0; // 5 seconds after last damage before regen starts
    let regen_delay = REGEN_DELAY;

    for mut health in health_query.iter_mut() {
        // Only regenerate if not at full health and enough time has passed since last damage
        if health.current < health.max && 
           current_time - health.last_damage_time >= regen_delay &&
           health.regeneration_rate > 0.0 {
            
            let regen_amount = health.regeneration_rate * time.delta_secs();
            health.current = (health.current + regen_amount).min(health.max);
        }
    }
}

// System to handle unit death
pub fn death_system(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    unit_query: Query<&RTSUnit>,
    mut combat_query: Query<&mut Combat>,
) {
    for death_event in death_events.read() {
        let dead_entity = death_event.entity;

        // Log death
        if let Ok(unit) = unit_query.get(dead_entity) {
            info!("Unit {:?} (Player {}) has died", dead_entity, unit.player_id);
        }

        // Clear this entity as a target from all combat components
        for mut combat in combat_query.iter_mut() {
            if combat.target == Some(dead_entity) {
                combat.target = None;
                combat.is_attacking = false;
            }
        }

        // Despawn the entity
        commands.entity(dead_entity).despawn_recursive();
    }
}

// System to update health bar visibility and position
pub fn health_bar_system(
    _health_bar_query: Query<(&mut Transform, &HealthBar), (With<HealthBar>, Without<RTSUnit>)>,
    unit_query: Query<(&Transform, &RTSHealth, &HealthBar), (With<RTSUnit>, Without<HealthBar>)>,
    _commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    // This is a simplified version - in a real implementation you'd want to use UI elements
    // or a more sophisticated health bar rendering system
    
    for (unit_transform, health, health_bar_config) in unit_query.iter() {
        // Only show health bar if damaged or always visible
        let should_show = health_bar_config.always_visible || health.current < health.max;
        
        if should_show {
            // Calculate health bar position
            let _health_bar_pos = unit_transform.translation + health_bar_config.offset;
            
            // Here you would update or create health bar UI elements
            // This is a placeholder for the actual health bar rendering
        }
    }
}

// Helper function to create a combat unit with default values
#[allow(dead_code)]
pub fn create_combat_unit(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    player_id: u8,
    unit_type: UnitType,
) -> Entity {
    let (health, mut combat, movement, mut vision, mesh_handle, material_handle) = match unit_type {
        UnitType::SoldierAnt => (
            RTSHealth {
                current: crate::constants::combat::SOLDIER_ANT_HEALTH,
                max: crate::constants::combat::SOLDIER_ANT_HEALTH,
                armor: 2.0,
                regeneration_rate: 0.5,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: crate::constants::combat::SOLDIER_ANT_DAMAGE,
                attack_range: 3.0,
                attack_speed: 1.5,
                last_attack_time: 0.0,
                target: None,
                attack_type: AttackType::Melee,
                attack_cooldown: 1.0 / 1.5, // 1.5 attacks per second
                is_attacking: false,
                auto_attack: true,
            },
            Movement::default(),
            Vision::default(),
            meshes.add(Cuboid::new(1.0, 2.0, 1.0)),
            materials.add(StandardMaterial {
                base_color: if player_id == 1 { Color::srgb(0.8, 0.2, 0.2) } else { Color::srgb(0.8, 0.4, 0.0) },
                ..default()
            }),
        ),
        UnitType::HunterWasp => (
            RTSHealth {
                current: crate::constants::combat::HUNTER_WASP_HEALTH,
                max: crate::constants::combat::HUNTER_WASP_HEALTH,
                armor: 0.0,
                regeneration_rate: 0.3,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: crate::constants::combat::HUNTER_WASP_DAMAGE,
                attack_range: 12.0,
                attack_speed: 2.0,
                last_attack_time: 0.0,
                target: None,
                attack_type: AttackType::Ranged,
                attack_cooldown: 1.0 / 2.0,
                is_attacking: false,
                auto_attack: true,
            },
            Movement::default(),
            Vision::default(),
            meshes.add(Cuboid::new(0.8, 2.0, 0.8)),
            materials.add(StandardMaterial {
                base_color: if player_id == 1 { Color::srgb(0.2, 0.8, 0.2) } else { Color::srgb(0.6, 0.2, 0.8) },
                ..default()
            }),
        ),
        UnitType::SpearMantis => (
            RTSHealth {
                current: 110.0,
                max: 110.0,
                armor: 1.0,
                regeneration_rate: 0.4,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: 22.0,
                attack_range: 8.0,
                attack_speed: 1.8,
                last_attack_time: 0.0,
                target: None,
                attack_type: AttackType::Melee,
                attack_cooldown: 1.0 / 1.8,
                is_attacking: false,
                auto_attack: true,
            },
            Movement {
                max_speed: 25.0,
                acceleration: 55.0,
                turning_speed: 2.8,
                ..default()
            },
            Vision {
                sight_range: 120.0,
                ..default()
            },
            meshes.add(Cuboid::new(1.2, 2.2, 1.2)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.8, 0.3),
                ..default()
            }),
        ),
        UnitType::ScoutAnt => (
            RTSHealth {
                current: 65.0,
                max: 65.0,
                armor: 0.0,
                regeneration_rate: 0.2,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: 12.0,
                attack_range: 6.0,
                attack_speed: 2.2,
                last_attack_time: 0.0,
                target: None,
                attack_type: AttackType::Melee,
                attack_cooldown: 1.0 / 2.2,
                is_attacking: false,
                auto_attack: true,
            },
            Movement {
                max_speed: 40.0,
                acceleration: 70.0,
                turning_speed: 3.2,
                ..default()
            },
            Vision {
                sight_range: 180.0,
                ..default()
            },
            meshes.add(Cuboid::new(0.9, 1.8, 0.9)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.7, 0.9),
                ..default()
            }),
        ),
        _ => (
            RTSHealth::default(),
            Combat {
                attack_damage: crate::constants::combat::DEFAULT_UNIT_DAMAGE,
                attack_range: crate::constants::combat::DEFAULT_ATTACK_RANGE,
                attack_speed: 1.0,
                last_attack_time: 0.0,
                target: None,
                attack_type: AttackType::Melee,
                attack_cooldown: 1.0,
                is_attacking: false,
                auto_attack: true,
            },
            Movement::default(),
            Vision::default(),
            meshes.add(Cuboid::new(1.0, 2.0, 1.0)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 0.5),
                ..default()
            }),
        ),
    };

    // Make enemy units slightly more active but balanced
    if player_id != 1 {
        combat.auto_attack = true;
        // DO NOT override movement stats - use unit's configured stats
        vision.sight_range = 80.0; // Reasonable vision range
    }

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::from_translation(position),
        RTSUnit {
            unit_id: rand::random(),
            player_id,
            size: 1.0,
            unit_type: Some(unit_type.clone()),
        },
        health,
        combat,
        movement,
        vision,
        Selectable::default(),
        HealthBar::default(),
        CombatStats {
            kills: 0,
            damage_dealt: 0.0,
            damage_taken: 0.0,
            experience: 0.0,
        },
        CollisionRadius::default(),
        EntityState::default(),
    )).id()
}