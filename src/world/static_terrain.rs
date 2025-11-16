//! Static Preloaded Terrain System
//! 
//! Simple, efficient terrain system designed specifically for RTS games.
//! Preloads the entire map at startup for instant camera response and zero lag.

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

// Terrain configuration optimized for RTS gameplay - extremely minimal for testing
pub const NOISE_HEIGHT: f32 = 15.0; // Moderate terrain height variation
pub const NOISE_SCALE: f32 = 400.0; // Terrain feature scale

pub struct StaticTerrainPlugin;

impl Plugin for StaticTerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainAssets::default())
            .insert_resource(StaticTerrainHeights::default())
            .add_systems(Startup, load_terrain_texture)
            .add_systems(PostStartup, generate_static_terrain);
    }
}

#[derive(Resource, Default)]
struct TerrainAssets {
    material: Option<Handle<StandardMaterial>>,
}

/// Simple terrain height provider for static terrain
#[derive(Resource)]
pub struct StaticTerrainHeights {
    noise: Perlin,
}

impl Default for StaticTerrainHeights {
    fn default() -> Self {
        Self {
            noise: Perlin::new(42),
        }
    }
}

impl StaticTerrainHeights {
    /// Get terrain height at any world position
    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        sample_terrain_height(&self.noise, x, z)
    }
}

#[derive(Component)]
pub struct TerrainChunk {
}

/// Load terrain texture at startup
fn load_terrain_texture(
    mut terrain_assets: ResMut<TerrainAssets>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let texture_handle = asset_server.load("textures/dirt.jpg");
    
    let material = StandardMaterial {
        base_color_texture: Some(texture_handle),
        base_color: Color::srgb(0.8, 0.9, 0.8), // Slight green tint
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    };
    
    let material_handle = materials.add(material);
    terrain_assets.material = Some(material_handle);
}

/// Generate a single minimal terrain chunk for testing
fn generate_static_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_assets: Res<TerrainAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Use Bevy's built-in plane mesh - cover full map area using unified size constant
    use crate::constants::movement::TERRAIN_SIZE;
    let terrain_diameter = TERRAIN_SIZE * 2.0; // Convert from radius to diameter
    
    info!("🌍 Generating terrain covering full map area ({} x {} units)...", terrain_diameter, terrain_diameter);
    let mesh_handle = meshes.add(Mesh::from(Plane3d::default().mesh().size(terrain_diameter, terrain_diameter)));
    
    // Use proper dirt texture or fallback to brown
    let material_handle = if let Some(texture_material) = &terrain_assets.material {
        texture_material.clone()
    } else {
        let terrain_material = StandardMaterial {
            base_color: Color::srgb(0.6, 0.4, 0.3), // Brown dirt color
            perceptual_roughness: 0.9,
            metallic: 0.0,
            ..default()
        };
        materials.add(terrain_material)
    };
    
    // Spawn single terrain chunk entity
    let entity = commands
        .spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)), // At ground level
            TerrainChunk {},
            Name::new("Terrain"),
        ))
        .id();
    
    info!("✅ Static terrain generation complete - spawned entity {:?}", entity);
}

/// Generate a simple flat mesh for testing


/// Sample terrain height at world coordinates
pub fn sample_terrain_height(noise: &Perlin, x: f32, z: f32) -> f32 {
    let nx = (x / NOISE_SCALE) as f64;
    let nz = (z / NOISE_SCALE) as f64;
    
    // Multi-octave noise for varied terrain
    let height1 = noise.get([nx, nz]) as f32 * NOISE_HEIGHT;
    let height2 = noise.get([nx * 2.0, nz * 2.0]) as f32 * NOISE_HEIGHT * 0.5;
    let height3 = noise.get([nx * 4.0, nz * 4.0]) as f32 * NOISE_HEIGHT * 0.25;
    
    height1 + height2 + height3
}

