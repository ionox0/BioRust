use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use rand::Rng;

#[derive(Component, Debug)]
pub struct AIPlayer {
    pub player_id: u8,
    pub ai_type: AIType,
    pub decision_timer: Timer,
    pub build_order_index: usize,
}

#[derive(Debug, Clone)]
pub enum AIType {
    Aggressive,
    Economic,
    Balanced,
}

#[derive(Debug, Clone)]
pub enum AIDecision {
    BuildWorkerAnt,
    BuildMilitary(UnitType),
    BuildBuilding(BuildingType),
    AttackPlayer(u8),
    GatherResources,
    Expand,
}

pub struct AISystemsPlugin;

impl Plugin for AISystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            ai_decision_system,
            ai_unit_management_system,
            ai_resource_management_system,
            ai_combat_system,
        ));
    }
}

// Main AI decision making system
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
    model_assets: Option<Res<crate::model_loader::ModelAssets>>,
) {
    use crate::constants::ai::*;
    for mut ai_player in ai_players.iter_mut() {
        ai_player.decision_timer.tick(time.delta());
        
        if ai_player.decision_timer.finished() {
            // Reset timer for next decision
            ai_player.decision_timer.reset();
            
            let player_resources = ai_resources.resources.get(&ai_player.player_id).cloned();
            if let Some(resources) = player_resources {
                
                // Count current units and buildings
                let mut villager_count = 0;
                let mut military_count = 0;
                // let mut town_centers = 0; // Not used in AI logic yet
                let mut houses = 0;
                let mut barracks = 0;
                
                for unit in units.iter() {
                    if unit.player_id == ai_player.player_id {
                        // Count different unit types
                        if let Some(unit_type) = &unit.unit_type {
                            match unit_type {
                                UnitType::WorkerAnt => villager_count += 1,
                                UnitType::SoldierAnt | UnitType::HunterWasp | UnitType::BeetleKnight | UnitType::SpearMantis | UnitType::ScoutAnt | UnitType::BatteringBeetle | UnitType::AcidSpitter => {
                                    military_count += 1;
                                },
                            }
                        }
                    }
                }
                
                for (_, _, building, unit) in buildings.iter() {
                    if unit.player_id == ai_player.player_id {
                        match building.building_type {
                            BuildingType::Queen => { /* town centers tracked but not used yet */ },
                            BuildingType::Nursery => houses += 1,
                            BuildingType::WarriorChamber => barracks += 1,
                            _ => {}
                        }
                    }
                }
                
                // Simple AI logic based on current state and AI type
                let decision = match ai_player.ai_type {
                    AIType::Economic => {
                        if resources.current_population >= resources.max_population && houses < 3 {
                            AIDecision::BuildBuilding(BuildingType::Nursery)
                        } else if villager_count < AI_MIN_VILLAGERS_FOR_BARRACKS && resources.has_population_space() {
                            AIDecision::BuildWorkerAnt
                        } else if barracks == 0 && resources.wood >= 175.0 {
                            AIDecision::BuildBuilding(BuildingType::WarriorChamber)
                        } else if barracks > 0 && military_count < AI_MAX_MILITARY_UNITS_EARLY {
                            AIDecision::BuildMilitary(UnitType::SoldierAnt)
                        } else {
                            AIDecision::GatherResources
                        }
                    },
                    AIType::Aggressive => {
                        if barracks == 0 && resources.wood >= 175.0 {
                            AIDecision::BuildBuilding(BuildingType::WarriorChamber)
                        } else if barracks > 0 && military_count < AI_MAX_MILITARY_UNITS_MID {
                            AIDecision::BuildMilitary(UnitType::SoldierAnt)
                        } else if military_count >= AI_MIN_MILITARY_FOR_ATTACK {
                            AIDecision::AttackPlayer(1) // Attack player 1
                        } else if villager_count < 3 {
                            AIDecision::BuildWorkerAnt
                        } else {
                            AIDecision::BuildBuilding(BuildingType::Nursery)
                        }
                    },
                    AIType::Balanced => {
                        if resources.current_population >= resources.max_population {
                            AIDecision::BuildBuilding(BuildingType::Nursery)
                        } else if villager_count < 3 {
                            AIDecision::BuildWorkerAnt
                        } else if barracks == 0 {
                            AIDecision::BuildBuilding(BuildingType::WarriorChamber)
                        } else if military_count < AI_MIN_MILITARY_FOR_DEFEND {
                            AIDecision::BuildMilitary(UnitType::SoldierAnt)
                        } else {
                            AIDecision::GatherResources
                        }
                    }
                };
                
                execute_ai_decision(
                    decision,
                    ai_player.player_id,
                    &mut ai_resources,
                    &game_costs,
                    &mut buildings,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    model_assets.as_deref(),
                );
            }
        }
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
    _model_assets: Option<&crate::model_loader::ModelAssets>,
) {
    match decision {
        AIDecision::BuildWorkerAnt => {
            // Find a queen to queue worker ant
            for (_building_entity, mut queue, building, unit) in buildings.iter_mut() {
                if unit.player_id == player_id && 
                   building.building_type == BuildingType::Queen &&
                   building.is_complete {
                    
                    if let Some(cost) = game_costs.unit_costs.get(&UnitType::WorkerAnt) {
                        if let Some(resources) = ai_resources.resources.get_mut(&player_id) {
                            if resources.can_afford(cost) && resources.has_population_space() {
                                // Spend resources and queue the unit
                                resources.spend_resources(cost);
                                queue.queue.push(UnitType::WorkerAnt);
                                info!("AI Player {} queuing worker ant", player_id);
                                break;
                            }
                        }
                    }
                }
            }
        },
        AIDecision::BuildMilitary(unit_type) => {
            // Find a warrior chamber to queue military unit
            for (_building_entity, mut queue, building, unit) in buildings.iter_mut() {
                if unit.player_id == player_id && 
                   building.building_type == BuildingType::WarriorChamber &&
                   building.is_complete {
                    
                    if let Some(cost) = game_costs.unit_costs.get(&unit_type) {
                        if let Some(resources) = ai_resources.resources.get_mut(&player_id) {
                            if resources.can_afford(cost) && resources.has_population_space() {
                                // Spend resources and queue the unit
                                resources.spend_resources(cost);
                                queue.queue.push(unit_type.clone());
                                info!("AI Player {} queuing {:?}", player_id, unit_type);
                                break;
                            }
                        }
                    }
                }
            }
        },
        AIDecision::BuildBuilding(building_type) => {
            // Spawn building (simplified - in real game would use construction system)
            if let Some(cost) = game_costs.building_costs.get(&building_type) {
                if let Some(resources) = ai_resources.resources.get_mut(&player_id) {
                    if resources.can_afford(cost) {
                        resources.spend_resources(cost);
                        
                        // Spawn building at random location near AI base
                        let mut rng = rand::thread_rng();
                        let position = Vec3::new(
                            rng.gen_range(-100.0..100.0),
                            10.0,
                            rng.gen_range(50.0..150.0), // AI builds in positive Z area
                        );
                        
                        match building_type {
                            BuildingType::Nursery => {
                                crate::rts_entities::RTSEntityFactory::spawn_house(
                                    commands, meshes, materials, position, player_id
                                );
                                resources.add_housing(5); // Nurseries provide 5 population
                            },
                            BuildingType::WarriorChamber => {
                                crate::rts_entities::RTSEntityFactory::spawn_barracks(
                                    commands, meshes, materials, position, player_id
                                );
                            },
                            _ => {}
                        }
                        
                        info!("AI Player {} built {:?}", player_id, building_type);
                    }
                }
            }
        },
        AIDecision::AttackPlayer(_target_player) => {
            info!("AI Player {} initiating attack!", player_id);
            // This would issue attack commands to military units
        },
        AIDecision::GatherResources => {
            // Issue gather commands to villagers (simplified)
            info!("AI Player {} focusing on resource gathering", player_id);
        },
        AIDecision::Expand => {
            info!("AI Player {} expanding", player_id);
        }
    }
}

// System to manage AI unit behavior
pub fn ai_unit_management_system(
    mut ai_units: Query<(&mut Movement, &RTSUnit), (With<RTSUnit>, Without<Combat>)>,
    resources: Query<&Transform, With<ResourceSource>>,
    _time: Res<Time>,
) {
    for (mut movement, unit) in ai_units.iter_mut() {
        if unit.player_id == 2 { // Only AI player 2
            // Simple villager AI - go gather resources if idle
            if movement.target_position.is_none() {
                // Find nearest resource
                if let Some(nearest_resource) = resources.iter().next() {
                    movement.target_position = Some(nearest_resource.translation);
                }
            }
        }
    }
}

// System to manage AI resource allocation
pub fn ai_resource_management_system(
    mut ai_resources: ResMut<AIResources>,
    time: Res<Time>,
) {
    use crate::constants::ai::*;
    
    // Simple passive resource income for AI players
    for (_, resources) in ai_resources.resources.iter_mut() {
        // AI gets small passive income to keep things moving
        resources.food += AI_FOOD_RATE * time.delta_secs();
        resources.wood += AI_WOOD_RATE * time.delta_secs();
        resources.stone += AI_STONE_RATE * time.delta_secs();
        resources.gold += AI_GOLD_RATE * time.delta_secs();
    }
}

// System for AI combat decisions
pub fn ai_combat_system(
    mut ai_units: Query<(&mut Movement, &mut Combat, &Transform, &RTSUnit), With<Combat>>,
    enemy_units: Query<(&Transform, &RTSUnit), (With<RTSUnit>, Without<Combat>)>,
) {
    for (mut movement, combat, unit_transform, unit) in ai_units.iter_mut() {
        if unit.player_id == 2 && combat.auto_attack { // AI player 2 combat units
            // Find nearest enemy unit
            let mut nearest_enemy = None;
            let mut nearest_distance = f32::INFINITY;
            
            for (enemy_transform, enemy_unit) in enemy_units.iter() {
                if enemy_unit.player_id != unit.player_id {
                    let distance = unit_transform.translation.distance(enemy_transform.translation);
                    if distance < nearest_distance && distance < 100.0 { // Within detection range
                        nearest_distance = distance;
                        nearest_enemy = Some(enemy_transform.translation);
                    }
                }
            }
            
            if let Some(enemy_pos) = nearest_enemy {
                if nearest_distance > combat.attack_range {
                    // Move towards enemy
                    movement.target_position = Some(enemy_pos);
                } else {
                    // In range - stop and attack
                    movement.target_position = None;
                }
            }
        }
    }
}

// Helper function to spawn AI players
pub fn spawn_ai_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_id: u8,
    ai_type: AIType,
    base_position: Vec3,
) {
    // Spawn AI player entity
    commands.spawn(AIPlayer {
        player_id,
        ai_type: ai_type.clone(),
        decision_timer: Timer::from_seconds(3.0, TimerMode::Repeating), // Make decisions every 3 seconds
        build_order_index: 0,
    });
    
    // Spawn AI town center
    crate::rts_entities::RTSEntityFactory::spawn_town_center(
        commands, meshes, materials, base_position, player_id
    );
    
    // Spawn a couple starting villagers
    for i in 0..3 {
        use crate::constants::ai::*;
        let villager_pos = base_position + Vec3::new(i as f32 * AI_SPAWN_RANGE, 0.0, AI_SPAWN_RANGE);
        crate::rts_entities::RTSEntityFactory::spawn_villager(
            commands, meshes, materials, villager_pos, player_id, rand::random()
        );
    }
    
    info!("Spawned AI Player {} ({:?}) at {:?}", player_id, ai_type, base_position);
}