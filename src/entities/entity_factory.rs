//! # Unified Entity Factory
//!
//! Consolidated factory for spawning all game entities with clean parameter passing
//! and efficient code organization. Replaces scattered spawn functions across the codebase.

use crate::core::components::*;
use crate::rendering::animation_systems::*;
use crate::rendering::model_loader::*;
use bevy::prelude::*;
use rand;

/// Configuration for spawning any entity in the game
/// This struct consolidates all possible spawn parameters into a clean interface
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    /// The type of entity to spawn
    pub entity_type: EntityType,
    /// World position for the entity
    pub position: Vec3,
    /// Player ID (1-4, or 0 for neutral)
    pub player_id: u8,
    /// Optional explicit unit ID (generates random if None)
    pub unit_id: Option<u32>,
    /// Optional explicit scale override (uses defaults if None)
    pub scale_override: Option<f32>,
    /// Whether to spawn directly as GLB model (if available)
    pub prefer_glb: bool,
}

impl SpawnConfig {
    /// Create a basic spawn config for a unit
    pub fn unit(entity_type: EntityType, position: Vec3, player_id: u8) -> Self {
        Self {
            entity_type,
            position,
            player_id,
            unit_id: None,
            scale_override: None,
            prefer_glb: true,
        }
    }

    /// Create a basic spawn config for a building
    pub fn building(entity_type: EntityType, position: Vec3, player_id: u8) -> Self {
        Self {
            entity_type,
            position,
            player_id,
            unit_id: None,
            scale_override: None,
            prefer_glb: true,
        }
    }
}

/// All possible entity types that can be spawned
#[derive(Debug, Clone)]
pub enum EntityType {
    // Units
    Unit(UnitType),
    // Buildings
    Building(BuildingType),
}

impl EntityType {
    /// Convert unit type to entity type
    pub fn from_unit(unit_type: UnitType) -> Self {
        Self::Unit(unit_type)
    }

    /// Convert building type to entity type
    pub fn from_building(building_type: BuildingType) -> Self {
        Self::Building(building_type)
    }
}

/// Unified entity factory that handles all spawning in the game
/// This replaces all the scattered spawn functions across different modules
pub struct EntityFactory;

impl EntityFactory {
    /// Single entry point for spawning any entity in the game
    /// This method handles units, buildings, and resources with consistent parameters
    pub fn spawn(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        config: SpawnConfig,
        model_assets: Option<&ModelAssets>,
    ) -> Entity {
        match &config.entity_type {
            EntityType::Unit(unit_type) => Self::spawn_unit(
                commands,
                meshes,
                materials,
                unit_type.clone(),
                config,
                model_assets,
            ),
            EntityType::Building(building_type) => Self::spawn_building(
                commands,
                meshes,
                materials,
                building_type.clone(),
                config,
                model_assets,
            ),
        }
    }

    /// Spawn a unit with the specified configuration
    fn spawn_unit(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        unit_type: UnitType,
        config: SpawnConfig,
        model_assets: Option<&ModelAssets>,
    ) -> Entity {
        let unit_id = config.unit_id.unwrap_or_else(rand::random);
        let unit_stats_config = crate::core::unit_stats::get_unit_stats(&unit_type);

        // Determine model scale - always use correct scale for GLB model upgrade compatibility
        let model_scale = if let Some(scale) = config.scale_override {
            scale
        } else {
            // Always determine the correct scale based on unit type, even for primitives
            // This ensures proper scaling when GLB models are loaded later
            let model_type = get_unit_insect_model(&unit_type);
            get_model_scale(&model_type)
        };

        // Create the visual representation
        let mut entity = if let (true, Some(assets)) = (config.prefer_glb, model_assets) {
            Self::spawn_unit_with_glb(commands, &unit_type, &config, model_scale, assets)
        } else {
            Self::spawn_unit_with_primitive(
                commands,
                meshes,
                materials,
                &unit_type,
                &config,
                model_scale,
            )
        };

        // Add common components
        Self::add_unit_components(
            &mut entity,
            unit_type,
            unit_id,
            config,
            unit_stats_config,
            model_scale,
        );

        entity.id()
    }

    /// Create GLB model entity for unit
    fn spawn_unit_with_glb<'a>(
        commands: &'a mut Commands,
        unit_type: &UnitType,
        config: &SpawnConfig,
        model_scale: f32,
        model_assets: &ModelAssets,
    ) -> EntityCommands<'a> {
        let model_type = get_unit_insect_model(unit_type);
        let model_handle = model_assets.get_model_handle(&model_type);

        // Determine correct rotation for each model type
        let rotation = Self::get_model_rotation(unit_type, &model_type);

        commands.spawn((
            SceneRoot(model_handle),
            Transform::from_translation(config.position)
                .with_scale(Vec3::splat(model_scale))
                .with_rotation(rotation),
            InsectModel {
                model_type,
                scale: model_scale,
            },
            UseGLBModel,
            TeamColor::new(config.player_id),
        ))
    }

    /// Create primitive shape entity for unit
    fn spawn_unit_with_primitive<'a>(
        commands: &'a mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        unit_type: &UnitType,
        config: &SpawnConfig,
        model_scale: f32,
    ) -> EntityCommands<'a> {
        let (mesh, _) = Self::get_unit_primitive_mesh(unit_type);
        // Use the calculated model_scale instead of the primitive's default scale

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: TeamColor::get_primitive_color(config.player_id),
                ..default()
            })),
            Transform::from_translation(config.position).with_scale(Vec3::splat(model_scale)),
            TeamColor::new(config.player_id),
        ))
    }

    /// Add components to a unit entity
    fn add_unit_components(
        entity: &mut EntityCommands,
        unit_type: UnitType,
        unit_id: u32,
        config: SpawnConfig,
        stats: crate::core::unit_stats::UnitStatsConfig,
        _model_scale: f32, // Unused - visual scale is now separate from gameplay mechanics
    ) {
        // Debug logging for DragonFly spawning
        if matches!(unit_type, UnitType::DragonFly) {
            info!(
                "Spawning DragonFly with stats - max_speed: {:.1}, acceleration: {:.1}",
                stats.movement.max_speed, stats.movement.acceleration
            );
        }

        // Add base components
        entity.insert((
            RTSUnit {
                unit_id,
                player_id: config.player_id,
                size: 1.0,
                unit_type: Some(unit_type.clone()),
            },
            Position {
                translation: config.position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: stats.movement.max_speed, // Don't scale movement speed - it should be consistent regardless of visual size
                acceleration: stats.movement.acceleration, // Don't scale acceleration either
                turning_speed: stats.movement.turning_speed,
                ..default()
            },
            RTSHealth {
                current: stats.health.current,
                max: stats.health.max,
                armor: stats.health.armor,
                regeneration_rate: stats.health.regeneration_rate,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: stats.combat.attack_damage,
                attack_range: stats.combat.attack_range,
                attack_speed: stats.combat.attack_speed,
                last_attack_time: 0.0,
                target: None,
                attack_type: stats.combat.attack_type,
                attack_cooldown: 1.0 / stats.combat.attack_speed, // Convert attack_speed to cooldown
                is_attacking: false,
                auto_attack: stats.combat.auto_attack,
            },
            Vision {
                sight_range: stats.vision.sight_range,
                line_of_sight: stats.vision.line_of_sight,
            },
            Selectable::default(),
            CollisionRadius {
                radius: stats.collision_radius,
            },
            // EntityState removed - using specialized state components instead
            GameEntity,
        ));

        // Add animation controller for units with animations
        entity.insert(UnitAnimationController {
            current_state: AnimationState::Idle,
            previous_state: AnimationState::Idle,
            animation_player: None, // Will be populated by find_animation_players system
            animation_node_index: None, // Will be populated by setup_glb_animations system
        });
        info!("Added animation controller to unit {:?}", unit_type);

        // Add special components for specific unit types
        if let UnitType::WorkerAnt = unit_type {
            entity.insert((
                ResourceGatherer {
                    gather_rate: 10.0,
                    capacity: 5.0, // Reduced from 10.0 for faster testing
                    carried_amount: 0.0,
                    resource_type: None,
                    target_resource: None,
                    drop_off_building: None,
                },
                Constructor {
                    build_speed: 1.0,
                    current_target: None,
                },
            ));
        }
    }

    /// Spawn a building with the specified configuration
    fn spawn_building(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        building_type: BuildingType,
        config: SpawnConfig,
        model_assets: Option<&ModelAssets>,
    ) -> Entity {
        let building_stats = Self::get_building_stats(&building_type);

        // Create the visual representation - prefer GLB models if available
        let mut entity = if let (true, Some(assets)) = (config.prefer_glb, model_assets) {
            Self::spawn_building_with_glb(commands, &building_type, &config, assets)
        } else {
            Self::spawn_building_with_primitive(
                commands,
                meshes,
                materials,
                &building_type,
                &config,
            )
        };

        // Add common building components
        entity.insert((
            RTSUnit {
                unit_id: config.unit_id.unwrap_or(0),
                player_id: config.player_id,
                size: building_stats.size,
                unit_type: None,
            },
            TeamColor::new(config.player_id),
            Position {
                translation: config.position,
                rotation: Quat::IDENTITY,
            },
            Building {
                building_type: building_type.clone(),
                construction_progress: 100.0,
                max_construction: 100.0,
                is_complete: true,
                rally_point: building_stats
                    .rally_point
                    .map(|offset| config.position + offset),
            },
            RTSHealth {
                current: building_stats.health,
                max: building_stats.health,
                armor: building_stats.armor,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            Selectable {
                is_selected: false,
                selection_radius: building_stats.selection_radius,
            },
            CollisionRadius {
                radius: building_stats.collision_radius,
            },
            GameEntity,
        ));

        // Add special components based on building type
        building_stats.special_components.add_to_entity(&mut entity);

        entity.id()
    }

    /// Create GLB model entity for building
    fn spawn_building_with_glb<'a>(
        commands: &'a mut Commands,
        building_type: &BuildingType,
        config: &SpawnConfig,
        model_assets: &ModelAssets,
    ) -> EntityCommands<'a> {
        let model_type = crate::rendering::model_loader::get_building_insect_model(building_type);
        let model_handle = model_assets.get_model_handle(&model_type);
        let model_scale = crate::rendering::model_loader::get_model_scale(&model_type);

        commands.spawn((
            SceneRoot(model_handle),
            Transform::from_translation(config.position).with_scale(Vec3::splat(model_scale)),
            InsectModel {
                model_type,
                scale: model_scale,
            },
            UseGLBModel,
            TeamColor::new(config.player_id),
        ))
    }

    /// Create primitive shape entity for building
    fn spawn_building_with_primitive<'a>(
        commands: &'a mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        building_type: &BuildingType,
        config: &SpawnConfig,
    ) -> EntityCommands<'a> {
        let (mesh, offset) = Self::get_building_mesh(building_type);

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: TeamColor::get_primitive_color(config.player_id),
                ..default()
            })),
            Transform::from_translation(config.position + offset),
            TeamColor::new(config.player_id),
        ))
    }
}

// Helper structs for organizing entity statistics

#[derive(Debug, Clone)]
struct BuildingStats {
    health: f32,
    armor: f32,
    size: f32,
    selection_radius: f32,
    collision_radius: f32,
    rally_point: Option<Vec3>,
    special_components: BuildingSpecialComponents,
}

#[derive(Debug, Clone)]
enum BuildingSpecialComponents {
    None,
    ProductionQueue {
        production_time: f32,
    },
    ProductionWithGarrison {
        production_time: f32,
        garrison_capacity: u32,
        protection_bonus: f32,
    },
}

impl BuildingSpecialComponents {
    fn add_to_entity(&self, entity: &mut EntityCommands) {
        match self {
            Self::None => {}
            Self::ProductionQueue { production_time } => {
                entity.insert(ProductionQueue {
                    queue: Vec::new(),
                    current_progress: 0.0,
                    production_time: *production_time,
                });
            }
            Self::ProductionWithGarrison {
                production_time,
                garrison_capacity,
                protection_bonus,
            } => {
                entity.insert((
                    ProductionQueue {
                        queue: Vec::new(),
                        current_progress: 0.0,
                        production_time: *production_time,
                    },
                    Garrison {
                        capacity: *garrison_capacity,
                        garrisoned_units: Vec::new(),
                        protection_bonus: *protection_bonus,
                    },
                    Vision {
                        sight_range: 200.0,
                        line_of_sight: true,
                    },
                ));
            }
        }
    }
}

impl EntityFactory {
    fn get_building_stats(building_type: &BuildingType) -> BuildingStats {
        match building_type {
            BuildingType::Queen => BuildingStats {
                health: 600.0,
                armor: 5.0,
                size: 4.0,
                selection_radius: 10.0,
                collision_radius: crate::constants::collision::QUEEN_COLLISION_RADIUS,
                rally_point: Some(Vec3::new(30.0, 0.0, 0.0)),
                special_components: BuildingSpecialComponents::ProductionWithGarrison {
                    production_time: 25.0,
                    garrison_capacity: 15,
                    protection_bonus: 2.0,
                },
            },
            BuildingType::Nursery => BuildingStats {
                health: 75.0,
                armor: 0.0,
                size: 2.0,
                selection_radius: 5.0,
                collision_radius: crate::constants::collision::NURSERY_COLLISION_RADIUS,
                rally_point: Some(Vec3::new(15.0, 0.0, 15.0)),
                special_components: BuildingSpecialComponents::ProductionQueue {
                    production_time: 18.0, // Faster than Queen for flying units
                },
            },
            BuildingType::WarriorChamber => BuildingStats {
                health: 200.0,
                armor: 2.0,
                size: 3.0,
                selection_radius: 8.0,
                collision_radius: crate::constants::collision::WARRIOR_CHAMBER_COLLISION_RADIUS,
                rally_point: Some(Vec3::new(20.0, 0.0, 0.0)),
                special_components: BuildingSpecialComponents::ProductionQueue {
                    production_time: 21.0,
                },
            },
            // Default case
            _ => BuildingStats {
                health: 100.0,
                armor: 1.0,
                size: 2.0,
                selection_radius: 5.0,
                collision_radius: crate::constants::collision::DEFAULT_BUILDING_COLLISION_RADIUS,
                rally_point: None,
                special_components: BuildingSpecialComponents::None,
            },
        }
    }

    /// Get the correct rotation for a model based on unit type and model type
    fn get_model_rotation(unit_type: &UnitType, model_type: &InsectModelType) -> Quat {
        match (unit_type, model_type) {
            // Ant models (Fourmi) need no additional rotation - they face forward correctly
            (UnitType::WorkerAnt, InsectModelType::Fourmi) => Quat::IDENTITY,
            (UnitType::SoldierAnt, InsectModelType::Fourmi) => Quat::IDENTITY,

            // Mantis models need 180° rotation - they're facing backwards
            (UnitType::SpearMantis, InsectModelType::QueenFacedBug) => {
                Quat::from_rotation_y(std::f32::consts::PI)
            }

            // Butterfly/Scout models face forward correctly - no rotation needed
            (UnitType::ScoutAnt, InsectModelType::CairnsBirdwing) => Quat::IDENTITY,

            // Hornet/Wasp models face forward correctly - no rotation needed
            (UnitType::HunterWasp, InsectModelType::Hornet) => Quat::IDENTITY,

            // Beetle models face forward correctly - no rotation needed
            (UnitType::BeetleKnight, InsectModelType::RhinoBeetle) => Quat::IDENTITY,

            // DragonFly model - may need specific rotation
            (UnitType::DragonFly, InsectModelType::DragonFly) => {
                Quat::from_rotation_y(std::f32::consts::PI)
            }

            // BatteringBeetle uses black_ox_beetle_small.glb
            (UnitType::BatteringBeetle, InsectModelType::Beetle) => {
                Quat::from_rotation_y(std::f32::consts::PI)
            }

            // Default case: 180° rotation for most models since they typically face backwards
            _ => Quat::from_rotation_y(std::f32::consts::PI),
        }
    }

    fn get_unit_primitive_mesh(unit_type: &UnitType) -> (Mesh, f32) {
        match unit_type {
            UnitType::WorkerAnt => (Capsule3d::new(1.0, 2.0).into(), 1.0),
            UnitType::SoldierAnt => (Capsule3d::new(1.2, 2.2).into(), 1.0),
            UnitType::HunterWasp => (Capsule3d::new(1.0, 2.0).into(), 1.0),
            UnitType::SpearMantis => (Capsule3d::new(1.2, 2.5).into(), 1.0),
            UnitType::ScoutAnt => (Capsule3d::new(0.9, 2.0).into(), 1.0),
            UnitType::BeetleKnight => (Capsule3d::new(1.5, 2.5).into(), 1.0),
            _ => (Capsule3d::new(1.0, 2.0).into(), 1.0),
        }
    }

    fn get_building_mesh(building_type: &BuildingType) -> (Mesh, Vec3) {
        match building_type {
            BuildingType::Queen => (
                Cuboid::new(20.0, 15.0, 20.0).into(),
                Vec3::new(0.0, 7.5, 0.0),
            ),
            BuildingType::Nursery => (Cuboid::new(8.0, 6.0, 8.0).into(), Vec3::new(0.0, 3.0, 0.0)),
            BuildingType::WarriorChamber => (
                Cuboid::new(12.0, 8.0, 12.0).into(),
                Vec3::new(0.0, 4.0, 0.0),
            ),
            _ => (Cuboid::new(8.0, 6.0, 8.0).into(), Vec3::new(0.0, 3.0, 0.0)),
        }
    }
}
