#![allow(dead_code)] // Allow unused constants for future features
                     // Game configuration constants
                     // This module contains all the magic numbers and configuration values used throughout the game

// === WINDOW AND DISPLAY ===
pub const WINDOW_TITLE: &str = "Flexible Rust Game";
pub const WINDOW_WIDTH: f32 = 1280.0;
pub const WINDOW_HEIGHT: f32 = 720.0;

// === MOVEMENT SYSTEM ===
pub mod movement {
    // Map boundaries - playable area limits
    pub const MAP_BOUNDARY: f32 = 1000.0; // Map extends from -1000 to +1000 (total 2000 units)
    pub const BOUNDARY_PUSH_FORCE: f32 = 200.0; // Force applied when near boundary
    pub const BOUNDARY_BUFFER: f32 = 50.0; // Distance from boundary where push force starts

    // Camera and terrain boundaries (slightly larger than map boundary)
    pub const CAMERA_BOUNDARY: f32 = 1200.0; // Camera can go 200 units beyond map boundary
    pub const TERRAIN_BOUNDARY: f32 = 1500.0; // Terrain generates 500 units beyond map boundary

    // Safety limits to prevent units from going to astronomical positions
    pub const MAX_POSITION: f32 = 100000.0; // Extreme position safety net (beyond map boundary)
    pub const MAX_VELOCITY: f32 = 500.0; // Increased to 500.0 to allow very fast units like DragonFly (400.0 max_speed)
    pub const MAX_DISTANCE: f32 = 50000.0;

    // Movement physics
    pub const UNIT_SPEED: f32 = 80.0; // 2x speed increase
    pub const ARRIVAL_THRESHOLD: f32 = 2.0; // Slightly larger threshold for faster units
    pub const DECELERATION_FACTOR: f32 = 2.5; // Slightly higher deceleration

    // Collision and separation
    pub const SEPARATION_MULTIPLIER: f32 = 2.0; // Units separate at 2x their radius for better flow
    pub const SEPARATION_FORCE_STRENGTH: f32 = 2.0; // Further reduced force for smoother group movement

    // Default spawn height
    pub const DEFAULT_SPAWN_HEIGHT: f32 = 10.0;
    pub const TERRAIN_SAMPLE_LIMIT: f32 = 10000.0;
}

// === CAMERA SYSTEM ===
pub mod camera {
    // Camera height constraints relative to terrain
    pub const MIN_HEIGHT_ABOVE_TERRAIN: f32 = 50.0; // Never go below 5 units above terrain
    pub const MAX_HEIGHT_ABOVE_TERRAIN: f32 = 1000.0; // Never go above 500 units above terrain
    pub const ZOOM_SPEED_BASE: f32 = 15.0; // Base zoom speed (unused for scroll wheel)
    pub const CAMERA_MOVE_SPEED: f32 = 250.0; // Base camera movement speed

    // Scroll wheel zoom settings
    pub const SCROLL_ZOOM_SENSITIVITY: f32 = 2.0; // Units of movement per scroll wheel click (reduced for better control)
    pub const ZOOM_SPEED_FAST_MULTIPLIER: f32 = 5.0; // Shift key multiplier (reduced)
    pub const ZOOM_SPEED_HYPER_MULTIPLIER: f32 = 15.0; // Alt key multiplier (reduced)

    // Camera look sensitivity
    pub const LOOK_SENSITIVITY: f32 = 0.02;
    pub const PITCH_LIMIT: f32 = 1.5; // Maximum pitch angle in radians
}

// === COLLISION SYSTEM ===
pub mod collision {
    // Collision radii for different unit
    pub const WORKER_ANT_COLLISION_RADIUS: f32 = 6.0; // Increased from 2.0 for better spacing
    pub const SOLDIER_ANT_COLLISION_RADIUS: f32 = 7.0; // Increased from 3.0 for better spacing
    pub const HUNTER_WASP_COLLISION_RADIUS: f32 = 6.5; // Increased from 2.5 for better spacing
    pub const BEETLE_KNIGHT_COLLISION_RADIUS: f32 = 8.0; // Increased from 4.0 for better spacing
    pub const DEFAULT_UNIT_COLLISION_RADIUS: f32 = 10.0;

    // Building collision radii
    pub const NURSERY_COLLISION_RADIUS: f32 = 8.0; // Increased for building spacing
    pub const WARRIOR_CHAMBER_COLLISION_RADIUS: f32 = 10.0; // Increased for building spacing
    pub const QUEEN_COLLISION_RADIUS: f32 = 12.0; // Increased for building spacing
    pub const DEFAULT_BUILDING_COLLISION_RADIUS: f32 = 8.0;
}

// === UI CONSTANTS ===
pub mod ui {
    use bevy::prelude::*;

    // Top resource bar
    pub const RESOURCE_BAR_HEIGHT: f32 = 60.0;
    pub const RESOURCE_BAR_PADDING: f32 = 10.0;
    pub const RESOURCE_ICON_SIZE: f32 = 20.0;
    pub const RESOURCE_TEXT_SIZE: f32 = 16.0;

    // Bottom building panel
    pub const BUILDING_PANEL_HEIGHT: f32 = 300.0;
    pub const BUILDING_PANEL_PADDING: f32 = 10.0;
    pub const BUILDING_PANEL_BUILDINGS_WIDTH: f32 = 70.0;
    pub const BUILDING_PANEL_UNITS_WIDTH: f32 = 30.0;

    // Building buttons
    pub const BUILDING_BUTTON_SIZE: f32 = 80.0;
    pub const BUILDING_BUTTON_BORDER: f32 = 2.0;
    pub const BUILDING_BUTTON_ICON_SIZE: f32 = 24.0;
    pub const BUILDING_BUTTON_TEXT_SIZE: f32 = 10.0;

    // Unit buttons
    pub const UNIT_BUTTON_HEIGHT: f32 = 10.0;
    pub const UNIT_BUTTON_BORDER: f32 = 1.0;
    pub const UNIT_BUTTON_PADDING: f32 = 5.0;
    pub const UNIT_BUTTON_TEXT_SIZE: f32 = 8.0;

    // Panel titles
    pub const PANEL_TITLE_SIZE: f32 = 18.0;

    // Layout spacing
    pub const BUTTON_GAP: f32 = 5.0;

    // Colors
    pub const BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
    pub const PANEL_COLOR: Color = Color::srgba(0.15, 0.15, 0.15, 0.9);
    pub const BUTTON_COLOR: Color = Color::srgba(0.2, 0.2, 0.2, 0.8);
    pub const BUTTON_HOVER_COLOR: Color = Color::srgba(0.3, 0.3, 0.3, 0.8);
    pub const UNIT_BUTTON_COLOR: Color = Color::srgba(0.3, 0.3, 0.3, 0.8);
    pub const UNIT_BUTTON_HOVER_COLOR: Color = Color::srgba(0.4, 0.4, 0.4, 0.8);
    pub const BORDER_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
    pub const BUTTON_BORDER_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
    pub const PANEL_BORDER_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
    pub const UNIT_BORDER_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);
    pub const TEXT_COLOR: Color = Color::WHITE;
}

// === TEAM COLORS ===
pub mod team_colors {
    use bevy::prelude::*;

    // Team color tints for GLB models and materials (very strong vibrant colors for visibility)
    pub const PLAYER_1_TINT: Color = Color::srgba(0.1, 0.3, 1.0, 1.0); // Bright Cyan-Blue
    pub const PLAYER_2_TINT: Color = Color::srgba(1.0, 0.1, 0.1, 1.0); // Bright Red
    pub const PLAYER_3_TINT: Color = Color::srgba(0.1, 1.0, 0.1, 1.0); // Bright Green
    pub const PLAYER_4_TINT: Color = Color::srgba(1.0, 0.9, 0.1, 1.0); // Bright Yellow
    pub const PLAYER_5_TINT: Color = Color::srgba(1.0, 0.1, 1.0, 1.0); // Bright Magenta
    pub const PLAYER_6_TINT: Color = Color::srgba(0.1, 1.0, 1.0, 1.0); // Bright Cyan
    pub const PLAYER_7_TINT: Color = Color::srgba(1.0, 0.5, 0.1, 1.0); // Bright Orange
    pub const PLAYER_8_TINT: Color = Color::srgba(0.6, 0.1, 1.0, 1.0); // Bright Purple
    pub const UNKNOWN_PLAYER_TINT: Color = Color::srgba(0.5, 0.5, 0.5, 1.0); // Dark Gray

    // Team colors for primitive models (very vibrant colors for strong visual distinction)
    pub const PLAYER_1_PRIMITIVE: Color = Color::srgb(0.1, 0.4, 1.0); // Bright cyan-blue
    pub const PLAYER_2_PRIMITIVE: Color = Color::srgb(1.0, 0.1, 0.1); // Bright red
    pub const PLAYER_3_PRIMITIVE: Color = Color::srgb(0.1, 1.0, 0.1); // Bright green
    pub const PLAYER_4_PRIMITIVE: Color = Color::srgb(1.0, 0.9, 0.1); // Bright yellow
    pub const PLAYER_5_PRIMITIVE: Color = Color::srgb(1.0, 0.1, 1.0); // Bright magenta
    pub const PLAYER_6_PRIMITIVE: Color = Color::srgb(0.1, 1.0, 1.0); // Bright cyan
    pub const PLAYER_7_PRIMITIVE: Color = Color::srgb(1.0, 0.5, 0.1); // Bright orange
    pub const PLAYER_8_PRIMITIVE: Color = Color::srgb(0.6, 0.1, 1.0); // Bright purple
    pub const UNKNOWN_PLAYER_PRIMITIVE: Color = Color::srgb(0.6, 0.6, 0.6); // Light gray
}

// === BUILDING SYSTEM ===
pub mod buildings {
    use bevy::prelude::*;

    // Building dimensions (width, height, depth)
    pub const NURSERY_SIZE: Vec3 = Vec3::new(4.0, 3.0, 4.0);
    pub const WARRIOR_CHAMBER_SIZE: Vec3 = Vec3::new(6.0, 4.0, 6.0);
    pub const HUNTER_CHAMBER_SIZE: Vec3 = Vec3::new(5.0, 4.0, 5.0);
    pub const FUNGAL_GARDEN_SIZE: Vec3 = Vec3::new(8.0, 1.0, 8.0);
    pub const DEFAULT_BUILDING_SIZE: Vec3 = Vec3::new(4.0, 3.0, 4.0);

    // Building colors
    pub const NURSERY_COLOR: Color = Color::srgb(0.8, 0.6, 0.4);
    pub const WARRIOR_CHAMBER_COLOR: Color = Color::srgb(0.6, 0.6, 0.6);
    pub const HUNTER_CHAMBER_COLOR: Color = Color::srgb(0.4, 0.6, 0.4);
    pub const FUNGAL_GARDEN_COLOR: Color = Color::srgb(0.3, 0.8, 0.3);
    pub const DEFAULT_BUILDING_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
    pub const PREVIEW_COLOR: Color = Color::srgba(0.5, 0.5, 1.0, 0.5);

    // Building stats
    pub const DEFAULT_BUILDING_HEALTH: f32 = 500.0;
    pub const DEFAULT_BUILDING_ARMOR: f32 = 5.0;
    pub const CONSTRUCTION_PROGRESS_MAX: f32 = 100.0;
}

// === MODEL SCALING ===
pub mod models {
    // === UNIFORM MODEL SCALING FOR CONSISTENCY ===
    // All models use the same scale for uniform appearance in RTS gameplay
    pub const UNIFORM_UNIT_SCALE: f32 = 2.5; // Standard scale for all unit models (increased)
    pub const UNIFORM_BUILDING_SCALE: f32 = 3.0; // Standard scale for all building models (increased)

    // === INDIVIDUAL MODEL SCALES
    // Classic models - all increased
    pub const SCORPION_SCALE: f32 = 8.0; // Increased from 5.5 for better visibility
    pub const BEE_SCALE: f32 = 1.2; // Increased from 0.5 for better visibility
    pub const SPIDER_SCALE: f32 = 2.5;
    pub const MANTIS_SCALE: f32 = 5.0; // Increased from 3.5 for better visibility
    pub const APIS_MELLIFERA_SCALE: f32 = 2.5;
    pub const BEETLE_SCALE: f32 = 2.5;
    pub const LADYBUG_SCALE: f32 = 2.5;

    // New models
    pub const MEGANEURA_SCALE: f32 = 25.0; // Massively increased - 10x bigger (dragonfly)
    pub const ANIMATED_SPIDER_SCALE: f32 = 2.5;
    pub const RHINO_BEETLE_SCALE: f32 = 2.5;
    pub const HORNET_SCALE: f32 = 5.0;
    pub const FOURMI_SCALE: f32 = 2.5;
    pub const CAIRNS_BIRDWING_SCALE: f32 = 8.0; // Increased much more for better visibility
    pub const LADYBUG_LOWPOLY_SCALE: f32 = 0.2;
    pub const ROLY_POLY_SCALE: f32 = 0.0005; // Made smaller to fix oversized model
    pub const DRAGONFLY_SCALE: f32 = 200.0; // Further increased for better visibility
    pub const WOLF_SPIDER_SCALE: f32 = 2.5;
    pub const QUEEN_FACED_BUG_SCALE: f32 = 8.0; // Increased much more for better visibility
    pub const HOUSEFLY_SCALE: f32 = 4.0; // Increased from UNIFORM_UNIT_SCALE (2.5) for better visibility
    pub const STINKBUG_SCALE: f32 = 5.0; // Increased from UNIFORM_UNIT_SCALE (2.5) for better visibility
    pub const FLYING_HORNET_SCALE: f32 = 0.625; // 20x smaller than 12.5 (12.5 ÷ 20)
    pub const MOTH_SCALE: f32 = 17.5; // 7x larger than UNIFORM_UNIT_SCALE (2.5 × 7)
    pub const ELEPHANT_HAWK_MOTH_SCALE: f32 = 25.0; // 10x larger than UNIFORM_UNIT_SCALE (2.5 × 10)
    pub const ANIMATED_PEACOCK_MOTH_SCALE: f32 = 25.0; // 10x larger than UNIFORM_UNIT_SCALE (2.5 × 10)
    pub const HAWKMOTH_LARVAE_SCALE: f32 = 50.0; // 20x larger than UNIFORM_UNIT_SCALE (2.5 × 20)
    pub const WOODLOUSE_SCALE: f32 = 0.125; // 20x smaller than UNIFORM_UNIT_SCALE (2.5 ÷ 20)

    // Environment object scales
    pub const MUSHROOMS_SCALE: f32 = 20.5; // Larger scale for environment objects
    pub const GRASS_SCALE: f32 = 50.0; // Natural grass scale
    pub const GRASS_2_SCALE: f32 = 10.2; // Slightly larger grass variant
    pub const HIVE_SCALE: f32 = 0.02; // Moderate hive structure (adjusted for visibility)
    pub const WOOD_STICK_SCALE: f32 = 1.5; // Small debris scale
    pub const SIMPLE_GRASS_CHUNKS_SCALE: f32 = 1.2; // Compact grass chunks (increased for visibility)

    // Building object scales
    pub const ANTHILL_SCALE: f32 = 21.0; // Anthill building scale - 2x larger for better visibility

    // New environment object scales (increased for visibility)
    pub const CHERRY_BLOSSOM_TREE_SCALE: f32 = 2.0; // Beautiful tree landmark (increased for visibility)
    pub const PINE_CONE_SCALE: f32 = 5.0; // Large pine cone size for resource visibility (5x bigger)
    pub const PLANTS_ASSET_SET_SCALE: f32 = 1.5; // Moderate plant collection scale (increased for visibility)
    pub const BEECH_FERN_SCALE: f32 = 1.5; // Medium fern undergrowth (increased for visibility)
    pub const TREES_PACK_SCALE: f32 = 2.5; // Tree landmarks (increased for visibility)
    pub const RIVER_ROCK_SCALE: f32 = 10.5; // Rock formations (increased for visibility)
    pub const SMALL_ROCKS_SCALE: f32 = 1.0; // Small scattered stones (increased for visibility)
}

// === RESOURCE SYSTEM ===
pub mod resources {
    // Starting resources (balanced to match AI starting resources)
    pub const STARTING_NECTAR: f32 = 200.0;
    pub const STARTING_CHITIN: f32 = 60.0;
    pub const STARTING_MINERALS: f32 = 30.0;
    pub const STARTING_PHEROMONES: f32 = 90.0;
    pub const STARTING_POPULATION_LIMIT: u32 = 200;

    // Resource costs for buildings (using new theme)
    pub const NURSERY_CHITIN_COST: f32 = 1.0;
    pub const WARRIOR_CHAMBER_CHITIN_COST: f32 = 1.0;
    pub const WARRIOR_CHAMBER_MINERALS_COST: f32 = 1.0;
    pub const HUNTER_CHAMBER_CHITIN_COST: f32 = 1.0;
    pub const FUNGAL_GARDEN_CHITIN_COST: f32 = 1.0;
    pub const WOOD_PROCESSOR_CHITIN_COST: f32 = 1.0;
    pub const MINERAL_PROCESSOR_CHITIN_COST: f32 = 1.0;
    pub const DEFAULT_BUILDING_CHITIN_COST: f32 = 1.0;

    // Resource costs for units (using new theme)
    pub const WORKER_ANT_NECTAR_COST: f32 = 1.0;
    pub const SOLDIER_ANT_NECTAR_COST: f32 = 1.0;
    pub const SOLDIER_ANT_PHEROMONES_COST: f32 = 1.0;
    pub const HUNTER_WASP_CHITIN_COST: f32 = 1.0;
    pub const HUNTER_WASP_PHEROMONES_COST: f32 = 1.0;

    // Housing values
    pub const NURSERY_POPULATION_CAPACITY: u32 = 5;
    pub const QUEEN_POPULATION_CAPACITY: u32 = 5;
}

// === AI SYSTEM ===
pub mod ai {
    // AI decision timing
    pub const AI_DECISION_INTERVAL_SECS: f32 = 5.0;

    // Resource generation rates for AI (per second)
    pub const AI_FOOD_RATE: f32 = 2.0;
    pub const AI_WOOD_RATE: f32 = 1.5;
    pub const AI_STONE_RATE: f32 = 0.5;
    pub const AI_GOLD_RATE: f32 = 0.8;

    // AI building thresholds
    pub const AI_MIN_WORKERS_FOR_WARRIOR_CHAMBER: i32 = 5;
    pub const AI_MAX_MILITARY_UNITS_EARLY: i32 = 3;
    pub const AI_MAX_MILITARY_UNITS_MID: i32 = 5;
    pub const AI_MIN_MILITARY_FOR_ATTACK: i32 = 3;
    pub const AI_MIN_MILITARY_FOR_DEFEND: i32 = 2;

    // AI spawn positions
    pub const AI_SPAWN_RANGE: f32 = 50.0;
    pub const AI_SPAWN_HEIGHT: f32 = 5.0;
}

// === COMBAT SYSTEM ===
pub mod combat {
    // Unit spawn positions
    pub const UNIT_SPAWN_RANGE: f32 = 10.0;
    pub const UNIT_SPAWN_OFFSET: f32 = 5.0;
}

// === POPULATION MANAGEMENT ===
pub mod population {
    use std::time::Duration;

    pub const UPDATE_INTERVAL: Duration = Duration::from_secs(1);
}

// === RESOURCE INTERACTION ===
pub mod resource_interaction {
    // Resource selection and gathering distances (increased for better usability)
    pub const RESOURCE_CLICK_RADIUS: f32 = 150.0; // Distance for right-clicking to target resources (reduced by half)
    pub const GATHERING_DISTANCE: f32 = 20.0; // Distance within which gathering occurs
    pub const DROPOFF_TRAVEL_DISTANCE: f32 = 30.0; // Distance threshold for delivering resources

    // Resource source collision
    pub const RESOURCE_COLLISION_RADIUS: f32 = 3.0; // Collision radius of resource sources
}

// === TERRAIN SYSTEM ===
pub mod terrain {
    // Terrain generation limits
    pub const MAX_TERRAIN_COORDINATE: f32 = 10000.0;

    // Chunk management
    pub const CHUNK_SIZE: f32 = 64.0;
    pub const VIEW_DISTANCE: f32 = 200.0;
}

// === GRID SYSTEM ===
pub mod grid {
    use bevy::prelude::*;

    // Grid spacing and rendering
    pub const GRID_SPACING: f32 = 50.0; // Grid lines every 50 units
    pub const GRID_SIZE: f32 = 2000.0; // Total grid size (extends from -1000 to +1000)
    pub const GRID_LINE_WIDTH: f32 = 0.5; // Width of grid lines
    pub const GRID_HEIGHT: f32 = 1.0; // Height above terrain

    // Grid colors
    pub const GRID_COLOR: Color = Color::srgba(0.5, 0.5, 0.5, 0.3); // Semi-transparent gray
    pub const GRID_MAJOR_COLOR: Color = Color::srgba(0.7, 0.7, 0.7, 0.5); // Slightly brighter for major lines
}

// === INPUT HOTKEYS ===
pub mod hotkeys {
    use bevy::prelude::KeyCode;

    pub const BUILD_WARRIOR_CHAMBER: KeyCode = KeyCode::KeyB;
    pub const BUILD_NURSERY: KeyCode = KeyCode::KeyH;
    pub const BUILD_FUNGAL_GARDEN: KeyCode = KeyCode::KeyF;
    pub const CANCEL_BUILD: KeyCode = KeyCode::Escape;

    // Time control hotkeys
    pub const SPEED_UP: KeyCode = KeyCode::Equal; // + key
    pub const SLOW_DOWN: KeyCode = KeyCode::Minus; // - key
    pub const RESET_SPEED: KeyCode = KeyCode::Backspace; // Reset to normal speed

    // Debug/utility hotkeys
    pub const TOGGLE_GRID: KeyCode = KeyCode::KeyL; // Toggle grid lines
}

// === BUILDING PLACEMENT ===
pub mod building_placement {
    // Minimum spacing between buildings to prevent overlap
    pub const MIN_SPACING_BETWEEN_BUILDINGS: f32 = 2.0;

    // Minimum spacing from units when placing buildings
    pub const MIN_SPACING_FROM_UNITS: f32 = 5.0;

    // Minimum spacing from environment objects (rocks, trees, etc.)
    pub const MIN_SPACING_FROM_ENVIRONMENT: f32 = 3.0;

    // Visual feedback colors for placement preview
    pub const VALID_PLACEMENT_COLOR: bevy::prelude::Color =
        bevy::prelude::Color::srgba(0.5, 1.0, 0.5, 0.5);
    pub const INVALID_PLACEMENT_COLOR: bevy::prelude::Color =
        bevy::prelude::Color::srgba(1.0, 0.5, 0.5, 0.5);
}

// === UI STYLING ===
pub mod ui_styling {
    use bevy::prelude::Color;

    // Health bar dimensions
    pub const HEALTH_BAR_WIDTH: f32 = 3.5;
    pub const HEALTH_BAR_HEIGHT: f32 = 0.6;
    pub const HEALTH_BAR_Y_OFFSET: f32 = 3.0;
    pub const HEALTH_BAR_FOREGROUND_OFFSET: f32 = 0.1;

    // Health bar colors
    pub const HEALTH_BAR_BACKGROUND_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);
    pub const HEALTH_BAR_HEALTHY_COLOR: Color = Color::srgb(0.2, 0.8, 0.2);
    pub const HEALTH_BAR_WOUNDED_COLOR: Color = Color::srgb(1.0, 0.8, 0.0);
    pub const HEALTH_BAR_CRITICAL_COLOR: Color = Color::srgb(1.0, 0.2, 0.2);

    // Health status thresholds
    pub const HEALTH_THRESHOLD_HEALTHY: f32 = 0.8; // 80%+
    pub const HEALTH_THRESHOLD_WOUNDED: f32 = 0.4; // 40-79%
                                                   // Below HEALTH_THRESHOLD_WOUNDED is critical (0-39%)
}
