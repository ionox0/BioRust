use bevy::prelude::*;
use crate::core::components::*;
use std::collections::HashMap;

pub struct CombatPlugin;

// Combat constants
mod combat_constants {
    pub const BASE_XP_FOR_KILL: f32 = 10.0;
    pub const REGEN_DELAY: f32 = 5.0;
    pub const ATTACK_RANGE_MARGIN: f32 = 0.8;
    pub const TARGET_POSITION_RATIO: f32 = 0.7;
    pub const GRID_CELL_SIZE: f32 = 100.0;
}

// Spatial grid for efficient target acquisition
struct TargetGrid {
    cells: HashMap<(i32, i32), Vec<(Entity, Vec3, u8)>>,
}

impl TargetGrid {
    fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    fn insert(&mut self, entity: Entity, position: Vec3, player_id: u8) {
        let cell = Self::get_cell(position);
        self.cells.entry(cell).or_default().push((entity, position, player_id));
    }

    fn get_cell(position: Vec3) -> (i32, i32) {
        (
            (position.x / combat_constants::GRID_CELL_SIZE).floor() as i32,
            (position.z / combat_constants::GRID_CELL_SIZE).floor() as i32,
        )
    }

    fn query_nearby(&self, position: Vec3, range: f32) -> Vec<(Entity, Vec3, u8)> {
        let cell = Self::get_cell(position);
        let cell_radius = (range / combat_constants::GRID_CELL_SIZE).ceil() as i32;
        let mut results = Vec::new();

        for dx in -cell_radius..=cell_radius {
            for dz in -cell_radius..=cell_radius {
                let neighbor_cell = (cell.0 + dx, cell.1 + dz);
                if let Some(entities) = self.cells.get(&neighbor_cell) {
                    for &(entity, pos, player_id) in entities {
                        if position.distance(pos) <= range {
                            results.push((entity, pos, player_id));
                        }
                    }
                }
            }
        }
        results
    }
}

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                target_validation_system,  // Clear invalid targets first
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

// System to validate and clear invalid combat targets (e.g., buildings, dead units)
pub fn target_validation_system(
    mut combat_query: Query<(&mut Combat, &RTSUnit)>,
    target_query: Query<&RTSUnit, (With<RTSHealth>, Without<Building>, Without<DeathEvent>)>,
) {
    for (mut combat, unit) in combat_query.iter_mut() {
        if let Some(target_entity) = combat.target {
            // Check if target still exists and is a valid target (not a building, not dead)
            if target_query.get(target_entity).is_err() {
                // Target is invalid (dead, building, or despawned) - clear it
                debug!("🛑 Clearing invalid combat target for unit (Player {})", unit.player_id);
                combat.target = None;
                combat.last_attack_time = 0.0; // Reset attack timing
            }
        }
    }
}

// System to find and acquire targets for units with combat capability
pub fn target_acquisition_system(
    mut combat_query: Query<(Entity, &mut Combat, &Transform, &RTSUnit, &Movement), With<Vision>>,
    potential_targets: Query<(Entity, &Transform, &RTSUnit), (With<RTSHealth>, Without<DeathEvent>, Without<Building>)>, // Exclude buildings
    vision_query: Query<&Vision>,
    _time: Res<Time>,
) {
    // Build spatial grid for potential targets (performance optimization)
    let mut target_grid = TargetGrid::new();
    for (entity, transform, unit) in potential_targets.iter() {
        target_grid.insert(entity, transform.translation, unit.player_id);
    }

    for (entity, mut combat, transform, unit, _movement) in combat_query.iter_mut() {
        // Allow all units to auto-attack when enemies are nearby regardless of movement commands
        // Player units should defend themselves when threatened


        // Skip if already has a valid target and is not auto-attacking
        if combat.target.is_some() && !combat.auto_attack {
            continue;
        }

        // Get vision range
        let vision = vision_query.get(entity).ok();
        let effective_vision_range = vision.map(|v| v.sight_range).unwrap_or(80.0);

        // Query nearby enemies using spatial grid
        let nearby_entities = target_grid.query_nearby(transform.translation, effective_vision_range);

        let mut closest_enemy: Option<(Entity, f32)> = None;

        for (target_entity, target_pos, target_player_id) in nearby_entities {
            // Skip same player units (no friendly fire)
            if target_player_id == unit.player_id {
                continue;
            }

            let distance = transform.translation.distance(target_pos);
            let is_enemy_seeking_player = unit.player_id != 1 && target_player_id == 1;

            // Check if target is within vision range OR very close (defensive reaction)
            let defensive_range = combat.attack_range * 1.5; // React to enemies 1.5x attack range
            let in_vision = distance <= effective_vision_range;
            let in_defensive_range = distance <= defensive_range;

            if in_vision || in_defensive_range {
                // Prioritize targets within attack range first
                if distance <= combat.attack_range * 1.2 {
                    // Target within or very close to attack range - highest priority
                    if closest_enemy.map_or(true, |(_, d)| distance < d) {
                        closest_enemy = Some((target_entity, distance));
                    }
                } else if closest_enemy.is_none() || distance < closest_enemy.map(|(_, d)| d).unwrap_or(f32::MAX) {
                    // No closer target found yet, or this is closer
                    closest_enemy = Some((target_entity, distance));
                }
            } else if is_enemy_seeking_player || closest_enemy.is_none() {
                // Enemies will actively seek player units at longer range
                closest_enemy = Some((target_entity, distance));
            }
        }

        // Set or clear target
        combat.target = if let Some((target_entity, _)) = closest_enemy {
            Some(target_entity)
        } else if combat.auto_attack {
            None
        } else {
            combat.target
        };
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
    mut unit_query: Query<(&mut Movement, &Combat, &Transform, &RTSUnit, &CollisionRadius)>,
    target_query: Query<(&Transform, &CollisionRadius), With<RTSHealth>>,
) {
    use combat_constants::*;

    for (mut movement, combat, unit_transform, unit, unit_collision) in unit_query.iter_mut() {
        if let Some(target_entity) = combat.target {
            if let Ok((target_transform, target_collision)) = target_query.get(target_entity) {
                let center_distance = unit_transform.translation.distance(target_transform.translation);
                let edge_distance = center_distance - unit_collision.radius - target_collision.radius;

                // Move towards target if out of effective attack range
                let effective_range = combat.attack_range * ATTACK_RANGE_MARGIN; // Use constant (0.8)

                if edge_distance > effective_range {
                    let direction = (target_transform.translation - unit_transform.translation).normalize();
                    let target_position = target_transform.translation - direction * (combat.attack_range * TARGET_POSITION_RATIO);

                    // For player units, only override movement if no explicit move command or if enemy is very close
                    let should_override_movement = if unit.player_id == 1 {
                        movement.target_position.is_none() || edge_distance < combat.attack_range * 2.0
                    } else {
                        true // AI units always pursue targets
                    };

                    if should_override_movement {
                        movement.target_position = Some(target_position);
                        debug!("🗡️ Unit {:?} (Player {}) pursuing target: edge_dist={:.1}, range={:.1}", 
                               unit_transform.translation, unit.player_id, edge_distance, combat.attack_range);
                    }
                } else if unit.player_id != 1 {
                    // Stop AI units when in range
                    movement.target_position = None;
                    movement.current_velocity = Vec3::ZERO;
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
    use combat_constants::BASE_XP_FOR_KILL;

    let current_time = time.elapsed_secs();

    for damage_event in damage_events.read() {
        if let Ok((target_entity, mut health)) = health_query.get_mut(damage_event.target) {
            let actual_damage = calculate_damage(damage_event.damage, health.armor, &damage_event.damage_type);

            health.current = (health.current - actual_damage).max(0.0);
            health.last_damage_time = current_time;

            // Update combat stats
            if let Ok(mut stats) = combat_stats_query.get_mut(damage_event.attacker) {
                stats.damage_dealt += actual_damage;
            }
            if let Ok(mut stats) = combat_stats_query.get_mut(damage_event.target) {
                stats.damage_taken += actual_damage;
            }

            info!("Entity {:?} takes {} damage (current health: {})",
                  target_entity, actual_damage, health.current);

            if health.current <= 0.0 {
                commands.entity(target_entity).insert(Dying);

                if let Ok(mut stats) = combat_stats_query.get_mut(damage_event.attacker) {
                    stats.kills += 1;
                    stats.experience += BASE_XP_FOR_KILL;
                }

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
    use combat_constants::REGEN_DELAY;

    let current_time = time.elapsed_secs();

    for mut health in health_query.iter_mut() {
        if health.current < health.max &&
           current_time - health.last_damage_time >= REGEN_DELAY &&
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

// Note: Unit creation is now handled by entity_factory.rs which provides
// a more maintainable approach using unit configuration data