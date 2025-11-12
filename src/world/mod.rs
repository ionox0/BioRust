/// World modules for terrain, environment, and world setup
pub mod terrain_v2;
pub mod systems;
pub mod grid;

// Re-export commonly used items
pub use grid::GridPlugin;