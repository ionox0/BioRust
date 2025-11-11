use bevy::prelude::*;
use crate::core::components::*;
use crate::ai::intelligence::{IntelligenceSystem, ThreatLevel};

/// Manages tactical decisions and army coordination
#[derive(Resource, Debug, Clone)]
pub struct TacticalManager {
    pub player_tactics: std::collections::HashMap<u8, PlayerTactics>,
}

#[derive(Debug, Clone)]
pub struct PlayerTactics {
    pub current_stance: TacticalStance,
    pub army_groups: Vec<ArmyGroup>,
    pub last_attack_time: f32,
    pub rally_point: Vec3,
    pub is_attacking: bool,
    pub target_player: Option<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TacticalStance {
    Defensive,      // Stay near base, defend only
    Harass,         // Send small raids to disrupt economy
    Aggressive,     // Full army attack
    Retreat,        // Pull back and regroup
    Expand,         // Secure expansion locations
}

#[derive(Debug, Clone)]
pub struct ArmyGroup {
    pub units: Vec<Entity>,
    pub group_position: Vec3,
    pub role: ArmyRole,
    pub target_location: Option<Vec3>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArmyRole {
    MainArmy,       // Primary fighting force
    Harassers,      // Small raiding party
    Defenders,      // Base defense
    Scouts,         // Exploration
}

/// Component to mark units as part of an army group
#[derive(Component, Debug, Clone)]
pub struct ArmyMember {
    pub group_role: ArmyRole,
    pub follow_group: bool,
}

impl Default for TacticalManager {
    fn default() -> Self {
        let mut player_tactics = std::collections::HashMap::new();

        // Initialize AI player 2
        player_tactics.insert(2, PlayerTactics {
            current_stance: TacticalStance::Defensive,
            army_groups: Vec::new(),
            last_attack_time: 0.0,
            rally_point: Vec3::new(200.0, 0.0, 0.0),
            is_attacking: false,
            target_player: Some(1),
        });

        Self { player_tactics }
    }
}

/// System to manage tactical decisions
pub fn tactical_decision_system(
    mut tactical_manager: ResMut<TacticalManager>,
    intelligence: Res<IntelligenceSystem>,
    units: Query<(&RTSUnit, &Transform, &RTSHealth), With<Combat>>,
    time: Res<Time>,
    mut last_log_time: Local<f32>,
    mut last_stance: Local<std::collections::HashMap<u8, TacticalStance>>,
) {
    let current_time = time.elapsed_secs();

    for (player_id, tactics) in tactical_manager.player_tactics.iter_mut() {
        // Get intelligence about enemy
        let intel = intelligence.get_intel(*player_id);

        // Count our military units
        let our_military_count = units.iter()
            .filter(|(unit, _, _)| unit.player_id == *player_id)
            .count();

        // Store previous stance
        let _prev_stance = tactics.current_stance.clone();

        // Decide tactical stance based on intelligence and army size
        tactics.current_stance = decide_tactical_stance(
            our_military_count,
            intel,
            current_time,
            tactics.last_attack_time,
        );

        // Update army grouping based on stance
        update_army_groups(tactics, &units, *player_id);

        // Only log when stance changes or every 5 seconds
        let stance_changed = last_stance.get(player_id) != Some(&tactics.current_stance);
        let should_log = stance_changed || (current_time - *last_log_time > 5.0);

        if should_log {
            info!("AI Player {} - Stance: {:?}, Military: {}, Attacking: {}",
                  player_id, tactics.current_stance, our_military_count, tactics.is_attacking);
            *last_log_time = current_time;
            last_stance.insert(*player_id, tactics.current_stance.clone());
        }
    }
}

fn decide_tactical_stance(
    our_military_count: usize,
    intel: Option<&crate::ai::intelligence::PlayerIntelligence>,
    current_time: f32,
    last_attack_time: f32,
) -> TacticalStance {
    if let Some(intel) = intel {
        // If under heavy threat, go defensive
        if intel.threat_level == ThreatLevel::Critical || intel.threat_level == ThreatLevel::High {
            return TacticalStance::Defensive;
        }

        let enemy_military = intel.enemy_unit_composition.military_units as usize;

        // If we have significant advantage (1.5x enemy force), attack
        if our_military_count >= 6 && our_military_count >= (enemy_military as f32 * 1.5) as usize {
            // Don't attack too frequently - wait at least 60 seconds between attacks
            if current_time - last_attack_time > 60.0 {
                return TacticalStance::Aggressive;
            }
        }

        // If enemy is economy focused and we have 3+ military, harass
        if intel.estimated_resources.is_eco_focused && our_military_count >= 3 {
            return TacticalStance::Harass;
        }

        // If we're outnumbered significantly, retreat
        if enemy_military > our_military_count * 2 {
            return TacticalStance::Retreat;
        }

        // Default to defensive in early game
        if our_military_count < 5 {
            return TacticalStance::Defensive;
        }
    }

    TacticalStance::Defensive
}

fn update_army_groups(
    tactics: &mut PlayerTactics,
    units: &Query<(&RTSUnit, &Transform, &RTSHealth), With<Combat>>,
    player_id: u8,
) {
    // Clear existing groups
    tactics.army_groups.clear();

    // Collect all military units for this player
    let military_units: Vec<(Entity, Vec3, f32)> = units.iter()
        .filter(|(unit, _, _)| unit.player_id == player_id)
        .map(|(unit, transform, health)| {
            (Entity::from_raw(unit.unit_id as u32), transform.translation, health.current / health.max)
        })
        .collect();

    if military_units.is_empty() {
        return;
    }

    // Based on stance, create army groups
    match tactics.current_stance {
        TacticalStance::Aggressive => {
            // All units in one main army
            let unit_entities: Vec<Entity> = military_units.iter().map(|(e, _, _)| *e).collect();
            let avg_position = military_units.iter()
                .map(|(_, pos, _)| *pos)
                .fold(Vec3::ZERO, |acc, pos| acc + pos) / military_units.len() as f32;

            tactics.army_groups.push(ArmyGroup {
                units: unit_entities,
                group_position: avg_position,
                role: ArmyRole::MainArmy,
                target_location: None, // Will be set by attack system
            });
            tactics.is_attacking = true;
        }
        TacticalStance::Harass => {
            // Split into harassers (2-3 units) and defenders (rest)
            let harass_count = (military_units.len() / 2).min(3).max(2);

            let harassers: Vec<Entity> = military_units.iter().take(harass_count).map(|(e, _, _)| *e).collect();
            let defenders: Vec<Entity> = military_units.iter().skip(harass_count).map(|(e, _, _)| *e).collect();

            if !harassers.is_empty() {
                tactics.army_groups.push(ArmyGroup {
                    units: harassers,
                    group_position: tactics.rally_point,
                    role: ArmyRole::Harassers,
                    target_location: None,
                });
            }

            if !defenders.is_empty() {
                tactics.army_groups.push(ArmyGroup {
                    units: defenders,
                    group_position: tactics.rally_point,
                    role: ArmyRole::Defenders,
                    target_location: Some(tactics.rally_point),
                });
            }
            tactics.is_attacking = false;
        }
        TacticalStance::Defensive | TacticalStance::Retreat => {
            // All units defend at rally point
            let unit_entities: Vec<Entity> = military_units.iter().map(|(e, _, _)| *e).collect();

            tactics.army_groups.push(ArmyGroup {
                units: unit_entities,
                group_position: tactics.rally_point,
                role: ArmyRole::Defenders,
                target_location: Some(tactics.rally_point),
            });
            tactics.is_attacking = false;
        }
        TacticalStance::Expand => {
            tactics.is_attacking = false;
        }
    }
}

/// System to coordinate army groups
pub fn army_coordination_system(
    _commands: Commands,
    tactical_manager: Res<TacticalManager>,
    intelligence: Res<IntelligenceSystem>,
    mut units: Query<(Entity, &mut Movement, &RTSUnit, &Transform), With<Combat>>,
) {
    for (player_id, tactics) in tactical_manager.player_tactics.iter() {
        for group in &tactics.army_groups {
            match group.role {
                ArmyRole::MainArmy => {
                    // Coordinate main army attack
                    if tactics.is_attacking {
                        if let Some(intel) = intelligence.get_intel(*player_id) {
                            if let Some(target_location) = intel.enemy_base_location {
                                // All units in group move together to enemy base
                                for unit_entity in &group.units {
                                    if let Ok((_entity, mut movement, _unit, _transform)) = units.get_mut(*unit_entity) {
                                        movement.target_position = Some(target_location);
                                    }
                                }
                            }
                        }
                    }
                }
                ArmyRole::Harassers => {
                    // Send harassers to attack enemy workers/resources
                    if let Some(intel) = intelligence.get_intel(*player_id) {
                        if let Some(enemy_base) = intel.enemy_base_location {
                            // Harassers target enemy base periphery
                            for (i, unit_entity) in group.units.iter().enumerate() {
                                if let Ok((_entity, mut movement, _unit, _transform)) = units.get_mut(*unit_entity) {
                                    // Spread harassers around enemy base
                                    let angle = (i as f32 / group.units.len() as f32) * std::f32::consts::PI * 2.0;
                                    let offset = Vec3::new(angle.cos() * 40.0, 0.0, angle.sin() * 40.0);
                                    movement.target_position = Some(enemy_base + offset);
                                }
                            }
                        }
                    }
                }
                ArmyRole::Defenders => {
                    // Keep defenders at rally point
                    for unit_entity in &group.units {
                        if let Ok((_entity, mut movement, _unit, transform)) = units.get_mut(*unit_entity) {
                            // Only move if far from rally point
                            if transform.translation.distance(tactics.rally_point) > 30.0 {
                                movement.target_position = Some(tactics.rally_point);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
