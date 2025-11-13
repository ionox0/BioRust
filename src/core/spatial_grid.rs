//! Spatial Grid utility for efficient spatial queries
//! 
//! This module provides a generic spatial partitioning system that can be used
//! for fast collision detection, pathfinding, and other spatial queries.

use bevy::prelude::*;
use hashbrown::HashMap;

/// Default grid cell size in world units
pub const DEFAULT_GRID_CELL_SIZE: f32 = 50.0;

/// A generic spatial grid for efficient spatial queries
pub struct SpatialGrid<T> {
    pub cells: HashMap<GridCoord, Vec<T>>,
    pub cell_size: f32,
}

/// Grid coordinate representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCoord {
    pub x: i32,
    pub z: i32,
}

impl GridCoord {
    /// Create a grid coordinate from a world position
    pub fn from_world_pos(pos: Vec3, cell_size: f32) -> Self {
        Self {
            x: (pos.x / cell_size).floor() as i32,
            z: (pos.z / cell_size).floor() as i32,
        }
    }
    
    /// Get all neighboring coordinates (including self) in a 3x3 grid
    pub fn get_neighboring_coords(self) -> [GridCoord; 9] {
        [
            GridCoord { x: self.x - 1, z: self.z - 1 },
            GridCoord { x: self.x, z: self.z - 1 },
            GridCoord { x: self.x + 1, z: self.z - 1 },
            GridCoord { x: self.x - 1, z: self.z },
            self, // center
            GridCoord { x: self.x + 1, z: self.z },
            GridCoord { x: self.x - 1, z: self.z + 1 },
            GridCoord { x: self.x, z: self.z + 1 },
            GridCoord { x: self.x + 1, z: self.z + 1 },
        ]
    }
}

impl<T> SpatialGrid<T> {
    /// Create a new spatial grid with the specified cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            cells: HashMap::new(),
            cell_size,
        }
    }
    
    /// Create a new spatial grid with default cell size
    pub fn with_default_size() -> Self {
        Self::new(DEFAULT_GRID_CELL_SIZE)
    }
    
    /// Clear all entries in the grid
    pub fn clear(&mut self) {
        self.cells.clear();
    }
    
    /// Insert an item at the specified world position
    pub fn insert(&mut self, position: Vec3, item: T) {
        let coord = GridCoord::from_world_pos(position, self.cell_size);
        self.cells.entry(coord).or_default().push(item);
    }
    
    /// Get all items in the cell containing the given position
    pub fn get_cell(&self, position: Vec3) -> Option<&Vec<T>> {
        let coord = GridCoord::from_world_pos(position, self.cell_size);
        self.cells.get(&coord)
    }
    
    /// Get all items in the cell and neighboring cells around the given position
    pub fn query_nearby(&self, position: Vec3) -> Vec<&T> {
        let coord = GridCoord::from_world_pos(position, self.cell_size);
        let neighboring_coords = coord.get_neighboring_coords();
        
        let mut results = Vec::new();
        for &neighbor_coord in &neighboring_coords {
            if let Some(items) = self.cells.get(&neighbor_coord) {
                for item in items {
                    results.push(item);
                }
            }
        }
        results
    }
}

/// Specialized spatial grid for obstacle data (position + radius)
pub type ObstacleSpatialGrid = SpatialGrid<(Vec3, f32)>;

impl ObstacleSpatialGrid {
    /// Create a new obstacle spatial grid from a list of obstacles
    pub fn from_obstacles(obstacles: &[(Vec3, f32)], cell_size: Option<f32>) -> Self {
        let mut grid = Self::new(cell_size.unwrap_or(DEFAULT_GRID_CELL_SIZE));
        
        for &(pos, radius) in obstacles {
            grid.insert(pos, (pos, radius));
        }
        
        grid
    }
    
    /// Query nearby obstacles within a certain radius
    pub fn query_nearby_obstacles(&self, position: Vec3, max_distance: f32) -> Vec<(Vec3, f32)> {
        self.query_nearby(position)
            .into_iter()
            .filter(|(obstacle_pos, obstacle_radius)| {
                position.distance(*obstacle_pos) <= max_distance + obstacle_radius + self.cell_size
            })
            .copied()
            .collect()
    }
}

/// Specialized spatial grid for entity data (entity + position + radius)
pub type EntitySpatialGrid = SpatialGrid<(Entity, Vec3, f32)>;

impl EntitySpatialGrid {
    /// Insert an entity with its position and radius
    pub fn insert_entity(&mut self, entity: Entity, position: Vec3, radius: f32) {
        self.insert(position, (entity, position, radius));
    }
    
    /// Query nearby entities within a certain radius, excluding the given entity
    pub fn query_nearby_entities(&self, position: Vec3, radius: f32, exclude_entity: Option<Entity>) -> Vec<(Entity, Vec3, f32)> {
        self.query_nearby(position)
            .into_iter()
            .filter(|(entity, entity_pos, entity_radius)| {
                if let Some(exclude) = exclude_entity {
                    if *entity == exclude {
                        return false;
                    }
                }
                position.distance(*entity_pos) <= radius + entity_radius + self.cell_size
            })
            .copied()
            .collect()
    }
}