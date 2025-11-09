use bevy::prelude::*;
use crate::components::*;
use crate::model_loader::*;

pub struct RTSEntityFactory;

impl RTSEntityFactory {
    pub fn spawn_villager(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
    ) -> Entity {
        Self::spawn_villager_with_models(commands, meshes, materials, position, player_id, unit_id, None)
    }

    pub fn spawn_villager_with_models(
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
                crate::model_loader::InsectModelType::QueenFacedBug => crate::constants::models::MANTIS_SCALE,
                crate::model_loader::InsectModelType::ApisMellifera => crate::constants::models::APIS_MELLIFERA_SCALE,
                _ => crate::model_loader::get_model_scale(&model_type),
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
            ))
        } else {
            // Fallback to primitive shapes
            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(1.0, 2.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: if player_id == 1 { 
                        Color::srgb(0.4, 0.6, 0.8) 
                    } else { 
                        Color::srgb(0.8, 0.4, 0.2) 
                    },
                    ..default()
                })),
                Transform::from_translation(position),
            ))
        };

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(UnitType::WorkerAnt) },
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
            max_speed: 25.0 / model_scale,
            acceleration: 15.0 / model_scale,
            turning_speed: 2.0,  // Turning speed doesn't need scaling
            ..default()
        },
            RTSHealth {
                current: 25.0,
                max: 25.0,
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

    pub fn spawn_militia(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_id: u32,
    ) -> Entity {
        Self::spawn_militia_with_models(commands, meshes, materials, position, player_id, unit_id, None)
    }

    pub fn spawn_militia_with_models(
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
                crate::model_loader::InsectModelType::QueenFacedBug => crate::constants::models::MANTIS_SCALE,
                crate::model_loader::InsectModelType::ApisMellifera => crate::constants::models::APIS_MELLIFERA_SCALE,
                _ => crate::model_loader::get_model_scale(&model_type),
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
            ))
        } else {
            // Fallback to primitive shapes
            commands.spawn((
                Mesh3d(meshes.add(Capsule3d::new(1.2, 2.2))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: if player_id == 1 { 
                        Color::srgb(0.8, 0.2, 0.2) 
                    } else { 
                        Color::srgb(0.2, 0.8, 0.2) 
                    },
                    ..default()
                })),
                Transform::from_translation(position),
            ))
        };

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(UnitType::SoldierAnt) },
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 30.0 / model_scale,
                acceleration: 20.0 / model_scale,
                turning_speed: 2.5,  // Turning speed doesn't need scaling
                ..default()
            },
            RTSHealth {
                current: 40.0,
                max: 40.0,
                armor: 1.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: 4.0,
                attack_range: 10.0,
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

    pub fn spawn_archer(
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
                base_color: Color::srgb(0.2, 0.8, 0.2),
                ..default()
            })),
            Transform::from_translation(position),
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(UnitType::HunterWasp) },
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 30.0,
                acceleration: 60.0,
                turning_speed: 2.5,
                ..default()
            },
            RTSHealth {
                current: 30.0,
                max: 30.0,
                armor: 0.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: 6.0,
                attack_range: 100.0,
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

    pub fn spawn_knight(
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
                base_color: Color::srgb(0.7, 0.7, 0.1),
                ..default()
            })),
            Transform::from_translation(position),
            RTSUnit { unit_id, player_id, size: 1.5, unit_type: Some(UnitType::BeetleKnight) },
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 35.0,
                acceleration: 70.0,
                turning_speed: 3.0,
                ..default()
            },
            RTSHealth {
                current: 100.0,
                max: 100.0,
                armor: 3.0,
                regeneration_rate: 0.0,
                last_damage_time: 0.0,
            },
            Combat {
                attack_damage: 12.0,
                attack_range: 15.0,
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

    pub fn spawn_town_center(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
    ) -> Entity {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(20.0, 15.0, 20.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.4, 0.2),
                ..default()
            })),
            Transform::from_translation(position),
            RTSUnit { unit_id: 0, player_id, size: 4.0, unit_type: None },
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

    pub fn spawn_house(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
    ) -> Entity {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(8.0, 6.0, 8.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.6, 0.4),
                ..default()
            })),
            Transform::from_translation(position),
            RTSUnit { unit_id: 0, player_id, size: 2.0, unit_type: None },
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

    pub fn spawn_barracks(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
    ) -> Entity {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(12.0, 8.0, 12.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.3, 0.1),
                ..default()
            })),
            Transform::from_translation(position),
            RTSUnit { unit_id: 0, player_id, size: 3.0, unit_type: None },
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

    pub fn spawn_wood_resource(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Entity {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(2.0, 8.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.2, 0.0),
                ..default()
            })),
            Transform::from_translation(position),
            ResourceSource {
                resource_type: ResourceType::Chitin,
                amount: 100.0,
                max_gatherers: 3,
                current_gatherers: 0,
            },
            Selectable {
                is_selected: false,
                selection_radius: 3.0,
            },
            CollisionRadius { radius: 5.0 }, // Wood resource collision radius
            GameEntity,
        )).id()
    }

    pub fn spawn_stone_resource(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Entity {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(3.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.5, 0.5),
                ..default()
            })),
            Transform::from_translation(position),
            ResourceSource {
                resource_type: ResourceType::Minerals,
                amount: 200.0,
                max_gatherers: 5,
                current_gatherers: 0,
            },
            Selectable {
                is_selected: false,
                selection_radius: 4.0,
            },
            CollisionRadius { radius: 6.0 }, // Stone resource collision radius
            GameEntity,
        )).id()
    }

    pub fn spawn_gold_resource(
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Entity {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(2.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.8, 0.0),
                ..default()
            })),
            Transform::from_translation(position),
            ResourceSource {
                resource_type: ResourceType::Pheromones,
                amount: 800.0,
                max_gatherers: 4,
                current_gatherers: 0,
            },
            Selectable {
                is_selected: false,
                selection_radius: 3.0,
            },
            CollisionRadius { radius: 5.5 }, // Gold resource collision radius
            GameEntity,
        )).id()
    }
}