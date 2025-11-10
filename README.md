# Rust RTS Game

A real-time strategy game built with Bevy Engine, featuring an insect-themed colony simulation with advanced terrain generation, 3D models, and AI systems.

## 🎮 Game Features

### Core Gameplay
- **Insect Colony RTS**: Command various insect species including Worker Ants, Soldier Ants, Hunter Wasps, Spear Mantises, Scout Ants, and Beetle Knights
- **Resource Management**: Collect and manage Nectar, Chitin, Minerals, and Pheromones
- **Building System**: Construct Queen Chambers, Nurseries, Warrior Chambers, and other specialized buildings
- **Combat System**: Tactical combat with different unit types (Melee, Ranged, Siege)
- **AI Opponents**: Intelligent AI players with decision-making systems

### Technical Features
- **3D Terrain Generation**: Procedural terrain using noise algorithms with dynamic chunk loading
- **GLB Model Support**: High-quality 3D insect models with fallback primitive shapes
- **Animation System**: Unit animations with state management
- **Team Coloring**: Dynamic color application for different players
- **Camera System**: RTS-style camera with terrain-following and smooth controls
- **Selection System**: Drag selection, click selection, and visual feedback

## 🏗️ Project Structure

The codebase has been recently refactored into a clean, modular architecture:

```
src/
├── core/           # Core game systems and shared components
│   ├── components.rs   # Game entities and component definitions
│   ├── resources.rs    # Global game resources
│   ├── constants.rs    # Configuration constants and settings
│   ├── unit_stats.rs   # Unit balance and statistics configuration
│   ├── game.rs         # Game state management
│   └── mod.rs          # Module exports
├── rendering/      # Visual rendering and model systems
│   ├── model_loader.rs     # GLB model loading and management
│   ├── animation_systems.rs # Animation controllers
│   ├── seamless_texture.rs # Texture systems
│   └── mod.rs              # Module exports
├── world/          # World generation and terrain
│   ├── terrain_v2.rs   # Advanced terrain generation
│   ├── systems.rs      # World setup and camera systems
│   └── mod.rs          # Module exports
├── entities/       # Entity creation and management
│   ├── rts_entities.rs    # RTS-specific entity factories
│   ├── entity_factory.rs  # Unified entity creation system
│   ├── entity_state_systems.rs # Entity state management
│   └── mod.rs             # Module exports
├── rts/            # RTS-specific game systems
│   ├── selection.rs    # Unit selection and interaction
│   ├── production.rs   # Building production queues
│   └── mod.rs          # Module exports
├── ui/             # User interface systems
│   ├── building_panel.rs # Building construction UI
│   ├── placement.rs      # Building placement system
│   └── mod.rs            # Module exports
├── ai/             # Artificial intelligence
│   ├── decision_making.rs # AI decision algorithms
│   ├── player_state.rs    # AI player state tracking
│   └── mod.rs             # Module exports
├── combat_systems.rs  # Combat mechanics and damage
├── rts_systems.rs     # Core RTS gameplay systems
└── main.rs            # Application entry point
```

## 🚀 Getting Started

### Prerequisites
- Rust 1.75+ with Cargo
- Git

### Installation
```bash
# Clone the repository
git clone <repository-url>
cd rust-game

# Run in development mode
cargo run

# Run optimized release build
cargo run --profile release
```

### Controls
- **WASD**: Move camera (W=North, S=South, A=West, D=East)
- **Mouse Wheel**: Zoom in/out
- **Right Mouse + Drag**: Look around
- **Left Click**: Select units
- **Left Drag**: Box selection
- **Shift + Movement**: Fast camera movement
- **Alt + Movement**: Hyper speed camera movement
- **G**: Toggle terrain following
- **F**: Toggle height clamping
- **B**: Build Warrior Chamber
- **H**: Build Nursery  
- **F**: Build Fungal Garden
- **Escape**: Cancel building

## ⚙️ Configuration

### Unit Speed Adjustment
Unit movement speed can be modified in `src/core/constants.rs`:
```rust
pub const UNIT_SPEED: f32 = 40.0;  // Current: 2x default speed
```

### Game Balance
Unit statistics and balance are now organized in separate modules:

**Unit Stats** (`src/core/unit_stats.rs`):
- Health, armor, regeneration rates
- Combat damage, range, attack speed
- Movement speed, acceleration, turning
- Vision range and collision radii

**General Constants** (`src/core/constants.rs`):
- Movement physics and limits
- Camera behavior settings
- UI layout and styling
- Resource costs and generation rates
- AI decision intervals and thresholds

Example unit configuration:
```rust
pub const WORKER_ANT_STATS: UnitStatsConfig = UnitStatsConfig {
    health: HealthStats {
        current: 75.0,
        max: 75.0,
        armor: 0.0,
        regeneration_rate: 0.1,
    },
    movement: MovementStats {
        max_speed: 25.0,
        acceleration: 15.0,
        turning_speed: 2.0,
    },
    // ... combat and vision stats
};
```

### Model Scaling
3D model scales can be adjusted in the `models` module of constants:
```rust
pub const UNIFORM_UNIT_SCALE: f32 = 2.5;
pub const UNIFORM_BUILDING_SCALE: f32 = 3.0;
```

## 🎨 Asset Requirements

### 3D Models (GLB Format)
The game supports various insect models placed in `assets/models/`:
- **Units**: fourmi.glb, hornet.glb, rhinobeetle.glb, etc.
- **Buildings**: anthill.glb, hive.glb
- **Environment**: mushrooms.glb, grass.glb, rocks.glb

### Fallback System
If GLB models are unavailable, the game automatically falls back to primitive shapes (capsules, cubes, spheres) with appropriate scaling and team colors.

## 🧠 AI System

The AI system features:
- **Decision Making**: Timer-based decisions every 5 seconds
- **Resource Management**: Automatic resource generation and spending
- **Military Strategy**: Unit production and attack coordination
- **Building Planning**: Strategic building placement and expansion

## 🔧 Development

### Adding New Units
1. Define unit type in `src/core/components.rs`
2. Add stats in `src/entities/entity_factory.rs`
3. Implement spawn function in `src/entities/rts_entities.rs`
4. Configure model mapping in `src/rendering/model_loader.rs`

### Adding New Buildings
1. Define building type in `src/core/components.rs`
2. Add construction costs in `src/core/constants.rs`
3. Implement spawn function in entity factories
4. Add to production queues if trainable

### Performance Optimization
- **Terrain Chunking**: Only loads visible terrain chunks
- **Model LOD**: Automatic level-of-detail for distant objects
- **Efficient Queries**: ECS queries optimized for performance
- **Resource Pooling**: Reused mesh and material assets

## 📈 Recent Changes

### Environment Objects Cleanup (Latest)
- ✅ Removed scattered random environment object spawning across the map
- ✅ Replaced with minimal, controlled spawning only in center area
- ✅ Changed from 6 large spawn zones with 2-4 objects each to 5 specific positions
- ✅ Cleaner map layout focused on RTS gameplay without visual clutter

### Unit Stats Extraction 
- ✅ Created `src/core/unit_stats.rs` for centralized unit balance configuration
- ✅ Moved all unit health, combat, movement, and vision stats to dedicated constants
- ✅ Simplified game balance tweaking with clear, organized stat structures
- ✅ Maintained backwards compatibility with existing EntityFactory system

### Major Refactoring
- ✅ Removed unused systems (model_showcase, debug_health, physics remnants)
- ✅ Reorganized code into logical module structure
- ✅ Split large files into focused, maintainable modules
- ✅ Updated all import paths and resolved compilation errors
- ✅ Increased unit movement speed from 20.0 to 40.0

### Architecture Improvements
- **Unified Entity Factory**: Single system for spawning all entities
- **Clean Module Structure**: Logical separation of concerns
- **Enhanced Model Support**: Better GLB model integration with fallbacks
- **Improved Selection**: Simplified selection system with better performance

## 🐛 Known Issues

- Selection system uses placeholder viewport conversion (TODO items in code)
- Some unit types need GLB model assignments
- Building model variety could be expanded

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes following the established module structure
4. Test with both GLB models and primitive fallbacks
5. Submit a pull request

## 📄 License

[Add your license information here]

## 🙏 Acknowledgments

- Built with [Bevy Engine](https://bevyengine.org/)
- Insect 3D models from various sources
- Inspired by classic RTS games like Age of Empires

---

**Note**: This project emphasizes clean, maintainable code architecture while delivering engaging RTS gameplay with beautiful 3D visuals and intelligent AI opponents.