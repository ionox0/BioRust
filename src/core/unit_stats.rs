//! # Unit Statistics Configuration
//!
//! This module contains all unit statistics and balance parameters.
//! Modify these values to adjust game balance without touching the core logic.

use crate::core::components::*;

// Unit role categories for easier balance management
// Base unit statistics constants
const BASE_HEALTH: f32 = 100.0;
const BASE_DAMAGE: f32 = 20.0;
const BASE_SPEED: f32 = 80.0;
const BASE_ACCELERATION: f32 = 160.0;
const BASE_TURNING_SPEED: f32 = 3.0;
const BASE_SIGHT_RANGE: f32 = 120.0;
const BASE_MELEE_RANGE: f32 = 3.0;
const BASE_SIEGE_RANGE: f32 = 8.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnitRole {
    Economic, // Workers, gatherers
    Tank,     // High health, heavy armor
    DPS,      // High damage dealers
    Scout,    // Fast reconnaissance
    Siege,    // Anti-building specialists
    Elite,    // Special units
}

// Base stat templates by role
#[derive(Debug, Clone)]
pub struct BaseStats {
    pub health_multiplier: f32,
    pub damage_multiplier: f32,
    pub speed_multiplier: f32,
    pub armor_base: f32,
    pub attack_type: AttackType,
}

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




/// Spear Mantis - Elite damage dealer (150 cost)
/// Role: DPS - High damage assassin
/// Effectiveness: Premium damage output justifies higher cost
pub const SPEAR_MANTIS_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 110.0, // Moderate health for DPS role
        max: 110.0,
        armor: 1.0,             // Light armor for mobility
        regeneration_rate: 0.5, // Good regen for sustained combat
    },
    combat: CombatStats {
        attack_damage: 40.0, // High damage for elite DPS role
        attack_range: 13.0,  // collision_radius (10.0) + base_melee_range (3.0)
        attack_speed: 1.6,   // Fast attacks for burst damage
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 85.0, // Good mobility for flanking
        acceleration: 60.0,
        turning_speed: 3.0,
    },
    vision: VisionStats {
        sight_range: 130.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Scout Ant - Fast reconnaissance (80 cost)
/// Role: SCOUT - Map control, vision, harassment
/// Effectiveness: Excellent mobility and vision for cost
pub const SCOUT_ANT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 65.0, // Low health for scout role
        max: 65.0,
        armor: 0.0,             // No armor - speed over protection
        regeneration_rate: 0.3, // Good regen for hit-and-run
    },
    combat: CombatStats {
        attack_damage: 15.0, // Light damage for harassment
        attack_range: 13.0,  // collision_radius (10.0) + base_melee_range (3.0)
        attack_speed: 2.5,   // Fast attacks for quick strikes
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 130.0, // Very fast for scout role
        acceleration: 80.0,
        turning_speed: 3.5, // Excellent maneuverability
    },
    vision: VisionStats {
        sight_range: 200.0, // Excellent vision for scouting
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Beetle Knight - Heavy tank (180 cost)
/// Role: TANK - Frontline armor, damage absorption
/// Effectiveness: Excellent survivability justifies high cost
pub const BEETLE_KNIGHT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 280.0, // Very high health for tank role
        max: 280.0,
        armor: 4.0,             // Heavy armor for damage reduction
        regeneration_rate: 0.2, // Slow but steady regen
    },
    combat: CombatStats {
        attack_damage: 25.0, // Moderate damage for tank role
        attack_range: 11.0,  // collision_radius (8.0) + base_melee_range (3.0)
        attack_speed: 1.2,   // Slower attacks for tank role
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 55.0, // Slower for tank role
        acceleration: 35.0,
        turning_speed: 2.0, // Poor maneuverability
    },
    vision: VisionStats {
        sight_range: 100.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::BEETLE_KNIGHT_COLLISION_RADIUS,
};

/// DragonFly - Ultimate air superiority (450 cost)
/// Role: ELITE - Game-ending unit, air domination
/// Effectiveness: Extreme stats justify extreme cost
pub const DRAGONFLY_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 300.0, // High health for elite unit
        max: 300.0,
        armor: 3.0,             // Moderate armor, relies on speed
        regeneration_rate: 0.8, // Good regen for elite status
    },
    combat: CombatStats {
        attack_damage: 65.0, // Very high damage for elite
        attack_range: 22.0,  // Excellent range
        attack_speed: 1.5,   // Good attack speed
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 400.0,    // EXTREMELY fast - 5x normal speed for air superiority
        acceleration: 250.0, // Very fast acceleration to match speed
        turning_speed: 5.0,  // Superior maneuverability for air unit
    },
    vision: VisionStats {
        sight_range: 250.0, // Excellent vision for elite
        line_of_sight: true,
    },
    collision_radius: 3.0,
};

/// Generate unit statistics dynamically based on unit type and role
fn generate_unit_stats(unit_type: &UnitType) -> UnitStatsConfig {
    let role = get_unit_role(unit_type);
    let role_stats = get_role_base_stats();
    let base = role_stats.get(&role).unwrap();

    // Get collision radius from constants
    let collision_radius = match unit_type {
        UnitType::WorkerAnt => crate::constants::collision::WORKER_ANT_COLLISION_RADIUS,
        UnitType::SoldierAnt => crate::constants::collision::SOLDIER_ANT_COLLISION_RADIUS,
        UnitType::BeetleKnight => crate::constants::collision::BEETLE_KNIGHT_COLLISION_RADIUS,
        UnitType::DragonFly => 3.0, // Special case
        _ => crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
    };

    // Calculate attack range based on type and collision radius
    let base_range = match base.attack_type {
        AttackType::Melee => BASE_MELEE_RANGE,
        AttackType::Siege => BASE_SIEGE_RANGE,
    };

    let attack_range = collision_radius + base_range;

    // Apply role-based multipliers
    let health = BASE_HEALTH * base.health_multiplier;
    let damage = BASE_DAMAGE * base.damage_multiplier;
    let speed = BASE_SPEED * base.speed_multiplier;

    UnitStatsConfig {
        health: HealthStats {
            current: health,
            max: health,
            armor: base.armor_base,
            regeneration_rate: if role == UnitRole::Economic { 0.1 } else { 0.3 },
        },
        combat: CombatStats {
            attack_damage: damage,
            attack_range,
            attack_speed: match base.attack_type {
                AttackType::Melee => 1.5,
                AttackType::Siege => 0.9,
            },
            attack_type: base.attack_type.clone(),
            auto_attack: role != UnitRole::Economic,
        },
        movement: MovementStats {
            max_speed: speed,
            acceleration: BASE_ACCELERATION * base.speed_multiplier,
            turning_speed: BASE_TURNING_SPEED,
        },
        vision: VisionStats {
            sight_range: BASE_SIGHT_RANGE * if role == UnitRole::Scout { 1.5 } else { 1.0 },
            line_of_sight: true,
        },
        collision_radius,
    }
}

/// Get the role of a unit type for stat generation
fn get_unit_role(unit_type: &UnitType) -> UnitRole {
    match unit_type {
        UnitType::WorkerAnt | UnitType::TermiteWorker => UnitRole::Economic,
        UnitType::SoldierAnt | UnitType::BeetleKnight | UnitType::SpiderHunter |
        UnitType::WolfSpider | UnitType::Scorpion | UnitType::TermiteWarrior |
        UnitType::LegBeetle => UnitRole::Tank,
        UnitType::SpearMantis | UnitType::Ladybug |
        UnitType::Housefly | UnitType::HoneyBee => UnitRole::DPS,
        UnitType::ScoutAnt | UnitType::LadybugScout => UnitRole::Scout,
        UnitType::BatteringBeetle | UnitType::Stinkbug => UnitRole::Siege,
        UnitType::DragonFly | UnitType::AcidSpitter | UnitType::EliteSpider => UnitRole::Elite,
        UnitType::DefenderBug => UnitRole::Tank,
    }
}

/// Get base statistics for each unit role
fn get_role_base_stats() -> std::collections::HashMap<UnitRole, BaseStats> {
    let mut stats = std::collections::HashMap::new();
    
    stats.insert(UnitRole::Economic, BaseStats {
        health_multiplier: 0.8,
        damage_multiplier: 0.4,
        speed_multiplier: 1.0,
        armor_base: 0.0,
        attack_type: AttackType::Melee,
    });
    
    stats.insert(UnitRole::Tank, BaseStats {
        health_multiplier: 1.8,
        damage_multiplier: 0.8,
        speed_multiplier: 0.7,
        armor_base: 3.0,
        attack_type: AttackType::Melee,
    });
    
    stats.insert(UnitRole::DPS, BaseStats {
        health_multiplier: 0.9,
        damage_multiplier: 1.4,
        speed_multiplier: 1.1,
        armor_base: 0.5,
        attack_type: AttackType::Melee,
    });
    
    stats.insert(UnitRole::Scout, BaseStats {
        health_multiplier: 0.7,
        damage_multiplier: 0.6,
        speed_multiplier: 1.6,
        armor_base: 0.0,
        attack_type: AttackType::Melee,
    });
    
    stats.insert(UnitRole::Siege, BaseStats {
        health_multiplier: 1.5,
        damage_multiplier: 2.0,
        speed_multiplier: 0.6,
        armor_base: 2.0,
        attack_type: AttackType::Siege,
    });
    
    stats.insert(UnitRole::Elite, BaseStats {
        health_multiplier: 2.5,
        damage_multiplier: 2.2,
        speed_multiplier: 2.0,
        armor_base: 3.0,
        attack_type: AttackType::Melee,
    });
    
    stats
}

/// Get unit statistics configuration for a given unit type
pub fn get_unit_stats(unit_type: &UnitType) -> UnitStatsConfig {
    // Use generated stats for most units, with special cases for unique units
    match unit_type {
        UnitType::DragonFly => DRAGONFLY_STATS, // Keep special elite stats
        _ => generate_unit_stats(unit_type),
    }
}
