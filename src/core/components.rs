use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// Removed unused old game components - using RTS components instead

#[allow(dead_code)]
#[derive(Component, Debug, Clone)]
pub struct Animator {
    pub current_animation: String,
    pub timer: Timer,
    pub frame: usize,
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Debug, Clone)]
pub struct RTSCamera {
    pub move_speed: f32,
}

#[derive(Component)]
pub struct UI;

#[derive(Component)]
pub struct GameEntity;

// Team color component for tinting GLB models
#[derive(Component, Debug, Clone)]
pub struct TeamColor {
    pub player_id: u8,
    pub tint_color: Color,
}

impl TeamColor {
    pub fn new(player_id: u8) -> Self {
        use crate::core::constants::team_colors::*;
        let tint_color = match player_id {
            1 => PLAYER_1_TINT,
            2 => PLAYER_2_TINT,
            3 => PLAYER_3_TINT,
            4 => PLAYER_4_TINT,
            5 => PLAYER_5_TINT,
            6 => PLAYER_6_TINT,
            7 => PLAYER_7_TINT,
            8 => PLAYER_8_TINT,
            _ => UNKNOWN_PLAYER_TINT,
        };
        Self {
            player_id,
            tint_color,
        }
    }

    /// Get the primitive color for a player (used for fallback geometric shapes)
    pub fn get_primitive_color(player_id: u8) -> Color {
        use crate::core::constants::team_colors::*;
        match player_id {
            1 => PLAYER_1_PRIMITIVE,
            2 => PLAYER_2_PRIMITIVE,
            3 => PLAYER_3_PRIMITIVE,
            4 => PLAYER_4_PRIMITIVE,
            5 => PLAYER_5_PRIMITIVE,
            6 => PLAYER_6_PRIMITIVE,
            7 => PLAYER_7_PRIMITIVE,
            8 => PLAYER_8_PRIMITIVE,
            _ => UNKNOWN_PLAYER_PRIMITIVE,
        }
    }
}

/// Component to track how many frames we've tried to apply team colors
#[derive(Component, Debug)]
pub struct TeamColorRetryCount {
    pub attempts: u8,
}

impl Default for TeamColorRetryCount {
    fn default() -> Self {
        Self { attempts: 0 }
    }
}

// RTS-specific components

#[derive(Component, Debug, Clone)]
pub struct RTSUnit {
    pub unit_id: u32,
    pub player_id: u8,
    #[allow(dead_code)]
    pub size: f32,
    pub unit_type: Option<UnitType>, // None for buildings
}

#[derive(Component, Debug, Clone)]
pub struct Position {
    pub translation: Vec3,
    #[allow(dead_code)]
    pub rotation: Quat,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::from_rotation_y(180.0f32.to_radians()),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Movement {
    pub max_speed: f32,
    pub acceleration: f32,
    pub turning_speed: f32,
    pub current_velocity: Vec3,
    pub target_position: Option<Vec3>,
    #[allow(dead_code)]
    pub path: Vec<Vec3>,
    #[allow(dead_code)]
    pub path_index: usize,
}

#[derive(Component, Debug, Clone)]
pub struct StuckDetection {
    pub last_position: Vec3,
    pub stuck_timer: f32,
    pub position_history: Vec<Vec3>,
    pub last_movement_time: f32,
    pub unstuck_attempts: u32,
    pub last_unstuck_time: f32,
}

/// Component to track spatial grid position for incremental updates
#[derive(Component, Debug, Clone)]
pub struct SpatialGridPosition {
    pub last_grid_coord: Option<crate::core::spatial_grid::GridCoord>,
    pub dirty: bool,
}

impl Default for StuckDetection {
    fn default() -> Self {
        Self {
            last_position: Vec3::ZERO,
            stuck_timer: 0.0,
            position_history: Vec::new(),
            last_movement_time: 0.0,
            unstuck_attempts: 0,
            last_unstuck_time: 0.0,
        }
    }
}

impl Default for SpatialGridPosition {
    fn default() -> Self {
        Self {
            last_grid_coord: None,
            dirty: true, // Start dirty to ensure initial grid insertion
        }
    }
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            max_speed: 200.0,    // 2x speed increase for default
            acceleration: 400.0, // 2x acceleration increase for default
            turning_speed: 4.0,  // Slightly faster turning
            current_velocity: Vec3::ZERO,
            target_position: None,
            path: Vec::new(),
            path_index: 0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct RTSHealth {
    pub current: f32,
    pub max: f32,
    pub armor: f32,
    pub regeneration_rate: f32,
    pub last_damage_time: f32,
}

impl Default for RTSHealth {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            armor: 0.0,
            regeneration_rate: 0.5,
            last_damage_time: 0.0,
        }
    }
}

// Alias for compatibility with existing code
pub type Health = RTSHealth;

#[derive(Component, Debug, Clone)]
pub struct Combat {
    pub attack_damage: f32,
    pub attack_range: f32,
    #[allow(dead_code)]
    pub attack_speed: f32,
    pub last_attack_time: f32,
    pub target: Option<Entity>,
    pub attack_type: AttackType,
    pub attack_cooldown: f32,
    pub is_attacking: bool,
    pub auto_attack: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackType {
    Melee,
    Siege,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct CombatState {
    pub state: CombatStateType,
    pub target_entity: Option<Entity>,
    pub target_position: Option<Vec3>,
    pub last_state_change: f32,
    pub engagement_start_time: f32,
    pub last_attack_attempt: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CombatStateType {
    /// Unit is not in combat, following normal movement/orders
    Idle,
    /// Unit is moving toward a combat engagement (initial movement to fight)
    MovingToCombat,
    /// Unit is moving toward an attack target but not yet in range
    MovingToAttack,
    /// Unit is actively engaged in combat, within attack range
    InCombat,
    /// Unit is in combat but temporarily moving (chasing fleeing enemy, repositioning)
    CombatMoving,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            state: CombatStateType::Idle,
            target_entity: None,
            target_position: None,
            last_state_change: 0.0,
            engagement_start_time: 0.0,
            last_attack_attempt: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct ResourceGatherer {
    pub gather_rate: f32,
    pub capacity: f32,
    pub carried_amount: f32,
    pub resource_type: Option<ResourceType>,
    pub target_resource: Option<Entity>,
    pub drop_off_building: Option<Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Nectar,
    Chitin,
    Minerals,
    Pheromones,
}

impl ResourceType {
    /// Get the current value of this resource from PlayerResources
    pub fn get_from(&self, resources: &crate::core::resources::PlayerResources) -> f32 {
        match self {
            Self::Nectar => resources.nectar,
            Self::Chitin => resources.chitin,
            Self::Minerals => resources.minerals,
            Self::Pheromones => resources.pheromones,
        }
    }

    /// Add an amount to this resource in PlayerResources
    pub fn add_to(&self, resources: &mut crate::core::resources::PlayerResources, amount: f32) {
        match self {
            Self::Nectar => resources.nectar += amount,
            Self::Chitin => resources.chitin += amount,
            Self::Minerals => resources.minerals += amount,
            Self::Pheromones => resources.pheromones += amount,
        }
    }

    /// Subtract an amount from this resource in PlayerResources
    pub fn subtract_from(
        &self,
        resources: &mut crate::core::resources::PlayerResources,
        amount: f32,
    ) {
        match self {
            Self::Nectar => resources.nectar -= amount,
            Self::Chitin => resources.chitin -= amount,
            Self::Minerals => resources.minerals -= amount,
            Self::Pheromones => resources.pheromones -= amount,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct ResourceSource {
    pub resource_type: ResourceType,
    pub amount: f32,
    #[allow(dead_code)]
    pub max_gatherers: u32,
    #[allow(dead_code)]
    pub current_gatherers: u32,
}

#[derive(Component, Debug, Clone)]
pub struct Building {
    pub building_type: BuildingType,
    pub construction_progress: f32,
    pub max_construction: f32,
    pub is_complete: bool,
    pub rally_point: Option<Vec3>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BuildingType {
    Queen,
    Nursery,
    WarriorChamber,
    HunterChamber,
    Stable,
    FungalGarden,
    WoodProcessor,
    MineralProcessor,
    StorageChamber,
    EvolutionChamber,
    TradingPost,
    ChitinWall,
    GuardTower,
}

#[derive(Component, Debug, Clone)]
pub struct ProductionQueue {
    pub queue: Vec<UnitType>,
    pub current_progress: f32,
    pub production_time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UnitType {
    WorkerAnt,
    SoldierAnt,
    BeetleKnight,
    SpearMantis,
    ScoutAnt,
    BatteringBeetle,
    AcidSpitter,

    // Additional unit types for new models
    DragonFly,   // Flying reconnaissance unit
    DefenderBug, // Defensive unit
    EliteSpider, // Elite predator unit

    // Units for previously unused models
    HoneyBee,     // Basic flying unit (bee-v1.glb)
    Scorpion,     // Heavy melee unit with armor
    SpiderHunter, // Light predator unit (spider_small.glb)
    WolfSpider,   // Heavy predator unit

    // Units for newly added models
    Housefly,       // Fast flying harassment unit
    TermiteWorker,  // Builder/gatherer specialist
    TermiteWarrior, // Heavy siege unit (giant_termite.glb)
    LegBeetle,      // Fast melee skirmisher
    Stinkbug,       // Area denial/support unit

    // Expanded unit categories for multi-team system

    // Beetles family
    StagBeetle,        // Heavy melee beetle
    DungBeetle,        // Worker/siege beetle
    RhinoBeetle,       // Armored assault beetle
    StinkBeetle,       // Area denial beetle (variant)
    JewelBug,          // Fast support beetle

    // Mantids family
    CommonMantis,      // Standard predator mantis
    OrchidMantis,      // Stealth/ambush mantis

    // Fourmi (Ants) family variants
    RedAnt,            // Fire/damage ant
    BlackAnt,          // Standard worker ant
    FireAnt,           // High damage/poison ant
    SoldierFourmi,     // Military ant variant
    WorkerFourmi,      // Economic ant variant

    // Cephalopoda family (Isopods/Crustaceans)
    Pillbug,           // Defensive rolling unit
    Silverfish,        // Fast sneaky unit
    Woodlouse,         // Armored defensive unit
    SandFleas,         // Jumping swarm unit

    // Small creatures family
    Aphids,            // Tiny swarm units
    Mites,             // Microscopic fast units
    Ticks,             // Parasitic units
    Fleas,             // Small jumping units
    Lice,              // Tiny fast units

    // Butterflies family
    Moths,             // Night flying units
    Caterpillars,      // Ground larvae units
    PeacockMoth,       // Large beautiful flyer

    // Spiders family
    WidowSpider,       // Venomous predator
    WolfSpiderVariant, // Pack hunter variant
    Tarantula,         // Large ground predator  
    DaddyLongLegs,     // Light fast spider

    // Flies family
    HouseflyVariant,   // Basic fly variant
    Horsefly,          // Large aggressive fly
    Firefly,           // Light/energy fly
    DragonFlies,       // Large aerial predator
    Dragonfly2,
    Damselfly,         // Light aerial scout

    // Bees family
    Hornets,           // Aggressive flying unit
    Wasps,             // Fast attack flyers
    Bumblebees,        // Heavy flying unit
    Honeybees,         // Economic flying unit
    MurderHornet,      // Elite aggressive flyer

    // Termites family
    Earwigs,           // Pincer assault unit

    // Individual species
    ScorpionVariant,   // Heavy ground predator
    StickBugs,         // Camouflaged units
    LeafBugs,          // Stealth units
    Cicadas,           // Sound/support units
    Grasshoppers,      // Jumping assault units
    Cockroaches,       // Tough survivor units
}

#[derive(Component, Debug, Clone)]
pub struct Selectable {
    pub is_selected: bool,
    pub selection_radius: f32,
}

impl Default for Selectable {
    fn default() -> Self {
        Self {
            is_selected: false,
            selection_radius: 5.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Formation {
    pub formation_type: FormationType,
    pub position_in_formation: Vec2,
    pub leader: Option<Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormationType {
    Line,
    Box,
    Wedge,
    Circle,
    Spread, // Wide spread formation with greater spacing
}

#[derive(Component, Debug, Clone)]
pub struct Vision {
    pub sight_range: f32,
    #[allow(dead_code)]
    pub line_of_sight: bool,
}

impl Default for Vision {
    fn default() -> Self {
        Self {
            sight_range: 150.0,
            line_of_sight: true,
        }
    }
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone)]
pub struct Constructable {
    pub building_type: BuildingType,
    pub build_time: f32,
    pub resource_cost: Vec<(ResourceType, f32)>,
}

#[derive(Component, Debug, Clone)]
pub struct Constructor {
    pub build_speed: f32,
    pub current_target: Option<Entity>,
}

/// Represents a planned building site that needs a worker to construct
#[derive(Component, Debug, Clone)]
pub struct BuildingSite {
    pub building_type: BuildingType,
    pub position: Vec3,
    pub player_id: u8,
    pub assigned_worker: Option<Entity>,
    pub construction_started: bool,
    pub site_reserved: bool,
}

/// Task assigned to a worker to construct a building
#[derive(Component, Debug, Clone)]
pub struct ConstructionTask {
    pub building_site: Entity,
    pub building_type: BuildingType,
    pub target_position: Vec3,
    pub is_moving_to_site: bool,
    pub construction_progress: f32,
    pub total_build_time: f32,
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone)]
pub struct Age {
    pub current_age: GameAge,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameAge {
    LarvalStage,
    PupalStage,
    AdultStage,
    SwarmStage,
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone)]
pub struct Technology {
    pub tech_type: TechnologyType,
    pub is_researched: bool,
    pub research_time: f32,
    pub research_cost: Vec<(ResourceType, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TechnologyType {
    ChitinWeaving,
    SharpMandibles,
    PheromoneBoost,
    CargoSacs,
    SwiftLegs,
    ColonyWatch,
    ColonyPatrol,
    StrongGenetics,
    Metamorphosis,
    ChitinForging,
    AcidGlands,
    VenomSacs,
}

#[derive(Component, Debug, Clone)]
pub struct Garrison {
    #[allow(dead_code)]
    pub capacity: u32,
    #[allow(dead_code)]
    pub garrisoned_units: Vec<Entity>,
    #[allow(dead_code)]
    pub protection_bonus: f32,
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone)]
pub struct Garrisonable {
    pub size: u32,
    pub garrisoned_in: Option<Entity>,
}

#[derive(Component)]
pub struct SelectionIndicator {
    pub target: Entity,
}

#[derive(Component)]
pub struct DragSelection {
    pub start_position: Vec2,
    pub current_position: Vec2,
    pub is_active: bool,
}

#[derive(Component)]
pub struct SelectionBox;

// Combat system components
#[derive(Event, Debug)]
pub struct DamageEvent {
    pub damage: f32,
    pub attacker: Entity,
    pub target: Entity,
    pub damage_type: DamageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Siege,
    True,
}

#[derive(Event, Debug)]
pub struct DeathEvent {
    pub entity: Entity,
    #[allow(dead_code)]
    pub killer: Option<Entity>,
}

/// Component to mark entities that are in the process of dying
/// This prevents duplicate death processing and race conditions
#[derive(Component, Debug)]
pub struct Dying;

/// Resource gathering state component
/// Tracks units that are collecting or returning resources
#[derive(Component, Debug, Clone, PartialEq)]
pub struct GatheringState {
    pub state: GatheringStateType,
    pub target_resource: Option<Entity>,
    pub return_building: Option<Entity>,
    pub gather_start_time: f32,
    pub last_state_change: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GatheringStateType {
    /// Moving to a resource to start gathering
    MovingToResource,
    /// Actively gathering from a resource
    Gathering,
    /// Moving back to base with gathered resources
    ReturningToBase,
    /// Delivering resources to a building
    DeliveringResources,
}

impl Default for GatheringState {
    fn default() -> Self {
        Self {
            state: GatheringStateType::MovingToResource,
            target_resource: None,
            return_building: None,
            gather_start_time: 0.0,
            last_state_change: 0.0,
        }
    }
}

/// Construction state component
/// Tracks units that are building structures
#[derive(Component, Debug, Clone, PartialEq)]
pub struct BuildingState {
    pub state: BuildingStateType,
    pub target_site: Option<Entity>,
    pub construction_progress: f32,
    pub build_start_time: f32,
    pub last_state_change: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuildingStateType {
    /// Moving to a construction site
    MovingToSite,
    /// Actively constructing the building
    #[allow(dead_code)] // Placeholder for building construction states
    Constructing,
    /// Construction completed, returning to idle
    #[allow(dead_code)] // Placeholder for building construction states
    ConstructionComplete,
}

impl Default for BuildingState {
    fn default() -> Self {
        Self {
            state: BuildingStateType::MovingToSite,
            target_site: None,
            construction_progress: 0.0,
            build_start_time: 0.0,
            last_state_change: 0.0,
        }
    }
}

/// Movement state component
/// Tracks units that are moving without other specific activities
#[derive(Component, Debug, Clone, PartialEq)]
pub struct MovementState {
    pub state: MovementStateType,
    pub destination: Vec3,
    pub move_start_time: f32,
    pub last_state_change: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementStateType {
    /// Moving to a destination
    Moving,
    /// Following another entity
    #[allow(dead_code)] // Placeholder for unit following behavior
    Following,
    /// Patrolling between waypoints
    #[allow(dead_code)] // Placeholder for patrol behavior
    Patrolling,
    /// Guarding a specific location
    #[allow(dead_code)] // Placeholder for guard behavior
    Guarding,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            state: MovementStateType::Moving,
            destination: Vec3::ZERO,
            move_start_time: 0.0,
            last_state_change: 0.0,
        }
    }
}

#[derive(Component, Debug)]
pub struct HealthBar {
    pub offset: Vec3,
    #[allow(dead_code)]
    pub size: Vec2,
    pub always_visible: bool,
}

impl Default for HealthBar {
    fn default() -> Self {
        Self {
            offset: Vec3::new(0.0, 3.0, 0.0),
            size: Vec2::new(2.0, 0.3),
            always_visible: false,
        }
    }
}

#[derive(Component, Debug)]
pub struct CombatStats {
    pub kills: u32,
    pub damage_dealt: f32,
    pub damage_taken: f32,
    pub experience: f32,
}

#[derive(Component, Debug, Clone)]
pub struct CollisionRadius {
    pub radius: f32,
}

impl Default for CollisionRadius {
    fn default() -> Self {
        Self {
            radius: 2.5, // Larger default radius for GLB models with scaling
        }
    }
}

/// Component for environment objects (non-interactive decorations)
#[derive(Component, Debug, Clone)]
pub struct EnvironmentObject {
    pub object_type: EnvironmentObjectType,
}

/// Types of environment objects available in the scene
#[derive(Debug, Clone, PartialEq)]
pub enum EnvironmentObjectType {
    /// Mushroom clusters for natural decoration
    Mushrooms,
    /// Hive structures for insect theme
    #[allow(dead_code)] // Placeholder for hive environment objects
    Hive,
    /// Wood stick debris for natural clutter
    WoodStick,

    // New environment object types
    /// Rock formations for geological features
    Rocks,
    /// Pine cone resources for natural pheromone gathering
    PineCone,
}

/// Team system for multi-player gameplay with specialized unit rosters
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamType {
    /// Heavy armored specialists - beetles and tanks
    BeetleSwarm,
    /// Predator specialists - spiders and mantids  
    Predators,
    /// Aerial superiority - flying units
    SkyDominion,
    /// Mass swarm tactics - small fast units
    TinyLegion,
    /// Balanced mixed forces - traditional RTS
    BalancedColony,
    /// Ant specialists - classic ant colony
    AntEmpire,
    /// Stealth and ambush specialists
    ShadowCrawlers,
    /// Siege and support specialists  
    HiveMind,
}

impl TeamType {
    /// Get the display name for this team
    pub fn display_name(&self) -> &str {
        match self {
            TeamType::BeetleSwarm => "Beetle Swarm",
            TeamType::Predators => "Predator Pack", 
            TeamType::SkyDominion => "Sky Dominion",
            TeamType::TinyLegion => "Tiny Legion",
            TeamType::BalancedColony => "Balanced Colony",
            TeamType::AntEmpire => "Ant Empire",
            TeamType::ShadowCrawlers => "Shadow Crawlers",
            TeamType::HiveMind => "Hive Mind",
        }
    }

    /// Get the description for this team
    pub fn description(&self) -> &str {
        match self {
            TeamType::BeetleSwarm => "Heavy armored beetles with powerful siege capabilities",
            TeamType::Predators => "Deadly spiders and mantids with stealth and venom",
            TeamType::SkyDominion => "Flying units dominate from the air",
            TeamType::TinyLegion => "Swarm tactics with mass small units",
            TeamType::BalancedColony => "Well-rounded forces for any situation", 
            TeamType::AntEmpire => "Classic ant colony with workers and soldiers",
            TeamType::ShadowCrawlers => "Stealth specialists and ambush predators",
            TeamType::HiveMind => "Support and siege units with area control",
        }
    }

    /// Get the available units for this team
    pub fn get_unit_roster(&self) -> Vec<UnitType> {
        match self {
            TeamType::BeetleSwarm => vec![
                // Workers (gatherers)
                UnitType::DungBeetle, // Primary gatherer
                // Combat beetles 
                UnitType::StagBeetle,
                UnitType::RhinoBeetle,
                UnitType::BeetleKnight,
                UnitType::JewelBug,
                UnitType::LegBeetle,
                // Siege beetles
                UnitType::BatteringBeetle,
                UnitType::StinkBeetle,
                // Support
                UnitType::Cockroaches, // Related to beetles
            ],
            TeamType::Predators => vec![
                // Workers (gatherers)
                UnitType::Silverfish, // Primary gatherer
                // Spider predators
                UnitType::WidowSpider,
                UnitType::WolfSpider,
                UnitType::WolfSpiderVariant,
                UnitType::Tarantula,
                UnitType::SpiderHunter,
                UnitType::EliteSpider,
                UnitType::DaddyLongLegs,
                // Mantis predators
                UnitType::CommonMantis,
                UnitType::OrchidMantis,
                UnitType::SpearMantis,
            ],
            TeamType::SkyDominion => vec![
                // Workers (gatherers)
                UnitType::Honeybees, // Primary gatherer
                UnitType::HoneyBee, // Secondary gatherer
                // Flying combat units
                UnitType::Bumblebees,
                UnitType::Hornets,
                UnitType::Wasps,
                UnitType::MurderHornet,
                UnitType::DragonFly,
                UnitType::DragonFlies,
                UnitType::Damselfly,
                // Flying support
                UnitType::Moths,
                UnitType::PeacockMoth,
                UnitType::Housefly,
                UnitType::HouseflyVariant,
                UnitType::Horsefly,
                UnitType::Firefly,
            ],
            TeamType::TinyLegion => vec![
                // Swarm workers (gatherers) 
                UnitType::Aphids, // Primary gatherer
                UnitType::Mites, // Secondary gatherer
                // Mass swarm units
                UnitType::Lice,
                UnitType::Fleas,
                UnitType::SandFleas,
                UnitType::Ticks,
                UnitType::Grasshoppers,
                UnitType::Caterpillars,
                // Support
                UnitType::Cicadas,
            ],
            TeamType::BalancedColony => vec![
                // Classic balanced roster - mixed species
                UnitType::WorkerAnt, // Primary gatherer
                UnitType::SoldierAnt,
                UnitType::ScoutAnt,
                UnitType::BeetleKnight,
                UnitType::SpearMantis,
                UnitType::DragonFly,
                UnitType::HoneyBee,
                UnitType::Scorpion,
                UnitType::DefenderBug,
                UnitType::BatteringBeetle,
                UnitType::AcidSpitter,
            ],
            TeamType::AntEmpire => vec![
                // Pure ant specialists
                UnitType::WorkerAnt, // Primary gatherer
                UnitType::SoldierAnt,
                UnitType::ScoutAnt,
                UnitType::WorkerFourmi, // Secondary gatherer
                UnitType::SoldierFourmi,
                UnitType::RedAnt,
                UnitType::BlackAnt,
                UnitType::FireAnt,
                // Termites (related to ants)
                UnitType::TermiteWorker, // Tertiary gatherer
                UnitType::TermiteWarrior,
                // Support
                UnitType::Earwigs, // Related social insects
            ],
            TeamType::ShadowCrawlers => vec![
                // Stealth workers (gatherers)
                UnitType::Silverfish, // Primary gatherer
                UnitType::Mites, // Secondary gatherer (small/stealthy)
                // Camouflage units
                UnitType::StickBugs,
                UnitType::LeafBugs,
                UnitType::OrchidMantis,
                // Night hunters
                UnitType::WidowSpider,
                UnitType::Moths,
                UnitType::DaddyLongLegs,
                // Ground stealth
                UnitType::Woodlouse,
                UnitType::Pillbug,
                UnitType::Ticks,
            ],
            TeamType::HiveMind => vec![
                // Social worker insects (gatherers)
                UnitType::Honeybees, // Primary gatherer
                UnitType::TermiteWorker, // Secondary gatherer
                // Area denial
                UnitType::Stinkbug,
                UnitType::StinkBeetle,
                UnitType::AcidSpitter,
                // Communication/support
                UnitType::Cicadas,
                UnitType::Firefly,
                // Heavy support
                UnitType::BatteringBeetle,
                UnitType::TermiteWarrior,
                // Defense
                UnitType::DefenderBug,
                UnitType::Cockroaches,
                UnitType::ScorpionVariant,
            ],
        }
    }

    /// Get all available team types
    pub fn all_teams() -> Vec<TeamType> {
        vec![
            TeamType::BeetleSwarm,
            TeamType::Predators,
            TeamType::SkyDominion,
            TeamType::TinyLegion,
            TeamType::BalancedColony,
            TeamType::AntEmpire,
            TeamType::ShadowCrawlers,
            TeamType::HiveMind,
        ]
    }
}

/// Component to track which team a player belongs to
#[derive(Component, Debug, Clone)]
pub struct PlayerTeam {
    #[allow(dead_code)] // Placeholder for team-based gameplay
    pub team_type: TeamType,
    #[allow(dead_code)] // Placeholder for team-based gameplay
    pub player_id: u8,
}

/// Resource to store game setup configuration
#[derive(Resource, Debug, Clone)]
pub struct GameSetup {
    pub player_team: TeamType,
    pub ai_teams: Vec<TeamType>,
    #[allow(dead_code)] // Placeholder for multiplayer setup
    pub player_count: u8,
}
