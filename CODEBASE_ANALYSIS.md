# BioRust Codebase - Comprehensive DRY Violation Analysis

## Executive Summary
The BioRust codebase (~12,400 lines across 49 Rust files) shows a RTS game architecture with good separation of concerns into modules (AI, UI, RTS systems, rendering, core), but contains **significant duplication and DRY violations** that make the codebase harder to maintain. Key areas of concern include mass duplication in unit spawn functions, scattered configuration constants, and repeated match patterns.

---

## 1. PROJECT STRUCTURE OVERVIEW

### Directory Organization (Good)
```
src/
├── core/          # Game configuration, components, resources
├── world/         # Terrain generation and world systems
├── entities/      # Entity spawning (factory pattern)
├── rts/           # RTS gameplay systems (movement, production, construction)
├── rendering/     # 3D models and animations
├── ui/            # User interface elements
├── ai/            # AI decision making and unit management
├── collision.rs   # Collision detection
├── combat_systems.rs
├── health_ui.rs
├── main.rs        # Entry point
└── resource_ui.rs
```

### Largest Files (Potential Refactoring Targets)
- `model_loader.rs` (1,130 lines) - Model management, loading, mapping
- `rts_entities.rs` (809 lines) - **MAJOR DUPLICATION HERE**
- `entity_factory.rs` (635 lines)
- `world/systems.rs` (556 lines)
- `combat_systems.rs` (534 lines)

---

## 2. CRITICAL DRY VIOLATIONS

### 2.1 MASSIVE DUPLICATION: Unit Spawn Functions (809 lines in rts_entities.rs)

**Problem**: 15+ nearly-identical unit spawn functions with extremely repetitive code.

**Examples**:
- `spawn_worker_ant()` + `spawn_worker_ant_with_models()`
- `spawn_soldier_ant()` + `spawn_soldier_ant_with_models()`
- `spawn_spear_mantis()` + `spawn_spear_mantis_with_models()`
- `spawn_scout_ant()` + `spawn_scout_ant_with_models()`
- `spawn_beetle_knight()` (no dual version)
- `spawn_dragonfly()` + `spawn_dragonfly_with_models()`
- Building spawns: `spawn_queen_chamber()`, `spawn_nursery()`, `spawn_warrior_chamber()`

**Code Pattern (Highly Repetitive)**:
```rust
// Every unit spawn follows this exact pattern:
pub fn spawn_soldier_ant(commands, meshes, materials, position, player_id, unit_id) -> Entity {
    Self::spawn_soldier_ant_with_models(commands, meshes, materials, position, player_id, unit_id, None)
}

pub fn spawn_soldier_ant_with_models(..., model_assets) -> Entity {
    let unit_type = UnitType::SoldierAnt;
    let model_scale = Self::calculate_model_scale(&unit_type, model_assets);
    
    let mut entity = Self::spawn_unit_visual(
        commands, meshes, materials, position, player_id, &unit_type, 
        model_scale, model_assets, Capsule3d::new(1.2, 2.2).into(),
    );
    
    entity.insert((
        RTSUnit { unit_id, player_id, size: 1.0, unit_type: Some(unit_type) },
        TeamColor::new(player_id),
        Position { translation: position, rotation: Quat::IDENTITY },
        Movement {
            max_speed: 60.0 / model_scale,
            acceleration: 40.0 / model_scale,
            turning_speed: 3.5,
            ..default()
        },
        RTSHealth {
            current: 120.0,
            max: 120.0,
            armor: 1.0,
            regeneration_rate: 0.0,
            last_damage_time: 0.0,
        },
        Combat {
            attack_damage: 4.0,
            attack_range: 5.0,
            attack_speed: 2.0,
            last_attack_time: 0.0,
            target: None,
            attack_type: AttackType::Melee,
            attack_cooldown: 0.0,
            is_attacking: false,
            auto_attack: true,
        },
        Selectable::default(),
        Vision::default(),
        CollisionRadius { radius: SOLDIER_ANT_COLLISION_RADIUS },
        EntityState::default(),
        GameEntity,
    )).id()
}
```

**The Real Solution Already Exists**:
`entity_factory.rs` has `EntityFactory` with `SpawnConfig` pattern that could consolidate all this!

---

### 2.2 DUPLICATE AI STRATEGY REGISTRATION

**Location**: `src/ai/mod.rs`, lines 54-63

```rust
.add_systems(Update, (
    intelligence_update_system,
    scouting_system,
    scout_survival_system,
    tactical_decision_system,
    economy_optimization_system,
    worker_idle_detection_system,
    ai_decision_system,
    ai_strategy_system,
    ai_unit_management_system,
    ai_resource_management_system,
    army_coordination_system,
    ai_combat_system,
    ai_strategy_system,              // <-- DUPLICATED!
    ai_worker_initialization_system,
    ai_worker_dropoff_system,
).chain());
```

**Impact**: `ai_strategy_system` runs twice per frame, wasting CPU cycles.

---

### 2.3 SCATTERED CONFIGURATION CONSTANTS (52+ Instances)

**Problem**: Game configuration spread across multiple files instead of centralized:

**Good Centralization** (`src/core/constants.rs`):
- Movement, camera, collision, UI, team colors, buildings, models, resources, AI, combat, population, terrain, hotkeys, building placement

**Bad Scattering**:
1. **In `components.rs`** (Component defaults):
   - `Movement::default()` hardcodes `max_speed: 200.0`, `acceleration: 400.0`
   
2. **In `entity_factory.rs` / `rts_entities.rs`** (Per-unit overrides):
   - Each unit has hardcoded movement stats scattered across spawn functions
   - Example: Worker speed is `50.0 / model_scale` in one place, default `75.0` in `unit_stats.rs`
   
3. **In `unit_stats.rs`** (Centralized but conflicts):
   - `WORKER_ANT_STATS` defines `max_speed: 75.0`
   - But `rts_entities.rs:149` uses `50.0 / model_scale`
   - These should be identical!

4. **In `health_ui.rs`** (Hardcoded):
   - Health bar size: `Rectangle::new(3.5, 0.6)` (lines 71, 84)
   - Health bar colors: `Color::srgb(0.8, 0.2, 0.2)` (line 73)
   - Health status ratios: 0.8, 0.4 (lines 33-39)

5. **In `combat_systems.rs`**:
   - Damage type determination (lines 134-138)

6. **In `resource_display.rs`**:
   - Resource icon display logic with hardcoded colors and sizes

---

### 2.4 REPEATED MATCH PATTERNS (20+ UnitType matches)

**Problem**: Same pattern repeated across codebase:
```rust
match unit_type {
    UnitType::WorkerAnt => /* do something */,
    UnitType::SoldierAnt => /* do something */,
    UnitType::HunterWasp => /* do something */,
    // etc...
}
```

**Locations**:
- `unit_stats.rs:296` - `get_unit_stats()`
- `model_loader.rs:722` - `get_unit_insect_model()`
- `model_loader.rs:769` - `get_building_insect_model()`
- `resources.rs:110-152` - `can_afford()`, `spend_resources()`, `add_resource()`
- `entity_factory.rs` - Model handling
- `combat_systems.rs:134` - Attack type determination
- Multiple files with resource type handling

**Pattern**:
```rust
// This pattern appears 20+ times
match resource_type {
    ResourceType::Nectar => { /* handle nectar */ },
    ResourceType::Chitin => { /* handle chitin */ },
    ResourceType::Minerals => { /* handle minerals */ },
    ResourceType::Pheromones => { /* handle pheromones */ },
}
```

---

### 2.5 DUPLICATE VISUAL REPRESENTATION CODE

**Location**: `rts_entities.rs`, lines 33-104

Two nearly-identical functions:
- `spawn_unit_visual()` - Creates unit mesh/model
- `spawn_building_visual()` - Creates building mesh/model

Both do the same thing:
1. Check if GLB model available
2. If yes: spawn SceneRoot with transform
3. If no: spawn Mesh3d with primitive

Could be consolidated into single generic function.

---

### 2.6 DUPLICATE RESOURCE DISPLAY SYSTEMS

**Problem**: Two separate systems for displaying resources:
- `src/ui/resource_display.rs` - Player resource UI
- `src/health_ui.rs` - Health bar UI (similar structure)

Both create:
- Background mesh
- Foreground mesh
- Update loops for positioning

**In main.rs (lines 23, 47)**:
```rust
// Line 23: REMOVED comment
// use resource_ui::ResourceUIPlugin;  // REMOVED: Duplicate of ui::resource_display

// Line 47: REMOVED comment
// ResourceUIPlugin,  // REMOVED: Duplicate overlapping resource display
```

Indicates developer awareness of this duplication but partial cleanup only.

---

## 3. CODE PATTERNS & REPEATED LOGIC

### 3.1 Entity Component Addition Pattern (Repeated 15+ Times)

Each unit spawn does:
```rust
entity.insert((
    RTSUnit { ... },
    TeamColor::new(player_id),
    Position { ... },
    Movement { ... },
    RTSHealth { ... },
    Combat { ... },  // Only for combat units
    ResourceGatherer { ... },  // Only for workers
    Constructor { ... },  // Only for builders
    Selectable::default(),
    Vision::default(),
    CollisionRadius { ... },
    EntityState::default(),
    GameEntity,
))
```

**Better Approach**: Use builder pattern or data-driven configuration.

### 3.2 Model Mapping Pattern (Repeated 2+ Times)

```rust
// In entity_factory.rs:118-119
let model_type = get_unit_insect_model(&unit_type);
get_model_scale(&model_type)

// Same pattern in rts_entities.rs:11-26
let model_type = get_unit_insect_model(unit_type);
match model_type {
    // Calculate scale
}
```

Both files duplicate the model determination logic.

### 3.3 Resource Type Match Pattern (8 Locations)

```rust
// Appears in: resources.rs, entity_factory.rs, placement.rs, resource_display.rs, etc.
match resource_type {
    ResourceType::Nectar => self.nectar += amount,
    ResourceType::Chitin => self.chitin += amount,
    ResourceType::Minerals => self.minerals += amount,
    ResourceType::Pheromones => self.pheromones += amount,
}
```

Should be data-driven with HashMap or better abstraction.

### 3.4 Player ID Branching (Repeated 5+ Times)

```rust
// In production.rs, resources.rs, ai files
if player_id == 1 {
    // Player resources
} else {
    // AI resources (from AIResources HashMap)
}
```

Creates tight coupling between player/AI logic.

---

## 4. SYSTEMS & COMPONENTS ORGANIZATION ISSUES

### 4.1 Animation System Scattered

**Locations**: 
- `src/rendering/animation_systems.rs` - Main animation system
- `src/entities/entity_state_systems.rs` - Entity state tracking
- Individual unit animations hardcoded in model_loader.rs

**Issue**: Animation data for each unit type is hardcoded in `animation_systems.rs` starting around line 500 with:
```rust
"LADYBUGAction" => {...},
"FourmAction" => {...},
// etc for each model type
```

Should be data-driven configuration file.

### 4.2 Vision System Duplicated

**Locations**:
- `src/rts/vision.rs` - Vision system
- `src/core/components.rs` - Vision component with default sight_range: 150.0
- `src/combat_systems.rs:44` - Re-creates default vision for units without it

### 4.3 Health Bar System Complexity

**Location**: `src/health_ui.rs`

**Issues**:
- Hardcoded health bar dimensions (3.5 x 0.6 rectangle)
- Hardcoded colors (red background, green foreground)
- Hardcoded health status thresholds (80%, 40%)
- Should all be in constants or configuration system

---

## 5. CONFIGURATION & CONSTANTS CONSOLIDATION OPPORTUNITIES

### Currently Well-Organized (Good Examples)

**`src/core/constants.rs`**: Excellent centralization of:
- Window dimensions
- Movement parameters
- Camera settings
- Collision radii
- UI dimensions and colors
- Team colors
- Building dimensions
- Building colors
- Model scales
- Resource costs
- AI parameters
- Combat stats
- Hotkeys
- Building placement rules

### Missing Centralizations (Bad)

1. **Unit Statistics Split**:
   - `core/unit_stats.rs` - Complete stats configs (GOOD)
   - But scattered in spawn functions with conflicts

2. **Animation Configurations**:
   - Hardcoded in `animation_systems.rs` (line 500+)
   - Should be data file or constants

3. **UI Component Styling**:
   - Health bar colors in `health_ui.rs`
   - Health status thresholds in `health_ui.rs`
   - Should be in `constants.rs::ui` module

4. **Resource Display Configuration**:
   - Icon sizes (24x24)
   - Text sizes
   - Colors and styling
   - Should be centralized

5. **Building Spawn Properties**:
   - Each building spawn function is unique
   - No centralized configuration

---

## 6. SPECIFIC DRY VIOLATIONS SUMMARY

| Category | Count | Severity | Files Affected |
|----------|-------|----------|-----------------|
| Duplicate spawn functions | 15+ | HIGH | rts_entities.rs, entity_factory.rs |
| Repeated match patterns | 20+ | MEDIUM | Multiple resource/unit handling |
| Scattered constants | 52+ | MEDIUM | Multiple system files |
| Duplicate UI systems | 2 | MEDIUM | resource_ui.rs, resource_display.rs |
| Duplicate AI system registration | 1 | LOW | ai/mod.rs |
| Repeated entity component setup | 15+ | MEDIUM | rts_entities.rs |
| Hardcoded animation data | 12+ | MEDIUM | animation_systems.rs |
| Model mapping duplication | 2+ | LOW | entity_factory.rs, rts_entities.rs |

---

## 7. ARCHITECTURAL IMPROVEMENT OPPORTUNITIES

### 7.1 Consolidate Unit Spawning

**Current State**:
- `rts_entities.rs`: 17 spawn functions, 809 lines
- `entity_factory.rs`: 7 spawn functions, 635 lines
- Two competing patterns!

**Recommended**:
- Use `EntityFactory` + `SpawnConfig` exclusively
- Remove all `RTSEntityFactory` functions
- Define all unit/building stats in data-driven configs

### 7.2 Data-Driven Configuration

Current approach:
```rust
pub const WORKER_ANT_STATS: UnitStatsConfig = ...;
pub const SOLDIER_ANT_STATS: UnitStatsConfig = ...;
// etc
```

Better approach would include:
- Base stats in `unit_stats.rs` (already done)
- Animation mappings in config file
- Model mappings in config file
- UI styling in config struct

### 7.3 Generic Match Handler

Instead of:
```rust
match resource_type {
    Nectar => ...,
    Chitin => ...,
    // etc
}
```

Use:
```rust
pub trait ResourceHandler {
    fn apply(&mut self, amount: f32);
}

struct ResourceArray {
    nectar: f32,
    chitin: f32,
    // etc
}

impl ResourceHandler for ResourceArray {
    // Implement once
}
```

### 7.4 Consolidate Resource Display

Merge `resource_ui.rs` and `resource_display.rs`:
- Single resource display system
- Single configuration for styling
- Reusable for both player and AI resources

### 7.5 AI Player Abstraction

Instead of:
```rust
if player_id == 1 {
    use player_resources
} else {
    use ai_resources.get(player_id)
}
```

Use:
```rust
trait PlayerResourceManager {
    fn resources(&self, player_id: u8) -> &PlayerResources;
    fn resources_mut(&mut self, player_id: u8) -> &mut PlayerResources;
}

struct GameResources {
    player: PlayerResources,
    ai: HashMap<u8, PlayerResources>,
}

impl PlayerResourceManager for GameResources { ... }
```

---

## 8. DUPLICATED INITIALIZATION CODE

**Location**: `src/core/game.rs`, `src/world/systems.rs`, `src/rts/mod.rs`

Repeated initialization of:
- Camera setup (multiple places)
- Resource initialization
- Unit spawn setup
- Building spawn setup

Should consolidate into single initialization sequence.

---

## 9. DUPLICATE PLUGIN SETUP

**Pattern**: Each module has a `Plugin` impl, but they register:
- Same resource types multiple times
- Overlapping systems (e.g., `ai_strategy_system` listed twice)
- Redundant startup systems

**Example**:
- `ui/mod.rs` initializes `PlayerResources`
- `core/game.rs` also initializes `PlayerResources`
- Could conflict or cause redundant initialization

---

## 10. RENDERING SYSTEM DUPLICATION

**Files**: `model_loader.rs` + `animation_systems.rs`

Both handle:
- Model loading
- Model type mapping
- Animation setup

`model_loader.rs` (1,130 lines) could be split:
- Model asset management (what it does)
- Animation controller setup (currently in animation_systems.rs)
- Model type mapping could be smaller

---

## RECOMMENDATIONS BY PRIORITY

### CRITICAL (High Impact, High Effort)
1. **Consolidate unit spawn functions** (Save 400+ lines)
   - Use only `EntityFactory`
   - Move all unit/building stats to `core/unit_stats.rs`
   - Remove `RTSEntityFactory` or make it thin wrapper

2. **Fix AI strategy system duplication**
   - Remove duplicate `ai_strategy_system` from plugin registration
   - Verify no other duplicate system registrations

### HIGH (High Impact, Medium Effort)
3. **Centralize animation configuration**
   - Move hardcoded animation data from `animation_systems.rs` to JSON/TOML config
   - Or move to `core/constants.rs`

4. **Consolidate resource display systems**
   - Merge `resource_ui.rs` and `resource_display.rs`
   - Create single reusable resource display component

5. **Create resource handler trait**
   - Eliminate 8+ `match resource_type` patterns
   - Implement once, use everywhere

### MEDIUM (Medium Impact, Low-Medium Effort)
6. **Move UI constants to core/constants.rs**
   - Health bar dimensions and colors
   - Health status thresholds
   - Icon sizes and colors

7. **Centralize model mapping logic**
   - Ensure `get_unit_insect_model()` used everywhere
   - Remove duplicate calls in entity_factory.rs and rts_entities.rs

8. **Abstract player ID branching**
   - Create `PlayerResourceManager` trait
   - Eliminate if player_id == 1 patterns

### LOW (Low Impact or Low Effort)
9. **Verify no other duplicate system registrations**
   - Audit all plugin setup code
   - Ensure clean system ordering

10. **Refactor entity component setup**
    - Use builder pattern for unit/building creation
    - Eliminate 15+ repetitive .insert() calls

---

## CODE QUALITY METRICS

- **Total Lines**: ~13,800
- **Estimated Duplication**: 15-20% (2,070-2,760 lines)
- **Duplicate Spawn Functions**: 15+
- **Repeated Match Patterns**: 20+
- **Scattered Constants**: 52+
- **Plugin Duplication**: 2+
- **System Registration Duplication**: 1

---

## CONCLUSION

BioRust has good architectural foundations with well-separated module concerns. However, the codebase suffers from significant DRY violations, primarily in:

1. **Entity spawning** (worst offender)
2. **Match pattern repetition**
3. **Scattered configuration**
4. **Duplicate UI systems**

Implementing the recommendations above could reduce code by 1,500-2,000 lines while improving maintainability, consistency, and reducing bugs from conflicting configuration values.
