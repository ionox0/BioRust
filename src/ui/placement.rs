use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use crate::core::components::*;
use crate::ui::resource_display::PlayerResources;
use crate::collision::validate_building_placement;

#[derive(Resource, Default)]
pub struct BuildingPlacement {
    pub active_building: Option<BuildingType>,
    pub placement_preview: Option<Entity>,
    pub is_valid_placement: bool,
}

#[derive(Component)]
pub struct PlacementPreview;

#[derive(Component)]
pub struct PlacementStatusText;

/// System parameter grouping collision-related queries to reduce parameter count
#[derive(SystemParam)]
pub struct CollisionQueries<'w, 's> {
    pub buildings: Query<'w, 's, (&'static Transform, &'static CollisionRadius), (With<Building>, Without<PlacementPreview>)>,
    pub units: Query<'w, 's, (&'static Transform, &'static CollisionRadius), (With<RTSUnit>, Without<PlacementPreview>)>,
    pub environment_objects: Query<'w, 's, (&'static Transform, &'static CollisionRadius, &'static EnvironmentObject), (With<EnvironmentObject>, Without<PlacementPreview>)>,
}

/// System parameter grouping preview-related queries to reduce parameter count
#[derive(SystemParam)]
pub struct PreviewQueries<'w, 's> {
    pub transforms: Query<'w, 's, &'static mut Transform, With<PlacementPreview>>,
    pub materials: Query<'w, 's, &'static MeshMaterial3d<StandardMaterial>, With<PlacementPreview>>,
}

/// System parameter grouping rendering resources to reduce parameter count
#[derive(SystemParam)]
pub struct RenderingResources<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub model_assets: Option<Res<'w, crate::rendering::model_loader::ModelAssets>>,
}

/// System parameter grouping terrain resources to reduce parameter count
#[derive(SystemParam)]
pub struct TerrainResources<'w> {
    pub manager: Res<'w, crate::world::terrain_v2::TerrainChunkManager>,
    pub settings: Res<'w, crate::world::terrain_v2::TerrainSettings>,
}

/// System parameter grouping player resources to reduce parameter count
#[derive(SystemParam)]
pub struct PlayerResourcesMut<'w> {
    pub ui_resources: ResMut<'w, PlayerResources>,
    pub main_resources: ResMut<'w, crate::core::resources::PlayerResources>,
}

pub fn handle_building_placement(
    mut placement: ResMut<BuildingPlacement>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut rendering_resources: RenderingResources,
    terrain_resources: TerrainResources,
    mut player_resources: PlayerResourcesMut,
    collision_queries: CollisionQueries,
    mut preview_queries: PreviewQueries,
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
            &mut rendering_resources,
            &mut player_resources,
            building_type,
            &windows,
            &camera_q,
            &terrain_resources,
            &mouse_button,
            &collision_queries,
            &mut preview_queries,
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
    rendering_resources: &mut RenderingResources,
    player_resources: &mut PlayerResourcesMut,
    building_type: BuildingType,
    windows: &Query<&Window>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
    terrain_resources: &TerrainResources,
    mouse_button: &ButtonInput<MouseButton>,
    collision_queries: &CollisionQueries,
    preview_queries: &mut PreviewQueries,
) {
    let window = windows.single();
    if let Some(cursor_position) = window.cursor_position() {
        let (camera, camera_transform) = camera_q.single();

        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            let placement_pos = calculate_placement_position(ray, &terrain_resources.manager, &terrain_resources.settings);

            // Get collision radius for this building type
            let building_radius = get_building_collision_radius(&building_type);

            // Validate placement position
            let is_valid = validate_building_placement(
                placement_pos,
                building_radius,
                &collision_queries.buildings,
                &collision_queries.units,
                &collision_queries.environment_objects,
            );

            // Update validation state
            placement.is_valid_placement = is_valid;

            update_placement_preview(
                placement,
                commands,
                &mut rendering_resources.meshes,
                &mut rendering_resources.materials,
                building_type.clone(),
                placement_pos,
                &mut preview_queries.transforms,
                is_valid,
                &mut preview_queries.materials,
            );

            if mouse_button.just_pressed(MouseButton::Left) {
                info!("Attempting to place building at: {:?}", placement_pos);
                attempt_building_placement(
                    placement,
                    commands,
                    &mut rendering_resources.meshes,
                    &mut rendering_resources.materials,
                    &mut player_resources.ui_resources,
                    &mut player_resources.main_resources,
                    building_type,
                    placement_pos,
                    &rendering_resources.model_assets,
                    is_valid,
                );
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
    is_valid: bool,
    preview_materials: &mut Query<&MeshMaterial3d<StandardMaterial>, With<PlacementPreview>>,
) {
    if placement.placement_preview.is_none() {
        let preview_entity = create_building_preview(
            commands,
            meshes,
            materials,
            building_type,
            placement_pos,
            is_valid,
        );
        placement.placement_preview = Some(preview_entity);
    } else if let Some(preview_entity) = placement.placement_preview {
        // Update position
        if let Ok(mut transform) = preview_transforms.get_mut(preview_entity) {
            transform.translation = placement_pos;
        }

        // Update material color based on validity
        if let Ok(material_handle) = preview_materials.get_mut(preview_entity) {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.base_color = if is_valid {
                    crate::constants::building_placement::VALID_PLACEMENT_COLOR
                } else {
                    crate::constants::building_placement::INVALID_PLACEMENT_COLOR
                };
            }
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
    _model_assets: &Option<Res<crate::rendering::model_loader::ModelAssets>>,
    is_valid: bool,
) {
    // Check if placement is valid (no collision)
    if !is_valid {
        warn!("Cannot place building: position is blocked by another building, unit, or obstacle");
        return;
    }

    let building_cost = get_building_cost(&building_type);
    if can_afford_building(&building_cost, player_resources) {
        deduct_building_cost(&building_cost, player_resources, main_resources);
        
        // Create a building site for player 1 (requires worker to complete)
        let placeholder_visual = create_placeholder_building_visual(meshes, materials, &building_type, placement_pos);
        let collision_radius = get_building_collision_radius(&building_type);
        
        let _building_site = commands.spawn((
            BuildingSite {
                building_type: building_type.clone(),
                position: placement_pos,
                player_id: 1, // Player 1
                assigned_worker: None,
                construction_started: false,
                site_reserved: false,
            },
            placeholder_visual.0, // Transform
            placeholder_visual.1, // Mesh3d
            placeholder_visual.2, // MeshMaterial3d
            Selectable {
                is_selected: false,
                selection_radius: collision_radius * 2.0, // Larger radius for easy clicking
            },
            CollisionRadius {
                radius: collision_radius,
            },
        ));

        // Clear placement
        placement.active_building = None;
        if let Some(preview_entity) = placement.placement_preview.take() {
            commands.entity(preview_entity).despawn();
        }

        info!("Created building site for player 1: {:?} at {:?}", building_type, placement_pos);
    } else {
        warn!("Cannot place building: insufficient resources");
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
    is_valid: bool,
) -> Entity {
    use crate::constants::buildings::*;
    use crate::constants::building_placement::*;

    let (mesh, size) = match building_type {
        BuildingType::Nursery => (meshes.add(Cuboid::from_size(NURSERY_SIZE)), NURSERY_SIZE),
        BuildingType::WarriorChamber => (meshes.add(Cuboid::from_size(WARRIOR_CHAMBER_SIZE)), WARRIOR_CHAMBER_SIZE),
        BuildingType::HunterChamber => (meshes.add(Cuboid::from_size(HUNTER_CHAMBER_SIZE)), HUNTER_CHAMBER_SIZE),
        BuildingType::FungalGarden => (meshes.add(Cuboid::from_size(FUNGAL_GARDEN_SIZE)), FUNGAL_GARDEN_SIZE),
        _ => (meshes.add(Cuboid::from_size(DEFAULT_BUILDING_SIZE)), DEFAULT_BUILDING_SIZE),
    };

    let preview_color = if is_valid {
        VALID_PLACEMENT_COLOR
    } else {
        INVALID_PLACEMENT_COLOR
    };

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: preview_color,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(position + Vec3::new(0.0, size.y / 2.0, 0.0)),
        PlacementPreview,
    )).id()
}

/// Create a placeholder visual for a building under construction
fn create_placeholder_building_visual(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    building_type: &BuildingType,
    position: Vec3,
) -> (Transform, Mesh3d, MeshMaterial3d<StandardMaterial>) {
    use crate::constants::buildings::*;
    

    // Get building size for placeholder
    let (mesh, size) = match building_type {
        BuildingType::Nursery => (meshes.add(Cuboid::from_size(NURSERY_SIZE)), NURSERY_SIZE),
        BuildingType::WarriorChamber => (meshes.add(Cuboid::from_size(WARRIOR_CHAMBER_SIZE)), WARRIOR_CHAMBER_SIZE),
        BuildingType::HunterChamber => (meshes.add(Cuboid::from_size(HUNTER_CHAMBER_SIZE)), HUNTER_CHAMBER_SIZE),
        BuildingType::FungalGarden => (meshes.add(Cuboid::from_size(FUNGAL_GARDEN_SIZE)), FUNGAL_GARDEN_SIZE),
        _ => (meshes.add(Cuboid::from_size(DEFAULT_BUILDING_SIZE)), DEFAULT_BUILDING_SIZE),
    };

    // Create a semi-transparent placeholder material
    let placeholder_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.8, 0.6, 0.6), // Yellow-ish semi-transparent
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    (
        Transform::from_translation(position + Vec3::new(0.0, size.y / 2.0, 0.0)),
        Mesh3d(mesh),
        MeshMaterial3d(placeholder_material),
    )
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

pub fn get_building_collision_radius(building_type: &BuildingType) -> f32 {
    use crate::constants::collision::*;
    match building_type {
        BuildingType::Nursery => NURSERY_COLLISION_RADIUS,
        BuildingType::WarriorChamber => WARRIOR_CHAMBER_COLLISION_RADIUS,
        BuildingType::Queen => QUEEN_COLLISION_RADIUS,
        _ => DEFAULT_BUILDING_COLLISION_RADIUS,
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