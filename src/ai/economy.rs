use bevy::prelude::*;
use crate::core::components::*;
use crate::core::resources::*;
use std::collections::HashMap;

/// Manages optimal worker distribution across resources
#[derive(Resource, Debug, Clone)]
pub struct EconomyManager {
    pub player_economy: HashMap<u8, PlayerEconomy>,
}

#[derive(Debug, Clone)]
pub struct PlayerEconomy {
    pub resource_priorities: Vec<ResourceAllocation>,
    pub ideal_worker_distribution: HashMap<ResourceType, u32>,
    pub last_optimization_time: f32,
}

#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub resource_type: ResourceType,
    pub current_workers: u32,
    pub ideal_workers: u32,
    pub priority: f32,
}

impl Default for EconomyManager {
    fn default() -> Self {
        let mut player_economy = HashMap::new();

        // Initialize AI player 2 with balanced economy
        let mut ideal_distribution = HashMap::new();
        ideal_distribution.insert(ResourceType::Nectar, 3);    // Food priority
        ideal_distribution.insert(ResourceType::Chitin, 3);    // Building materials
        ideal_distribution.insert(ResourceType::Minerals, 2);  // Secondary
        ideal_distribution.insert(ResourceType::Pheromones, 2); // Least priority

        player_economy.insert(2, PlayerEconomy {
            resource_priorities: Vec::new(),
            ideal_worker_distribution: ideal_distribution,
            last_optimization_time: 0.0,
        });

        Self { player_economy }
    }
}

/// System to optimize worker distribution
pub fn economy_optimization_system(
    mut economy_manager: ResMut<EconomyManager>,
    ai_resources: Res<AIResources>,
    mut workers: Query<(Entity, &mut ResourceGatherer, &mut Movement, &RTSUnit, &Transform), With<ResourceGatherer>>,
    resource_sources: Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    buildings: Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (player_id, economy) in economy_manager.player_economy.iter_mut() {
        // Only optimize every 10 seconds
        if current_time - economy.last_optimization_time < 10.0 {
            continue;
        }
        economy.last_optimization_time = current_time;

        // Get current resources to adjust priorities dynamically
        if let Some(resources) = ai_resources.resources.get(player_id) {
            adjust_resource_priorities(economy, resources);
        }

        // Count current worker distribution
        let mut current_distribution = HashMap::new();
        for (_entity, gatherer, unit, _transform) in workers.iter() {
            if unit.player_id == *player_id {
                if let Some(resource_type) = &gatherer.resource_type {
                    *current_distribution.entry(resource_type.clone()).or_insert(0) += 1;
                }
            }
        }

        // Calculate allocations
        let mut allocations = Vec::new();
        for (resource_type, ideal_count) in &economy.ideal_worker_distribution {
            let current = *current_distribution.get(resource_type).unwrap_or(&0);
            allocations.push(ResourceAllocation {
                resource_type: resource_type.clone(),
                current_workers: current,
                ideal_workers: *ideal_count,
                priority: calculate_resource_priority(resource_type, current, *ideal_count),
            });
        }

        // Sort by priority (highest first)
        allocations.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
        economy.resource_priorities = allocations.clone();

        // Reassign workers based on priorities
        reassign_workers(
            *player_id,
            &allocations,
            &mut workers,
            &resource_sources,
            &buildings,
        );
    }
}

/// Adjust resource priorities dynamically based on current resources
fn adjust_resource_priorities(economy: &mut PlayerEconomy, resources: &PlayerResources) {
    // If low on a resource, increase its priority
    if resources.nectar < 100.0 {
        economy.ideal_worker_distribution.insert(ResourceType::Nectar, 4);
    }
    if resources.chitin < 100.0 {
        economy.ideal_worker_distribution.insert(ResourceType::Chitin, 4);
    }

    // If high on a resource, reduce its priority
    if resources.nectar > 500.0 {
        economy.ideal_worker_distribution.insert(ResourceType::Nectar, 2);
    }
    if resources.chitin > 500.0 {
        economy.ideal_worker_distribution.insert(ResourceType::Chitin, 2);
    }

    // Always need some minerals and pheromones
    economy.ideal_worker_distribution.entry(ResourceType::Minerals).or_insert(2);
    economy.ideal_worker_distribution.entry(ResourceType::Pheromones).or_insert(2);
}

/// Calculate priority for a resource type
fn calculate_resource_priority(
    _resource_type: &ResourceType,
    current_workers: u32,
    ideal_workers: u32,
) -> f32 {
    // Higher priority if we have fewer workers than ideal
    if current_workers < ideal_workers {
        (ideal_workers - current_workers) as f32 * 2.0
    } else if current_workers > ideal_workers {
        // Negative priority if we have too many workers
        -((current_workers - ideal_workers) as f32)
    } else {
        0.0
    }
}

/// Reassign workers to balance resource gathering
fn reassign_workers(
    player_id: u8,
    allocations: &[ResourceAllocation],
    workers: &mut Query<(Entity, &mut ResourceGatherer, &mut Movement, &RTSUnit, &Transform), With<ResourceGatherer>>,
    resource_sources: &Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    buildings: &Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) {
    use crate::core::components::BuildingType;

    // First, check if player has any suitable drop-off buildings
    let has_dropoff_building = buildings
        .iter()
        .any(|(_, _, building, building_unit)| {
            building_unit.player_id == player_id &&
            building.is_complete &&
            matches!(building.building_type, BuildingType::Queen | BuildingType::StorageChamber | BuildingType::Nursery)
        });

    // Don't reassign workers if no buildings can accept resources
    if !has_dropoff_building {
        return;
    }
    // First, check resource availability for each allocation
    let mut available_resources: HashMap<ResourceType, usize> = HashMap::new();
    for (_, source, _) in resource_sources.iter() {
        *available_resources.entry(source.resource_type.clone()).or_insert(0) += 1;
    }

    // Find workers that need reassignment (on low priority resources)
    let mut workers_to_reassign = Vec::new();

    for allocation in allocations.iter().rev() {
        // If we have too many workers on this resource
        if allocation.current_workers > allocation.ideal_workers {
            let excess = allocation.current_workers - allocation.ideal_workers;

            // Find workers on this resource to reassign
            for (worker_entity, gatherer, unit, transform) in workers.iter() {
                if unit.player_id == player_id &&
                   gatherer.resource_type.as_ref() == Some(&allocation.resource_type) &&
                   workers_to_reassign.len() < excess as usize {
                    workers_to_reassign.push((worker_entity, transform.translation));
                }
            }
        }
    }

    // Reassign workers to high priority resources
    for allocation in allocations.iter() {
        if allocation.current_workers < allocation.ideal_workers && !workers_to_reassign.is_empty() {
            let needed = (allocation.ideal_workers - allocation.current_workers) as usize;

            // Check if this resource type is actually available
            let available_count = available_resources.get(&allocation.resource_type).copied().unwrap_or(0);
            if available_count == 0 {
                warn!("Player {} wants to assign workers to {:?}, but no sources available!",
                      player_id, allocation.resource_type);
                continue;
            }

            // Find resource sources of this type
            let mut sources: Vec<(Entity, Vec3)> = resource_sources
                .iter()
                .filter(|(_, source, _)| source.resource_type == allocation.resource_type)
                .map(|(entity, _, transform)| (entity, transform.translation))
                .collect();

            if sources.is_empty() {
                continue;
            }

            // Reassign workers
            for _ in 0..needed.min(workers_to_reassign.len()) {
                if let Some((worker_entity, worker_pos)) = workers_to_reassign.pop() {
                    // Find closest resource source
                    sources.sort_by(|a, b| {
                        let dist_a = worker_pos.distance(a.1);
                        let dist_b = worker_pos.distance(b.1);
                        dist_a.partial_cmp(&dist_b).unwrap()
                    });

                    if let Some((source_entity, source_position)) = sources.first() {
                        if let Ok((_, mut gatherer, mut movement, unit, worker_transform)) = workers.get_mut(worker_entity) {
                            // Find nearest suitable drop-off building
                            // Only complete Queens, StorageChambers, and Nurseries can accept resources
                            let nearest_building = buildings
                                .iter()
                                .filter(|(_, _, building, building_unit)| {
                                    building_unit.player_id == player_id &&
                                    building.is_complete &&
                                    matches!(building.building_type, BuildingType::Queen | BuildingType::StorageChamber | BuildingType::Nursery)
                                })
                                .min_by(|a, b| {
                                    let dist_a = worker_transform.translation.distance(a.1.translation);
                                    let dist_b = worker_transform.translation.distance(b.1.translation);
                                    dist_a.partial_cmp(&dist_b).unwrap()
                                })
                                .map(|(entity, _, _, _)| entity);

                            // Only reassign if we found a suitable drop-off building
                            if let Some(building) = nearest_building {
                                gatherer.target_resource = Some(*source_entity);
                                gatherer.resource_type = Some(allocation.resource_type.clone());
                                gatherer.carried_amount = 0.0;
                                gatherer.drop_off_building = Some(building);
                                // CRITICAL: Tell worker to actually MOVE to the resource!
                                movement.target_position = Some(*source_position);
                                info!("🚀 Reassigning worker {} (player {}) to gather {:?} at distance {:.1}",
                                      unit.unit_id, player_id, allocation.resource_type,
                                      worker_transform.translation.distance(*source_position));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// System to handle worker idle detection and assignment
pub fn worker_idle_detection_system(
    economy_manager: Res<EconomyManager>,
    mut workers: Query<(Entity, &mut ResourceGatherer, &mut Movement, &RTSUnit, &Transform), With<ResourceGatherer>>,
    resource_sources: Query<(Entity, &ResourceSource, &Transform), Without<RTSUnit>>,
    buildings: Query<(Entity, &Transform, &Building, &RTSUnit), With<Building>>,
) {
    use crate::core::components::BuildingType;

    for (player_id, economy) in &economy_manager.player_economy {
        // First, check if player has any suitable drop-off buildings
        // Only Queens, StorageChambers, and Nurseries can accept resources
        let has_dropoff_building = buildings
            .iter()
            .any(|(_, _, building, building_unit)| {
                building_unit.player_id == *player_id &&
                building.is_complete &&
                matches!(building.building_type, BuildingType::Queen | BuildingType::StorageChamber | BuildingType::Nursery)
            });

        // Don't assign workers to gather if no buildings can accept resources
        if !has_dropoff_building {
            continue;
        }

        // Try to find an available resource type from priorities
        let mut assigned_resource_type = None;

        for priority_allocation in &economy.resource_priorities {
            // Check if this resource type has available sources
            let has_sources = resource_sources
                .iter()
                .any(|(_, source, _)| source.resource_type == priority_allocation.resource_type);

            if has_sources {
                assigned_resource_type = Some(priority_allocation.resource_type.clone());
                break;
            }
        }

        // If no resource priorities or no available sources, skip
        let Some(target_resource_type) = assigned_resource_type else {
            continue;
        };

        // Find idle workers
        for (_worker_entity, mut gatherer, mut movement, unit, worker_transform) in workers.iter_mut() {
            if unit.player_id == *player_id &&
               gatherer.target_resource.is_none() &&
               gatherer.carried_amount == 0.0 {
                // Assign to nearest available resource of the target type
                if let Some((source_entity, _, resource_transform)) = resource_sources
                    .iter()
                    .filter(|(_, source, _)| source.resource_type == target_resource_type)
                    .min_by(|a, b| {
                        let dist_a = worker_transform.translation.distance(a.2.translation);
                        let dist_b = worker_transform.translation.distance(b.2.translation);
                        dist_a.partial_cmp(&dist_b).unwrap()
                    })
                {
                    // Find nearest suitable drop-off building
                    // Only complete Queens, StorageChambers, and Nurseries can accept resources
                    let nearest_building = buildings
                        .iter()
                        .filter(|(_, _, building, building_unit)| {
                            building_unit.player_id == *player_id &&
                            building.is_complete &&
                            matches!(building.building_type, BuildingType::Queen | BuildingType::StorageChamber | BuildingType::Nursery)
                        })
                        .min_by(|a, b| {
                            let dist_a = worker_transform.translation.distance(a.1.translation);
                            let dist_b = worker_transform.translation.distance(b.1.translation);
                            dist_a.partial_cmp(&dist_b).unwrap()
                        })
                        .map(|(entity, _, _, _)| entity);

                    // Only assign if we found a suitable drop-off building
                    if let Some(building) = nearest_building {
                        gatherer.target_resource = Some(source_entity);
                        gatherer.resource_type = Some(target_resource_type.clone());
                        gatherer.drop_off_building = Some(building);
                        // CRITICAL: Tell worker to actually MOVE to the resource!
                        movement.target_position = Some(resource_transform.translation);
                        info!("🚀 Assigned worker {} (player {}) to gather {:?} at distance {:.1}",
                              unit.unit_id, unit.player_id, target_resource_type,
                              worker_transform.translation.distance(resource_transform.translation));
                    }
                }
            }
        }
    }
}
