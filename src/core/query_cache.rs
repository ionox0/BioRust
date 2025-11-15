/// High-performance query cache system to eliminate redundant ECS queries
/// This system consolidates all RTSUnit queries into shared cached resources
/// to dramatically improve performance in scenarios with many units.

use crate::core::components::*;
use bevy::prelude::*;
use std::collections::HashMap;

/// Cached query results for all RTSUnit entities with comprehensive data
#[derive(Resource, Default, Debug)]
pub struct UnitQueryCache {
    /// All units with basic info (Entity, Transform, RTSUnit)
    pub basic_units: HashMap<Entity, BasicUnitData>,
    /// Units with health info
    pub health_units: HashMap<Entity, HealthUnitData>,
    /// Units with combat info  
    pub combat_units: HashMap<Entity, CombatUnitData>,
    /// Units with movement info
    pub movement_units: HashMap<Entity, MovementUnitData>,
    /// Units with gathering info
    pub gatherer_units: HashMap<Entity, GathererUnitData>,
    /// Dirty flag to indicate when cache needs refresh
    pub dirty: bool,
    /// Last update frame for cache invalidation
    pub last_update_frame: u32,
    /// Cache statistics for monitoring
    pub stats: CacheStats,
}

#[derive(Debug, Clone)]
pub struct BasicUnitData {
    pub entity: Entity,
    pub transform: Transform,
    pub unit: RTSUnit,
    pub player_id: u8,
}

#[derive(Debug, Clone)]
pub struct HealthUnitData {
    pub basic: BasicUnitData,
    pub health: RTSHealth,
    pub max_health: f32,
}

#[derive(Debug, Clone)]
pub struct CombatUnitData {
    pub basic: BasicUnitData,
    pub combat: Combat,
    pub health: RTSHealth,
}

#[derive(Debug, Clone)]
pub struct MovementUnitData {
    pub basic: BasicUnitData,
    pub movement: Movement,
}

#[derive(Debug, Clone)]
pub struct GathererUnitData {
    pub basic: BasicUnitData,
    pub gatherer: ResourceGatherer,
    pub movement: Movement,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_units: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub last_update_ms: f64,
}

impl UnitQueryCache {
    pub fn new() -> Self {
        Self {
            basic_units: HashMap::new(),
            health_units: HashMap::new(),
            combat_units: HashMap::new(),
            movement_units: HashMap::new(),
            gatherer_units: HashMap::new(),
            dirty: true,
            last_update_frame: 0,
            stats: CacheStats::default(),
        }
    }

    /// Mark cache as dirty (needs refresh)
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    /// Get all units for a specific player
    pub fn get_player_units(&self, player_id: u8) -> Vec<&BasicUnitData> {
        self.basic_units
            .values()
            .filter(|unit| unit.player_id == player_id)
            .collect()
    }

    /// Get all enemy units relative to a player
    pub fn get_enemy_units(&self, player_id: u8) -> Vec<&BasicUnitData> {
        self.basic_units
            .values()
            .filter(|unit| unit.player_id != player_id)
            .collect()
    }

    /// Get units within a certain range of a position
    pub fn get_units_in_range(&self, position: Vec3, range: f32) -> Vec<&BasicUnitData> {
        let range_sq = range * range;
        self.basic_units
            .values()
            .filter(|unit| unit.transform.translation.distance_squared(position) <= range_sq)
            .collect()
    }

    /// Get combat units for a player
    pub fn get_player_combat_units(&self, player_id: u8) -> Vec<&CombatUnitData> {
        self.combat_units
            .values()
            .filter(|unit| unit.basic.player_id == player_id)
            .collect()
    }

    /// Get gatherer units for a player
    pub fn get_player_gatherers(&self, player_id: u8) -> Vec<&GathererUnitData> {
        self.gatherer_units
            .values()
            .filter(|unit| unit.basic.player_id == player_id)
            .collect()
    }

    /// Get idle gatherers (not currently gathering or moving)
    pub fn get_idle_gatherers(&self, player_id: u8) -> Vec<&GathererUnitData> {
        self.gatherer_units
            .values()
            .filter(|unit| {
                unit.basic.player_id == player_id
                    && unit.gatherer.target_resource.is_none()
                    && unit.movement.target_position.is_none()
            })
            .collect()
    }

    /// Update cache statistics
    pub fn update_stats(&mut self, update_time_ms: f64) {
        self.stats.total_units = self.basic_units.len();
        self.stats.last_update_ms = update_time_ms;
    }

    /// Record cache hit for monitoring
    pub fn record_hit(&mut self) {
        self.stats.cache_hits += 1;
    }

    /// Record cache miss for monitoring
    pub fn record_miss(&mut self) {
        self.stats.cache_misses += 1;
    }
}

/// System to update the unit query cache
pub fn update_unit_cache_system(
    mut cache: ResMut<UnitQueryCache>,
    basic_query: Query<(Entity, &Transform, &RTSUnit), With<RTSUnit>>,
    health_query: Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
    combat_query: Query<(Entity, &Transform, &RTSUnit, &Combat, &RTSHealth), (With<RTSUnit>, With<Combat>)>,
    movement_query: Query<(Entity, &Transform, &RTSUnit, &Movement), (With<RTSUnit>, With<Movement>)>,
    gatherer_query: Query<(Entity, &Transform, &RTSUnit, &ResourceGatherer, &Movement), (With<RTSUnit>, With<ResourceGatherer>)>,
    time: Res<Time>,
) {
    let start_time = std::time::Instant::now();
    
    // Only update if cache is dirty or if significant time has passed
    let current_frame = time.elapsed_secs() as u32;
    if !cache.dirty && current_frame <= cache.last_update_frame + 1 {
        return;
    }

    // Clear existing cache data
    cache.basic_units.clear();
    cache.health_units.clear();
    cache.combat_units.clear();
    cache.movement_units.clear();
    cache.gatherer_units.clear();

    // Update basic unit cache
    for (entity, transform, unit) in basic_query.iter() {
        cache.basic_units.insert(entity, BasicUnitData {
            entity,
            transform: *transform,
            unit: unit.clone(),
            player_id: unit.player_id,
        });
    }

    // Update health unit cache
    for (entity, transform, unit, health) in health_query.iter() {
        let basic = BasicUnitData {
            entity,
            transform: *transform,
            unit: unit.clone(),
            player_id: unit.player_id,
        };
        cache.health_units.insert(entity, HealthUnitData {
            basic: basic.clone(),
            health: health.clone(),
            max_health: health.max,
        });
    }

    // Update combat unit cache
    for (entity, transform, unit, combat, health) in combat_query.iter() {
        let basic = BasicUnitData {
            entity,
            transform: *transform,
            unit: unit.clone(),
            player_id: unit.player_id,
        };
        cache.combat_units.insert(entity, CombatUnitData {
            basic,
            combat: combat.clone(),
            health: health.clone(),
        });
    }

    // Update movement unit cache
    for (entity, transform, unit, movement) in movement_query.iter() {
        let basic = BasicUnitData {
            entity,
            transform: *transform,
            unit: unit.clone(),
            player_id: unit.player_id,
        };
        cache.movement_units.insert(entity, MovementUnitData {
            basic,
            movement: movement.clone(),
        });
    }

    // Update gatherer unit cache
    for (entity, transform, unit, gatherer, movement) in gatherer_query.iter() {
        let basic = BasicUnitData {
            entity,
            transform: *transform,
            unit: unit.clone(),
            player_id: unit.player_id,
        };
        cache.gatherer_units.insert(entity, GathererUnitData {
            basic,
            gatherer: gatherer.clone(),
            movement: movement.clone(),
        });
    }

    // Update cache metadata
    cache.dirty = false;
    cache.last_update_frame = current_frame;
    
    let update_time = start_time.elapsed().as_secs_f64() * 1000.0;
    cache.update_stats(update_time);

    // Log performance metrics periodically
    if current_frame % 300 == 0 { // Every 5 seconds at 60fps
        debug!(
            "Unit Cache: {} units, {:.2}ms update, {:.1}% hit rate",
            cache.stats.total_units,
            cache.stats.last_update_ms,
            if cache.stats.cache_hits + cache.stats.cache_misses > 0 {
                cache.stats.cache_hits as f64 / (cache.stats.cache_hits + cache.stats.cache_misses) as f64 * 100.0
            } else { 0.0 }
        );
    }
}

/// System to detect when cache should be invalidated
pub fn cache_invalidation_system(
    mut cache: ResMut<UnitQueryCache>,
    changed_units: Query<Entity, (With<RTSUnit>, Or<(Changed<Transform>, Changed<RTSHealth>, Changed<Combat>, Changed<Movement>, Changed<ResourceGatherer>)>)>,
    added_units: Query<Entity, Added<RTSUnit>>,
    mut removed: RemovedComponents<RTSUnit>,
) {
    // Check for any changes that would invalidate the cache
    if !changed_units.is_empty() || !added_units.is_empty() || removed.read().count() > 0 {
        cache.invalidate();
    }
}

/// Plugin to add the query cache system
pub struct QueryCachePlugin;

impl Plugin for QueryCachePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UnitQueryCache>()
            .add_systems(
                Update,
                (
                    cache_invalidation_system,
                    update_unit_cache_system.after(cache_invalidation_system),
                )
                .chain(),
            );
    }
}