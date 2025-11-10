use bevy::prelude::*;
use crate::core::components::*;
use crate::rendering::model_loader::*;

pub struct RTSEntityFactory;

impl RTSEntityFactory {
    // Helper function to calculate model scale
    fn calculate_model_scale(unit_type: &UnitType, model_assets: Option<&ModelAssets>) -> f32 {
        if let Some(_models) = model_assets {
            let model_type = get_unit_insect_model(unit_type);
            match model_type {
                crate::rendering::model_loader::InsectModelType::QueenFacedBug => {
                    match unit_type {
                        UnitType::SpearMantis => crate::constants::models::QUEEN_FACED_BUG_SCALE,
                        _ => crate::constants::models::MANTIS_SCALE,
                    }
                }
                crate::rendering::model_loader::InsectModelType::ApisMellifera => {
                    crate::constants::models::APIS_MELLIFERA_SCALE
                }
                crate::rendering::model_loader::InsectModelType::CairnsBirdwing => {
                    crate::constants::models::CAIRNS_BIRDWING_SCALE
                }
                _ => crate::rendering::model_loader::get_model_scale(&model_type),
            }
        } else {
            1.0 // Primitive models use default scale
        }
    }

    // Helper function to spawn unit visual representation (GLB model or primitive)
    fn spawn_unit_visual<'a>(
        commands: &'a mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        unit_type: &UnitType,
        model_scale: f32,
        model_assets: Option<&ModelAssets>,
        primitive_mesh: Mesh,
    ) -> EntityCommands<'a> {
        if let Some(models) = model_assets {
            // Use GLB model
            let model_type = get_unit_insect_model(unit_type);
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
                TeamColor::new(player_id),
            ))
        } else {
            // Fallback to primitive shapes
            commands.spawn((
                Mesh3d(meshes.add(primitive_mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: TeamColor::get_primitive_color(player_id),
                    ..default()
                })),
                Transform::from_translation(position),
            ))
        }
    }

    // Helper function to spawn building visual representation (GLB model or primitive)
    fn spawn_building_visual<'a>(
        commands: &'a mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
        player_id: u8,
        model_handle: Option<Handle<Scene>>,
        model_scale: Vec3,
        primitive_size: (f32, f32, f32),
        position_offset: Vec3,
    ) -> EntityCommands<'a> {
        if let Some(model) = model_handle {
            // Use GLB model
            commands.spawn((
                SceneRoot(model),
                Transform::from_translation(position + position_offset).with_scale(model_scale),
                UseGLBModel,
            ))
        } else {
            // Fallback to primitive shapes
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(primitive_size.0, primitive_size.1, primitive_size.2))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: TeamColor::get_primitive_color(player_id),
                    ..default()
                })),
                Transform::from_translation(position + position_offset),
            ))
        }
    }

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
        let unit_type = UnitType::WorkerAnt;
        let model_scale = Self::calculate_model_scale(&unit_type, model_assets);

        let mut entity = Self::spawn_unit_visual(
            commands,
            meshes,
            materials,
            position,
            player_id,
            &unit_type,
            model_scale,
            model_assets,
            Capsule3d::new(1.0, 2.0).into(),
        );

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(unit_type) },
            TeamColor::new(player_id),
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 50.0 / model_scale,
                acceleration: 30.0 / model_scale,
                turning_speed: 3.0,
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
        let unit_type = UnitType::SoldierAnt;
        let model_scale = Self::calculate_model_scale(&unit_type, model_assets);

        let mut entity = Self::spawn_unit_visual(
            commands,
            meshes,
            materials,
            position,
            player_id,
            &unit_type,
            model_scale,
            model_assets,
            Capsule3d::new(1.2, 2.2).into(),
        );

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(unit_type) },
            TeamColor::new(player_id),
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 60.0 / model_scale,
                acceleration: 40.0 / model_scale,
                turning_speed: 3.5,
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
        let unit_type = UnitType::HunterWasp;
        let model_scale = 1.0;

        let mut entity = Self::spawn_unit_visual(
            commands,
            meshes,
            materials,
            position,
            player_id,
            &unit_type,
            model_scale,
            None,
            Capsule3d::new(1.0, 2.0).into(),
        );

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(unit_type) },
            TeamColor::new(player_id),
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 60.0,
                acceleration: 120.0,
                turning_speed: 3.5,
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
        let unit_type = UnitType::SpearMantis;
        let model_scale = Self::calculate_model_scale(&unit_type, model_assets);

        let mut entity = Self::spawn_unit_visual(
            commands,
            meshes,
            materials,
            position,
            player_id,
            &unit_type,
            model_scale,
            model_assets,
            Capsule3d::new(1.2, 2.5).into(),
        );

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(unit_type) },
            TeamColor::new(player_id),
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 100.0 / model_scale.max(1.5),
                acceleration: 200.0 / model_scale.max(1.5),
                turning_speed: 4.5,
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
        let unit_type = UnitType::ScoutAnt;
        let model_scale = Self::calculate_model_scale(&unit_type, model_assets);

        let mut entity = Self::spawn_unit_visual(
            commands,
            meshes,
            materials,
            position,
            player_id,
            &unit_type,
            model_scale,
            model_assets,
            Capsule3d::new(0.9, 2.0).into(),
        );

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(unit_type) },
            TeamColor::new(player_id),
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 80.0 / model_scale.max(2.0),
                acceleration: 140.0 / model_scale.max(2.0),
                turning_speed: 4.2,
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
        let unit_type = UnitType::BeetleKnight;
        let model_scale = 1.0;

        let mut entity = Self::spawn_unit_visual(
            commands,
            meshes,
            materials,
            position,
            player_id,
            &unit_type,
            model_scale,
            None,
            Capsule3d::new(1.5, 2.5).into(),
        );

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.5, unit_type: Some(unit_type) },
            TeamColor::new(player_id),
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 70.0,
                acceleration: 140.0,
                turning_speed: 4.0,
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
        let unit_type = UnitType::DragonFly;
        let model_scale = Self::calculate_model_scale(&unit_type, model_assets);

        let mut entity = Self::spawn_unit_visual(
            commands,
            meshes,
            materials,
            position,
            player_id,
            &unit_type,
            model_scale,
            model_assets,
            Capsule3d::new(0.6, 3.0).into(),
        );

        entity.insert((
            RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(unit_type) },
            TeamColor::new(player_id),
            Position {
                translation: position,
                rotation: Quat::IDENTITY,
            },
            Movement {
                max_speed: 250.0 / model_scale.max(1.0),
                acceleration: 400.0 / model_scale.max(1.0),
                turning_speed: 6.5,
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
                attack_range: 12.0,
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
                sight_range: 200.0,
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
        let model_handle = model_assets
            .filter(|m| m.models_loaded && m.anthill != Handle::default())
            .map(|m| m.anthill.clone());

        let mut entity_commands = Self::spawn_building_visual(
            commands,
            meshes,
            materials,
            position,
            player_id,
            model_handle,
            Vec3::splat(10.0),
            (20.0, 15.0, 20.0),
            Vec3::new(0.0, if model_handle.is_some() { 5.0 } else { 7.5 }, 0.0),
        );

        entity_commands.insert((
            RTSUnit { unit_id: 0, player_id, size: 4.0, unit_type: None },
            TeamColor::new(player_id),
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
        let model_handle = model_assets
            .filter(|m| m.models_loaded && m.hive != Handle::default())
            .map(|m| m.hive.clone());

        let mut entity_commands = Self::spawn_building_visual(
            commands,
            meshes,
            materials,
            position,
            player_id,
            model_handle,
            Vec3::splat(0.05),
            (8.0, 6.0, 8.0),
            Vec3::new(0.0, 3.0, 0.0),
        );

        entity_commands.insert((
            RTSUnit { unit_id: 0, player_id, size: 2.0, unit_type: None },
            TeamColor::new(player_id),
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
        let model_handle = model_assets
            .filter(|m| m.models_loaded && m.pine_cone != Handle::default())
            .map(|m| m.pine_cone.clone());

        let mut entity_commands = Self::spawn_building_visual(
            commands,
            meshes,
            materials,
            position,
            player_id,
            model_handle,
            Vec3::splat(7.0),
            (12.0, 8.0, 12.0),
            if model_handle.is_some() {
                Vec3::new(0.0, 2.0, 10.0)
            } else {
                Vec3::new(0.0, 4.0, 0.0)
            },
        );

        entity_commands.insert((
            RTSUnit { unit_id: 0, player_id, size: 3.0, unit_type: None },
            TeamColor::new(player_id),
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