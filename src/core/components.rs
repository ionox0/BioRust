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
            _ => UNKNOWN_PLAYER_TINT,
        };
        Self { player_id, tint_color }
    }
    
    /// Get the primitive color for a player (used for fallback geometric shapes)
    pub fn get_primitive_color(player_id: u8) -> Color {
        use crate::core::constants::team_colors::*;
        match player_id {
            1 => PLAYER_1_PRIMITIVE,
            2 => PLAYER_2_PRIMITIVE,
            3 => PLAYER_3_PRIMITIVE,
            4 => PLAYER_4_PRIMITIVE,
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

impl Default for Movement {
    fn default() -> Self {
        Self {
            max_speed: 200.0,  // 2x speed increase for default
            acceleration: 400.0,  // 2x acceleration increase for default
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
    Ranged,
    Siege,
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
    pub fn subtract_from(&self, resources: &mut crate::core::resources::PlayerResources, amount: f32) {
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
    HunterWasp,
    BeetleKnight,
    SpearMantis,
    ScoutAnt,
    BatteringBeetle,
    AcidSpitter,
    
    // Additional unit types for new models
    DragonFly,      // Flying reconnaissance unit
    DefenderBug,    // Defensive unit
    EliteSpider,    // Elite predator unit

    // Units for previously unused models
    HoneyBee,       // Basic flying unit (bee-v1.glb)
    Scorpion,       // Heavy melee unit with armor
    SpiderHunter,   // Light predator unit (spider_small.glb)
    WolfSpider,     // Heavy predator unit
    Ladybug,        // Balanced mid-tier unit
    LadybugScout,   // Light scout variant (ladybug_simple.glb)

    // Units for newly added models
    Housefly,       // Fast flying harassment unit
    TermiteWorker,  // Builder/gatherer specialist
    TermiteWarrior, // Heavy siege unit (giant_termite.glb)
    LegBeetle,      // Fast melee skirmisher
    Stinkbug,       // Area denial/support unit
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
    Pierce,
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

/// General entity state for game logic (separate from animation states)
/// This tracks the high-level state of what an entity is doing
#[derive(Component, Debug, Clone, PartialEq)]
pub struct EntityState {
    pub current: UnitState,
    pub previous: UnitState,
    pub state_timer: f32, // How long the entity has been in this state
}

impl Default for EntityState {
    fn default() -> Self {
        Self {
            current: UnitState::Idle,
            previous: UnitState::Idle,
            state_timer: 0.0,
        }
    }
}

/// High-level states that entities can be in
/// This affects AI behavior, decision making, and game logic
#[derive(Debug, Clone, PartialEq)]
pub enum UnitState {
    /// Entity is not doing anything specific
    Idle,
    /// Entity is moving to a target location
    Moving,
    /// Entity is in combat (attacking or being attacked)
    Fighting,
    /// Entity is gathering resources
    Gathering,
    /// Entity is returning to base with gathered resources
    ReturningWithResources,
    /// Entity is constructing a building
    #[allow(dead_code)]
    Building,
    /// Entity is dead and should be cleaned up
    Dead,
    /// Entity is following another entity
    #[allow(dead_code)]
    Following,
    /// Entity is patrolling between waypoints
    #[allow(dead_code)]
    Patrolling,
    /// Entity is guarding a specific location or entity
    #[allow(dead_code)]
    Guarding,
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
    Hive,
    /// Wood stick debris for natural clutter
    WoodStick,
    
    // New environment object types
    /// Rock formations for geological features
    Rocks,
}