use bevy::prelude::*;
use crate::core::components::*;
use crate::rendering::model_loader::*;

pub struct RTSEntityFactory;

impl RTSEntityFactory {
    pub fn spawn_worker_ant(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
    ) -> Entity {
        Self::spawn_worker_ant_with_models(commands, meshes, materials, position, player_id, unit_id, None)
    }

    pub fn spawn_worker_ant_with_models(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
        model_assets: Option<&ModelAssets>,
    ) -> Entity {
        // Calculate model scale factor for movement adjustment
        let model_scale = if let Some(_models) = model_assets {
            let model_type = get_unit_insect_model(&UnitType::WorkerAnt);
            match model_type {
                crate::rendering::model_loader::InsectModelType::QueenFacedBug => crate::constants::models::MANTIS_SCALE,
                crate::rendering::model_loader::InsectModelType::ApisMellifera => crate::constants::models::APIS_MELLIFERA_SCALE,
                _ => crate::rendering::model_loader::get_model_scale(&model_type),
            }
        } else {
            1.0 // Primitive models use default scale
        };
        
        let mut entity = if let Some(models) = model_assets {
            // Use GLB model
            let model_type = get_unit_insect_model(&UnitType::WorkerAnt);
            let model_handle = models.get_model_handle(&model_type);
            
            commands.spawn((
                SceneRoot(model_handle),
                Transform::from_translation(position)
                    .with_scale(Vec3::splat(model_scale))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                InsectModel {
                    model_type,
                    scale: model_scale,
                },
                UseGLBModel,
                TeamColor::new(player_id), // Add team coloring
            ))
        } else {
            // Fallback to primitive shapes
            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(1.0, 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: TeamColor::get_primitive_color(player_id),
                    ..default()
                })),
                Transform::from_translation(position),
            ))
        };

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(UnitType::WorkerAnt) },
            TeamColor::new(player_id), // Add team coloring for primitive units too
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
            max_speed: 50.0 / model_scale,  // 2x speed increase
            acceleration: 30.0 / model_scale,  // 2x acceleration increase
            turning_speed: 3.0,  // Slightly faster turning
            ..default()
        },
            RTSHealth {
                current: 75.0,
                max: 75.0,
                armor: 0.0,
                regeneration_rate: 0.1,
                last_damage_time: 0.0,
            },
            ResourceGatherer {
                gather_rate: 10.0,
                capacity: 10.0,
                carried_amount: 0.0,
                resource_type: None,
                target_resource: None,
                drop_off_building: None,
            },
            Constructor {
                build_speed: 1.0,
                current_target: None,
            },
            Selectable::default(),
            Vision::default(),
            CollisionRadius { radius: crate::constants::collision::WORKER_ANT_COLLISION_RADIUS },
            EntityState::default(),
            GameEntity,
        )).id()
    }

    pub fn spawn_soldier_ant(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
    ) -> Entity {
        Self::spawn_soldier_ant_with_models(commands, meshes, materials, position, player_id, unit_id, None)
    }

    pub fn spawn_soldier_ant_with_models(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
        model_assets: Option<&ModelAssets>,
    ) -> Entity {
        // Calculate model scale factor for movement adjustment
        let model_scale = if let Some(_models) = model_assets {
            let model_type = get_unit_insect_model(&UnitType::SoldierAnt);
            match model_type {
                crate::rendering::model_loader::InsectModelType::QueenFacedBug => crate::constants::models::MANTIS_SCALE,
                crate::rendering::model_loader::InsectModelType::ApisMellifera => crate::constants::models::APIS_MELLIFERA_SCALE,
                _ => crate::rendering::model_loader::get_model_scale(&model_type),
            }
        } else {
            1.0 // Primitive models use default scale
        };
        
        let mut entity = if let Some(models) = model_assets {
            // Use GLB model
            let model_type = get_unit_insect_model(&UnitType::SoldierAnt);
            let model_handle = models.get_model_handle(&model_type);
            
            commands.spawn((
                SceneRoot(model_handle),
                Transform::from_translation(position)
                    .with_scale(Vec3::splat(model_scale))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                InsectModel {
                    model_type,
                    scale: model_scale,
                },
                UseGLBModel,
                TeamColor::new(player_id), // Add team coloring
            ))
        } else {
            // Fallback to primitive shapes
            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(1.2, 2.2))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: TeamColor::get_primitive_color(player_id),
                    ..default()
                })),
                Transform::from_translation(position),
            ))
        };

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(UnitType::SoldierAnt) },
            TeamColor::new(player_id), // Add team coloring
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 60.0 / model_scale,  // 2x speed increase
                acceleration: 40.0 / model_scale,  // 2x acceleration increase
                turning_speed: 3.5,  // Slightly faster turning
                ..default()
            },
            RTSHealth {
                current: 120.0,
                max: 120.0,
                armor: 1.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: 4.0,
                attack_range: 5.0,
                attack_speed: 2.0,
                last_attack_time: 0.0,
                target: None,
                attack_type: AttackType::Melee,
                attack_cooldown: 0.0,
                is_attacking: false,
                auto_attack: true,
            },
            Selectable::default(),
            Vision::default(),
            CollisionRadius { radius: crate::constants::collision::SOLDIER_ANT_COLLISION_RADIUS },
            EntityState::default(),
            GameEntity,
        )).id()
    }

    pub fn spawn_hunter_wasp(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
    ) -> Entity {
        commands.spawn((
            Mesh3d(meshes.add(Capsule3d::new(1.0, 2.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: TeamColor::get_primitive_color(player_id),
                ..default()
            })),
            Transform::from_translation(position),
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(UnitType::HunterWasp) },
            TeamColor::new(player_id), // Add team coloring
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 60.0,  // 2x speed increase
                acceleration: 120.0,  // 2x acceleration increase
                turning_speed: 3.5,  // Slightly faster turning
                ..default()
            },
            RTSHealth {
                current: 90.0,
                max: 90.0,
                armor: 0.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: 6.0,
                attack_range: 15.0,
                attack_speed: 1.5,
                last_attack_time: 0.0,
                target: None,
                attack_type: AttackType::Ranged,
                attack_cooldown: 0.0,
                is_attacking: false,
                auto_attack: true,
            },
            Selectable::default(),
            Vision {
                sight_range: 200.0,
                line_of_sight: true,
            },
            CollisionRadius { radius: crate::constants::collision::HUNTER_WASP_COLLISION_RADIUS },
            EntityState::default(),
            GameEntity,
        )).id()
    }

    pub fn spawn_spear_mantis(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
    ) -> Entity {
        Self::spawn_spear_mantis_with_models(commands, meshes, materials, position, player_id, unit_id, None)
    }

    pub fn spawn_spear_mantis_with_models(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
        model_assets: Option<&ModelAssets>,
    ) -> Entity {
        // Calculate model scale factor for movement adjustment
        let model_scale = if let Some(_models) = model_assets {
            let model_type = get_unit_insect_model(&UnitType::SpearMantis);
            match model_type {
                crate::rendering::model_loader::InsectModelType::QueenFacedBug => crate::constants::models::QUEEN_FACED_BUG_SCALE,
                _ => crate::rendering::model_loader::get_model_scale(&model_type),
            }
        } else {
            1.0 // Primitive models use default scale
        };
        
        let mut entity = if let Some(models) = model_assets {
            // Use GLB model
            let model_type = get_unit_insect_model(&UnitType::SpearMantis);
            let model_handle = models.get_model_handle(&model_type);
            
            commands.spawn((
                SceneRoot(model_handle),
                Transform::from_translation(position)
                    .with_scale(Vec3::splat(model_scale))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                InsectModel {
                    model_type,
                    scale: model_scale,
                },
                UseGLBModel,
                TeamColor::new(player_id), // Add team coloring
            ))
        } else {
            // Fallback to primitive shapes
            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(1.2, 2.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: TeamColor::get_primitive_color(player_id),
                    ..default()
                })),
                Transform::from_translation(position),
            ))
        };

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(UnitType::SpearMantis) },
            TeamColor::new(player_id), // Add team coloring
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 100.0 / model_scale.max(1.5), // 4x speed increase - faster mantis
                acceleration: 200.0 / model_scale.max(1.5),  // 4x acceleration increase
                turning_speed: 4.5,  // Faster turning
                ..default()
            },
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
                attack_cooldown: 0.0,
                is_attacking: false,
                auto_attack: true,
            },
            Selectable::default(),
            Vision {
                sight_range: 120.0,
                line_of_sight: true,
            },
            CollisionRadius { radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS },
            EntityState::default(),
            GameEntity,
        )).id()
    }

    pub fn spawn_scout_ant(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
    ) -> Entity {
        Self::spawn_scout_ant_with_models(commands, meshes, materials, position, player_id, unit_id, None)
    }

    pub fn spawn_scout_ant_with_models(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
        model_assets: Option<&ModelAssets>,
    ) -> Entity {
        // Calculate model scale factor for movement adjustment
        let model_scale = if let Some(_models) = model_assets {
            let model_type = get_unit_insect_model(&UnitType::ScoutAnt);
            match model_type {
                crate::rendering::model_loader::InsectModelType::CairnsBirdwing => crate::constants::models::CAIRNS_BIRDWING_SCALE,
                _ => crate::rendering::model_loader::get_model_scale(&model_type),
            }
        } else {
            1.0 // Primitive models use default scale
        };
        
        let mut entity = if let Some(models) = model_assets {
            // Use GLB model
            let model_type = get_unit_insect_model(&UnitType::ScoutAnt);
            let model_handle = models.get_model_handle(&model_type);
            
            commands.spawn((
                SceneRoot(model_handle),
                Transform::from_translation(position)
                    .with_scale(Vec3::splat(model_scale))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                InsectModel {
                    model_type,
                    scale: model_scale,
                },
                UseGLBModel,
                TeamColor::new(player_id), // Add team coloring
            ))
        } else {
            // Fallback to primitive shapes
            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(0.9, 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: TeamColor::get_primitive_color(player_id),
                    ..default()
                })),
                Transform::from_translation(position),
            ))
        };

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(UnitType::ScoutAnt) },
            TeamColor::new(player_id), // Add team coloring
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 80.0 / model_scale.max(2.0), // 2x speed increase
                acceleration: 140.0 / model_scale.max(2.0),  // 2x acceleration increase
                turning_speed: 4.2,  // Slightly faster turning
                ..default()
            },
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
                attack_cooldown: 0.0,
                is_attacking: false,
                auto_attack: true,
            },
            Selectable::default(),
            Vision {
                sight_range: 180.0,
                line_of_sight: true,
            },
            CollisionRadius { radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS },
            EntityState::default(),
            GameEntity,
        )).id()
    }

    pub fn spawn_beetle_knight(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
    ) -> Entity {
        commands.spawn((
            Mesh3d(meshes.add(Capsule3d::new(1.5, 2.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: TeamColor::get_primitive_color(player_id),
                ..default()
            })),
            Transform::from_translation(position),
            RTSUnit { unit_id, player_id, size: 1.5, unit_type: Some(UnitType::BeetleKnight) },
            TeamColor::new(player_id), // Add team coloring
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 70.0,  // 2x speed increase
                acceleration: 140.0,  // 2x acceleration increase
                turning_speed: 4.0,  // Slightly faster turning
                ..default()
            },
            RTSHealth {
                current: 200.0,
                max: 200.0,
                armor: 3.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: 12.0,
                attack_range: 8.0,
                attack_speed: 1.8,
                last_attack_time: 0.0,
                target: None,
                attack_type: AttackType::Melee,
                attack_cooldown: 0.0,
                is_attacking: false,
                auto_attack: true,
            },
            Selectable::default(),
            Vision::default(),
            CollisionRadius { radius: crate::constants::collision::BEETLE_KNIGHT_COLLISION_RADIUS },
            EntityState::default(),
            GameEntity,
        )).id()
    }

    pub fn spawn_dragonfly(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
    ) -> Entity {
        Self::spawn_dragonfly_with_models(commands, meshes, materials, position, player_id, unit_id, None)
    }

    pub fn spawn_dragonfly_with_models(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
        model_assets: Option<&ModelAssets>,
    ) -> Entity {
        // Calculate model scale factor for movement adjustment
        let model_scale = if let Some(_models) = model_assets {
            let model_type = get_unit_insect_model(&UnitType::DragonFly);
            match model_type {
                crate::rendering::model_loader::InsectModelType::DragonFly => crate::rendering::model_loader::get_model_scale(&model_type),
                _ => crate::rendering::model_loader::get_model_scale(&model_type),
            }
        } else {
            1.0 // Primitive models use default scale
        };

        let mut entity = if let Some(models) = model_assets {
            // Use GLB model
            let model_type = get_unit_insect_model(&UnitType::DragonFly);
            let model_handle = models.get_model_handle(&model_type);

            commands.spawn((
                SceneRoot(model_handle),
                Transform::from_translation(position)
                    .with_scale(Vec3::splat(model_scale))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                InsectModel {
                    model_type,
                    scale: model_scale,
                },
                UseGLBModel,
                TeamColor::new(player_id), // Add team coloring
            ))
        } else {
            // Fallback to primitive shapes - elongated for dragonfly
            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(0.6, 3.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: TeamColor::get_primitive_color(player_id),
                    ..default()
                })),
                Transform::from_translation(position),
            ))
        };

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(UnitType::DragonFly) },
            TeamColor::new(player_id), // Add team coloring
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 250.0 / model_scale.max(1.0), // Extremely fast flying scout
                acceleration: 400.0 / model_scale.max(1.0),  // Very quick acceleration
                turning_speed: 6.5,  // Excellent maneuverability
                ..default()
            },
            RTSHealth {
                current: 50.0,
                max: 50.0,
                armor: 0.0,
                regeneration_rate: 0.3,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: 10.0,
                attack_range: 12.0, // Ranged attack
                attack_speed: 2.5,
                last_attack_time: 0.0,
                target: None,
                attack_type: AttackType::Ranged,
                attack_cooldown: 0.0,
                is_attacking: false,
                auto_attack: true,
            },
            Selectable::default(),
            Vision {
                sight_range: 200.0, // Excellent vision for reconnaissance
                line_of_sight: true,
            },
            CollisionRadius { radius: crate::constants::collision::DEFAULT_UNIT_COLLISION_RADIUS * 0.8 },
            EntityState::default(),
            GameEntity,
        )).id()
    }

    pub fn spawn_queen_chamber(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        model_assets: Option<&crate::rendering::model_loader::ModelAssets>,
    ) -> Entity {
        let mut entity_commands = if let Some(models) = model_assets {
            if models.models_loaded && models.anthill != Handle::default() {
                // Use GLB model - anthill raised 5 units above terrain to sit on top
                commands.spawn((
                    SceneRoot(models.anthill.clone()),
                    Transform::from_translation(position + Vec3::new(0.0, 5.0, 0.0))
                        .with_scale(Vec3::splat(10.0)), // Large main building
                    UseGLBModel,
                ))
            } else {
                // Fallback to primitive
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(20.0, 15.0, 20.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: TeamColor::get_primitive_color(player_id),
                        ..default()
                    })),
                    Transform::from_translation(position + Vec3::new(0.0, 7.5, 0.0)),
                ))
            }
        } else {
            // No models available, use primitive
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(20.0, 15.0, 20.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: TeamColor::get_primitive_color(player_id),
                    ..default()
                })),
                Transform::from_translation(position + Vec3::new(0.0, 7.5, 0.0)),
            ))
        };
        
        entity_commands.insert((
            RTSUnit { unit_id: 0, player_id, size: 4.0, unit_type: None },
            TeamColor::new(player_id), // Add team coloring
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Building {
                building_type: BuildingType::Queen,
                construction_progress: 100.0,
                max_construction: 100.0,
                is_complete: true,
                rally_point: Some(position + Vec3::new(30.0, 0.0, 0.0)),
            },
            RTSHealth {
                current: 600.0,
                max: 600.0,
                armor: 5.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            ProductionQueue {
                queue: Vec::new(),
                current_progress: 0.0,
                production_time: 25.0,
            },
            Garrison {
                capacity: 15,
                garrisoned_units: Vec::new(),
                protection_bonus: 2.0,
            },
            Selectable {
                is_selected: false,
                selection_radius: 10.0,
            },
            Vision {
                sight_range: 200.0,
                line_of_sight: true,
            },
            CollisionRadius { radius: crate::constants::collision::QUEEN_COLLISION_RADIUS },
            GameEntity,
        )).id()
    }

    pub fn spawn_nursery(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        model_assets: Option<&crate::rendering::model_loader::ModelAssets>,
    ) -> Entity {
        let mut entity_commands = if let Some(models) = model_assets {
            if models.models_loaded && models.hive != Handle::default() {
                // Use GLB model - hive placed on terrain surface
                commands.spawn((
                    SceneRoot(models.hive.clone()),
                    Transform::from_translation(position + Vec3::new(0.0, 3.0, 0.0))
                        .with_scale(Vec3::splat(0.05)), // Medium-sized building
                    UseGLBModel,
                ))
            } else {
                // Fallback to primitive
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(8.0, 6.0, 8.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: TeamColor::get_primitive_color(player_id),
                        ..default()
                    })),
                    Transform::from_translation(position + Vec3::new(0.0, 3.0, 0.0)),
                ))
            }
        } else {
            // No models available, use primitive
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(8.0, 6.0, 8.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: TeamColor::get_primitive_color(player_id),
                    ..default()
                })),
                Transform::from_translation(position + Vec3::new(0.0, 3.0, 0.0)),
            ))
        };
        
        entity_commands.insert((
            RTSUnit { unit_id: 0, player_id, size: 2.0, unit_type: None },
            TeamColor::new(player_id), // Add team coloring
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Building {
                building_type: BuildingType::Nursery,
                construction_progress: 100.0,
                max_construction: 100.0,
                is_complete: true,
                rally_point: None,
            },
            RTSHealth {
                current: 75.0,
                max: 75.0,
                armor: 0.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            Selectable {
                is_selected: false,
                selection_radius: 5.0,
            },
            CollisionRadius { radius: crate::constants::collision::NURSERY_COLLISION_RADIUS },
            GameEntity,
        )).id()
    }

    pub fn spawn_warrior_chamber(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        model_assets: Option<&crate::rendering::model_loader::ModelAssets>,
    ) -> Entity {
        let mut entity_commands = if let Some(models) = model_assets {
            if models.models_loaded && models.stick_shelter != Handle::default() {
                // Use GLB model - stick shelter placed on terrain surface
                commands.spawn((
                    SceneRoot(models.stick_shelter.clone()),
                    Transform::from_translation(position + Vec3::new(0.0, 2.0, 0.0))
                        .with_scale(Vec3::splat(0.08)), // Medium building size
                    UseGLBModel,
                ))
            } else {
                // Fallback to primitive
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(12.0, 8.0, 12.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: TeamColor::get_primitive_color(player_id),
                        ..default()
                    })),
                    Transform::from_translation(position + Vec3::new(0.0, 4.0, 0.0)),
                ))
            }
        } else {
            // No models available, use primitive
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(12.0, 8.0, 12.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: TeamColor::get_primitive_color(player_id),
                    ..default()
                })),
                Transform::from_translation(position + Vec3::new(0.0, 4.0, 0.0)),
            ))
        };
        
        entity_commands.insert((
            RTSUnit { unit_id: 0, player_id, size: 3.0, unit_type: None },
            TeamColor::new(player_id), // Add team coloring
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Building {
                building_type: BuildingType::WarriorChamber,
                construction_progress: 100.0,
                max_construction: 100.0,
                is_complete: true,
                rally_point: Some(position + Vec3::new(20.0, 0.0, 0.0)),
            },
            RTSHealth {
                current: 200.0,
                max: 200.0,
                armor: 2.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            ProductionQueue {
                queue: Vec::new(),
                current_progress: 0.0,
                production_time: 21.0,
            },
            Selectable {
                is_selected: false,
                selection_radius: 8.0,
            },
            CollisionRadius { radius: crate::constants::collision::WARRIOR_CHAMBER_COLLISION_RADIUS },
            GameEntity,
        )).id()
    }

    // Note: Old primitive resource spawn functions removed
    // Resources are now provided by environment objects (mushrooms, rocks) with ResourceSource components
    // This provides better visual integration and uses GLB models instead of primitive cubes/spheres
}