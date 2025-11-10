use bevy::prelude::*;
use crate::core::components::*;
use crate::ui::resource_display::PlayerResources;

#[derive(Resource, Default)]
pub struct BuildingPlacement {
    pub active_building: Option<BuildingType>,
    pub placement_preview: Option<Entity>,
    #[allow(dead_code)]
    pub is_valid_placement: bool,
}

#[derive(Component)]
pub struct PlacementPreview;

#[derive(Component)]
pub struct PlacementStatusText;

// Removed PlacementContext to avoid lifetime issues - using direct parameters instead

pub fn handle_building_placement(
    mut placement: ResMut<BuildingPlacement>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_manager: Res<crate::world::terrain_v2::TerrainChunkManager>,
    terrain_settings: Res<crate::world::terrain_v2::TerrainSettings>,
    mut player_resources: ResMut<PlayerResources>,
    mut main_resources: ResMut<crate::core::resources::PlayerResources>,
    model_assets: Option<Res<crate::rendering::model_loader::ModelAssets>>,
    mut preview_transforms: Query<&mut Transform, With<PlacementPreview>>,
) {
    // Cancel building placement with ESC
    if keyboard.just_pressed(crate::constants::hotkeys::CANCEL_BUILD) {
        cancel_placement(&mut placement, &mut commands);
        return;
    }

    if let Some(building_type) = placement.active_building.clone() {
        handle_active_placement(
            &mut placement,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut player_resources,
            &mut main_resources,
            building_type,
            &windows,
            &camera_q,
            &terrain_manager,
            &terrain_settings,
            &mouse_button,
            &model_assets,
            &mut preview_transforms,
        );
    }
}

fn cancel_placement(placement: &mut BuildingPlacement, commands: &mut Commands) {
    if placement.active_building.is_some() {
        placement.active_building = None;
        if let Some(preview_entity) = placement.placement_preview.take() {
            commands.entity(preview_entity).despawn();
        }
    }
}

fn handle_active_placement(
    placement: &mut ResMut<BuildingPlacement>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_resources: &mut ResMut<PlayerResources>,
    main_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    building_type: BuildingType,
    windows: &Query<&Window>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
    terrain_manager: &crate::world::terrain_v2::TerrainChunkManager,
    terrain_settings: &crate::world::terrain_v2::TerrainSettings,
    mouse_button: &ButtonInput<MouseButton>,
    model_assets: &Option<Res<crate::rendering::model_loader::ModelAssets>>,
    preview_transforms: &mut Query<&mut Transform, With<PlacementPreview>>,
) {
    let window = windows.single();
    if let Some(cursor_position) = window.cursor_position() {
        let (camera, camera_transform) = camera_q.single();

        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            let placement_pos = calculate_placement_position(ray, terrain_manager, terrain_settings);
            
            update_placement_preview(placement, commands, meshes, materials, building_type.clone(), placement_pos, preview_transforms);
            
            if mouse_button.just_pressed(MouseButton::Left) {
                info!("Attempting to place building at: {:?}", placement_pos);
                attempt_building_placement(placement, commands, meshes, materials, player_resources, main_resources, building_type, placement_pos, model_assets);
            }
        }
    }
}

fn calculate_placement_position(
    ray: Ray3d,
    terrain_manager: &crate::world::terrain_v2::TerrainChunkManager,
    terrain_settings: &crate::world::terrain_v2::TerrainSettings,
) -> Vec3 {
    // Method 1: Try ray-plane intersection with y=0 plane first (most common case)
    if ray.direction.y.abs() > 0.001 {
        let t = -ray.origin.y / ray.direction.y;
        if t > 0.0 {
            let intersection = ray.origin + ray.direction * t;
            let terrain_height = crate::world::terrain_v2::sample_terrain_height(
                intersection.x,
                intersection.z,
                &terrain_manager.noise_generator,
                terrain_settings,
            );
            return Vec3::new(intersection.x, terrain_height, intersection.z);
        }
    }
    
    // Method 2: If looking too horizontal, project forward and down to terrain
    let forward_distance = 100.0;
    let projected_point = ray.origin + ray.direction * forward_distance;
    
    // Sample terrain at the projected point
    let terrain_height = crate::world::terrain_v2::sample_terrain_height(
        projected_point.x,
        projected_point.z,
        &terrain_manager.noise_generator,
        terrain_settings,
    );
    
    Vec3::new(projected_point.x, terrain_height, projected_point.z)
}

fn update_placement_preview(
    placement: &mut ResMut<BuildingPlacement>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    building_type: BuildingType,
    placement_pos: Vec3,
    preview_transforms: &mut Query<&mut Transform, With<PlacementPreview>>,
) {
    if placement.placement_preview.is_none() {
        let preview_entity = create_building_preview(
            commands,
            meshes,
            materials,
            building_type,
            placement_pos,
        );
        placement.placement_preview = Some(preview_entity);
    } else if let Some(preview_entity) = placement.placement_preview {
        if let Ok(mut transform) = preview_transforms.get_mut(preview_entity) {
            transform.translation = placement_pos;
        }
    }
}

fn attempt_building_placement(
    placement: &mut ResMut<BuildingPlacement>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_resources: &mut ResMut<PlayerResources>,
    main_resources: &mut ResMut<crate::core::resources::PlayerResources>,
    building_type: BuildingType,
    placement_pos: Vec3,
    model_assets: &Option<Res<crate::rendering::model_loader::ModelAssets>>,
) {
    let building_cost = get_building_cost(&building_type);
    if can_afford_building(&building_cost, player_resources) {
        deduct_building_cost(&building_cost, player_resources, main_resources);
        place_building(commands, meshes, materials, building_type.clone(), placement_pos, model_assets);
        
        // Clear placement
        placement.active_building = None;
        if let Some(preview_entity) = placement.placement_preview.take() {
            commands.entity(preview_entity).despawn();
        }

        info!("Placed building: {:?} at {:?}", building_type, placement_pos);
    }
}

fn deduct_building_cost(
    cost: &[(ResourceType, f32)],
    player_resources: &mut PlayerResources,
    main_resources: &mut crate::core::resources::PlayerResources,
) {
    for (resource_type, cost_amount) in cost {
        match resource_type {
            ResourceType::Nectar => {
                player_resources.nectar -= cost_amount;
                main_resources.nectar -= cost_amount;
            },
            ResourceType::Chitin => {
                player_resources.chitin -= cost_amount;
                main_resources.chitin -= cost_amount;
            },
            ResourceType::Minerals => {
                player_resources.minerals -= cost_amount;
                main_resources.minerals -= cost_amount;
            },
            ResourceType::Pheromones => {
                player_resources.pheromones -= cost_amount;
                main_resources.pheromones -= cost_amount;
            },
        }
    }
}

pub fn create_building_preview(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    building_type: BuildingType,
    position: Vec3,
) -> Entity {
    use crate::constants::buildings::*;
    let (mesh, size) = match building_type {
        BuildingType::Nursery => (meshes.add(Cuboid::from_size(NURSERY_SIZE)), NURSERY_SIZE),
        BuildingType::WarriorChamber => (meshes.add(Cuboid::from_size(WARRIOR_CHAMBER_SIZE)), WARRIOR_CHAMBER_SIZE),
        BuildingType::HunterChamber => (meshes.add(Cuboid::from_size(HUNTER_CHAMBER_SIZE)), HUNTER_CHAMBER_SIZE),
        BuildingType::FungalGarden => (meshes.add(Cuboid::from_size(FUNGAL_GARDEN_SIZE)), FUNGAL_GARDEN_SIZE),
        _ => (meshes.add(Cuboid::from_size(DEFAULT_BUILDING_SIZE)), DEFAULT_BUILDING_SIZE),
    };

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: PREVIEW_COLOR,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(position + Vec3::new(0.0, size.y / 2.0, 0.0)),
        PlacementPreview,
    )).id()
}

fn place_building(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    building_type: BuildingType,
    position: Vec3,
    model_assets: &Option<Res<crate::rendering::model_loader::ModelAssets>>,
) {
    use crate::entities::rts_entities::RTSEntityFactory;
    
    let model_assets_ref = model_assets.as_ref().map(|r| &**r);
    
    let _building_entity = match building_type {
        BuildingType::Queen => RTSEntityFactory::spawn_queen_chamber(commands, meshes, materials, position, 1, model_assets_ref),
        BuildingType::Nursery => RTSEntityFactory::spawn_nursery(commands, meshes, materials, position, 1, model_assets_ref),
        BuildingType::WarriorChamber => RTSEntityFactory::spawn_warrior_chamber(commands, meshes, materials, position, 1, model_assets_ref),
        _ => create_fallback_building(commands, meshes, materials, building_type, position),
    };
}

fn create_fallback_building(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    building_type: BuildingType,
    position: Vec3,
) -> Entity {
    use crate::constants::buildings::*;
    let (mesh, material, size) = (
        meshes.add(Cuboid::from_size(DEFAULT_BUILDING_SIZE)),
        materials.add(StandardMaterial {
            base_color: DEFAULT_BUILDING_COLOR,
            ..default()
        }),
        DEFAULT_BUILDING_SIZE,
    );
    
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(position + Vec3::new(0.0, size.y / 2.0, 0.0)),
        Building {
            building_type: building_type.clone(),
            construction_progress: CONSTRUCTION_PROGRESS_MAX,
            max_construction: CONSTRUCTION_PROGRESS_MAX,
            is_complete: true,
            rally_point: None,
        },
        crate::core::components::RTSHealth {
            current: DEFAULT_BUILDING_HEALTH,
            max: DEFAULT_BUILDING_HEALTH,
            armor: DEFAULT_BUILDING_ARMOR,
            regeneration_rate: 0.0,
            last_damage_time: 0.0,
        },
        crate::core::components::Selectable::default(),
        crate::core::components::GameEntity,
    )).id()
}

// Helper functions
pub fn can_afford_building(cost: &[(ResourceType, f32)], resources: &PlayerResources) -> bool {
    for (resource_type, amount) in cost {
        let available = match resource_type {
            ResourceType::Nectar => resources.nectar,
            ResourceType::Chitin => resources.chitin,
            ResourceType::Minerals => resources.minerals,
            ResourceType::Pheromones => resources.pheromones,
        };
        if available < *amount {
            return false;
        }
    }
    true
}

pub fn get_building_cost(building_type: &BuildingType) -> Vec<(ResourceType, f32)> {
    use crate::constants::resources::*;
    match building_type {
        BuildingType::Nursery => vec![(ResourceType::Chitin, NURSERY_CHITIN_COST)],
        BuildingType::WarriorChamber => vec![(ResourceType::Chitin, WARRIOR_CHAMBER_CHITIN_COST), (ResourceType::Minerals, WARRIOR_CHAMBER_MINERALS_COST)],
        BuildingType::HunterChamber => vec![(ResourceType::Chitin, HUNTER_CHAMBER_CHITIN_COST)],
        BuildingType::FungalGarden => vec![(ResourceType::Chitin, FUNGAL_GARDEN_CHITIN_COST)],
        BuildingType::WoodProcessor => vec![(ResourceType::Chitin, WOOD_PROCESSOR_CHITIN_COST)],
        BuildingType::MineralProcessor => vec![(ResourceType::Chitin, MINERAL_PROCESSOR_CHITIN_COST)],
        _ => vec![(ResourceType::Chitin, DEFAULT_BUILDING_CHITIN_COST)],
    }
}

pub fn update_placement_status(
    placement: Res<BuildingPlacement>,
    mut status_query: Query<&mut Text, With<PlacementStatusText>>,
) {
    if let Ok(mut text) = status_query.get_single_mut() {
        match &placement.active_building {
            Some(building_type) => {
                **text = format!("Placing {:?} - Click terrain or ESC to cancel", building_type);
            }
            None => {
                **text = "".to_string();
            }
        }
    }
}