#![allow(dead_code)]
use bevy::prelude::*;
use crate::model_loader::*;
use crate::components::*;

// === MODEL SHOWCASE CONSTANTS ===
const MODELS_PER_ROW: usize = 9;
const MODEL_SPACING: f32 = 20.0; // Increased spacing for larger models
const GROUP_SIZE: usize = 3; // Number of instances per model type
const GROUP_SPACING: f32 = 6.0; // Increased spacing between instances in a group
const BASE_HEIGHT: f32 = 5.0; // Height above ground
const ROW_SPACING_MULTIPLIER: f32 = 2.0; // Extra spacing between rows

pub struct ModelShowcasePlugin;

impl Plugin for ModelShowcasePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShowcaseState::default())
           .add_systems(Update, (handle_showcase_input, setup_model_showcase_on_demand));
    }
}

#[derive(Resource, Default)]
pub struct ShowcaseState {
    pub spawned: bool,
}

#[derive(Component)]
pub struct ShowcaseModel {
    pub model_name: String,
}

pub fn handle_showcase_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut showcase_state: ResMut<ShowcaseState>,
) {
    if keyboard.just_pressed(crate::constants::hotkeys::SPAWN_MODEL_SHOWCASE) {
        if !showcase_state.spawned {
            info!("Model showcase requested via 'M' key");
            showcase_state.spawned = true;
        } else {
            info!("Model showcase already spawned");
        }
    }
}

pub fn setup_model_showcase_on_demand(
    mut commands: Commands,
    model_assets: Res<ModelAssets>,
    scenes: Res<Assets<Scene>>,
    mut showcase_state: ResMut<ShowcaseState>,
) {
    if !showcase_state.spawned {
        return;
    }
    // Wait for models to be loaded
    if !model_assets.models_loaded {
        return;
    }
    
    // Check if at least one model is actually loaded
    let bee_loaded = scenes.get(&model_assets.bee).is_some();
    if !bee_loaded {
        info!("Models not yet loaded, waiting...");
        return;
    }
    
    // Mark as processed to prevent re-spawning
    showcase_state.spawned = false;
    
    // Define all model types to showcase - ensure we have them all
    let model_types = get_all_model_types();
    
    info!("Setting up model showcase with all {} model types", model_types.len());
    
    // Verify we have all expected model types
    if model_types.len() < 18 {
        warn!("Expected at least 18 model types, but only found {}. Some models may be missing!", model_types.len());
    }
    
    for (index, model_type) in model_types.iter().enumerate() {
        let row = index / MODELS_PER_ROW;
        let col = index % MODELS_PER_ROW;
        
        // Calculate base position for this model type
        let base_x = (col as f32 - (MODELS_PER_ROW as f32 - 1.0) / 2.0) * MODEL_SPACING;
        let base_z = (row as f32 - 1.0) * MODEL_SPACING * ROW_SPACING_MULTIPLIER;
        let base_y = BASE_HEIGHT;
        
        // Get model-specific scale
        let model_scale = get_model_scale(model_type);
        let model_handle = model_assets.get_model_handle(model_type);
        let model_name = format!("{:?}", model_type);
        
        // Check if model handle is valid
        if model_handle == Handle::default() {
            warn!("Invalid model handle for {}, skipping...", model_name);
            continue;
        }
        
        // Spawn a small group of each model type
        for group_index in 0..GROUP_SIZE {
            let offset_x = (group_index as f32 - (GROUP_SIZE as f32 - 1.0) / 2.0) * GROUP_SPACING;
            let position = Vec3::new(base_x + offset_x, base_y, base_z);
            
            // Spawn the model with proper components
            let _entity = commands.spawn((
                SceneRoot(model_handle.clone()),
                Transform::from_translation(position)
                    .with_scale(Vec3::splat(model_scale))
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)), // Face forward
                InsectModel {
                    model_type: model_type.clone(),
                    scale: model_scale,
                },
                UseGLBModel,
                ShowcaseModel {
                    model_name: format!("{} #{}", model_name, group_index + 1),
                },
                // Add basic RTS components so they work with existing systems
                RTSUnit {
                    unit_id: (index * GROUP_SIZE + group_index) as u32,
                    player_id: 1,
                    size: model_scale,
                    unit_type: Some(get_showcase_unit_type(model_type)),
                },
                Position {
                    translation: position,
                    rotation: Quat::from_rotation_y(std::f32::consts::PI),
                },
                Movement::default(),
                RTSHealth {
                    current: 100.0,
                    max: 100.0,
                    armor: 0.0,
                    regeneration_rate: 0.0,
                    last_damage_time: 0.0,
                },
                Selectable {
                    is_selected: false,
                    selection_radius: model_scale * 2.0,
                },
                Vision::default(),
                CollisionRadius { radius: model_scale * 2.0 }, // Increased for better spacing in showcase
                EntityState::default(),
                GameEntity,
            )).id();
            
            info!("Spawned showcase model: {} at {:?} (scale: {:.1})", 
                  model_name, position, model_scale);
        }
    }
    
    // Spawn a few information markers
    spawn_info_markers(&mut commands);
    
    info!("Model showcase setup complete! {} model types with {} instances each", 
          model_types.len(), GROUP_SIZE);
}

fn spawn_info_markers(commands: &mut Commands) {
    // Add text markers to identify sections (using basic cube entities as placeholders)
    let marker_positions = vec![
        (Vec3::new(-MODEL_SPACING * 2.0, 8.0, -MODEL_SPACING * 2.0), "Classic Models"),
        (Vec3::new(-MODEL_SPACING * 2.0, 8.0, MODEL_SPACING * 0.5), "New High-Quality Models"),
    ];
    
    for (position, label) in marker_positions {
        commands.spawn((
            Name::new(label.to_string()),
            Transform::from_translation(position),
            ShowcaseModel {
                model_name: format!("Marker: {}", label),
            },
        ));
        
        info!("Placed marker '{}' at {:?}", label, position);
    }
}

/// Get all available model types for showcase
fn get_all_model_types() -> Vec<InsectModelType> {
    vec![
        // Classic models
        InsectModelType::Bee,
        InsectModelType::Beetle,
        InsectModelType::Spider,
        InsectModelType::Scorpion,
        InsectModelType::WolfSpider,
        InsectModelType::QueenFacedBug,
        InsectModelType::ApisMellifera,
        
        // New high-quality models
        InsectModelType::Meganeura,
        InsectModelType::AnimatedSpider,
        InsectModelType::RhinoBeetle,
        InsectModelType::Hornet,
        InsectModelType::Fourmi,
        InsectModelType::CairnsBirdwing,
        InsectModelType::LadybugLowpoly,
        InsectModelType::RolyPoly,
        InsectModelType::MysteryModel,
        InsectModelType::Mushrooms, // Environment objects
    ]
}

/// Get an appropriate unit type for showcase purposes
fn get_showcase_unit_type(model_type: &InsectModelType) -> UnitType {
    match model_type {
        InsectModelType::Fourmi => UnitType::WorkerAnt,
        InsectModelType::RhinoBeetle => UnitType::BeetleKnight,
        InsectModelType::Hornet => UnitType::HunterWasp,
        InsectModelType::CairnsBirdwing => UnitType::ScoutAnt,
        InsectModelType::QueenFacedBug => UnitType::SpearMantis,
        InsectModelType::Meganeura => UnitType::DragonFly,
        InsectModelType::RolyPoly => UnitType::DefenderBug,
        InsectModelType::AnimatedSpider => UnitType::EliteSpider,
        InsectModelType::MysteryModel => UnitType::SpecialOps,
        _ => UnitType::SoldierAnt, // Default fallback
    }
}

/// System to add labels above showcase models (optional enhancement)
pub fn update_showcase_labels(
    showcase_models: Query<(&Transform, &ShowcaseModel), With<ShowcaseModel>>,
    mut gizmos: Gizmos,
) {
    for (transform, _showcase) in showcase_models.iter() {
        // Draw a small indicator above each model
        let label_pos = transform.translation + Vec3::new(0.0, 3.0, 0.0);
        gizmos.sphere(label_pos, 0.2, Color::WHITE);
        
        // Note: For full text labels, you'd need a UI system
        // This is a simple visual indicator for now
    }
}