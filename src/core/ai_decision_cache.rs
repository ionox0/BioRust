/// AI Decision Cache System for Performance Optimization
/// Caches expensive AI decisions and only recomputes them when game state changes significantly

use bevy::prelude::*;
use crate::core::components::*;
use std::collections::HashMap;

/// Cached AI decision for a specific entity
#[derive(Debug, Clone)]
pub struct CachedDecision {
    /// The actual decision made
    pub decision: AIDecisionType,
    /// When this decision was made
    pub timestamp: f32,
    /// How long this decision should remain valid
    pub validity_duration: f32,
    /// Hash of the game state when decision was made
    pub state_hash: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AIDecisionType {
    /// Attack a specific target
    Attack { target: Entity, position: Vec3 },
    /// Move to a specific location
    Move { position: Vec3, purpose: MovementPurpose },
    /// Gather from a specific resource
    Gather { resource: Entity, resource_type: ResourceType },
    /// Return resources to a building
    ReturnResources { building: Entity },
    /// Build a specific structure
    Build { building_type: BuildingType, position: Vec3 },
    /// Train a specific unit
    TrainUnit { unit_type: UnitType, building: Entity },
    /// Wait/idle for a specified duration
    Wait { duration: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementPurpose {
    Patrol,
    Explore,
    Retreat,
    Support,
    Formation,
}

/// Resource managing all AI decision caches
#[derive(Resource, Default)]
pub struct AIDecisionCache {
    /// Cached decisions for each entity
    decisions: HashMap<Entity, CachedDecision>,
    /// Global state hash for cache invalidation
    global_state_hash: u64,
    /// Last time global state was checked
    last_global_check: f32,
    /// Statistics for monitoring cache performance
    stats: CacheStats,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_decisions: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub invalidations: u64,
    pub state_changes: u64,
}

impl AIDecisionCache {
    /// Check if a cached decision exists and is still valid
    pub fn get_valid_decision(&mut self, entity: Entity, current_time: f32, current_state_hash: u64) -> Option<AIDecisionType> {
        if let Some(cached) = self.decisions.get(&entity) {
            // Check if decision is still temporally valid
            let age = current_time - cached.timestamp;
            if age <= cached.validity_duration {
                // Check if game state hasn't changed significantly
                if cached.state_hash == current_state_hash {
                    self.stats.cache_hits += 1;
                    return Some(cached.decision.clone());
                }
            }
            
            // Decision is invalid, remove it
            self.decisions.remove(&entity);
            self.stats.invalidations += 1;
        }
        
        self.stats.cache_misses += 1;
        None
    }

    /// Store a new AI decision
    pub fn cache_decision(
        &mut self, 
        entity: Entity, 
        decision: AIDecisionType, 
        current_time: f32,
        validity_duration: f32,
        state_hash: u64
    ) {
        let cached_decision = CachedDecision {
            decision,
            timestamp: current_time,
            validity_duration,
            state_hash,
        };
        
        self.decisions.insert(entity, cached_decision);
        self.stats.total_decisions = self.decisions.len();
    }

    /// Invalidate decisions for a specific entity
    pub fn invalidate_entity(&mut self, entity: Entity) {
        if self.decisions.remove(&entity).is_some() {
            self.stats.invalidations += 1;
        }
    }

    /// Invalidate all decisions (e.g., when major game state changes)
    pub fn invalidate_all(&mut self) {
        let count = self.decisions.len();
        self.decisions.clear();
        self.stats.invalidations += count as u64;
        self.stats.total_decisions = 0;
    }

    /// Update global state hash and invalidate decisions if it changed
    pub fn update_global_state(&mut self, new_hash: u64, current_time: f32) {
        if self.global_state_hash != new_hash {
            self.global_state_hash = new_hash;
            self.stats.state_changes += 1;
            
            // Invalidate decisions that depend on global state
            self.decisions.retain(|_, decision| {
                decision.state_hash == new_hash || 
                (current_time - decision.timestamp) < 1.0 // Keep very recent decisions
            });
        }
        self.last_global_check = current_time;
    }

    /// Get cache hit rate for monitoring
    pub fn get_hit_rate(&self) -> f64 {
        let total_requests = self.stats.cache_hits + self.stats.cache_misses;
        if total_requests > 0 {
            self.stats.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }
}

/// Calculate a hash representing the current game state for cache invalidation
pub fn calculate_game_state_hash(
    unit_count: usize,
    building_count: usize,
    resource_sources: usize,
    combat_units: usize,
) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    unit_count.hash(&mut hasher);
    building_count.hash(&mut hasher);
    resource_sources.hash(&mut hasher);
    combat_units.hash(&mut hasher);
    hasher.finish()
}

/// System to update AI decision cache with current game state
pub fn update_ai_decision_cache_system(
    mut cache: ResMut<AIDecisionCache>,
    units: Query<Entity, With<RTSUnit>>,
    buildings: Query<Entity, With<Building>>,
    resources: Query<Entity, With<ResourceSource>>,
    combat_units: Query<Entity, (With<RTSUnit>, With<Combat>)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    // Only update global state hash periodically
    if current_time - cache.last_global_check > 1.0 {
        let unit_count = units.iter().count();
        let building_count = buildings.iter().count();
        let resource_count = resources.iter().count();
        let combat_count = combat_units.iter().count();
        
        let new_hash = calculate_game_state_hash(
            unit_count,
            building_count, 
            resource_count,
            combat_count
        );
        
        cache.update_global_state(new_hash, current_time);
    }
}

/// System to clean up cache for despawned entities
pub fn cleanup_ai_decision_cache_system(
    mut cache: ResMut<AIDecisionCache>,
    mut removed_units: RemovedComponents<RTSUnit>,
) {
    for entity in removed_units.read() {
        cache.invalidate_entity(entity);
    }
}

/// Helper function to get validity duration based on decision type
pub fn get_decision_validity(decision: &AIDecisionType) -> f32 {
    match decision {
        AIDecisionType::Attack { .. } => 2.0,      // Combat decisions change quickly
        AIDecisionType::Move { purpose, .. } => match purpose {
            MovementPurpose::Patrol => 10.0,       // Patrol routes are stable
            MovementPurpose::Explore => 15.0,      // Exploration targets are stable  
            MovementPurpose::Retreat => 3.0,       // Retreats need quick updates
            MovementPurpose::Support => 5.0,       // Support positions change
            MovementPurpose::Formation => 8.0,     // Formation positions are stable
        },
        AIDecisionType::Gather { .. } => 12.0,     // Resource gathering is stable
        AIDecisionType::ReturnResources { .. } => 8.0,  // Return trips are predictable
        AIDecisionType::Build { .. } => 20.0,      // Building decisions are long-term
        AIDecisionType::TrainUnit { .. } => 15.0,  // Unit training is planned
        AIDecisionType::Wait { duration } => *duration, // Wait as long as specified
    }
}

/// Macro to simplify caching AI decisions
#[macro_export]
macro_rules! cache_ai_decision {
    ($cache:expr, $entity:expr, $decision:expr, $time:expr, $state_hash:expr) => {{
        let validity = $crate::core::ai_decision_cache::get_decision_validity(&$decision);
        $cache.cache_decision($entity, $decision.clone(), $time, validity, $state_hash);
        $decision
    }};
}

/// Plugin to add AI decision caching
pub struct AIDecisionCachePlugin;

impl Plugin for AIDecisionCachePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AIDecisionCache>()
            .add_systems(
                Update,
                (
                    update_ai_decision_cache_system,
                    cleanup_ai_decision_cache_system,
                )
            );
    }
}