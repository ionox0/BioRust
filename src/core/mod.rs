/// Core game modules containing fundamental components, resources, and game structure
pub mod components;
pub mod constants;
pub mod game;
pub mod resources;
pub mod spatial_grid;
pub mod time_controls;
pub mod unit_stats;

// Re-export commonly used items for convenience
// Only re-export what's actually used externally
