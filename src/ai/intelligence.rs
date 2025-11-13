use crate::core::components::*;
use bevy::prelude::*;
use std::collections::HashMap;

/// Tracks intelligence about enemy players - scouting data, army composition, threats
#[derive(Resource, Debug, Clone)]
pub struct IntelligenceSystem {
    pub player_intel: HashMap<u8, PlayerIntelligence>,
}

#[derive(Debug, Clone)]
pub struct PlayerIntelligence {
    pub player_id: u8,
    pub last_scouted: f32,
    pub enemy_base_location: Option<Vec3>,
    pub enemy_unit_composition: UnitComposition,
    pub enemy_buildings: Vec<BuildingType>,
    pub threat_level: ThreatLevel,
    pub estimated_resources: EstimatedResources,
    pub enemy_strategy: EnemyStrategy,
    pub scout_unit: Option<Entity>,
}

#[derive(Debug, Clone, Default)]
pub struct UnitComposition {
    pub workers: u32,
    pub military_units: u32,
    pub soldier_ants: u32,
    pub hunter_wasps: u32,
    pub beetle_knights: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThreatLevel {
    None,     // No military threat
    Low,      // 1-2 military units
    Medium,   // 3-5 military units
    High,     // 6-10 military units
    Critical, // 10+ military units or imminent attack
}

#[derive(Debug, Clone)]
pub struct EstimatedResources {
    pub total_estimated: f32,
    pub is_eco_focused: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyStrategy {
    Unknown,
    EconomyRush,  // Many workers, few military
    MilitaryRush, // Early military pressure
    #[allow(dead_code)]
    FastExpansion, // Quick second base
    Defensive,    // Building defenses
    Aggressive,   // Active harassment
}

impl Default for IntelligenceSystem {
    fn default() -> Self {
        Self {
            player_intel: HashMap::new(),
        }
    }
}

impl IntelligenceSystem {
    pub fn initialize_player(&mut self, ai_player_id: u8, enemy_player_id: u8) {
        self.player_intel.insert(
            ai_player_id,
            PlayerIntelligence {
                player_id: enemy_player_id,
                last_scouted: 0.0,
                enemy_base_location: None,
                enemy_unit_composition: UnitComposition::default(),
                enemy_buildings: Vec::new(),
                threat_level: ThreatLevel::None,
                estimated_resources: EstimatedResources {
                    total_estimated: 0.0,
                    is_eco_focused: false,
                },
                enemy_strategy: EnemyStrategy::Unknown,
                scout_unit: None,
            },
        );
    }

    pub fn get_intel(&self, player_id: u8) -> Option<&PlayerIntelligence> {
        self.player_intel.get(&player_id)
    }

    pub fn get_intel_mut(&mut self, player_id: u8) -> Option<&mut PlayerIntelligence> {
        self.player_intel.get_mut(&player_id)
    }
}

/// System to update intelligence based on scouting
pub fn intelligence_update_system(
    mut intelligence: ResMut<IntelligenceSystem>,
    units: Query<(&RTSUnit, &Transform, &RTSHealth), With<RTSUnit>>,
    buildings: Query<(&Building, &Transform, &RTSUnit), With<Building>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (_ai_player_id, intel) in intelligence.player_intel.iter_mut() {
        let enemy_player_id = intel.player_id;

        // Update unit composition
        let mut composition = UnitComposition::default();
        let mut enemy_positions = Vec::new();

        for (unit, transform, _health) in units.iter() {
            if unit.player_id == enemy_player_id {
                enemy_positions.push(transform.translation);

                if let Some(unit_type) = &unit.unit_type {
                    match unit_type {
                        UnitType::WorkerAnt => composition.workers += 1,
                        UnitType::SoldierAnt => {
                            composition.military_units += 1;
                            composition.soldier_ants += 1;
                        }
                        UnitType::BeetleKnight => {
                            composition.military_units += 1;
                            composition.beetle_knights += 1;
                        }
                        _ => {}
                    }
                }
            }
        }

        intel.enemy_unit_composition = composition.clone();

        // Estimate enemy base location (average of unit positions)
        if !enemy_positions.is_empty() {
            let avg_pos = enemy_positions
                .iter()
                .fold(Vec3::ZERO, |acc, pos| acc + *pos)
                / enemy_positions.len() as f32;
            intel.enemy_base_location = Some(avg_pos);
        }

        // Update building information
        intel.enemy_buildings.clear();
        for (building, _transform, unit) in buildings.iter() {
            if unit.player_id == enemy_player_id && building.is_complete {
                intel.enemy_buildings.push(building.building_type.clone());
            }
        }

        // Calculate threat level
        intel.threat_level = match composition.military_units {
            0 => ThreatLevel::None,
            1..=2 => ThreatLevel::Low,
            3..=5 => ThreatLevel::Medium,
            6..=10 => ThreatLevel::High,
            _ => ThreatLevel::Critical,
        };

        // Estimate enemy strategy
        intel.enemy_strategy =
            determine_enemy_strategy(&composition, &intel.enemy_buildings, current_time);

        // Estimate resources based on units and buildings
        intel.estimated_resources.total_estimated = (composition.workers as f32 * 50.0)
            + (composition.military_units as f32 * 100.0)
            + (intel.enemy_buildings.len() as f32 * 200.0);
        intel.estimated_resources.is_eco_focused =
            composition.workers > composition.military_units * 2;

        intel.last_scouted = current_time;
    }
}

fn determine_enemy_strategy(
    composition: &UnitComposition,
    buildings: &[BuildingType],
    current_time: f32,
) -> EnemyStrategy {
    // Early game military rush detection
    if current_time < 120.0 && composition.military_units >= 3 && composition.workers < 5 {
        return EnemyStrategy::MilitaryRush;
    }

    // Economy rush detection
    if composition.workers >= 8 && composition.military_units <= 2 {
        return EnemyStrategy::EconomyRush;
    }

    // Defensive detection
    if buildings.iter().any(|b| matches!(b, BuildingType::Nursery))
        && composition.military_units < 5
    {
        return EnemyStrategy::Defensive;
    }

    // Aggressive detection - if military > workers
    if composition.military_units > composition.workers && composition.military_units >= 3 {
        return EnemyStrategy::Aggressive;
    }

    EnemyStrategy::Unknown
}

/// Recommends counter-strategy based on enemy strategy
#[allow(dead_code)]
pub fn recommend_counter_strategy(enemy_strategy: &EnemyStrategy) -> &'static str {
    match enemy_strategy {
        EnemyStrategy::EconomyRush => "COUNTER: Early military pressure - rush with 4-5 units",
        EnemyStrategy::MilitaryRush => "COUNTER: Quick defense - build workers and defensive units",
        EnemyStrategy::FastExpansion => "COUNTER: Harass expansion with small raids",
        EnemyStrategy::Defensive => "COUNTER: Expand economy, prepare for large army",
        EnemyStrategy::Aggressive => "COUNTER: Defensive posture, counter-attack when ready",
        EnemyStrategy::Unknown => "STRATEGY: Scout more to gather intelligence",
    }
}
