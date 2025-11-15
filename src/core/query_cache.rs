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
}


#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_units: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub last_update_ms: f64,
}

impl UnitQueryCache {

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


    /// Update cache statistics
    pub fn update_stats(&mut self, update_time_ms: f64) {
        self.stats.total_units = self.basic_units.len();
        self.stats.last_update_ms = update_time_ms;
    }

}

/// System to update the unit query cache
pub fn update_unit_cache_system(
    mut cache: ResMut<UnitQueryCache>,
    basic_query: Query<(Entity, &Transform, &RTSUnit), With<RTSUnit>>,
    health_query: Query<(Entity, &Transform, &RTSUnit, &RTSHealth), With<RTSUnit>>,
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