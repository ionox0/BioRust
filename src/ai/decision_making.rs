use bevy::prelude::*;
use crate::core::components::*;
use crate::core::resources::*;
use crate::ai::player_state::{AIPlayer, AIType, AIDecision, PlayerCounts};
use crate::ai::intelligence::{IntelligenceSystem, EnemyStrategy, ThreatLevel};

pub fn ai_decision_system(
    mut ai_players: Query<&mut AIPlayer>,
    mut ai_resources: ResMut<AIResources>,
    game_costs: Res<GameCosts>,
    mut buildings: Query<(Entity, &mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    units: Query<&RTSUnit, With<RTSUnit>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    model_assets: Option<Res<crate::rendering::model_loader::ModelAssets>>,
    intelligence: Option<Res<IntelligenceSystem>>,
) {
    for mut ai_player in ai_players.iter_mut() {
        ai_player.decision_timer.tick(time.delta());

        if !ai_player.decision_timer.finished() {
            continue;
        }

        ai_player.decision_timer.reset();

        if let Some(resources) = ai_resources.resources.get(&ai_player.player_id).cloned() {
            let mut counts = PlayerCounts::new();
            counts.count_units(&units, ai_player.player_id);
            counts.count_buildings(&buildings, ai_player.player_id);

            // Get intelligence data for adaptive decision making
            let enemy_intel = intelligence.as_ref()
                .and_then(|intel| intel.get_intel(ai_player.player_id));

            let decision = make_adaptive_ai_decision(&ai_player.ai_type, &resources, &counts, enemy_intel);

            execute_ai_decision(
                decision,
                ai_player.player_id,
                &mut ai_resources,
                &game_costs,
                &mut buildings,
                &mut commands,
                &mut meshes,
                &mut materials,
                &model_assets,
            );
        }
    }
}

/// Adaptive decision making that considers enemy intelligence
fn make_adaptive_ai_decision(
    ai_type: &AIType,
    resources: &crate::core::resources::PlayerResources,
    counts: &PlayerCounts,
    enemy_intel: Option<&crate::ai::intelligence::PlayerIntelligence>,
) -> AIDecision {
    // First check if we need to counter enemy strategy
    if let Some(intel) = enemy_intel {
        // Counter enemy rush with defensive units
        if intel.enemy_strategy == EnemyStrategy::MilitaryRush && counts.military_count < 5 {
            if counts.barracks > 0 {
                return AIDecision::BuildMilitary(UnitType::SoldierAnt);
            } else if resources.chitin >= 175.0 {
                return AIDecision::BuildBuilding(BuildingType::WarriorChamber);
            }
        }

        // If enemy is economy focused, we can be aggressive early
        if intel.enemy_strategy == EnemyStrategy::EconomyRush && counts.military_count >= 3 {
            return AIDecision::AttackPlayer(intel.player_id);
        }

        // If under high threat, prioritize defense
        if intel.threat_level == ThreatLevel::High || intel.threat_level == ThreatLevel::Critical {
            if counts.military_count < (intel.enemy_unit_composition.military_units + 2) as i32 {
                if counts.barracks > 0 && resources.has_population_space() {
                    // Build counter units based on enemy composition
                    if intel.enemy_unit_composition.hunter_wasps > 2 {
                        // Counter ranged with melee rush
                        return AIDecision::BuildMilitary(UnitType::BeetleKnight);
                    } else {
                        return AIDecision::BuildMilitary(UnitType::SoldierAnt);
                    }
                }
            }
        }
    }

    // Fall back to personality-based decisions
    match ai_type {
        AIType::Economic => make_economic_decision(resources, counts),
        AIType::Aggressive => make_aggressive_decision(resources, counts),
        AIType::Balanced => make_balanced_decision(resources, counts),
    }
}

fn make_economic_decision(resources: &crate::core::resources::PlayerResources, counts: &PlayerCounts) -> AIDecision {
    use crate::constants::ai::{AI_MIN_WORKERS_FOR_WARRIOR_CHAMBER, AI_MAX_MILITARY_UNITS_EARLY};
    
    if resources.current_population >= resources.max_population && counts.houses < 3 {
        AIDecision::BuildBuilding(BuildingType::Nursery)
    } else if counts.villager_count < AI_MIN_WORKERS_FOR_WARRIOR_CHAMBER && resources.has_population_space() {
        AIDecision::BuildWorkerAnt
    } else if counts.barracks == 0 && resources.chitin >= 175.0 {
        AIDecision::BuildBuilding(BuildingType::WarriorChamber)
    } else if counts.barracks > 0 && counts.military_count < AI_MAX_MILITARY_UNITS_EARLY {
        AIDecision::BuildMilitary(UnitType::SoldierAnt)
    } else {
        AIDecision::GatherResources
    }
}

fn make_aggressive_decision(resources: &crate::core::resources::PlayerResources, counts: &PlayerCounts) -> AIDecision {
    use crate::constants::ai::{AI_MAX_MILITARY_UNITS_MID, AI_MIN_MILITARY_FOR_ATTACK};
    
    if counts.barracks == 0 && resources.chitin >= 175.0 {
        AIDecision::BuildBuilding(BuildingType::WarriorChamber)
    } else if counts.barracks > 0 && counts.military_count < AI_MAX_MILITARY_UNITS_MID {
        AIDecision::BuildMilitary(UnitType::SoldierAnt)
    } else if counts.military_count >= AI_MIN_MILITARY_FOR_ATTACK {
        AIDecision::AttackPlayer(1) // Attack player 1
    } else if counts.villager_count < 3 {
        AIDecision::BuildWorkerAnt
    } else {
        AIDecision::BuildBuilding(BuildingType::Nursery)
    }
}

fn make_balanced_decision(resources: &crate::core::resources::PlayerResources, counts: &PlayerCounts) -> AIDecision {
    use crate::constants::ai::AI_MIN_MILITARY_FOR_DEFEND;
    
    if resources.current_population >= resources.max_population {
        AIDecision::BuildBuilding(BuildingType::Nursery)
    } else if counts.villager_count < 3 {
        AIDecision::BuildWorkerAnt
    } else if counts.barracks == 0 {
        AIDecision::BuildBuilding(BuildingType::WarriorChamber)
    } else if counts.military_count < AI_MIN_MILITARY_FOR_DEFEND {
        AIDecision::BuildMilitary(UnitType::SoldierAnt)
    } else {
        AIDecision::GatherResources
    }
}

fn execute_ai_decision(
    decision: AIDecision,
    player_id: u8,
    ai_resources: &mut ResMut<AIResources>,
    game_costs: &Res<GameCosts>,
    buildings: &mut Query<(Entity, &mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    model_assets: &Option<Res<crate::rendering::model_loader::ModelAssets>>,
) {
    match decision {
        AIDecision::BuildWorkerAnt => {
            execute_build_worker(player_id, ai_resources, game_costs, buildings);
        },
        AIDecision::BuildMilitary(unit_type) => {
            execute_build_military(player_id, unit_type, ai_resources, game_costs, buildings);
        },
        AIDecision::BuildBuilding(building_type) => {
            execute_build_building(player_id, building_type, ai_resources, game_costs, commands, meshes, materials, &model_assets);
        },
        AIDecision::AttackPlayer(_target_player) => {
            info!("AI Player {} initiating attack!", player_id);
        },
        AIDecision::GatherResources => {
            info!("AI Player {} focusing on resource gathering", player_id);
        },
        AIDecision::Expand => {
            info!("AI Player {} expanding", player_id);
        }
    }
}

fn execute_build_worker(
    player_id: u8,
    ai_resources: &mut ResMut<AIResources>,
    game_costs: &Res<GameCosts>,
    buildings: &mut Query<(Entity, &mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
) {
    if let Some(cost) = game_costs.unit_costs.get(&UnitType::WorkerAnt) {
        for (_, mut queue, building, unit) in buildings.iter_mut() {
            if unit.player_id == player_id && 
               building.building_type == BuildingType::Queen &&
               building.is_complete {
                
                if let Some(resources) = ai_resources.resources.get_mut(&player_id) {
                    if resources.can_afford(cost) && resources.has_population_space() {
                        resources.spend_resources(cost);
                        queue.queue.push(UnitType::WorkerAnt);
                        info!("AI Player {} queuing worker ant", player_id);
                        break;
                    }
                }
            }
        }
    }
}

fn execute_build_military(
    player_id: u8,
    unit_type: UnitType,
    ai_resources: &mut ResMut<AIResources>,
    game_costs: &Res<GameCosts>,
    buildings: &mut Query<(Entity, &mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
) {
    if let Some(cost) = game_costs.unit_costs.get(&unit_type) {
        for (_, mut queue, building, unit) in buildings.iter_mut() {
            if unit.player_id == player_id && 
               building.building_type == BuildingType::WarriorChamber &&
               building.is_complete {
                
                if let Some(resources) = ai_resources.resources.get_mut(&player_id) {
                    if resources.can_afford(cost) && resources.has_population_space() {
                        resources.spend_resources(cost);
                        queue.queue.push(unit_type.clone());
                        info!("AI Player {} queuing {:?}", player_id, unit_type);
                        break;
                    }
                }
            }
        }
    }
}

fn execute_build_building(
    player_id: u8,
    building_type: BuildingType,
    ai_resources: &mut ResMut<AIResources>,
    game_costs: &Res<GameCosts>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    model_assets: &Option<Res<crate::rendering::model_loader::ModelAssets>>,
) {
    if let Some(cost) = game_costs.building_costs.get(&building_type) {
        if let Some(resources) = ai_resources.resources.get_mut(&player_id) {
            if resources.can_afford(cost) {
                resources.spend_resources(cost);
                
                let position = generate_ai_building_position();
                
                use crate::entities::entity_factory::{EntityFactory, SpawnConfig, EntityType};

                let config = SpawnConfig::building(EntityType::Building(building_type.clone()), position, player_id);
                let model_assets_ref = model_assets.as_ref().map(|r| &**r);
                EntityFactory::spawn(commands, meshes, materials, config, model_assets_ref);

                // Add housing for nurseries
                if building_type == BuildingType::Nursery {
                    resources.add_housing(5); // Nurseries provide 5 population
                }
                
                info!("AI Player {} built {:?}", player_id, building_type);
            }
        }
    }
}

fn generate_ai_building_position() -> Vec3 {
    let mut rng = rand::thread_rng();
    use rand::Rng;
    
    Vec3::new(
        rng.gen_range(-100.0..100.0),
        10.0,
        rng.gen_range(50.0..150.0), // AI builds in positive Z area
    )
}