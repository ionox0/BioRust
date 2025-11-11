//! # Unit Statistics Configuration
//! 
//! This module contains all unit statistics and balance parameters.
//! Modify these values to adjust game balance without touching the core logic.

use crate::core::components::*;

/// Health statistics for a unit type
#[derive(Debug, Clone, Copy)]
pub struct HealthStats {
    pub current: f32,
    pub max: f32,
    pub armor: f32,
    pub regeneration_rate: f32,
}

/// Movement statistics for a unit type
#[derive(Debug, Clone, Copy)]
pub struct MovementStats {
    pub max_speed: f32,
    pub acceleration: f32,
    pub turning_speed: f32,
}

/// Combat statistics for a unit type
#[derive(Debug, Clone)]
pub struct CombatStats {
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
    pub attack_type: AttackType,
    pub auto_attack: bool,
}

/// Vision statistics for a unit type
#[derive(Debug, Clone, Copy)]
pub struct VisionStats {
    pub sight_range: f32,
    pub line_of_sight: bool,
}

/// Complete unit statistics configuration
#[derive(Debug, Clone)]
pub struct UnitStatsConfig {
    pub health: HealthStats,
    pub combat: CombatStats,
    pub movement: MovementStats,
    pub vision: VisionStats,
    pub collision_radius: f32,
}

/// Worker Ant (Villager equivalent) - Basic resource gathering unit
pub const WORKER_ANT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 75.0,
        max: 75.0,
        armor: 0.0,
        regeneration_rate: 0.1,
    },
    combat: CombatStats {
        attack_damage: 10.0,
        attack_range: 4.0,
        attack_speed: 1.0,
        attack_type: AttackType::Melee,
        auto_attack: false, // Workers don't auto-attack
    },
    movement: MovementStats {
        max_speed: 75.0,  // 3x faster - workers need to move efficiently
        acceleration: 45.0,
        turning_speed: 2.0,
    },
    vision: VisionStats {
        sight_range: 100.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::WORKER_ANT_COLLISION_RADIUS,
};

/// Soldier Ant (Militia equivalent) - Basic melee infantry
pub const SOLDIER_ANT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 120.0,
        max: 120.0,
        armor: 1.0,
        regeneration_rate: 0.0,
    },
    combat: CombatStats {
        attack_damage: 4.0,
        attack_range: 5.0,
        attack_speed: 2.0,
        attack_type: AttackType::Melee,
        auto_attack: true,
    },
    movement: MovementStats {
        max_speed: 80.0,  // 2.6x faster - strong melee units
        acceleration: 60.0,
        turning_speed: 2.5,
    },
    vision: VisionStats {
        sight_range: 120.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::SOLDIER_ANT_COLLISION_RADIUS,
};

/// Hunter Wasp (Archer equivalent) - Ranged attack unit
pub const HUNTER_WASP_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 90.0,
        max: 90.0,
        armor: 0.0,
        regeneration_rate: 0.0,
    },
    combat: CombatStats {
        attack_damage: 6.0,
        attack_range: 15.0,
        attack_speed: 1.5,
        attack_type: AttackType::Ranged,
        auto_attack: true,
    },
    movement: MovementStats {
        max_speed: 85.0,  // 2.8x faster - flying ranged units should be fast
        acceleration: 120.0,
        turning_speed: 2.5,
    },
    vision: VisionStats {
        sight_range: 200.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::HUNTER_WASP_COLLISION_RADIUS,
};

/// Spear Mantis - Elite melee unit with high damage and regeneration
pub const SPEAR_MANTIS_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 110.0,
        max: 110.0,
        armor: 1.0,
        regeneration_rate: 0.4,
    },
    combat: CombatStats {
        attack_damage: 22.0,
        attack_range: 8.0,
        attack_speed: 1.8,
        attack_type: AttackType::Melee,
        auto_attack: true,
    },
    movement: MovementStats {
        max_speed: 70.0,  // 2.8x faster - elite mantis units
        acceleration: 110.0,
        turning_speed: 2.8,
    },
    vision: VisionStats {
        sight_range: 120.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Scout Ant - Fast reconnaissance unit with excellent vision
pub const SCOUT_ANT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 65.0,
        max: 65.0,
        armor: 0.0,
        regeneration_rate: 0.2,
    },
    combat: CombatStats {
        attack_damage: 12.0,
        attack_range: 6.0,
        attack_speed: 2.2,
        attack_type: AttackType::Melee,
        auto_attack: true,
    },
    movement: MovementStats {
        max_speed: 120.0,  // 3x faster - scouts should be very fast
        acceleration: 140.0,
        turning_speed: 3.2,
    },
    vision: VisionStats {
        sight_range: 180.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Beetle Knight - Heavy armored unit with high health
pub const BEETLE_KNIGHT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 200.0,
        max: 200.0,
        armor: 3.0,
        regeneration_rate: 0.0,
    },
    combat: CombatStats {
        attack_damage: 12.0,
        attack_range: 8.0,
        attack_speed: 1.8,
        attack_type: AttackType::Melee,
        auto_attack: true,
    },
    movement: MovementStats {
        max_speed: 90.0,  // 2.5x faster - heavy units but still mobile
        acceleration: 140.0,
        turning_speed: 3.0,
    },
    vision: VisionStats {
        sight_range: 100.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::BEETLE_KNIGHT_COLLISION_RADIUS,
};

/// DragonFly - Elite ranged unit with exceptional stats and EXTREME SPEED
pub const DRAGONFLY_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 250.0,
        max: 250.0,
        armor: 5.0,
        regeneration_rate: 1.0,
    },
    combat: CombatStats {
        attack_damage: 50.0,
        attack_range: 20.0,
        attack_speed: 1.2,
        attack_type: AttackType::Ranged,
        auto_attack: true,
    },
    movement: MovementStats {
        max_speed: 1200.0,  // Balanced speed for smooth collision handling (1200 / 25 / 2 = 24 final speed)
        acceleration: 1200.0,  // Matched acceleration for smooth movement
        turning_speed: 3.0,  // Fast turning without jitter
    },
    vision: VisionStats {
        sight_range: 250.0,
        line_of_sight: true,
    },
    collision_radius: 2.0,
};

/// Battering Beetle - Siege unit specialized for destroying buildings
pub const BATTERING_BEETLE_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 180.0,
        max: 180.0,
        armor: 4.0,
        regeneration_rate: 0.2,
    },
    combat: CombatStats {
        attack_damage: 30.0,
        attack_range: 6.0,
        attack_speed: 1.0,
        attack_type: AttackType::Siege,
        auto_attack: true,
    },
    movement: MovementStats {
        max_speed: 15.0,
        acceleration: 25.0,
        turning_speed: 1.2,
    },
    vision: VisionStats {
        sight_range: 100.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Default stats for unimplemented or fallback unit types
pub const DEFAULT_UNIT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 100.0,
        max: 100.0,
        armor: 0.0,
        regeneration_rate: 0.0,
    },
    combat: CombatStats {
        attack_damage: 20.0,
        attack_range: 8.0,
        attack_speed: 1.0,
        attack_type: AttackType::Melee,
        auto_attack: true,
    },
    movement: MovementStats {
        max_speed: 25.0,
        acceleration: 50.0,
        turning_speed: 2.0,
    },
    vision: VisionStats {
        sight_range: 100.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Get unit statistics configuration for a given unit type
pub fn get_unit_stats(unit_type: &UnitType) -> UnitStatsConfig {
    match unit_type {
        UnitType::WorkerAnt => WORKER_ANT_STATS,
        UnitType::SoldierAnt => SOLDIER_ANT_STATS,
        UnitType::HunterWasp => HUNTER_WASP_STATS,
        UnitType::SpearMantis => SPEAR_MANTIS_STATS,
        UnitType::ScoutAnt => SCOUT_ANT_STATS,
        UnitType::BeetleKnight => BEETLE_KNIGHT_STATS,
        UnitType::DragonFly => DRAGONFLY_STATS,
        UnitType::BatteringBeetle => BATTERING_BEETLE_STATS,
        // Add new unit types here
        _ => DEFAULT_UNIT_STATS,
    }
}