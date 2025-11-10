/// Core game modules containing fundamental components, resources, and game structure
pub mod components;
pub mod resources;
pub mod constants;
pub mod game;
pub mod unit_stats;

// Re-export commonly used items for convenience
pub use components::*;
pub use resources::*;
pub use constants::*;
pub use game::*;
pub use unit_stats::*;