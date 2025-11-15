//! Static Preloaded Terrain System
//! 
//! Simple, efficient terrain system designed specifically for RTS games.
//! Preloads the entire map at startup for instant camera response and zero lag.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use noise::{NoiseFn, Perlin};

// Terrain configuration optimized for RTS gameplay - extremely minimal for testing
pub const CHUNK_SIZE: f32 = 3000.0; // Huge chunks for minimal chunk count
pub const CHUNK_RESOLUTION: u32 = 8; // Very low resolution for fastest generation
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
    texture: Option<Handle<Image>>,
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
    pub chunk_x: i32,
    pub chunk_z: i32,
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
    info!("🌍 Generating minimal static terrain...");

    // Use Bevy's built-in plane mesh - cover full terrain boundary (3000x3000)
    let mesh_handle = meshes.add(Mesh::from(Plane3d::default().mesh().size(3000.0, 3000.0)));
    
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
            TerrainChunk { chunk_x: 0, chunk_z: 0 },
            Name::new("Terrain"),
        ))
        .id();
    
    info!("✅ Static terrain generation complete - spawned entity {:?}", entity);
}

/// Generate a simple flat mesh for testing
fn generate_minimal_flat_mesh() -> Mesh {
    info!("🔧 Creating minimal flat mesh...");
    
    let size = 500.0; // Much smaller for easier testing
    
    // Just 4 vertices for a flat square at Y=50 (raised up)
    let vertices = vec![
        [-size, 50.0, -size], // Bottom left
        [size, 50.0, -size],  // Bottom right
        [size, 50.0, size],   // Top right
        [-size, 50.0, size],  // Top left
    ];
    
    // Counter-clockwise winding for proper face culling
    let indices = vec![
        0, 2, 1, // First triangle (counter-clockwise)
        0, 3, 2, // Second triangle (counter-clockwise)
    ];
    
    let normals = vec![
        [0.0, 1.0, 0.0], // All pointing up
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    
    let uvs = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];
    
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    
    info!("🔧 Mesh created with {} vertices: {:?}", vertices.len(), vertices);
    
    mesh
}

/// Generate mesh for a single terrain chunk
fn generate_chunk_mesh(noise: &Perlin, chunk_x: f32, chunk_z: f32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    let step_size = CHUNK_SIZE / CHUNK_RESOLUTION as f32;
    
    // Generate vertices
    for z in 0..=CHUNK_RESOLUTION {
        for x in 0..=CHUNK_RESOLUTION {
            let world_x = chunk_x + x as f32 * step_size - CHUNK_SIZE * 0.5;
            let world_z = chunk_z + z as f32 * step_size - CHUNK_SIZE * 0.5;
            
            // Sample noise for height
            let height = sample_terrain_height(noise, world_x, world_z);
            
            vertices.push([world_x, height, world_z]);
            normals.push([0.0, 1.0, 0.0]); // Will be recalculated below
            
            // UV coordinates for tiling
            let u = (x as f32 / CHUNK_RESOLUTION as f32) * 4.0; // 4x texture tiling
            let v = (z as f32 / CHUNK_RESOLUTION as f32) * 4.0;
            uvs.push([u, v]);
        }
    }
    
    // Generate indices for triangles
    for z in 0..CHUNK_RESOLUTION {
        for x in 0..CHUNK_RESOLUTION {
            let i = z * (CHUNK_RESOLUTION + 1) + x;
            
            // First triangle
            indices.push(i);
            indices.push(i + CHUNK_RESOLUTION + 1);
            indices.push(i + 1);
            
            // Second triangle
            indices.push(i + 1);
            indices.push(i + CHUNK_RESOLUTION + 1);
            indices.push(i + CHUNK_RESOLUTION + 2);
        }
    }
    
    // Use simple upward normals for now (much faster)
    for normal in normals.iter_mut() {
        *normal = [0.0, 1.0, 0.0];
    }
    
    // Create mesh
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    
    mesh
}

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

/// Calculate smooth normals for terrain mesh
fn calculate_normals(vertices: &[[f32; 3]], indices: &[u32], normals: &mut [[f32; 3]]) {
    // Reset normals
    for normal in normals.iter_mut() {
        *normal = [0.0, 0.0, 0.0];
    }
    
    // Accumulate face normals
    for triangle in indices.chunks(3) {
        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;
        
        let v0 = Vec3::from_array(vertices[i0]);
        let v1 = Vec3::from_array(vertices[i1]);
        let v2 = Vec3::from_array(vertices[i2]);
        
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = edge1.cross(edge2).normalize();
        
        normals[i0][0] += face_normal.x;
        normals[i0][1] += face_normal.y;
        normals[i0][2] += face_normal.z;
        
        normals[i1][0] += face_normal.x;
        normals[i1][1] += face_normal.y;
        normals[i1][2] += face_normal.z;
        
        normals[i2][0] += face_normal.x;
        normals[i2][1] += face_normal.y;
        normals[i2][2] += face_normal.z;
    }
    
    // Normalize accumulated normals
    for normal in normals.iter_mut() {
        let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if length > 0.0 {
            normal[0] /= length;
            normal[1] /= length;
            normal[2] /= length;
        }
    }
}