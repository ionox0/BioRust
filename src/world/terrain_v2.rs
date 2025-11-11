#![allow(dead_code)] // Allow unused terrain functionality for future features
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::executor::block_on;
use hashbrown::HashMap;
use noise::{NoiseFn, Perlin};
use std::collections::VecDeque;
use crate::rendering::seamless_texture::create_seamless_terrain_texture;

// Enhanced terrain constants for gentler hills and better textures
pub const QT_MIN_CELL_SIZE: f32 = 64.0;         // Good chunk size for RTS tactical gameplay
pub const QT_MIN_CELL_RESOLUTION: u32 = 16;     // Vertices per chunk edge
// Removed unused PLANET_RADIUS constant
pub const NOISE_HEIGHT: f32 = 15.0;             // Moderate height variation for RTS gameplay
pub const NOISE_SCALE: f32 = 400.0;             // Better scale for tactical terrain features
pub const LOD_DISTANCE_MULTIPLIER: f32 = 6.0;   // Less aggressive subdivision to maintain more chunks

pub struct TerrainPluginV2;

impl Plugin for TerrainPluginV2 {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TerrainChunkManager>()
            .init_resource::<TerrainSettings>()
            .add_systems(Startup, load_terrain_texture)
            .add_systems(Update, (
                update_visible_chunks_quadtree,
                start_chunk_generation_tasks,
                collect_generated_chunks,
                process_sync_chunk_generation,
                retire_old_chunks,
            ).chain());
    }
}

#[derive(Resource, Clone)]
pub struct TerrainSettings {
    pub min_cell_size: f32,
    pub min_cell_resolution: u32,
    pub noise_height: f32,
    pub noise_scale: f32,
    pub lod_distance_multiplier: f32,
    // terrain_seed removed as it's unused
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkKey {
    pub x: i32,
    pub z: i32,
    pub size: u32, // Size in world units
}

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

#[derive(Component)]
pub struct TerrainChunk {
    pub key: ChunkKey,
    pub generation_time: f32,
    pub is_ready: bool,
}

#[derive(Resource)]
pub struct TerrainChunkManager {
    pub chunks: HashMap<ChunkKey, Entity>,        // Active chunks
    pub old_chunks: VecDeque<Entity>,             // Chunks being retired
    pub chunk_pool: Vec<ChunkData>,               // Pooled chunk data
    pub generation_queue: VecDeque<ChunkKey>,     // Chunks to generate
    pub generating_tasks: HashMap<ChunkKey, Entity>, // Active generation tasks
    pub noise_generator: Perlin,                  // Keep for compatibility with existing systems
    pub noise_seed: u32,                          // Noise seed for thread-safe generation
    pub last_player_position: Vec3,
    pub dirty: bool,
    pub terrain_material: Option<Handle<StandardMaterial>>, // Cached terrain material
    pub last_update_time: f64,                    // Track when chunks were last updated
    pub terrain_texture: Option<Handle<Image>>,   // Cached terrain texture
    pub max_concurrent_tasks: usize,              // Limit concurrent generation tasks
}

impl Default for TerrainChunkManager {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            old_chunks: VecDeque::new(),
            chunk_pool: Vec::new(),
            generation_queue: VecDeque::new(),
            generating_tasks: HashMap::new(),
            noise_generator: Perlin::new(42),
            noise_seed: 42,
            last_player_position: Vec3::ZERO,
            dirty: true,
            terrain_material: None,
            last_update_time: 0.0,
            terrain_texture: None,
            max_concurrent_tasks: 4, // Allow up to 4 concurrent terrain generation tasks
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChunkData {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
}

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

// Async terrain generation result
#[derive(Debug)]
pub struct TerrainGenerationResult {
    pub chunk_key: ChunkKey,
    pub chunk_data: ChunkData,
}

// Task handle for async terrain generation
#[derive(Component)]
pub struct TerrainGenerationTask {
    pub task: Task<TerrainGenerationResult>,
    pub chunk_key: ChunkKey,
    pub started_time: f64,
}

// Thread-safe terrain generation parameters
#[derive(Clone)]
pub struct TerrainGenParams {
    pub chunk_key: ChunkKey,
    pub settings: TerrainSettings,
    pub noise_seed: u32,
}

// Bug_Game style quadtree node
#[derive(Debug, Clone)]
pub struct QuadTreeNode {
    pub center: Vec3,
    pub size: Vec3,
    pub children: Option<Box<[QuadTreeNode; 4]>>,
}

impl QuadTreeNode {
    pub fn new(center: Vec3, size: Vec3) -> Self {
        Self {
            center,
            size,
            children: None,
        }
    }
    
    pub fn create_children(&self, _min_size: f32) -> [QuadTreeNode; 4] {
        let half_size = self.size * 0.5;
        let quarter_size = half_size * 0.5;
        
        [
            // Bottom-left
            QuadTreeNode::new(
                self.center + Vec3::new(-quarter_size.x, 0.0, -quarter_size.z),
                half_size,
            ),
            // Bottom-right
            QuadTreeNode::new(
                self.center + Vec3::new(quarter_size.x, 0.0, -quarter_size.z),
                half_size,
            ),
            // Top-left
            QuadTreeNode::new(
                self.center + Vec3::new(-quarter_size.x, 0.0, quarter_size.z),
                half_size,
            ),
            // Top-right
            QuadTreeNode::new(
                self.center + Vec3::new(quarter_size.x, 0.0, quarter_size.z),
                half_size,
            ),
        ]
    }
    
    pub fn should_subdivide(&self, player_pos: Vec3, min_size: f32, distance_multiplier: f32) -> bool {
        let distance = self.center.distance(player_pos);
        let threshold = self.size.x * distance_multiplier;
        distance < threshold && self.size.x > min_size
    }
    
    pub fn insert(&mut self, player_pos: Vec3, min_size: f32, distance_multiplier: f32) {
        if self.should_subdivide(player_pos, min_size, distance_multiplier) {
            if self.children.is_none() {
                let children = self.create_children(min_size);
                self.children = Some(Box::new(children));
            }
            
            if let Some(ref mut children) = self.children {
                for child in children.iter_mut() {
                    child.insert(player_pos, min_size, distance_multiplier);
                }
            }
        }
    }
    
    pub fn get_leaf_nodes(&self) -> Vec<ChunkKey> {
        let mut leaves = Vec::new();
        self.collect_leaves(&mut leaves);
        leaves
    }
    
    fn collect_leaves(&self, leaves: &mut Vec<ChunkKey>) {
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.collect_leaves(leaves);
            }
        } else {
            // This is a leaf node - snap to exact grid boundaries
            let size = self.size.x as u32;
            let half_size = size as f32 * 0.5;
            
            // Round to nearest grid boundary to prevent floating point precision issues
            let x = ((self.center.x - half_size).round()) as i32;
            let z = ((self.center.z - half_size).round()) as i32;
            
            leaves.push(ChunkKey::new(x, z, size));
        }
    }
}

// Bug_Game style quadtree
pub struct CubeQuadTree {
    pub root: QuadTreeNode,
    pub min_node_size: f32,
    pub distance_multiplier: f32,
}

impl CubeQuadTree {
    pub fn new(radius: f32, min_node_size: f32, distance_multiplier: f32) -> Self {
        let size = Vec3::splat(radius * 2.0);
        let center = Vec3::ZERO;
        
        Self {
            root: QuadTreeNode::new(center, size),
            min_node_size,
            distance_multiplier,
        }
    }
    
    pub fn new_centered(center: Vec3, size: f32, min_node_size: f32, distance_multiplier: f32) -> Self {
        let node_size = Vec3::splat(size);
        
        Self {
            root: QuadTreeNode::new(center, node_size),
            min_node_size,
            distance_multiplier,
        }
    }
    
    pub fn insert(&mut self, player_pos: Vec3) {
        self.root.insert(player_pos, self.min_node_size, self.distance_multiplier);
    }
    
    pub fn get_leaf_chunks(&self) -> Vec<ChunkKey> {
        self.root.get_leaf_nodes()
    }
}

// Main update system following Bug_Game's pattern
pub fn update_visible_chunks_quadtree(
    camera_query: Query<&Transform, (With<Camera3d>, Without<TerrainChunk>)>,
    mut terrain_manager: ResMut<TerrainChunkManager>,
    terrain_settings: Res<TerrainSettings>,
) {
    if let Ok(camera_transform) = camera_query.get_single() {
        let camera_pos = camera_transform.translation;
        
        // Debug: Log camera position occasionally
        static mut LAST_LOG_TIME: f64 = 0.0;
        let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64();
        unsafe {
            if current_time - LAST_LOG_TIME > 2.0 {
                debug!("Camera position: {:?}", camera_pos);
                LAST_LOG_TIME = current_time;
            }
        }
        
        // Add minimum time between updates to prevent flashing
        let min_update_interval = 1.0; // At least 1 second between terrain updates
        if current_time - terrain_manager.last_update_time < min_update_interval {
            return;
        }
        
        // Check if player moved significantly (optimization from Bug_Game)
        let movement_threshold = 1000.0; // Even larger threshold to prevent frequent chunk updates and flashing
        if camera_pos.distance(terrain_manager.last_player_position) < movement_threshold && !terrain_manager.dirty {
            return;
        }
        
        debug!("Terrain update triggered - distance moved: {:.2}, dirty: {}", 
              camera_pos.distance(terrain_manager.last_player_position), 
              terrain_manager.dirty);
        
        terrain_manager.last_player_position = camera_pos;
        terrain_manager.last_update_time = current_time;
        terrain_manager.dirty = false;
        
        // Build quadtree with grid-aligned center to prevent seaming
        let grid_size = terrain_settings.min_cell_size * 16.0; // Align to multiple of min cell size
        let view_distance = 15000.0; // Much larger view distance to keep more chunks loaded
        
        // Round to exact grid boundaries to ensure consistent chunk alignment
        let quadtree_center = Vec3::new(
            (camera_pos.x / grid_size).round() * grid_size, // Use round for exact grid alignment
            0.0,
            (camera_pos.z / grid_size).round() * grid_size,
        );
        
        let mut quadtree = CubeQuadTree::new_centered(
            quadtree_center,
            view_distance,
            terrain_settings.min_cell_size,
            terrain_settings.lod_distance_multiplier,
        );
        
        quadtree.insert(camera_pos);
        let new_chunks = quadtree.get_leaf_chunks();
        
        // Convert to HashMap for set operations (Bug_Game pattern)
        let new_chunk_set: HashMap<ChunkKey, ()> = new_chunks
            .into_iter()
            .map(|key| (key, ()))
            .collect();
        
        // Calculate intersection, difference, and recycle sets with improved logic
        let intersection: HashMap<ChunkKey, Entity> = terrain_manager
            .chunks
            .iter()
            .filter(|(key, _)| new_chunk_set.contains_key(*key))
            .map(|(key, entity)| (*key, *entity))
            .collect();
        
        let difference: Vec<ChunkKey> = new_chunk_set
            .keys()
            .filter(|key| !terrain_manager.chunks.contains_key(*key))
            .copied()
            .collect();
        
        let recycle: Vec<Entity> = terrain_manager
            .chunks
            .iter()
            .filter(|(key, _)| !new_chunk_set.contains_key(*key))
            .map(|(_, entity)| *entity)
            .collect();
        
        // Log chunk status for debugging (debug level)
        debug!("Terrain chunks - Active: {}, Kept: {}, New: {}, Removed: {}", 
              terrain_manager.chunks.len(),
              intersection.len(), 
              difference.len(),
              recycle.len());
        
        // Don't immediately retire chunks - keep them visible until new chunks are generated
        // This prevents the flashing effect
        if !difference.is_empty() {
            // Only mark chunks for retirement if we have new chunks to generate
            for entity in recycle {
                debug!("Marking chunk for retirement: {:?}", entity);
                terrain_manager.old_chunks.push_back(entity);
            }
        }
        
        // Update active chunks (but keep old ones visible until new ones are ready)
        terrain_manager.chunks = intersection;
        
        // Queue new chunks for generation
        for chunk_key in difference {
            terrain_manager.generation_queue.push_back(chunk_key);
        }
    }
}


pub fn retire_old_chunks(
    mut commands: Commands,
    mut terrain_manager: ResMut<TerrainChunkManager>,
) {
    // Be very conservative with chunk retirement to prevent flashing
    // Only retire if we have accumulated many old chunks
    if terrain_manager.old_chunks.len() > 20 {
        // Retire only 1 chunk per frame to minimize visual pop
        if let Some(entity) = terrain_manager.old_chunks.pop_front() {
            debug!("Actually despawning old chunk entity: {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn generate_terrain_chunk_v2(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    _images: &mut ResMut<Assets<Image>>,
    chunk_key: ChunkKey,
    settings: &TerrainSettings,
    noise: &Perlin,
    cached_material: &Option<Handle<StandardMaterial>>,
    terrain_texture: &Option<Handle<Image>>,
) -> Entity {
    let resolution = calculate_resolution_for_chunk_size(chunk_key.size, settings.min_cell_resolution);
    let step = chunk_key.size as f32 / resolution as f32;
    
    // Generate heightmap data
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    
    // Store height data for normal calculation
    let mut height_data = vec![vec![0.0; (resolution + 1) as usize]; (resolution + 1) as usize];
    
    // Generate vertices with consistent boundary alignment and vertex colors for variation
    let mut vertex_colors = Vec::new();
    for z in 0..=resolution {
        for x in 0..=resolution {
            // Use consistent stepping for all vertices (prevents edge misalignment)
            let world_x = chunk_key.x as f32 + x as f32 * step;
            let world_z = chunk_key.z as f32 + z as f32 * step;
            
            let height = generate_height_v2(world_x, world_z, settings, noise);
            height_data[z as usize][x as usize] = height;
            
            vertices.push([world_x, height, world_z]);

            // Use world coordinates for UV mapping so texture tiles at consistent world-space scale
            // This makes textures appear at the same scale regardless of chunk size
            let texture_tile_size = 10.0; // Texture repeats every 10 world units
            let u = world_x / texture_tile_size;
            let v = world_z / texture_tile_size;
            uvs.push([u, v]);
            
            // Add vertex colors for terrain variation based on height and noise
            let color = calculate_terrain_color_v2(world_x, world_z, height, settings, noise);
            vertex_colors.push(color);
        }
    }
    
    // Calculate normals
    for z in 0..=resolution {
        for x in 0..=resolution {
            let normal = calculate_normal_v2(&height_data, x as usize, z as usize, resolution as usize, step);
            normals.push([normal.x, normal.y, normal.z]);
        }
    }
    
    // Generate indices with Bug_Game winding order
    for z in 0..resolution {
        for x in 0..resolution {
            let i = z * (resolution + 1) + x;
            let i_next_row = (z + 1) * (resolution + 1) + x;
            
            // Counter-clockwise winding
            indices.extend_from_slice(&[
                i as u32, i_next_row as u32, (i + 1) as u32,
                (i + 1) as u32, i_next_row as u32, (i_next_row + 1) as u32,
            ]);
        }
    }
    
    // Create mesh with indices for solid triangles
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    
    let vertex_count = vertices.len();
    let index_count = indices.len();
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    
    // Use cached material or create one if it doesn't exist
    let material = if let Some(cached_mat) = cached_material {
        cached_mat.clone()
    } else {
        // Create a material with grass texture if available
        materials.add(StandardMaterial {
            base_color: if terrain_texture.is_some() { 
                Color::WHITE // Use white to show texture colors naturally
            } else { 
                Color::srgb(0.6, 0.7, 0.4) // Fallback green-brown color
            },
            base_color_texture: terrain_texture.clone(),
            perceptual_roughness: 0.8, // Slightly less rough for grass
            metallic: 0.0, // No metallic reflection
            reflectance: 0.2, // Slightly higher reflectance for natural grass
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            double_sided: false,
            unlit: false, // Use proper lighting
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })
    };
    
    // Debug mesh info (debug level)
    debug!("Generated mesh with {} vertices, {} indices for chunk ({}, {})", 
          vertex_count, index_count, chunk_key.x, chunk_key.z);
    
    // Spawn chunk entity - explicitly solid (not wireframe)
    let entity = commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::ZERO),
        TerrainChunk {
            key: chunk_key,
            generation_time: 0.0, // Could use actual time
            is_ready: true,
        },
    )).id();
    
    // Make sure no Wireframe component is accidentally added
    commands.entity(entity).remove::<bevy::pbr::wireframe::Wireframe>();
    
    entity
}

fn calculate_resolution_for_chunk_size(chunk_size: u32, min_resolution: u32) -> u32 {
    // Improved resolution calculation for better detail distribution
    match chunk_size {
        0..=50 => min_resolution * 4,       // 64 vertices for smallest chunks - very high detail
        51..=100 => min_resolution * 2,     // 32 vertices for small chunks
        101..=250 => min_resolution,        // 16 vertices for medium chunks  
        251..=500 => min_resolution / 2,    // 8 vertices for large chunks
        _ => (min_resolution / 4).max(4),   // 4 vertices minimum for huge chunks
    }
}

fn generate_height_v2(x: f32, z: f32, settings: &TerrainSettings, noise: &Perlin) -> f32 {
    // Simple, clean terrain generation without artifacts
    let scale = 1.0 / settings.noise_scale;
    
    // Single octave for base terrain
    let base_noise = noise.get([x as f64 * scale as f64, z as f64 * scale as f64]) as f32;
    
    // Add one detail octave
    let detail_noise = noise.get([x as f64 * scale as f64 * 3.0, z as f64 * scale as f64 * 3.0]) as f32;
    
    // Simple combination without complex smoothing that causes artifacts
    let combined = base_noise + detail_noise * 0.3;
    
    // Direct scaling without tanh compression that causes circular holes
    combined * settings.noise_height
}

fn calculate_normal_v2(height_data: &Vec<Vec<f32>>, x: usize, z: usize, resolution: usize, step: f32) -> Vec3 {
    let get_height = |x: i32, z: i32| -> f32 {
        if x < 0 || z < 0 || x > resolution as i32 || z > resolution as i32 {
            height_data[z.max(0).min(resolution as i32) as usize][x.max(0).min(resolution as i32) as usize]
        } else {
            height_data[z as usize][x as usize]
        }
    };
    
    let hx = get_height(x as i32 + 1, z as i32) - get_height(x as i32 - 1, z as i32);
    let hz = get_height(x as i32, z as i32 + 1) - get_height(x as i32, z as i32 - 1);
    
    Vec3::new(-hx / (2.0 * step), 1.0, -hz / (2.0 * step)).normalize()
}

fn calculate_terrain_color_v2(world_x: f32, world_z: f32, height: f32, _settings: &TerrainSettings, noise: &Perlin) -> [f32; 4] {
    // Multi-scale noise for rich texture variation per chunk
    let large_noise = noise.get([world_x as f64 * 0.001, world_z as f64 * 0.001]) as f32;
    let medium_noise = noise.get([world_x as f64 * 0.005, world_z as f64 * 0.005]) as f32;
    let detail_noise = noise.get([world_x as f64 * 0.02, world_z as f64 * 0.02]) as f32;
    let micro_detail = noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]) as f32;
    
    // Height-based biome with adjusted thresholds for new height range
    let base_color = if height < -10.0 {
        // Deep water - blue
        Color::srgb(0.1, 0.3, 0.7)
    } else if height < -2.0 {
        // Shore/shallow water - blue-green
        Color::srgb(0.2, 0.5, 0.6)
    } else if height < 2.0 {
        // Beach/lowlands - sandy
        Color::srgb(0.7, 0.6, 0.4)
    } else if height < 8.0 {
        // Grass plains - green
        Color::srgb(0.3, 0.6, 0.3)
    } else if height < 15.0 {
        // Forest hills - dark green
        Color::srgb(0.2, 0.5, 0.2)
    } else {
        // Mountains - brown/gray
        Color::srgb(0.5, 0.4, 0.3)
    };
    
    // Create rich texture variation using multiple noise scales
    let texture_variation = large_noise * 0.1 + medium_noise * 0.08 + detail_noise * 0.06 + micro_detail * 0.03;
    
    // Add rock/dirt patches based on noise
    let rock_factor = (medium_noise * 0.5 + 0.5).powf(3.0); // Make patches more distinct
    let dirt_color = Color::srgb(0.4, 0.3, 0.2);
    
    // Blend base color with dirt patches
    let mixed_color = Color::srgb(
        base_color.to_srgba().red * (1.0 - rock_factor * 0.4) + dirt_color.to_srgba().red * rock_factor * 0.4,
        base_color.to_srgba().green * (1.0 - rock_factor * 0.4) + dirt_color.to_srgba().green * rock_factor * 0.4,
        base_color.to_srgba().blue * (1.0 - rock_factor * 0.4) + dirt_color.to_srgba().blue * rock_factor * 0.4,
    );
    
    [
        (mixed_color.to_srgba().red + texture_variation).clamp(0.0, 1.0),
        (mixed_color.to_srgba().green + texture_variation).clamp(0.0, 1.0),
        (mixed_color.to_srgba().blue + texture_variation).clamp(0.0, 1.0),
        1.0
    ]
}

// Create a fallback procedural terrain texture with seamless tiling
pub fn create_terrain_texture_fallback(images: &mut ResMut<Assets<Image>>) -> Handle<Image> {
    let texture_size = 512u32;
    let mut texture_data = Vec::with_capacity((texture_size * texture_size * 4) as usize);
    
    // Create a detailed terrain texture using multiple noise layers
    let noise = Perlin::new(12345);
    
    for y in 0..texture_size {
        for x in 0..texture_size {
            let nx = x as f64 / texture_size as f64;
            let ny = y as f64 / texture_size as f64;
            
            // Multiple noise octaves for detail
            let mut value = 0.0;
            let mut amplitude = 1.0;
            let mut frequency = 8.0;
            
            // Add 4 octaves of noise for texture detail
            for _ in 0..4 {
                value += noise.get([nx * frequency, ny * frequency]) * amplitude;
                amplitude *= 0.5;
                frequency *= 2.0;
            }
            
            // Normalize and create terrain-like pattern
            value = (value + 1.0) * 0.5; // Normalize to 0-1
            
            // Create varied terrain colors based on noise
            let (r, g, b) = if value < 0.3 {
                // Rocky/stone - gray with brown tint
                let base = (value * 0.8 + 0.2) as f32;
                (base * 0.6, base * 0.5, base * 0.4)
            } else if value < 0.6 {
                // Grass/vegetation - greens
                let grass_intensity = ((value - 0.3) / 0.3) as f32;
                (0.2 * grass_intensity + 0.1, 0.4 + grass_intensity * 0.3, 0.15 + grass_intensity * 0.2)
            } else {
                // Dirt/earth - browns
                let dirt_intensity = ((value - 0.6) / 0.4) as f32;
                (0.4 + dirt_intensity * 0.3, 0.25 + dirt_intensity * 0.2, 0.1 + dirt_intensity * 0.1)
            };
            
            // Add some micro-detail noise for surface texture
            let detail_noise = (noise.get([nx * 64.0, ny * 64.0]) * 0.1 + 1.0) as f32;
            
            texture_data.push((r * detail_noise * 255.0).clamp(0.0, 255.0) as u8); // R
            texture_data.push((g * detail_noise * 255.0).clamp(0.0, 255.0) as u8); // G
            texture_data.push((b * detail_noise * 255.0).clamp(0.0, 255.0) as u8); // B
            texture_data.push(255); // A
        }
    }
    
    // Create the image with proper settings for terrain
    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        texture_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    
    // Use default texture sampling
    
    images.add(image)
}

// Sample terrain height at any world position for camera following
pub fn sample_terrain_height(world_x: f32, world_z: f32, noise: &Perlin, settings: &TerrainSettings) -> f32 {
    generate_height_v2(world_x, world_z, settings, noise)
}

// System to load terrain texture at startup
pub fn load_terrain_texture(
    asset_server: Res<AssetServer>,
    mut terrain_manager: ResMut<TerrainChunkManager>,
) {
    info!("Loading terrain texture: grass.jpg");
    let texture_handle: Handle<Image> = asset_server.load("textures/grass.jpg");
    terrain_manager.terrain_texture = Some(texture_handle);
    info!("Terrain texture loaded and cached");
}

// Thread-safe terrain generation function that runs on async compute thread pool
async fn generate_terrain_chunk_async(params: TerrainGenParams) -> TerrainGenerationResult {
    let chunk_key = params.chunk_key;
    let settings = &params.settings;
    let noise = Perlin::new(params.noise_seed);
    
    let resolution = calculate_resolution_for_chunk_size(chunk_key.size, settings.min_cell_resolution);
    let step = chunk_key.size as f32 / resolution as f32;
    
    // Generate terrain data on background thread
    let mut chunk_data = ChunkData::new();
    
    // Store height data for normal calculation
    let mut height_data = vec![vec![0.0; (resolution + 1) as usize]; (resolution + 1) as usize];
    
    // Generate vertices and store heights
    for z in 0..=resolution {
        for x in 0..=resolution {
            let world_x = chunk_key.x as f32 + x as f32 * step;
            let world_z = chunk_key.z as f32 + z as f32 * step;
            
            let height = generate_height_v2(world_x, world_z, settings, &noise);
            height_data[z as usize][x as usize] = height;
            
            chunk_data.vertices.push([world_x, height, world_z]);
            
            // UV mapping
            let texture_tile_size = 10.0;
            let u = world_x / texture_tile_size;
            let v = world_z / texture_tile_size;
            chunk_data.uvs.push([u, v]);
            
            // Terrain color
            let color = calculate_terrain_color_v2(world_x, world_z, height, settings, &noise);
            chunk_data.colors.push(color);
        }
    }
    
    // Calculate normals
    for z in 0..=resolution {
        for x in 0..=resolution {
            let normal = calculate_normal_v2(&height_data, x as usize, z as usize, resolution as usize, step);
            chunk_data.normals.push([normal.x, normal.y, normal.z]);
        }
    }
    
    // Generate indices
    for z in 0..resolution {
        for x in 0..resolution {
            let i = z * (resolution + 1) + x;
            let i_next_row = (z + 1) * (resolution + 1) + x;
            
            chunk_data.indices.extend_from_slice(&[
                i as u32, i_next_row as u32, (i + 1) as u32,
                (i + 1) as u32, i_next_row as u32, (i_next_row + 1) as u32,
            ]);
        }
    }
    
    TerrainGenerationResult {
        chunk_key,
        chunk_data,
    }
}

// System to start async terrain generation tasks
pub fn start_chunk_generation_tasks(
    mut commands: Commands,
    mut terrain_manager: ResMut<TerrainChunkManager>,
    terrain_settings: Res<TerrainSettings>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
) {
    // Initialize shared terrain texture if needed
    if terrain_manager.terrain_texture.is_none() {
        let terrain_texture = create_seamless_terrain_texture(&mut images);
        terrain_manager.terrain_texture = Some(terrain_texture);
    }
    
    // Limit concurrent generation tasks
    if terrain_manager.generating_tasks.len() >= terrain_manager.max_concurrent_tasks {
        return;
    }
    
    // Start new generation tasks
    let tasks_to_start = terrain_manager.max_concurrent_tasks - terrain_manager.generating_tasks.len();
    
    for _ in 0..tasks_to_start {
        if let Some(chunk_key) = terrain_manager.generation_queue.pop_front() {
            // Skip if already generating this chunk
            if terrain_manager.generating_tasks.contains_key(&chunk_key) {
                continue;
            }
            
            debug!("Starting async generation for terrain chunk at ({}, {}) size {}", 
                   chunk_key.x, chunk_key.z, chunk_key.size);
            
            let params = TerrainGenParams {
                chunk_key,
                settings: (*terrain_settings).clone(),
                noise_seed: terrain_manager.noise_seed,
            };
            
            let task_pool = AsyncComputeTaskPool::get();
            let task = task_pool.spawn(generate_terrain_chunk_async(params));
            
            let task_entity = commands.spawn(TerrainGenerationTask {
                task,
                chunk_key,
                started_time: time.elapsed_secs_f64(),
            }).id();
            
            terrain_manager.generating_tasks.insert(chunk_key, task_entity);
        } else {
            break;
        }
    }
}

// System to collect completed terrain generation tasks
pub fn collect_generated_chunks(
    mut commands: Commands,
    mut terrain_manager: ResMut<TerrainChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut generation_tasks: Query<&mut TerrainGenerationTask>,
    time: Res<Time>,
) {
    let mut completed_tasks = Vec::new();
    
    // Check all active generation tasks - collect data first to avoid borrowing conflicts
    let task_data: Vec<(ChunkKey, Entity)> = terrain_manager.generating_tasks.iter().map(|(k, e)| (*k, *e)).collect();
    
    for (chunk_key, task_entity) in task_data {
        if let Ok(mut task) = generation_tasks.get_mut(task_entity) {
            // Check if task is completed
            if task.task.is_finished() {
                // Extract the result from the completed task
                let result = block_on(&mut task.task);
                
                info!("✅ Async terrain generation completed for chunk ({}, {}) size {}", 
                       result.chunk_key.x, result.chunk_key.z, result.chunk_key.size);
                
                // Create mesh from the generated data
                let chunk_entity = create_mesh_from_chunk_data(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    result.chunk_key,
                    result.chunk_data,
                    &terrain_manager.terrain_material,
                    &terrain_manager.terrain_texture,
                );
                
                // Add to active chunks
                terrain_manager.chunks.insert(result.chunk_key, chunk_entity);
                completed_tasks.push((chunk_key, task_entity));
            } else {
                // Check for timeout (optional)
                let elapsed = time.elapsed_secs_f64() - task.started_time;
                if elapsed > 10.0 { // 10 second timeout
                    warn!("Terrain generation task timed out for chunk ({}, {}) size {}", 
                          chunk_key.x, chunk_key.z, chunk_key.size);
                    completed_tasks.push((chunk_key, task_entity));
                }
            }
        }
    }
    
    // Clean up completed tasks
    for (chunk_key, task_entity) in completed_tasks {
        terrain_manager.generating_tasks.remove(&chunk_key);
        commands.entity(task_entity).despawn();
    }
}

// Fallback sync terrain generation for when async fails
pub fn process_sync_chunk_generation(
    mut commands: Commands,
    mut terrain_manager: ResMut<TerrainChunkManager>,
    terrain_settings: Res<TerrainSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Initialize shared terrain texture and material if not already created
    if terrain_manager.terrain_texture.is_none() {
        let terrain_texture = create_seamless_terrain_texture(&mut images);
        terrain_manager.terrain_texture = Some(terrain_texture.clone());
        
        let shared_material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(terrain_texture),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            reflectance: 0.1,
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            double_sided: false,
            unlit: false,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        });
        terrain_manager.terrain_material = Some(shared_material);
    }

    // Process up to 1 chunk per frame to keep framerate smooth
    if let Some(chunk_key) = terrain_manager.generation_queue.pop_front() {
        debug!("Generating terrain chunk synchronously at ({}, {}) size {}", 
               chunk_key.x, chunk_key.z, chunk_key.size);
        
        let chunk_entity = generate_terrain_chunk_v2(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            chunk_key,
            &terrain_settings,
            &terrain_manager.noise_generator,
            &terrain_manager.terrain_material,
            &terrain_manager.terrain_texture,
        );
        
        terrain_manager.chunks.insert(chunk_key, chunk_entity);
    }
}

// Helper function to create mesh from generated chunk data
fn create_mesh_from_chunk_data(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_key: ChunkKey,
    chunk_data: ChunkData,
    cached_material: &Option<Handle<StandardMaterial>>,
    terrain_texture: &Option<Handle<Image>>,
) -> Entity {
    // Create mesh
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, chunk_data.vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, chunk_data.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, chunk_data.uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, chunk_data.colors);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(chunk_data.indices));
    
    // Use cached material or create new one
    let material = if let Some(cached_mat) = cached_material {
        cached_mat.clone()
    } else {
        materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: terrain_texture.clone(),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            reflectance: 0.1,
            cull_mode: Some(bevy::render::render_resource::Face::Back),
            double_sided: false,
            unlit: false,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })
    };
    
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::ZERO),
        TerrainChunk {
            key: chunk_key,
            generation_time: 0.0,
            is_ready: true,
        },
    )).id()
}