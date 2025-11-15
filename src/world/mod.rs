pub mod grid;
pub mod systems;
/// Static terrain system for RTS gameplay
pub mod static_terrain;

// Re-export commonly used items
pub use grid::GridPlugin;
pub use static_terrain::StaticTerrainPlugin;
