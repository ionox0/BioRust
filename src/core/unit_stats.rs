//! # Unit Statistics Configuration
//!
//! This module contains all unit statistics and balance parameters.
//! Modify these values to adjust game balance without touching the core logic.

use crate::core::components::*;
use std::collections::HashMap;

// Unit role categories for easier balance management
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnitRole {
    Economic, // Workers, gatherers
    Infantry, // Basic combat units
    Ranged,   // Ranged attack units
    Heavy,    // Tanky units with high HP/armor
    Scout,    // Fast, lightly armored
    Elite,    // High-cost, powerful units
    Siege,    // Anti-building specialists
}

// Base stat templates by role
#[derive(Debug, Clone)]
pub struct BaseStats {
    pub health_multiplier: f32,
    pub damage_multiplier: f32,
    pub speed_multiplier: f32,
    pub armor_base: f32,
    pub attack_type: AttackType,
    pub base_cost_multiplier: f32,
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

/// Worker Ant - Economic foundation unit (50 cost)
/// Role: Resource gathering, building construction
/// Effectiveness: Essential utility unit, reasonable survivability for cost
pub const WORKER_ANT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 75.0,
        max: 75.0,
        armor: 0.0,
        regeneration_rate: 0.1,
    },
    combat: CombatStats {
        attack_damage: 10.0,
        attack_range: 9.0, // collision_radius (6.0) + base_melee_range (3.0)
        attack_speed: 1.0,
        attack_type: AttackType::Melee,
        auto_attack: false, // Workers don't auto-attack
    },
    movement: MovementStats {
        max_speed: 75.0,
        acceleration: 45.0,
        turning_speed: 2.0,
    },
    vision: VisionStats {
        sight_range: 100.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::WORKER_ANT_COLLISION_RADIUS,
};

/// Soldier Ant - Balanced infantry (90 cost)
/// Role: Core army unit, general combat
/// Effectiveness: Solid stats for cost, good all-around fighter
pub const SOLDIER_ANT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 120.0,
        max: 120.0,
        armor: 1.0,
        regeneration_rate: 0.0,
    },
    combat: CombatStats {
        attack_damage: 20.0, // Increased from 4.0 for proper cost effectiveness
        attack_range: 12.0,  // collision_radius (7.0) + base_melee_range (5.0)
        attack_speed: 1.5,   // Improved attack speed
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 80.0,
        acceleration: 60.0,
        turning_speed: 2.5,
    },
    vision: VisionStats {
        sight_range: 120.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::SOLDIER_ANT_COLLISION_RADIUS,
};

/// Hunter Wasp - Ranged support (100 cost)
/// Role: Ranged damage, air support
/// Effectiveness: Good range and mobility for cost, glass cannon
pub const HUNTER_WASP_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 85.0, // Reduced for glass cannon role
        max: 85.0,
        armor: 0.0, // No armor - ranged glass cannon
        regeneration_rate: 0.0,
    },
    combat: CombatStats {
        attack_damage: 22.0, // Increased from 6.0 for proper DPS
        attack_range: 21.5,  // collision_radius (6.5) + base_ranged_range (15.0)
        attack_speed: 1.8,   // Faster attacks for sustained DPS
        attack_type: AttackType::Ranged,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 95.0, // Increased for hit-and-run tactics
        acceleration: 120.0,
        turning_speed: 2.8,
    },
    vision: VisionStats {
        sight_range: 180.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::HUNTER_WASP_COLLISION_RADIUS,
};

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
        attack_type: AttackType::Ranged,
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

/// Battering Beetle - Siege specialist (200 cost)
/// Role: SIEGE - Anti-building, fortress breaker
/// Effectiveness: Specialized role with unique value
pub const BATTERING_BEETLE_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 220.0, // High health to survive approach to buildings
        max: 220.0,
        armor: 5.0, // Heavy armor vs building defenses
        regeneration_rate: 0.1,
    },
    combat: CombatStats {
        attack_damage: 80.0, // Very high damage vs buildings
        attack_range: 13.0,  // collision_radius (10.0) + base_siege_range (3.0)
        attack_speed: 0.8,   // Slower but devastating attacks
        attack_type: AttackType::Siege,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 35.0, // Slow siege unit
        acceleration: 25.0,
        turning_speed: 1.5,
    },
    vision: VisionStats {
        sight_range: 110.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Honey Bee - Basic flying ranged unit
pub const HONEY_BEE_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 80.0,
        max: 80.0,
        armor: 0.0,
        regeneration_rate: 0.2,
    },
    combat: CombatStats {
        attack_damage: 8.0,
        attack_range: 22.0, // collision_radius (10.0) + base_ranged_range (12.0)
        attack_speed: 1.8,
        attack_type: AttackType::Ranged,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 90.0,
        acceleration: 60.0,
        turning_speed: 2.8,
    },
    vision: VisionStats {
        sight_range: 150.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Scorpion - Armored bruiser (160 cost)
/// Role: TANK/DPS hybrid - Heavy melee with sustained damage
/// Effectiveness: Strong stats justify mid-tier pricing
pub const SCORPION_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 200.0,
        max: 200.0,
        armor: 3.5, // High armor for tanky role
        regeneration_rate: 0.4,
    },
    combat: CombatStats {
        attack_damage: 32.0, // High damage for cost
        attack_range: 13.0,  // collision_radius (10.0) + base_melee_range (3.0)
        attack_speed: 1.4,   // Good attack speed
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 65.0, // Moderate speed for balance
        acceleration: 45.0,
        turning_speed: 2.2,
    },
    vision: VisionStats {
        sight_range: 110.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Spider Hunter - Fast predator (130 cost)
/// Role: SKIRMISHER - Fast melee, hit-and-run tactics
/// Effectiveness: Good mobility and damage for mid-tier cost
pub const SPIDER_HUNTER_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 100.0,
        max: 100.0,
        armor: 1.0,             // Light armor for skirmisher role
        regeneration_rate: 0.5, // Good regen for sustained skirmishing
    },
    combat: CombatStats {
        attack_damage: 28.0, // Good damage for cost
        attack_range: 13.0,  // collision_radius (10.0) + base_melee_range (3.0)
        attack_speed: 2.2,   // Fast attacks for skirmishing
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 105.0, // Fast for skirmisher role
        acceleration: 70.0,
        turning_speed: 3.2,
    },
    vision: VisionStats {
        sight_range: 140.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Wolf Spider - Elite predator (250 cost)
/// Role: ELITE DPS - High-tier damage dealer
/// Effectiveness: Strong stats justify high-tier cost
pub const WOLF_SPIDER_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 180.0,
        max: 180.0,
        armor: 2.0,             // Good armor for elite unit
        regeneration_rate: 0.6, // Excellent regen for elite status
    },
    combat: CombatStats {
        attack_damage: 45.0, // High damage for elite DPS
        attack_range: 13.0,  // collision_radius (10.0) + base_melee_range (3.0)
        attack_speed: 1.8,   // Good attack speed
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 85.0, // Good mobility for elite
        acceleration: 55.0,
        turning_speed: 2.8,
    },
    vision: VisionStats {
        sight_range: 150.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Ladybug - Balanced fighter (140 cost)
/// Role: BALANCED - Jack-of-all-trades combat unit
/// Effectiveness: Reliable stats for mid-tier cost
pub const LADYBUG_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 140.0, // Good health for balanced role
        max: 140.0,
        armor: 2.0, // Decent armor for survivability
        regeneration_rate: 0.3,
    },
    combat: CombatStats {
        attack_damage: 25.0, // Balanced damage for cost
        attack_range: 13.0,  // collision_radius (10.0) + base_melee_range (3.0)
        attack_speed: 1.6,   // Good attack speed
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 80.0, // Balanced mobility
        acceleration: 55.0,
        turning_speed: 2.6,
    },
    vision: VisionStats {
        sight_range: 125.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Ladybug Scout - Light scout variant
pub const LADYBUG_SCOUT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 70.0,
        max: 70.0,
        armor: 0.5,
        regeneration_rate: 0.3,
    },
    combat: CombatStats {
        attack_damage: 10.0,
        attack_range: 13.0, // collision_radius (10.0) + base_melee_range (3.0)
        attack_speed: 2.0,
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 110.0,
        acceleration: 60.0,
        turning_speed: 3.5,
    },
    vision: VisionStats {
        sight_range: 170.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Housefly - Fast harasser (60 cost)
/// Role: SKIRMISHER - Fast hit-and-run, harassment
/// Effectiveness: Excellent mobility for very low cost
pub const HOUSEFLY_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 55.0, // Very low health for cheap harasser
        max: 55.0,
        armor: 0.0,             // No armor - pure speed
        regeneration_rate: 0.3, // Good regen for hit-and-run
    },
    combat: CombatStats {
        attack_damage: 12.0, // Light but consistent damage
        attack_range: 20.0, // collision_radius (10.0) + base_ranged_range (10.0)  // Good range for harassment
        attack_speed: 2.8,  // Fast attacks for harassment
        attack_type: AttackType::Ranged,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 140.0, // Very fast for harassment role
        acceleration: 90.0,
        turning_speed: 4.0, // Excellent maneuverability
    },
    vision: VisionStats {
        sight_range: 160.0, // Good vision for harassment
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Termite Worker - Builder and gatherer specialist
pub const TERMITE_WORKER_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 70.0,
        max: 70.0,
        armor: 0.5,
        regeneration_rate: 0.1,
    },
    combat: CombatStats {
        attack_damage: 8.0,
        attack_range: 13.0, // collision_radius (10.0) + base_melee_range (3.0)
        attack_speed: 1.2,
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 70.0,
        acceleration: 40.0,
        turning_speed: 2.0,
    },
    vision: VisionStats {
        sight_range: 100.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Termite Warrior - Elite siege (350 cost)
/// Role: ELITE SIEGE - Ultimate building destroyer
/// Effectiveness: Specialized siege role justifies very high cost
pub const TERMITE_WARRIOR_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 350.0, // Very high health for elite siege
        max: 350.0,
        armor: 6.0, // Heavy armor for approaching buildings
        regeneration_rate: 0.3,
    },
    combat: CombatStats {
        attack_damage: 120.0, // Massive damage vs buildings
        attack_range: 13.0,   // collision_radius (10.0) + base_siege_range (3.0)
        attack_speed: 0.9,    // Slow but devastating
        attack_type: AttackType::Siege,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 45.0, // Slow but unstoppable
        acceleration: 30.0,
        turning_speed: 1.8,
    },
    vision: VisionStats {
        sight_range: 120.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Leg Beetle - Fast melee skirmisher
pub const LEG_BEETLE_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 100.0,
        max: 100.0,
        armor: 0.5,
        regeneration_rate: 0.2,
    },
    combat: CombatStats {
        attack_damage: 12.0,
        attack_range: 13.0, // collision_radius (10.0) + base_melee_range (3.0)
        attack_speed: 2.2,
        attack_type: AttackType::Melee,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 105.0,
        acceleration: 65.0,
        turning_speed: 3.2,
    },
    vision: VisionStats {
        sight_range: 125.0,
        line_of_sight: true,
    },
    collision_radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
};

/// Stinkbug - Area denial and support unit
pub const STINKBUG_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 110.0,
        max: 110.0,
        armor: 1.5,
        regeneration_rate: 0.4,
    },
    combat: CombatStats {
        attack_damage: 10.0,
        attack_range: 20.0, // collision_radius (10.0) + base_ranged_range (10.0)
        attack_speed: 1.5,
        attack_type: AttackType::Ranged,
        auto_attack: false,
    },
    movement: MovementStats {
        max_speed: 60.0,
        acceleration: 35.0,
        turning_speed: 2.0,
    },
    vision: VisionStats {
        sight_range: 140.0,
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
        attack_range: 13.0, // collision_radius (10.0) + base_melee_range (3.0)
        attack_speed: 1.0,
        attack_type: AttackType::Melee,
        auto_attack: false,
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

// Role-based stat templates
fn get_role_base_stats() -> HashMap<UnitRole, BaseStats> {
    let mut base_stats = HashMap::new();

    base_stats.insert(
        UnitRole::Economic,
        BaseStats {
            health_multiplier: 0.8,
            damage_multiplier: 0.5,
            speed_multiplier: 1.0,
            armor_base: 0.0,
            attack_type: AttackType::Melee,
            base_cost_multiplier: 0.5,
        },
    );

    base_stats.insert(
        UnitRole::Infantry,
        BaseStats {
            health_multiplier: 1.0,
            damage_multiplier: 1.0,
            speed_multiplier: 1.0,
            armor_base: 1.0,
            attack_type: AttackType::Melee,
            base_cost_multiplier: 1.0,
        },
    );

    base_stats.insert(
        UnitRole::Ranged,
        BaseStats {
            health_multiplier: 0.7,
            damage_multiplier: 1.1,
            speed_multiplier: 1.2,
            armor_base: 0.0,
            attack_type: AttackType::Ranged,
            base_cost_multiplier: 1.1,
        },
    );

    base_stats.insert(
        UnitRole::Heavy,
        BaseStats {
            health_multiplier: 2.0,
            damage_multiplier: 1.2,
            speed_multiplier: 0.7,
            armor_base: 4.0,
            attack_type: AttackType::Melee,
            base_cost_multiplier: 1.8,
        },
    );

    base_stats.insert(
        UnitRole::Scout,
        BaseStats {
            health_multiplier: 0.6,
            damage_multiplier: 0.8,
            speed_multiplier: 1.6,
            armor_base: 0.0,
            attack_type: AttackType::Melee,
            base_cost_multiplier: 0.8,
        },
    );

    base_stats.insert(
        UnitRole::Elite,
        BaseStats {
            health_multiplier: 1.5,
            damage_multiplier: 2.0,
            speed_multiplier: 1.3,
            armor_base: 2.0,
            attack_type: AttackType::Ranged,
            base_cost_multiplier: 4.5,
        },
    );

    base_stats.insert(
        UnitRole::Siege,
        BaseStats {
            health_multiplier: 1.8,
            damage_multiplier: 3.0,
            speed_multiplier: 0.4,
            armor_base: 5.0,
            attack_type: AttackType::Siege,
            base_cost_multiplier: 2.0,
        },
    );

    base_stats
}

// Unit role mappings
fn get_unit_role(unit_type: &UnitType) -> UnitRole {
    match unit_type {
        UnitType::WorkerAnt | UnitType::TermiteWorker => UnitRole::Economic,
        UnitType::SoldierAnt | UnitType::Ladybug | UnitType::Scorpion => UnitRole::Infantry,
        UnitType::HunterWasp | UnitType::HoneyBee | UnitType::Housefly | UnitType::Stinkbug => {
            UnitRole::Ranged
        }
        UnitType::BeetleKnight => UnitRole::Heavy,
        UnitType::ScoutAnt
        | UnitType::SpiderHunter
        | UnitType::LadybugScout
        | UnitType::LegBeetle => UnitRole::Scout,
        UnitType::DragonFly | UnitType::SpearMantis | UnitType::WolfSpider => UnitRole::Elite,
        UnitType::BatteringBeetle | UnitType::TermiteWarrior => UnitRole::Siege,
        _ => UnitRole::Infantry, // Default
    }
}

// Base values for stat calculation
const BASE_HEALTH: f32 = 100.0;
const BASE_DAMAGE: f32 = 20.0;
const BASE_SPEED: f32 = 80.0;
const BASE_ACCELERATION: f32 = 60.0;
const BASE_TURNING_SPEED: f32 = 2.5;
const BASE_SIGHT_RANGE: f32 = 120.0;
const BASE_MELEE_RANGE: f32 = 3.0;
const BASE_RANGED_RANGE: f32 = 12.0;
const BASE_SIEGE_RANGE: f32 = 3.0;

/// Generate unit statistics based on role and modifiers
fn generate_unit_stats(unit_type: &UnitType) -> UnitStatsConfig {
    let role = get_unit_role(unit_type);
    let role_stats = get_role_base_stats();
    let base = role_stats.get(&role).unwrap();

    // Get collision radius from constants
    let collision_radius = match unit_type {
        UnitType::WorkerAnt => crate::constants::collision::WORKER_ANT_COLLISION_RADIUS,
        UnitType::SoldierAnt => crate::constants::collision::SOLDIER_ANT_COLLISION_RADIUS,
        UnitType::HunterWasp => crate::constants::collision::HUNTER_WASP_COLLISION_RADIUS,
        UnitType::BeetleKnight => crate::constants::collision::BEETLE_KNIGHT_COLLISION_RADIUS,
        UnitType::DragonFly => 3.0, // Special case
        _ => crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS,
    };

    // Calculate attack range based on type and collision radius
    let base_range = match base.attack_type {
        AttackType::Melee => BASE_MELEE_RANGE,
        AttackType::Ranged => BASE_RANGED_RANGE,
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
                AttackType::Ranged => 1.8,
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

/// Get unit statistics configuration for a given unit type
pub fn get_unit_stats(unit_type: &UnitType) -> UnitStatsConfig {
    // Use generated stats for most units, with special cases for unique units
    match unit_type {
        UnitType::DragonFly => DRAGONFLY_STATS, // Keep special elite stats
        _ => generate_unit_stats(unit_type),
    }
}
