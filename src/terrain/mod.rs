// Terrain module containing all terrain-related functionality
// Re-export from terrain_v2 for now

use bevy::prelude::*;
use hashbrown::HashMap;
use noise::Perlin;
use std::collections::VecDeque;

// Enhanced terrain constants for gentler hills and better textures
#[allow(dead_code)]
pub const QT_MIN_CELL_SIZE: f32 = 64.0;
#[allow(dead_code)]
pub const QT_MIN_CELL_RESOLUTION: u32 = 16;
#[allow(dead_code)]
pub const NOISE_HEIGHT: f32 = 15.0;
#[allow(dead_code)]
pub const NOISE_SCALE: f32 = 400.0;
#[allow(dead_code)]
pub const LOD_DISTANCE_MULTIPLIER: f32 = 6.0;

#[allow(dead_code)]
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TerrainChunkManager>()
            .init_resource::<TerrainSettings>()
            .add_systems(Update, (
                crate::terrain_v2::update_visible_chunks_quadtree,
                crate::terrain_v2::process_chunk_generation,
                crate::terrain_v2::retire_old_chunks,
            ).chain());
    }
}

#[allow(dead_code)]
#[derive(Resource)]
pub struct TerrainSettings {
    pub min_cell_size: f32,
    pub min_cell_resolution: u32,
    pub noise_height: f32,
    pub noise_scale: f32,
    pub lod_distance_multiplier: f32,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            min_cell_size: QT_MIN_CELL_SIZE,
            min_cell_resolution: QT_MIN_CELL_RESOLUTION,
            noise_height: NOISE_HEIGHT,
            noise_scale: NOISE_SCALE,
            lod_distance_multiplier: LOD_DISTANCE_MULTIPLIER,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkKey {
    pub x: i32,
    pub z: i32,
    pub size: u32,
}

#[allow(dead_code)]
impl ChunkKey {
    pub fn new(x: i32, z: i32, size: u32) -> Self {
        Self { x, z, size }
    }
    
    pub fn center(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 + self.size as f32 * 0.5,
            0.0,
            self.z as f32 + self.size as f32 * 0.5,
        )
    }
}

#[allow(dead_code)]
#[derive(Component)]
pub struct TerrainChunk {
    pub key: ChunkKey,
    pub generation_time: f32,
    pub is_ready: bool,
}

#[allow(dead_code)]
#[derive(Resource)]
pub struct TerrainChunkManager {
    pub chunks: HashMap<ChunkKey, Entity>,
    pub old_chunks: VecDeque<Entity>,
    pub chunk_pool: Vec<ChunkData>,
    pub generation_queue: VecDeque<ChunkKey>,
    pub noise_generator: Perlin,
    pub last_player_position: Vec3,
    pub dirty: bool,
    pub terrain_material: Option<Handle<StandardMaterial>>,
    pub terrain_texture: Option<Handle<Image>>,
}

impl Default for TerrainChunkManager {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            old_chunks: VecDeque::new(),
            chunk_pool: Vec::new(),
            generation_queue: VecDeque::new(),
            noise_generator: Perlin::new(42),
            last_player_position: Vec3::ZERO,
            dirty: true,
            terrain_material: None,
            terrain_texture: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ChunkData {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
}

#[allow(dead_code)]
impl ChunkData {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            uvs: Vec::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.normals.clear();
        self.colors.clear();
        self.uvs.clear();
    }
}

// Sample terrain height at any world position for camera following
// Note: terrain_v2::sample_terrain_height is available when needed