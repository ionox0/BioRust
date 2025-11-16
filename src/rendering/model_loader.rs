//! # Model Loader Module
//!
//! This module handles loading and managing 3D GLB (GL Transmission Format Binary) models
//! for the insect-themed RTS game. It provides:
//!
//! - Asset loading and management for all insect models
//! - Automatic upgrade from primitive meshes to GLB models
//! - Model-to-unit-type mappings for consistent theming
//! - Animation controller setup for animated models
//! - Scale normalization for consistent RTS gameplay
//!
//! ## Architecture
//!
//! The system uses a three-phase approach:
//! 1. **Startup**: Load all GLB model assets from disk
//! 2. **Runtime**: Automatically replace primitive shapes with GLB models when available
//! 3. **Animation**: Setup animation controllers for units with animated models

#![allow(dead_code)] // Allow unused model functionality for future features

use crate::core::components::*;
use crate::rendering::animation_systems::*;
use bevy::prelude::*;

/// Plugin that manages 3D model loading and lifecycle for the game.
///
/// This plugin coordinates:
/// - Initial model asset loading at startup
/// - Runtime model upgrades (primitive → GLB)
/// - Animation controller initialization
/// - Model transform smoothing to prevent visual jitter
pub struct ModelLoaderPlugin;

impl Plugin for ModelLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModelAssets>()
            .add_systems(Startup, load_models)
            .add_systems(
                Update,
                (
                    upgrade_to_glb_models,
                    setup_animation_controllers,
                    smooth_glb_model_updates,
                    apply_team_colors_to_glb_models.after(upgrade_to_glb_models),
                ),
            );
    }
}

/// Resource containing handles to all loaded GLB model assets.
///
/// This acts as a centralized registry for all 3D models in the game.
/// Models are loaded asynchronously at startup and marked ready via `models_loaded`.
///
/// # Model Categories
///
/// - **Classic Models**: Original insect models (bee, beetle, ladybug, spider, etc.)
/// - **High-Quality Models**: Newer, more detailed models with better animations
/// - **Environment**: Non-unit objects like mushrooms for decoration
#[derive(Resource)]
pub struct ModelAssets {
    // Classic models - original set of insect models
    pub bee: Handle<Scene>,
    pub beetle: Handle<Scene>,
    pub spider: Handle<Scene>,
    pub scorpion: Handle<Scene>,
    pub wolf_spider: Handle<Scene>,
    pub queen_faced_bug: Handle<Scene>,
    pub apis_mellifera: Handle<Scene>,

    // New high-quality models - enhanced visuals and animations
    pub meganeura: Handle<Scene>,       // Ancient dragonfly
    pub animated_spider: Handle<Scene>, // Animated spider
    pub rhino_beetle: Handle<Scene>,    // Rhino beetle
    pub hornet: Handle<Scene>,          // Hornet
    pub fourmi: Handle<Scene>,          // Ant (French: "fourmi")
    pub cairns_birdwing: Handle<Scene>, // Butterfly
    pub roly_poly: Handle<Scene>,       // Pill bug (isopod)
    pub dragonfly_2: Handle<Scene>,   // Unknown/special model
    pub common_housefly: Handle<Scene>, // Common housefly
    pub giant_termite: Handle<Scene>,   // Giant termite
    pub leg_beetle: Handle<Scene>,      // Leg beetle
    pub jewel_bug: Handle<Scene>,       // Jewel bug
    pub stinkbug: Handle<Scene>,        // Stinkbug
    pub termite: Handle<Scene>,         // Regular termite

    // Newly added models - expanded insect variety
    pub animated_peacock_moth: Handle<Scene>,     // Animated peacock moth
    pub aphid: Handle<Scene>,                     // Small aphid
    pub black_widow_spider: Handle<Scene>,        // Black widow spider
    pub elephant_hawk_moth: Handle<Scene>,        // Elephant hawk moth
    pub flea: Handle<Scene>,                      // Flea
    pub flying_hornet: Handle<Scene>,             // Flying hornet
    pub goliath_birdeater: Handle<Scene>,         // Goliath birdeater spider
    pub hawkmoth_larvae: Handle<Scene>,           // Hawkmoth larvae
    pub japanese_rhinoceros_beetle: Handle<Scene>, // Japanese rhinoceros beetle
    pub mantis_tenodera_aridifolia: Handle<Scene>, // Mantis tenodera aridifolia
    pub mite: Handle<Scene>,                      // Mite
    pub moth: Handle<Scene>,                      // Generic moth
    pub tick: Handle<Scene>,                      // Tick
    pub unknown_species1: Handle<Scene>,          // Unknown model 1
    pub unknown_species2: Handle<Scene>,          // Unknown model 2
    pub woodlouse: Handle<Scene>,                 // Woodlouse

    // Environment objects
    pub mushrooms: Handle<Scene>,  // Mushroom environment objects
    pub grass: Handle<Scene>,      // Grass patches
    pub grass_2: Handle<Scene>,    // Grass variant 2
    pub hive: Handle<Scene>,       // Hive structure
    pub wood_stick: Handle<Scene>, // Wood stick debris
    pub simple_grass_chunks: Handle<Scene>, // Simple grass chunks

    // Building objects
    pub anthill: Handle<Scene>, // Anthill building model

    // New environment objects
    pub cherry_blossom_tree: Handle<Scene>, // Emissive energy cherry blossom tree
    pub pine_cone: Handle<Scene>,           // Pine cone natural debris
    pub plants_asset_set: Handle<Scene>,    // Various plants collection
    pub beech_fern: Handle<Scene>,          // Realistic beech fern plant
    pub trees_pack: Handle<Scene>,          // Realistic trees pack
    pub river_rock: Handle<Scene>,          // River rock formations
    pub small_rocks: Handle<Scene>,         // Small rock debris

    /// Flag indicating whether all models have been queued for loading.
    /// Does not guarantee models are fully loaded in memory yet.
    pub models_loaded: bool,
}

impl Default for ModelAssets {
    fn default() -> Self {
        Self {
            // Classic models
            bee: Handle::default(),
            beetle: Handle::default(),
            spider: Handle::default(),
            scorpion: Handle::default(),
            wolf_spider: Handle::default(),
            queen_faced_bug: Handle::default(),
            apis_mellifera: Handle::default(),

            // New high-quality models
            meganeura: Handle::default(),
            animated_spider: Handle::default(),
            rhino_beetle: Handle::default(),
            hornet: Handle::default(),
            fourmi: Handle::default(),
            cairns_birdwing: Handle::default(),
            roly_poly: Handle::default(),
            dragonfly_2: Handle::default(),
            common_housefly: Handle::default(),
            giant_termite: Handle::default(),
            leg_beetle: Handle::default(),
            jewel_bug: Handle::default(),
            stinkbug: Handle::default(),
            termite: Handle::default(),

            // Newly added models
            animated_peacock_moth: Handle::default(),
            aphid: Handle::default(),
            black_widow_spider: Handle::default(),
            elephant_hawk_moth: Handle::default(),
            flea: Handle::default(),
            flying_hornet: Handle::default(),
            goliath_birdeater: Handle::default(),
            hawkmoth_larvae: Handle::default(),
            japanese_rhinoceros_beetle: Handle::default(),
            mantis_tenodera_aridifolia: Handle::default(),
            mite: Handle::default(),
            moth: Handle::default(),
            tick: Handle::default(),
            unknown_species1: Handle::default(),
            unknown_species2: Handle::default(),
            woodlouse: Handle::default(),

            // Environment objects
            mushrooms: Handle::default(),
            grass: Handle::default(),
            grass_2: Handle::default(),
            hive: Handle::default(),
            wood_stick: Handle::default(),
            simple_grass_chunks: Handle::default(),

            // Building objects
            anthill: Handle::default(),

            // New environment objects
            cherry_blossom_tree: Handle::default(),
            pine_cone: Handle::default(),
            plants_asset_set: Handle::default(),
            beech_fern: Handle::default(),
            trees_pack: Handle::default(),
            river_rock: Handle::default(),
            small_rocks: Handle::default(),

            models_loaded: false,
        }
    }
}

/// Component that attaches metadata about a 3D insect model to an entity.
///
/// This component is used to track which model type is being used and at what scale,
/// allowing for proper animation, collision, and rendering adjustments.
#[derive(Component, Debug, Clone)]
pub struct InsectModel {
    /// The specific type of insect model being displayed
    pub model_type: InsectModelType,
    /// The scale factor applied to the model (typically 1.5 for uniform RTS gameplay)
    pub scale: f32,
}

/// Enumeration of all available insect model types in the game.
///
/// Models are categorized into:
/// - Classic models: Original set of basic insect models
/// - High-quality models: Newer models with better detail and animations
/// - Environment: Non-unit decorative objects
///
/// Each model type maps to a specific GLB file in the assets directory.
#[derive(Debug, Clone, PartialEq)]
pub enum InsectModelType {
    // Classic models - original insect model set
    Bee,           // Basic bee model
    Beetle,        // Black ox beetle
    Spider,        // Small spider
    Scorpion,      // Scorpion with pincers and tail
    WolfSpider,    // Larger wolf spider variant
    QueenFacedBug, // Mantis/praying mantis model
    ApisMellifera, // High-quality honey bee (Apis mellifera)

    // New high-quality models - enhanced detail and animations
    Meganeura,      // Ancient dragonfly - perfect for flying units
    AnimatedSpider, // Spider with animation support - great for predators
    RhinoBeetle,    // Rhino beetle with horn - heavy armored units
    Hornet,         // Hornet - aggressive flying units
    Fourmi,         // French ant model - perfect for worker/soldier ants
    CairnsBirdwing, // Butterfly - scouts/light flying units
    RolyPoly,       // Pill bug (isopod) - defensive units
    DragonFly,      // Unknown special model - unique units
    CommonHousefly, // Common housefly - fast flying unit
    GiantTermite,   // Giant termite - heavy siege unit
    LegBeetle,      // Leg beetle - specialized melee unit
    JewelBug,       // Jewel bug - fast support beetle
    Stinkbug,       // Stinkbug - area denial unit
    Termite,        // Regular termite - builder/worker variant

    // Newly added models - expanded insect variety for multi-team system
    AnimatedPeacockMoth,     // Animated peacock moth - flying unit
    Aphid,                   // Small aphid - swarm unit
    BlackWidowSpider,        // Black widow spider - venomous predator
    ElephantHawkMoth,        // Elephant hawk moth - large flying unit
    Flea,                    // Flea - tiny fast jumping unit
    FlyingHornet,            // Flying hornet - aerial assault unit
    GoliathBirdeater,        // Goliath birdeater spider - massive predator
    HawkmothLarvae,          // Hawkmoth larvae - caterpillar unit
    JapaneseRhinocerosBeetle, // Japanese rhinoceros beetle - heavy armor
    MantisTenoderaAridifolia, // Mantis tenodera aridifolia - elite predator
    Mite,                    // Mite - microscopic unit
    Moth,                    // Generic moth - night flying unit
    Tick,                    // Tick - parasitic unit
    Dragonfly2,         // Unknown model 1 - special unit
    UnknownSpecies2,         // Unknown model 2 - special unit
    Woodlouse,               // Woodlouse - defensive unit

    // Environment objects - decorative non-interactive models
    Mushrooms,         // Mushroom cluster - environment decoration
    Grass,             // Grass patches - ground coverage
    Grass2,            // Grass variant 2 - ground coverage
    Hive,              // Hive structure - insect habitat
    WoodStick,         // Wood stick debris - natural clutter
    SimpleGrassChunks, // Simple grass chunks - ground decoration

    // Building objects - interactive structures
    Anthill, // Anthill building model - base structures

    // New environment objects - natural scenery
    CherryBlossomTree, // Emissive energy cherry blossom tree - beautiful landmark
    PineCone,          // Pine cone - forest floor debris
    PlantsAssetSet,    // Various plants collection - diverse vegetation
    BeechFern,         // Realistic beech fern plant - forest undergrowth
    TreesPack,         // Realistic trees pack - major landmarks
    RiverRock,         // River rock formations - geological features
    SmallRocks,        // Small rock debris - scattered stones
}

// Environment objects

#[derive(Debug, Clone, PartialEq)]
pub enum SceneryModelType {
    Mushrooms,
    Grass,
    Grass2,
    Hive,
    Stick,
}

/// Configuration for loading a single model from the asset directory.
///
/// This struct uses a factory pattern to centralize model loading configuration,
/// making it easier to add new models without modifying the loading logic.
struct ModelConfig {
    /// Internal identifier for the model (used in code)
    name: &'static str,
    /// File path to the GLB model asset relative to the assets directory
    path: &'static str,
    /// Human-readable description of what the model represents
    description: &'static str,
}

impl ModelConfig {
    /// Creates a new model configuration.
    ///
    /// # Arguments
    ///
    /// * `name` - Internal identifier (e.g., "bee", "spider")
    /// * `path` - Asset path (e.g., "models/insects/bee-v1.glb#Scene0")
    /// * `description` - Human-readable description (e.g., "Classic bee model")
    const fn new(name: &'static str, path: &'static str, description: &'static str) -> Self {
        Self {
            name,
            path,
            description,
        }
    }
}

/// Master list of all model definitions using the factory pattern.
///
/// This constant array centralizes all model loading configurations,
/// making it easy to add new models by simply adding a new entry.
/// Each entry maps a name to its file path and description.
///
/// # Format
///
/// GLB paths use the format: "path/to/file.glb#Scene0"
/// The "#Scene0" suffix is required by Bevy to load the root scene from the GLB file.
const MODEL_DEFINITIONS: &[ModelConfig] = &[
    // Classic models - original insect set
    ModelConfig::new(
        "bee",
        "models/insects/bee-v1.glb#Scene0",
        "Classic bee model",
    ),
    ModelConfig::new(
        "beetle",
        "models/insects/black_ox_beetle_small.glb#Scene0",
        "Black ox beetle",
    ),
    ModelConfig::new(
        "spider",
        "models/insects/spider_small.glb#Scene0",
        "Small spider",
    ),
    ModelConfig::new(
        "scorpion",
        "models/insects/scorpion.glb#Scene0",
        "Scorpion model",
    ),
    ModelConfig::new(
        "wolf_spider",
        "models/insects/wolf_spider.glb#Scene0",
        "Wolf spider",
    ),
    ModelConfig::new(
        "queen_faced_bug",
        "models/insects/mantis.glb#Scene0",
        "Mantis (Queen faced bug)",
    ),
    // ModelConfig::new("apis_mellifera", "models/insects/apis_mellifera.glb#Scene0", "High-quality honey bee"), // Temporarily disabled due to unsupported extension

    // New high-quality models - enhanced visuals
    ModelConfig::new(
        "meganeura",
        "models/insects/meganeura_dinoraul.glb#Scene0",
        "Ancient dragonfly",
    ),
    ModelConfig::new(
        "animated_spider",
        "models/insects/animated_spider.glb#Scene0",
        "Animated spider",
    ),
    ModelConfig::new(
        "rhino_beetle",
        "models/insects/a_rhino_beetle.glb#Scene0",
        "Rhino beetle",
    ),
    ModelConfig::new("hornet", "models/insects/hornet.glb#Scene0", "Hornet"),
    ModelConfig::new("fourmi", "models/insects/fourmi.glb#Scene0", "French ant"),
    ModelConfig::new(
        "cairns_birdwing",
        "models/insects/cairns_birdwing.glb#Scene0",
        "Butterfly",
    ),
    ModelConfig::new(
        "roly_poly",
        "models/insects/roly_poly.glb#Scene0",
        "Pill bug",
    ),
    ModelConfig::new(
        "dragonfly_2",
        "models/insects/dragonfly_2.glb#Scene0",
        "Dragonfly 2",
    ),
    ModelConfig::new(
        "common_housefly",
        "models/insects/common_housefly.glb#Scene0",
        "Common housefly",
    ),
    ModelConfig::new(
        "giant_termite",
        "models/insects/giant_termite.glb#Scene0",
        "Giant termite",
    ),
    ModelConfig::new(
        "leg_beetle",
        "models/insects/leg_beetle.glb#Scene0",
        "Leg beetle",
    ),
    ModelConfig::new(
        "jewel_bug",
        "models/insects/jewel_bug.glb#Scene0",
        "Jewel bug",
    ),
    ModelConfig::new("stinkbug", "models/insects/stinkbug.glb#Scene0", "Stinkbug"),
    ModelConfig::new(
        "termite",
        "models/insects/termite.glb#Scene0",
        "Regular termite",
    ),
    // Environment objects
    ModelConfig::new(
        "mushrooms",
        "models/objects/mushrooms.glb#Scene0",
        "Mushroom environment objects",
    ),
    ModelConfig::new("grass", "models/objects/grass.glb#Scene0", "Grass patches"),
    ModelConfig::new(
        "grass_2",
        "models/objects/grass_2.glb#Scene0",
        "Grass variant 2",
    ),
    ModelConfig::new("hive", "models/objects/hive.glb#Scene0", "Hive structure"),
    ModelConfig::new(
        "wood_stick",
        "models/objects/wood_stick.glb#Scene0",
        "Wood stick debris",
    ),
    ModelConfig::new(
        "simple_grass_chunks",
        "models/objects/simple_grass_chunks.glb#Scene0",
        "Simple grass chunks",
    ),
    // Building objects
    ModelConfig::new(
        "anthill",
        "models/objects/anthill.glb#Scene0",
        "Anthill building model",
    ),
    // New environment objects
    ModelConfig::new(
        "cherry_blossom_tree",
        "models/objects/emissive_energy_cherry_blossom_tree.glb#Scene0",
        "Emissive energy cherry blossom tree",
    ),
    ModelConfig::new(
        "pine_cone",
        "models/objects/pine_cone.glb#Scene0",
        "Pine cone natural debris",
    ),
    ModelConfig::new(
        "plants_asset_set",
        "models/objects/plants_asset_set.glb#Scene0",
        "Various plants collection",
    ),
    ModelConfig::new(
        "beech_fern",
        "models/objects/realistic_beech_fern_plant__games__film.glb#Scene0",
        "Realistic beech fern plant",
    ),
    ModelConfig::new(
        "trees_pack",
        "models/objects/realistic_trees_pack_of_2_free.glb#Scene0",
        "Realistic trees pack",
    ),
    ModelConfig::new(
        "river_rock",
        "models/objects/river_rock.glb#Scene0",
        "River rock formations",
    ),
    ModelConfig::new(
        "small_rocks",
        "models/objects/smallrocks1.glb#Scene0",
        "Small rock debris",
    ),

    // Additional insect models for expanded team system
    ModelConfig::new(
        "animated_peacock_moth",
        "models/insects/animated_peacock_moth.glb#Scene0",
        "Animated peacock moth",
    ),
    ModelConfig::new(
        "aphid",
        "models/insects/aphid.glb#Scene0",
        "Small aphid",
    ),
    ModelConfig::new(
        "black_widow_spider",
        "models/insects/black_widow_spider.glb#Scene0",
        "Black widow spider",
    ),
    ModelConfig::new(
        "elephant_hawk_moth",
        "models/insects/elephant_hawk_moth.glb#Scene0",
        "Elephant hawk moth",
    ),
    ModelConfig::new(
        "flea",
        "models/insects/flea.glb#Scene0",
        "Flea",
    ),
    ModelConfig::new(
        "flying_hornet",
        "models/insects/flying_hornet.glb#Scene0",
        "Flying hornet",
    ),
    ModelConfig::new(
        "goliath_birdeater",
        "models/insects/goliath_birdeater.glb#Scene0",
        "Goliath birdeater spider",
    ),
    ModelConfig::new(
        "hawkmoth_larvae",
        "models/insects/hawkmoth_larvae_2.glb#Scene0",
        "Hawkmoth larvae",
    ),
    ModelConfig::new(
        "japanese_rhinoceros_beetle",
        "models/insects/japanese_rhinoceros_beetle.glb#Scene0",
        "Japanese rhinoceros beetle",
    ),
    ModelConfig::new(
        "mantis_tenodera_aridifolia",
        "models/insects/mantis_tenodera_aridifolia.glb#Scene0",
        "Mantis tenodera aridifolia",
    ),
    ModelConfig::new(
        "mite",
        "models/insects/mite.glb#Scene0",
        "Mite",
    ),
    ModelConfig::new(
        "moth",
        "models/insects/moth.glb#Scene0",
        "Generic moth",
    ),
    ModelConfig::new(
        "tick",
        "models/insects/tick.glb#Scene0",
        "Tick",
    ),
    ModelConfig::new(
        "unknown_species1",
        "models/insects/unknown_species1.glb#Scene0",
        "Unknown species 1",
    ),
    ModelConfig::new(
        "unknown_species2",
        "models/insects/unknown_species2.glb#Scene0",
        "Unknown species 2",
    ),
    ModelConfig::new(
        "woodlouse",
        "models/insects/woodlouse.glb#Scene0",
        "Woodlouse",
    ),
];

/// Startup system that loads all GLB model assets.
///
/// This system runs once at game startup and queues all model files for loading.
/// Models are loaded asynchronously by Bevy's asset server.
///
/// # Process
///
/// 1. Iterates through all models defined in `MODEL_DEFINITIONS`
/// 2. Loads each model using the asset server
/// 3. Stores the handle in the `ModelAssets` resource
/// 4. Sets `models_loaded` flag to true (indicates loading has been queued)
///
/// # Note
///
/// Setting `models_loaded = true` means models have been *queued* for loading,
/// not that they are fully loaded in memory. The `upgrade_to_glb_models` system
/// checks if assets are actually available before using them.
pub fn load_models(mut model_assets: ResMut<ModelAssets>, asset_server: Res<AssetServer>) {
    info!(
        "Loading {} insect GLB models using factory pattern...",
        MODEL_DEFINITIONS.len()
    );

    // Load each model by name using helper function
    // This approach keeps the code DRY and makes it easy to add new models
    load_model_handle(&mut model_assets.bee, &asset_server, "bee");
    load_model_handle(&mut model_assets.beetle, &asset_server, "beetle");
    load_model_handle(&mut model_assets.spider, &asset_server, "spider");
    load_model_handle(&mut model_assets.scorpion, &asset_server, "scorpion");
    load_model_handle(&mut model_assets.wolf_spider, &asset_server, "wolf_spider");
    load_model_handle(
        &mut model_assets.queen_faced_bug,
        &asset_server,
        "queen_faced_bug",
    );
    // load_model_handle(&mut model_assets.apis_mellifera, &asset_server, "apis_mellifera"); // Temporarily disabled

    // Load new high-quality models
    load_model_handle(&mut model_assets.meganeura, &asset_server, "meganeura");
    load_model_handle(
        &mut model_assets.animated_spider,
        &asset_server,
        "animated_spider",
    );
    load_model_handle(
        &mut model_assets.rhino_beetle,
        &asset_server,
        "rhino_beetle",
    );
    load_model_handle(&mut model_assets.hornet, &asset_server, "hornet");
    load_model_handle(&mut model_assets.fourmi, &asset_server, "fourmi");
    load_model_handle(
        &mut model_assets.cairns_birdwing,
        &asset_server,
        "cairns_birdwing",
    );
    load_model_handle(&mut model_assets.roly_poly, &asset_server, "roly_poly");
    load_model_handle(
        &mut model_assets.dragonfly_2,
        &asset_server,
        "dragonfly_2",
    );
    load_model_handle(
        &mut model_assets.common_housefly,
        &asset_server,
        "common_housefly",
    );
    load_model_handle(
        &mut model_assets.giant_termite,
        &asset_server,
        "giant_termite",
    );
    load_model_handle(&mut model_assets.leg_beetle, &asset_server, "leg_beetle");
    load_model_handle(&mut model_assets.jewel_bug, &asset_server, "jewel_bug");
    load_model_handle(&mut model_assets.stinkbug, &asset_server, "stinkbug");
    load_model_handle(&mut model_assets.termite, &asset_server, "termite");

    // Load newly added models (skip problematic ones with unsupported glTF extensions)
    load_model_handle(&mut model_assets.animated_peacock_moth, &asset_server, "animated_peacock_moth");
    load_model_handle(&mut model_assets.aphid, &asset_server, "aphid");
    load_model_handle(&mut model_assets.black_widow_spider, &asset_server, "black_widow_spider");
    load_model_handle(&mut model_assets.elephant_hawk_moth, &asset_server, "elephant_hawk_moth");
    load_model_handle(&mut model_assets.flea, &asset_server, "flea");
    load_model_handle(&mut model_assets.flying_hornet, &asset_server, "flying_hornet");
    load_model_handle(&mut model_assets.goliath_birdeater, &asset_server, "goliath_birdeater");
    load_model_handle(&mut model_assets.hawkmoth_larvae, &asset_server, "hawkmoth_larvae");
    load_model_handle(&mut model_assets.japanese_rhinoceros_beetle, &asset_server, "japanese_rhinoceros_beetle");
    load_model_handle(&mut model_assets.mantis_tenodera_aridifolia, &asset_server, "mantis_tenodera_aridifolia");
    load_model_handle(&mut model_assets.mite, &asset_server, "mite");
    load_model_handle(&mut model_assets.moth, &asset_server, "moth");
    load_model_handle(&mut model_assets.tick, &asset_server, "tick");
    // load_model_handle(&mut model_assets.unknown_species1, &asset_server, "unknown_species1"); // Disabled: File doesn't exist
    // load_model_handle(&mut model_assets.unknown_species2, &asset_server, "unknown_species2"); // Disabled: File doesn't exist
    load_model_handle(&mut model_assets.woodlouse, &asset_server, "woodlouse");

    // Load environment objects
    load_model_handle(&mut model_assets.mushrooms, &asset_server, "mushrooms");
    load_model_handle(&mut model_assets.grass, &asset_server, "grass");
    load_model_handle(&mut model_assets.grass_2, &asset_server, "grass_2");
    load_model_handle(&mut model_assets.hive, &asset_server, "hive");
    load_model_handle(&mut model_assets.wood_stick, &asset_server, "wood_stick");
    // load_model_handle(
    //     &mut model_assets.simple_grass_chunks,
    //     &asset_server,
    //     "simple_grass_chunks",
    // ); // Disabled: KHR_materials_pbrSpecularGlossiness

    // Load building objects
    load_model_handle(&mut model_assets.anthill, &asset_server, "anthill");

    // Load new environment objects
    load_model_handle(
        &mut model_assets.cherry_blossom_tree,
        &asset_server,
        "cherry_blossom_tree",
    );
    load_model_handle(&mut model_assets.pine_cone, &asset_server, "pine_cone");
    load_model_handle(
        &mut model_assets.plants_asset_set,
        &asset_server,
        "plants_asset_set",
    );
    load_model_handle(&mut model_assets.beech_fern, &asset_server, "beech_fern");
    load_model_handle(&mut model_assets.trees_pack, &asset_server, "trees_pack");
    load_model_handle(&mut model_assets.river_rock, &asset_server, "river_rock");
    load_model_handle(&mut model_assets.small_rocks, &asset_server, "small_rocks");

    model_assets.models_loaded = true;
    info!("All GLB models queued for loading via factory pattern");
}

/// Helper function to load a single model handle.
///
/// This function looks up the model configuration by name and loads the asset.
/// It follows the DRY (Don't Repeat Yourself) principle to reduce code duplication.
///
/// # Arguments
///
/// * `handle` - Mutable reference to the scene handle to populate
/// * `asset_server` - Bevy's asset server for loading files
/// * `name` - The model's internal name (e.g., "bee", "spider")
///
/// # Errors
///
/// Logs a warning if the model name is not found in `MODEL_DEFINITIONS`.
fn load_model_handle(handle: &mut Handle<Scene>, asset_server: &AssetServer, name: &str) {
    if let Some(config) = MODEL_DEFINITIONS.iter().find(|c| c.name == name) {
        *handle = asset_server.load(config.path);
        info!(
            "Loading {}: {} from {}",
            name, config.description, config.path
        );
    } else {
        warn!("Model configuration not found for: {}", name);
    }
}

/// System that automatically upgrades primitive mesh units to GLB models.
///
/// This system runs every frame and checks for entities that:
/// 1. Don't have the `UseGLBModel` component (haven't been upgraded yet)
/// 2. Have an `RTSUnit` or `Building` component
/// 3. Are not dying/dead
///
/// When GLB models become available, it replaces the primitive shapes
/// (spheres, capsules, cuboids) with detailed 3D models.
///
/// # Process
///
/// 1. Check if models are queued and actually loaded in memory
/// 2. For each unit without a GLB model:
///    - Determine appropriate model type based on unit type
///    - Remove primitive mesh components
///    - Add GLB model scene and components
///    - Adjust scale and rotation for proper display
///    - Scale movement speed to compensate for visual size changes
/// 3. Mark buildings as processed (currently using placeholders)
///
/// # Movement Speed Adjustment
///
/// Since GLB models are visually larger than primitives (scaled up),
/// movement speeds are scaled down proportionally to maintain the same
/// perceived movement speed. This prevents units from appearing to
/// slide or move too fast for their size.
pub fn upgrade_to_glb_models(
    mut commands: Commands,
    model_assets: Res<ModelAssets>,
    scenes: Res<Assets<Scene>>,
    mut units_without_models: Query<
        (Entity, &RTSUnit, &mut Transform, Option<&mut Movement>),
        (Without<UseGLBModel>, Without<Building>, Without<Dying>),
    >,
    mut buildings_without_models: Query<
        (Entity, &Building, &RTSUnit, &mut Transform),
        (Without<UseGLBModel>, With<Building>, Without<Dying>),
    >,
) {
    // Early exit if models haven't been queued for loading yet
    if !model_assets.models_loaded {
        return;
    }

    // Verify models are actually loaded in memory (not just queued)
    // We check the bee model as a representative sample
    let bee_loaded = scenes.get(&model_assets.bee).is_some();
    if !bee_loaded {
        return; // Models not yet loaded, try again next frame
    }

    // Upgrade units from primitive meshes to GLB models
    upgrade_units_to_glb(
        &mut commands,
        &model_assets,
        units_without_models.iter_mut(),
    );

    // Handle buildings - upgrade to GLB models
    upgrade_buildings_to_glb(
        &mut commands,
        &model_assets,
        buildings_without_models.iter_mut(),
    );
}

/// Upgrades units from primitive shapes to GLB models.
///
/// This helper function processes the actual model upgrade for units,
/// separated for clarity and testability.
fn upgrade_units_to_glb<'a>(
    commands: &mut Commands,
    model_assets: &ModelAssets,
    units_iter: impl Iterator<
        Item = (
            Entity,
            &'a RTSUnit,
            Mut<'a, Transform>,
            Option<Mut<'a, Movement>>,
        ),
    >,
) {
    for (entity, unit, mut transform, movement_opt) in units_iter {
        // Skip entities without a specific unit type
        let Some(unit_type) = &unit.unit_type else {
            continue;
        };

        // Determine which model to use for this unit type
        let model_type = get_unit_insect_model(unit_type);
        let model_handle = model_assets.get_model_handle(&model_type);
        
        // Debug logging for model assignment
        info!("🎨 MODEL DEBUG: Unit {:?} -> Model {:?} (Player {})", 
              unit_type, model_type, unit.player_id);

        // Remove primitive mesh components (no longer needed)
        commands
            .entity(entity)
            .remove::<Mesh3d>()
            .remove::<MeshMaterial3d<StandardMaterial>>();

        // Calculate appropriate scale for this model
        let model_scale = calculate_model_scale(&model_type);

        // Add GLB model components
        commands.entity(entity).insert((
            SceneRoot(model_handle),
            InsectModel {
                model_type: model_type.clone(),
                scale: model_scale,
            },
            UseGLBModel,
            TeamColor::new(unit.player_id), // Add team coloring
        ));

        // Update transform for proper model display
        apply_model_transform(&mut transform, model_scale, &model_type);

        // Adjust movement speed to compensate for visual scale
        if let Some(mut movement) = movement_opt {
            adjust_movement_for_scale(&mut movement, model_scale);
        }

        info!("Upgraded unit to GLB model: {:?}", model_type);
    }
}

/// Calculates the appropriate scale for a given model type.
///
/// Special handling for certain models, otherwise uses the standard scale function.
fn calculate_model_scale(model_type: &InsectModelType) -> f32 {
    match model_type {
        InsectModelType::QueenFacedBug => crate::constants::models::MANTIS_SCALE,
        InsectModelType::ApisMellifera => crate::constants::models::BEE_SCALE, // Use bee scale as fallback
        _ => get_model_scale(model_type),
    }
}

/// Calculates the appropriate Y-axis rotation for a given model type.
///
/// Most models need a 180° rotation to face forward (from default backwards orientation).
/// Some models need additional rotation adjustments.
fn calculate_model_rotation(model_type: &InsectModelType) -> f32 {
    let base_rotation = std::f32::consts::PI; // 180° to face forward

    match model_type {
        // Termite needs 90° clockwise rotation (when viewed from above)
        InsectModelType::Termite | InsectModelType::GiantTermite => {
            base_rotation + std::f32::consts::FRAC_PI_2 // 180° + 90° = 270°
        }
        
        // Models that need 180° rotation (double the base rotation)
        InsectModelType::Flea | InsectModelType::Aphid => {
            base_rotation + std::f32::consts::PI // 180° + 180° = 360° (0°)
        }
        
        // CairnsBirdwing needs just the base 180° rotation
        InsectModelType::CairnsBirdwing => {
            base_rotation // 180° to face forward
        }
        
        // Tick needs 90° clockwise rotation (same as termite)
        InsectModelType::Tick => {
            base_rotation + std::f32::consts::FRAC_PI_2 // 180° + 90° = 270°
        }
        
        // All other models use default forward-facing rotation
        _ => base_rotation,
    }
}

/// Applies proper transform settings for a GLB model.
///
/// Sets the scale to match the model's intended size and rotates it to face forward
/// (GLB models typically face backwards by default).
/// Some models need special rotation adjustments.
fn apply_model_transform(
    transform: &mut Transform,
    model_scale: f32,
    model_type: &InsectModelType,
) {
    let target_scale = Vec3::splat(model_scale);

    // Only update scale if it's significantly different to avoid jitter
    if (transform.scale - target_scale).length() > 0.1 {
        transform.scale = target_scale;
    }

    // Apply model-specific rotation
    transform.rotation = Quat::from_rotation_y(calculate_model_rotation(model_type));
    
    // Apply model-specific Y position offsets
    match model_type {
        InsectModelType::Stinkbug => {
            transform.translation.y += 10.0; // Raise stinkbug 10 units higher
        }
        _ => {} // No Y offset for other models
    }
}

/// Visual scaling and gameplay mechanics are now completely separate.
/// Movement speeds are no longer adjusted based on model scale.
/// This function is disabled to prevent interference with unit balance.
fn adjust_movement_for_scale(_movement: &mut Movement, _scale_factor: f32) {
    // DISABLED: Movement speed is now independent of visual model scale
    // This ensures DragonFly and other units maintain their intended gameplay speeds
    // regardless of how large they appear visually.
    debug!("Movement speed adjustment disabled - visual scaling is separate from gameplay");
}

/// Upgrades buildings from primitive shapes to GLB models.
///
/// This function processes the actual model upgrade for buildings,
/// similar to units but handling building-specific logic.
fn upgrade_buildings_to_glb<'a>(
    commands: &mut Commands,
    model_assets: &ModelAssets,
    buildings_iter: impl Iterator<Item = (Entity, &'a Building, &'a RTSUnit, Mut<'a, Transform>)>,
) {
    for (entity, building, unit, mut transform) in buildings_iter {
        // Get the appropriate model for this building type
        let model_type = get_building_insect_model(&building.building_type);
        let model_handle = model_assets.get_model_handle(&model_type);

        // Get the appropriate scale for this model
        let model_scale = get_model_scale(&model_type);

        // Remove existing primitive mesh and material
        commands
            .entity(entity)
            .remove::<Mesh3d>()
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((
                // Add GLB model
                SceneRoot(model_handle),
                // Scale the model appropriately
                Transform::from_translation(transform.translation)
                    .with_scale(Vec3::splat(model_scale))
                    .with_rotation(transform.rotation),
                // Mark as using GLB model
                UseGLBModel,
            ));

        // Update the transform to match the new scale
        transform.scale = Vec3::splat(model_scale);

        info!(
            "Upgraded building {:?} (player {}) to GLB model {:?} with scale {:.1}",
            building.building_type, unit.player_id, model_type, model_scale
        );
    }
}

/// Marks buildings as processed without upgrading to GLB models.
///
/// Buildings are currently kept as placeholder models due to placement issues.
/// This function marks them with `UseGLBModel` to prevent re-processing.
#[allow(dead_code)]
fn mark_buildings_as_processed<'a>(
    commands: &mut Commands,
    buildings_iter: impl Iterator<Item = (Entity, &'a Building, &'a RTSUnit, Mut<'a, Transform>)>,
) {
    for (entity, building, _unit, _transform) in buildings_iter {
        info!(
            "Keeping building as placeholder model: {:?}",
            building.building_type
        );
        // Mark as using GLB model to prevent re-processing
        commands.entity(entity).insert(UseGLBModel);
    }
}

impl ModelAssets {
    /// Retrieves the scene handle for a specific insect model type.
    ///
    /// This method maps from the enum-based model type to the actual
    /// loaded asset handle. Used when spawning or upgrading entities.
    ///
    /// # Arguments
    ///
    /// * `model_type` - The type of insect model to retrieve
    ///
    /// # Returns
    ///
    /// A cloned handle to the GLB scene asset
    pub fn get_model_handle(&self, model_type: &InsectModelType) -> Handle<Scene> {
        match model_type {
            // Classic models
            InsectModelType::Bee => self.bee.clone(),
            InsectModelType::Beetle => self.beetle.clone(),
            InsectModelType::Spider => self.spider.clone(),
            InsectModelType::Scorpion => self.scorpion.clone(),
            InsectModelType::WolfSpider => self.wolf_spider.clone(),
            InsectModelType::QueenFacedBug => self.queen_faced_bug.clone(),
            InsectModelType::ApisMellifera => self.bee.clone(), // Fallback to regular bee model

            // New high-quality models
            InsectModelType::Meganeura => self.meganeura.clone(),
            InsectModelType::AnimatedSpider => self.animated_spider.clone(),
            InsectModelType::RhinoBeetle => self.rhino_beetle.clone(),
            InsectModelType::Hornet => self.hornet.clone(),
            InsectModelType::Fourmi => self.fourmi.clone(),
            InsectModelType::CairnsBirdwing => self.cairns_birdwing.clone(),
            InsectModelType::RolyPoly => self.roly_poly.clone(),
            InsectModelType::DragonFly => self.dragonfly_2.clone(),
            InsectModelType::CommonHousefly => self.common_housefly.clone(),
            InsectModelType::GiantTermite => self.giant_termite.clone(),
            InsectModelType::LegBeetle => self.leg_beetle.clone(),
            InsectModelType::JewelBug => self.jewel_bug.clone(),
            InsectModelType::Stinkbug => self.stinkbug.clone(),
            InsectModelType::Termite => self.termite.clone(),

            // Newly added models (with fallbacks for problematic ones)
            InsectModelType::AnimatedPeacockMoth => self.animated_peacock_moth.clone(),
            InsectModelType::Aphid => self.aphid.clone(),
            InsectModelType::BlackWidowSpider => self.black_widow_spider.clone(),
            InsectModelType::ElephantHawkMoth => self.elephant_hawk_moth.clone(),
            InsectModelType::Flea => self.flea.clone(),
            InsectModelType::FlyingHornet => self.flying_hornet.clone(),
            InsectModelType::GoliathBirdeater => self.goliath_birdeater.clone(),
            InsectModelType::HawkmothLarvae => self.hawkmoth_larvae.clone(),
            InsectModelType::JapaneseRhinocerosBeetle => self.japanese_rhinoceros_beetle.clone(),
            InsectModelType::MantisTenoderaAridifolia => self.mantis_tenodera_aridifolia.clone(),
            InsectModelType::Mite => self.mite.clone(),
            InsectModelType::Moth => self.moth.clone(),
            InsectModelType::Tick => self.tick.clone(),
            InsectModelType::Dragonfly2 => self.dragonfly_2.clone(), // Fallback to mystery
            InsectModelType::UnknownSpecies2 => self.dragonfly_2.clone(), // Fallback to mystery 2
            InsectModelType::Woodlouse => self.woodlouse.clone(),

            // Environment objects
            InsectModelType::Mushrooms => self.mushrooms.clone(),
            InsectModelType::Grass => self.grass.clone(),
            InsectModelType::Grass2 => self.grass_2.clone(),
            InsectModelType::Hive => self.hive.clone(),
            InsectModelType::WoodStick => self.wood_stick.clone(),
            InsectModelType::SimpleGrassChunks => self.simple_grass_chunks.clone(),

            // Building objects
            InsectModelType::Anthill => self.anthill.clone(),

            // New environment objects
            InsectModelType::CherryBlossomTree => self.cherry_blossom_tree.clone(),
            InsectModelType::PineCone => self.pine_cone.clone(),
            InsectModelType::PlantsAssetSet => self.plants_asset_set.clone(),
            InsectModelType::BeechFern => self.beech_fern.clone(),
            InsectModelType::TreesPack => self.trees_pack.clone(),
            InsectModelType::RiverRock => self.river_rock.clone(),
            InsectModelType::SmallRocks => self.small_rocks.clone(),

            // Defensive catch-all: fallback to a basic model for any new types
            #[allow(unreachable_patterns)] // This is intentional for defensive programming
            _ => {
                warn!(
                    "Model handle not found for type: {:?}. Using bee model as fallback.",
                    model_type
                );
                self.bee.clone() // Safe fallback to a basic model that should always exist
            }
        }
    }
}

/// Component marker for entities that use GLB models instead of primitives.
///
/// This component is added to entities when they've been upgraded from
/// primitive shapes to GLB models. It prevents the upgrade system from
/// processing the same entity multiple times.
#[derive(Component)]
pub struct UseGLBModel;

/// Spawns an entity with a GLB insect model.
///
/// This is a convenience function for directly spawning entities with GLB models,
/// bypassing the primitive-to-GLB upgrade process.
///
/// # Arguments
///
/// * `commands` - Bevy commands for entity spawning
/// * `model_assets` - Resource containing all loaded model handles
/// * `model_type` - Which insect model to use
/// * `position` - World position for the entity
/// * `scale` - Scale factor (will use model-specific scale if default is provided)
///
/// # Returns
///
/// The entity ID of the spawned model
pub fn spawn_insect_model(
    commands: &mut Commands,
    model_assets: &Res<ModelAssets>,
    model_type: InsectModelType,
    position: Vec3,
    scale: f32,
) -> Entity {
    let model_handle = model_assets.get_model_handle(&model_type);

    // Use provided scale or model-specific scale if scale is the uniform default
    let final_scale = if (scale - crate::constants::models::UNIFORM_UNIT_SCALE).abs() < 0.01 {
        get_model_scale(&model_type)
    } else {
        scale
    };

    commands
        .spawn((
            SceneRoot(model_handle),
            Transform::from_translation(position)
                .with_scale(Vec3::splat(final_scale))
                .with_rotation(Quat::from_rotation_y(calculate_model_rotation(&model_type))),
            InsectModel {
                model_type: model_type.clone(),
                scale: final_scale,
            },
            UseGLBModel,
        ))
        .id()
}

/// Maps unit types to their corresponding insect models.
///
/// This function provides the thematic mapping between game units (e.g., WorkerAnt,
/// SoldierAnt) and their visual representation (e.g., Fourmi ant model).
///
/// # Design Philosophy
///
/// - Worker units use ant models for thematic consistency
/// - Combat units use aggressive-looking insects (beetles, spiders, wasps)
/// - Flying units use winged models (hornets, dragonflies, butterflies)
/// - Special units use unique or mysterious models
///
/// # Arguments
///
/// * `unit_type` - The game unit type to map
///
/// # Returns
///
/// The appropriate `InsectModelType` for visual representation
pub fn get_unit_insect_model(unit_type: &crate::core::components::UnitType) -> InsectModelType {
    match unit_type {
        // Worker units - ant models for thematic consistency
        crate::core::components::UnitType::WorkerAnt => InsectModelType::Fourmi,

        // Combat units - aggressive insects
        crate::core::components::UnitType::SoldierAnt => InsectModelType::Fourmi, // Soldier ant variant
        crate::core::components::UnitType::BeetleKnight => InsectModelType::RhinoBeetle, // Heavy armored unit
        crate::core::components::UnitType::BatteringBeetle => InsectModelType::Beetle, // Uses black_ox_beetle_small.glb

        // Flying units - winged models

        // Scout/Support units - specialized models
        crate::core::components::UnitType::ScoutAnt => InsectModelType::CairnsBirdwing, // Fast butterfly
        crate::core::components::UnitType::SpearMantis => InsectModelType::QueenFacedBug, // Mantis

        // Special unit types with unique models
        crate::core::components::UnitType::DragonFly => InsectModelType::DragonFly, // Dragonfly 2 model
        crate::core::components::UnitType::DefenderBug => InsectModelType::RolyPoly, // Defensive pill bug
        crate::core::components::UnitType::EliteSpider => InsectModelType::AnimatedSpider, // Predator

        // Units for previously unused models
        crate::core::components::UnitType::HoneyBee => InsectModelType::Bee, // Classic bee
        crate::core::components::UnitType::Scorpion => InsectModelType::Scorpion, // Scorpion
        crate::core::components::UnitType::SpiderHunter => InsectModelType::Spider, // Small spider
        crate::core::components::UnitType::WolfSpider => InsectModelType::WolfSpider, // Wolf spider

        // Units for newly added models
        crate::core::components::UnitType::Housefly => InsectModelType::CommonHousefly, // Housefly
        crate::core::components::UnitType::TermiteWorker => InsectModelType::Termite, // Regular termite
        crate::core::components::UnitType::TermiteWarrior => InsectModelType::GiantTermite, // Giant termite
        crate::core::components::UnitType::LegBeetle => InsectModelType::LegBeetle, // Leg beetle
        crate::core::components::UnitType::Stinkbug => InsectModelType::Stinkbug,   // Stinkbug

        // Expanded unit categories for multi-team system

        // Beetles family
        crate::core::components::UnitType::StagBeetle => InsectModelType::JapaneseRhinocerosBeetle, // Stag beetle
        crate::core::components::UnitType::DungBeetle => InsectModelType::Beetle, // Dung beetle
        crate::core::components::UnitType::RhinoBeetle => InsectModelType::RhinoBeetle, // Rhino beetle
        crate::core::components::UnitType::StinkBeetle => InsectModelType::Stinkbug, // Stink beetle variant
        crate::core::components::UnitType::JewelBug => InsectModelType::JewelBug, // Jewel bug

        // Mantids family
        crate::core::components::UnitType::CommonMantis => InsectModelType::QueenFacedBug, // Common mantis
        crate::core::components::UnitType::OrchidMantis => InsectModelType::MantisTenoderaAridifolia, // Orchid mantis

        // Fourmi (Ants) family variants
        crate::core::components::UnitType::RedAnt => InsectModelType::Fourmi, // Red ant
        crate::core::components::UnitType::BlackAnt => InsectModelType::Fourmi, // Black ant
        crate::core::components::UnitType::FireAnt => InsectModelType::Fourmi, // Fire ant
        crate::core::components::UnitType::SoldierFourmi => InsectModelType::Fourmi, // Soldier ant
        crate::core::components::UnitType::WorkerFourmi => InsectModelType::Fourmi, // Worker ant

        // Cephalopoda family (Isopods/Crustaceans)
        crate::core::components::UnitType::Pillbug => InsectModelType::RolyPoly, // Pillbug
        crate::core::components::UnitType::Dragonfly2 => InsectModelType::Dragonfly2, // Dragonfly2
        crate::core::components::UnitType::Woodlouse => InsectModelType::Woodlouse, // Woodlouse
        crate::core::components::UnitType::SandFleas => InsectModelType::Flea, // Sand fleas

        // Small creatures family
        crate::core::components::UnitType::Aphids => InsectModelType::Aphid, // Aphids
        crate::core::components::UnitType::Mites => InsectModelType::Mite, // Mites
        crate::core::components::UnitType::Ticks => InsectModelType::Tick, // Ticks
        crate::core::components::UnitType::Fleas => InsectModelType::Flea, // Fleas
        crate::core::components::UnitType::Lice => InsectModelType::Mite, // Lice (use mite model)

        // Butterflies family
        crate::core::components::UnitType::Moths => InsectModelType::Moth, // Moths
        crate::core::components::UnitType::Caterpillars => InsectModelType::HawkmothLarvae, // Caterpillars
        crate::core::components::UnitType::PeacockMoth => InsectModelType::AnimatedPeacockMoth, // Peacock moth

        // Spiders family
        crate::core::components::UnitType::WidowSpider => InsectModelType::BlackWidowSpider, // Widow spider
        crate::core::components::UnitType::WolfSpiderVariant => InsectModelType::WolfSpider, // Wolf spider variant
        crate::core::components::UnitType::Tarantula => InsectModelType::GoliathBirdeater, // Tarantula
        crate::core::components::UnitType::DaddyLongLegs => InsectModelType::Spider, // Daddy long legs

        // Flies family
        crate::core::components::UnitType::HouseflyVariant => InsectModelType::CommonHousefly, // Housefly variant
        crate::core::components::UnitType::Horsefly => InsectModelType::FlyingHornet, // Horsefly (use hornet model)
        crate::core::components::UnitType::Firefly => InsectModelType::UnknownSpecies2, // Firefly
        crate::core::components::UnitType::DragonFlies => InsectModelType::Meganeura, // Dragonflies
        crate::core::components::UnitType::Damselfly => InsectModelType::CairnsBirdwing, // Damselfly

        // Bees family
        crate::core::components::UnitType::Hornets => InsectModelType::Hornet, // Hornets
        crate::core::components::UnitType::Wasps => InsectModelType::FlyingHornet, // Wasps
        crate::core::components::UnitType::Bumblebees => InsectModelType::Bee, // Bumblebees
        crate::core::components::UnitType::Honeybees => InsectModelType::ApisMellifera, // Honeybees
        crate::core::components::UnitType::MurderHornet => InsectModelType::FlyingHornet, // Murder hornet

        // Termites family
        crate::core::components::UnitType::Earwigs => InsectModelType::Termite, // Earwigs

        // Individual species
        crate::core::components::UnitType::ScorpionVariant => InsectModelType::Scorpion, // Scorpion variant
        crate::core::components::UnitType::StickBugs => InsectModelType::MantisTenoderaAridifolia, // Stick bugs
        crate::core::components::UnitType::LeafBugs => InsectModelType::QueenFacedBug, // Leaf bugs
        crate::core::components::UnitType::Cicadas => InsectModelType::Dragonfly2, // ?
        crate::core::components::UnitType::Grasshoppers => InsectModelType::CairnsBirdwing, // Grasshoppers
        crate::core::components::UnitType::Cockroaches => InsectModelType::Beetle, // Cockroaches

        // Default fallback - high-quality honey bee
        _ => InsectModelType::ApisMellifera,
    }
}

/// Maps building types to their corresponding insect models.
///
/// This function provides thematic mapping for buildings, using insect models
/// that match the building's purpose and theme.
///
/// # Design Philosophy
///
/// - Main structures use large, imposing models (Queen → Mantis)
/// - Production buildings use thematic insects (Nursery → Ladybug)
/// - Military buildings use aggressive models (Warrior Chamber → Rhino Beetle)
/// - Resource buildings use functional insects (Fungal Garden → Mushrooms)
///
/// # Arguments
///
/// * `building_type` - The building type to map
///
/// # Returns
///
/// The appropriate `InsectModelType` for visual representation
pub fn get_building_insect_model(
    building_type: &crate::core::components::BuildingType,
) -> InsectModelType {
    use crate::core::components::BuildingType;

    match building_type {
        // Main structures use anthill
        BuildingType::Queen => InsectModelType::Anthill,
        BuildingType::Nursery => InsectModelType::Anthill,

        // Production buildings
        BuildingType::WarriorChamber => InsectModelType::Hive, // Bee hive for warrior barracks
        BuildingType::HunterChamber => InsectModelType::Anthill,
        BuildingType::Stable => InsectModelType::Anthill,

        // Resource buildings
        BuildingType::FungalGarden => InsectModelType::Anthill, // Changed to anthill model
        BuildingType::WoodProcessor => InsectModelType::Anthill,
        BuildingType::MineralProcessor => InsectModelType::Anthill,
        BuildingType::StorageChamber => InsectModelType::Anthill,
        BuildingType::EvolutionChamber => InsectModelType::Anthill,
        BuildingType::TradingPost => InsectModelType::Anthill,
        BuildingType::ChitinWall => InsectModelType::Anthill,
        BuildingType::GuardTower => InsectModelType::Anthill,
    }
}

/// Returns the appropriate scale factor for a specific insect model.
///
/// All models use uniform scaling (1.5) for consistent RTS gameplay, with the exception
/// of environment objects like mushrooms which can be larger for visual impact.
///
/// # Uniform Scaling Philosophy
///
/// Using consistent scale across all units provides:
/// - Predictable collision detection
/// - Balanced unit size perception
/// - Easier game balance (size doesn't affect power)
/// - Simplified pathfinding and spatial calculations
///
/// # Arguments
///
/// * `model_type` - The insect model type to get scaling for
///
/// # Returns
///
/// Scale factor (typically 1.5, except for environment objects)
pub fn get_model_scale(model_type: &InsectModelType) -> f32 {
    use crate::constants::models::*;

    match model_type {
        // Classic models - all standardized to 1.5 for uniform gameplay
        InsectModelType::Scorpion => SCORPION_SCALE, // 1.5
        InsectModelType::Bee => BEE_SCALE,           // 1.5
        InsectModelType::ApisMellifera => BEE_SCALE, // Use bee scale as fallback
        InsectModelType::Spider => SPIDER_SCALE,     // 1.5
        InsectModelType::WolfSpider => WOLF_SPIDER_SCALE, // 1.5
        InsectModelType::QueenFacedBug => QUEEN_FACED_BUG_SCALE, // 1.5
        InsectModelType::Beetle => BEETLE_SCALE,     // 1.5

        // New high-quality models - all standardized to 1.5
        InsectModelType::Meganeura => MEGANEURA_SCALE, // 1.5
        InsectModelType::AnimatedSpider => ANIMATED_SPIDER_SCALE, // 1.5
        InsectModelType::RhinoBeetle => RHINO_BEETLE_SCALE, // 1.5
        InsectModelType::Hornet => HORNET_SCALE,       // 1.5
        InsectModelType::Fourmi => FOURMI_SCALE,       // 1.5
        InsectModelType::CairnsBirdwing => CAIRNS_BIRDWING_SCALE, // 1.5
        InsectModelType::RolyPoly => ROLY_POLY_SCALE,  // 1.5
        InsectModelType::DragonFly => DRAGONFLY_SCALE, // 1.5
        InsectModelType::CommonHousefly => HOUSEFLY_SCALE, // Increased for better visibility
        InsectModelType::GiantTermite => UNIFORM_UNIT_SCALE, // 1.5
        InsectModelType::LegBeetle => UNIFORM_UNIT_SCALE, // 1.5
        InsectModelType::JewelBug => UNIFORM_UNIT_SCALE, // 1.5
        InsectModelType::Stinkbug => STINKBUG_SCALE,   // Increased for better visibility
        InsectModelType::Termite => UNIFORM_UNIT_SCALE, // 1.5

        // Newly added models - all use uniform scale for consistency
        InsectModelType::AnimatedPeacockMoth => crate::constants::models::ANIMATED_PEACOCK_MOTH_SCALE, // 25.0 (10x larger)
        InsectModelType::Aphid => 3.0,                  // Larger scale for tiny aphid visibility
        InsectModelType::BlackWidowSpider => UNIFORM_UNIT_SCALE, // 1.5
        InsectModelType::ElephantHawkMoth => crate::constants::models::ELEPHANT_HAWK_MOTH_SCALE, // 25.0 (10x larger)
        InsectModelType::Flea => 4.0,                   // Much larger scale for tiny flea visibility
        InsectModelType::FlyingHornet => crate::constants::models::FLYING_HORNET_SCALE, // 12.5 (5x larger)
        InsectModelType::GoliathBirdeater => 1.0, // 2x larger than before (0.5 × 2) - now 2.5x smaller than uniform scale
        InsectModelType::HawkmothLarvae => crate::constants::models::HAWKMOTH_LARVAE_SCALE, // 50.0 (20x larger)
        InsectModelType::JapaneseRhinocerosBeetle => crate::constants::models::JAPANESE_RHINOCEROS_BEETLE_SCALE, // 20.0 (8x larger)
        InsectModelType::MantisTenoderaAridifolia => UNIFORM_UNIT_SCALE, // 1.5
        InsectModelType::Mite => 5.0,                   // Very large scale for microscopic mite
        InsectModelType::Moth => crate::constants::models::MOTH_SCALE,    // 17.5 (7x larger)
        InsectModelType::Tick => 4.0,                   // Large scale for tiny tick visibility
        InsectModelType::Dragonfly2 => UNIFORM_UNIT_SCALE, // 1.5
        InsectModelType::UnknownSpecies2 => UNIFORM_UNIT_SCALE, // 1.5
        InsectModelType::Woodlouse => crate::core::constants::models::WOODLOUSE_SCALE, // 20x smaller

        // Environment objects - various scales for different types
        InsectModelType::Mushrooms => MUSHROOMS_SCALE, // 2.5
        InsectModelType::Grass => GRASS_SCALE,         // 1.0
        InsectModelType::Grass2 => GRASS_2_SCALE,      // 1.2
        InsectModelType::Hive => HIVE_SCALE,           // 3.0
        InsectModelType::WoodStick => WOOD_STICK_SCALE, // 1.5
        InsectModelType::SimpleGrassChunks => SIMPLE_GRASS_CHUNKS_SCALE, // 0.8

        // Building objects - interactive structures
        InsectModelType::Anthill => ANTHILL_SCALE, // 3.5

        // New environment objects - various scales for natural scenery
        InsectModelType::CherryBlossomTree => CHERRY_BLOSSOM_TREE_SCALE, // 4.0
        InsectModelType::PineCone => PINE_CONE_SCALE,                    // 1.0
        InsectModelType::PlantsAssetSet => PLANTS_ASSET_SET_SCALE,       // 1.5
        InsectModelType::BeechFern => BEECH_FERN_SCALE,                  // 2.0
        InsectModelType::TreesPack => TREES_PACK_SCALE,                  // 5.0
        InsectModelType::RiverRock => RIVER_ROCK_SCALE,                  // 2.5
        InsectModelType::SmallRocks => SMALL_ROCKS_SCALE,                // 1.2

        // Defensive catch-all: provide reasonable default scale for any new types
        #[allow(unreachable_patterns)] // This is intentional for defensive programming
        _ => {
            warn!(
                "Model scale not defined for type: {:?}. Using default scale of 2.0.",
                model_type
            );
            2.0 // Reasonable default scale for most objects
        }
    }
}

/// Defensive helper function to safely get a model handle with fallback.
///
/// This function provides a safe way to get model handles even for newly added
/// model types that might not be explicitly handled everywhere.
pub fn get_model_handle_safe(
    model_assets: &ModelAssets,
    model_type: &InsectModelType,
) -> Handle<Scene> {
    // Use the defensive method that includes fallbacks
    model_assets.get_model_handle(model_type) // This already has defensive fallback
}

/// Defensive helper function to safely get a model scale with fallback.
///
/// This function provides a safe way to get model scales even for newly added
/// model types that might not have specific scale constants defined.
pub fn get_model_scale_safe(model_type: &InsectModelType) -> f32 {
    get_model_scale(model_type) // This already has defensive fallback
}

/// System that sets up animation controllers for newly spawned GLB units.
///
/// This system runs when a unit is first upgraded to use a GLB model and adds
/// the necessary animation controller component. The controller manages animation
/// state transitions (idle, walking, attacking, dying) for units with animations.
///
/// # Process
///
/// 1. Detects units that just received the `UseGLBModel` component
/// 2. Creates appropriate animation clips for the unit type
/// 3. Adds `UnitAnimationController` component
/// 4. Animation player will be found later by `find_animation_players` system
///
/// # Safety
///
/// Uses `try_insert` to handle potential race conditions with the death system.
/// If an entity is destroyed between query and insertion, the operation fails gracefully.
pub fn setup_animation_controllers(
    mut commands: Commands,
    new_glb_units: Query<(Entity, &RTSUnit), (Added<UseGLBModel>, With<RTSUnit>, Without<Dying>)>,
) {
    for (entity, unit) in new_glb_units.iter() {
        // Only add animation controller to units with a specific type
        let Some(unit_type) = &unit.unit_type else {
            continue;
        };

        let animation_controller = UnitAnimationController {
            current_state: AnimationState::Idle,
            previous_state: AnimationState::Idle,
            animation_player: None, // Will be populated by find_animation_players system
            animation_node_index: None, // Will be populated by setup_glb_animations system
        };

        // Use try_insert to avoid panicking if entity was destroyed
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.try_insert(animation_controller);
            info!("Added animation controller to unit {:?}", unit_type);
        }
    }
}

/// System that prevents GLB model transform conflicts and visual jitter.
///
/// This system runs every frame to maintain stable transforms for GLB models,
/// preventing visual glitches that can occur when multiple systems try to
/// modify the same transform simultaneously.
///
/// # Issues Prevented
///
/// 1. **Scale Jitter**: Smoothly corrects scale drift from competing systems
/// 2. **Rotation Issues**: Ensures rotations stay normalized (unit quaternions)
/// 3. **Position Extremes**: Clamps positions to prevent astronomical coordinates
///
/// # Performance
///
/// Uses threshold checks to avoid unnecessary updates. Only modifies transforms
/// when deviations exceed acceptable tolerances.
pub fn smooth_glb_model_updates(
    mut glb_models: Query<
        (&mut Transform, &InsectModel, &RTSUnit),
        (With<UseGLBModel>, Without<Dying>),
    >,
    time: Res<Time>,
) {
    for (mut transform, model, _unit) in glb_models.iter_mut() {
        // Prevent scale jitter from competing systems
        smooth_scale_drift(&mut transform, model.scale, &time);

        // Ensure quaternion remains normalized
        normalize_rotation(&mut transform);

        // Prevent extreme positions from physics/movement bugs
        clamp_extreme_positions(&mut transform);
    }
}

/// Smoothly corrects scale drift to prevent jiggling.
///
/// Uses lerp to gradually adjust scale back to target, preventing
/// sudden jumps that would cause visual jitter.
fn smooth_scale_drift(transform: &mut Transform, target_scale: f32, time: &Time) {
    let target_scale_vec = Vec3::splat(target_scale);
    let scale_diff = (transform.scale - target_scale_vec).length();

    // Only adjust if drift is significant (threshold: 0.05)
    if scale_diff > 0.05 {
        // Slow adjustment to prevent interference with other systems
        transform.scale = transform
            .scale
            .lerp(target_scale_vec, 2.0 * time.delta_secs());
    }
}

/// Ensures rotation quaternion stays normalized.
///
/// Non-normalized quaternions can cause rendering issues and
/// unpredictable rotation behavior.
fn normalize_rotation(transform: &mut Transform) {
    if !transform.rotation.is_normalized() {
        transform.rotation = transform.rotation.normalize();
    }
}

/// Clamps positions to prevent astronomical coordinates.
///
/// Extremely large coordinates can occur from physics bugs or
/// floating-point precision issues. This prevents entities from
/// drifting into unreachable space.
fn clamp_extreme_positions(transform: &mut Transform) {
    const MAX_COORDINATE: f32 = 50000.0;

    let is_extreme = transform.translation.x.abs() > MAX_COORDINATE
        || transform.translation.z.abs() > MAX_COORDINATE;

    if is_extreme {
        warn!(
            "GLB model at extreme position {:?}, clamping",
            transform.translation
        );
        transform.translation.x = transform
            .translation
            .x
            .clamp(-MAX_COORDINATE, MAX_COORDINATE);
        transform.translation.z = transform
            .translation
            .z
            .clamp(-MAX_COORDINATE, MAX_COORDINATE);
    }
}

/// System that applies team colors to GLB model materials.
///
/// This system finds GLB models that have team colors but haven't been tinted yet,
/// then creates unique material instances for each entity to avoid shared material issues.
///
/// IMPORTANT: Each entity gets its own unique material instances to prevent color bleeding.
pub fn apply_team_colors_to_glb_models(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    team_color_query: Query<
        (Entity, &TeamColor, Option<&TeamColorRetryCount>),
        (With<UseGLBModel>, Without<TeamColorApplied>),
    >,
    all_mesh_query: Query<&MeshMaterial3d<StandardMaterial>>,
    children_query: Query<&Children>,
) {
    let entity_count = team_color_query.iter().count();
    if entity_count > 0 {
        debug!(
            "Processing {} entities for team color application",
            entity_count
        );
    }

    for (entity, team_color, retry_count_opt) in team_color_query.iter() {
        // Check if entity still exists before trying to modify it
        let Some(mut entity_commands) = commands.get_entity(entity) else {
            // Entity has been despawned, skip processing
            continue;
        };

        // Get retry count and update it
        let retry_count = if let Some(count) = retry_count_opt {
            let new_count = count.attempts + 1;
            entity_commands.insert(TeamColorRetryCount {
                attempts: new_count,
            });
            new_count
        } else {
            entity_commands.insert(TeamColorRetryCount { attempts: 1 });
            1
        };

        // Give up after reasonable attempts (about 2 seconds at 60fps)
        if retry_count > 120 {
            warn!(
                "Giving up on team color application for player {} after {} attempts",
                team_color.player_id, retry_count
            );
            // Check entity exists before inserting component
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(TeamColorApplied);
            }
            continue;
        }

        debug!(
            "Attempting to apply team color for player {} to entity {:?} (attempt {})",
            team_color.player_id, entity, retry_count
        );

        // Collect all material updates needed for this entity tree
        let mut material_updates = Vec::new();
        collect_material_updates_recursive(
            entity,
            &children_query,
            &all_mesh_query,
            &materials,
            &team_color.tint_color,
            &mut material_updates,
            0,
        );

        if !material_updates.is_empty() {
            // Apply all material updates
            for (entity_to_update, _old_handle, new_material) in material_updates {
                let new_handle = materials.add(new_material);
                // Check if entity exists before updating material
                if let Some(mut entity_commands) = commands.get_entity(entity_to_update) {
                    entity_commands.insert(MeshMaterial3d(new_handle));
                    debug!(
                        "Updated entity {:?} with new unique team-colored material",
                        entity_to_update
                    );
                }
            }

            // Mark as processed (check entity exists first)
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(TeamColorApplied);
                info!(
                    "Successfully applied team color tint to GLB model for player {} entity {:?}",
                    team_color.player_id, entity
                );
            }
        } else {
            // Don't mark as processed - let it retry next frame
            debug!(
                "No materials found yet for player {} entity {:?} - will retry next frame",
                team_color.player_id, entity
            );
        }
    }
}

/// Recursively collects materials that need to be updated with team colors
/// Returns a list of (entity, old_handle, new_material) tuples for batch processing
fn collect_material_updates_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &Assets<StandardMaterial>,
    tint_color: &Color,
    material_updates: &mut Vec<(Entity, Handle<StandardMaterial>, StandardMaterial)>,
    depth: usize,
) {
    debug!(
        "Checking entity {:?} at depth {} for materials",
        entity, depth
    );

    // Try to get the material handle from this entity
    if let Ok(mesh_material) = mesh_query.get(entity) {
        debug!("Found mesh material on entity {:?}", entity);
        if let Some(original_material) = materials.get(&mesh_material.0) {
            // Create a unique material instance for this entity
            let mut new_material = original_material.clone();

            // Apply team color tint to the cloned material
            let original_color = new_material.base_color;
            let tint_rgba = tint_color.to_srgba();
            let original_rgba = original_color.to_srgba();

            // Apply team color tint - blend with original color (strong tint for better visibility)
            let new_color = Color::srgba(
                (original_rgba.red * 0.4 + tint_rgba.red * 0.6).min(1.0),
                (original_rgba.green * 0.4 + tint_rgba.green * 0.6).min(1.0),
                (original_rgba.blue * 0.4 + tint_rgba.blue * 0.6).min(1.0),
                original_rgba.alpha,
            );
            new_material.base_color = new_color;

            // Add to updates list
            material_updates.push((entity, mesh_material.0.clone(), new_material));
            info!(
                "Prepared unique team-colored material for entity {:?}: {:?} -> {:?}",
                entity, original_color, new_color
            );
        }
    }

    // Recursively check children
    if let Ok(children) = children_query.get(entity) {
        for &child in children.iter() {
            collect_material_updates_recursive(
                child,
                children_query,
                mesh_query,
                materials,
                tint_color,
                material_updates,
                depth + 1,
            );
        }
    }
}

/// Marker component to prevent re-applying team colors
#[derive(Component)]
pub struct TeamColorApplied;
