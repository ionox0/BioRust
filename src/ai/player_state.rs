use bevy::prelude::*;
use crate::core::components::*;

#[derive(Component, Debug)]
pub struct AIPlayer {
    pub player_id: u8,
    pub ai_type: AIType,
    pub decision_timer: Timer,
    #[allow(dead_code)]
    pub build_order_index: usize,
}

#[derive(Debug, Clone)]
pub enum AIType {
    Aggressive,
    #[allow(dead_code)]
    Economic,
    #[allow(dead_code)]
    Balanced,
}

#[derive(Debug, Clone)]
pub enum AIDecision {
    BuildWorkerAnt,
    BuildMilitary(UnitType),
    BuildBuilding(BuildingType),
    AttackPlayer(u8),
    GatherResources,
    #[allow(dead_code)]
    Expand,
}

#[derive(Default)]
pub struct PlayerCounts {
    pub villager_count: i32,
    pub military_count: i32,
    pub houses: i32,
    pub barracks: i32,
}

impl PlayerCounts {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn count_units(&mut self, units: &Query<&RTSUnit, With<RTSUnit>>, player_id: u8) {
        self.reset();
        
        for unit in units.iter() {
            if unit.player_id == player_id {
                if let Some(unit_type) = &unit.unit_type {
                    match unit_type {
                        UnitType::WorkerAnt => self.villager_count += 1,
                        UnitType::SoldierAnt | UnitType::HunterWasp | 
                        UnitType::BeetleKnight | UnitType::SpearMantis | 
                        UnitType::ScoutAnt | UnitType::BatteringBeetle | 
                        UnitType::AcidSpitter | UnitType::DragonFly |
                        UnitType::DefenderBug | UnitType::EliteSpider |
                        UnitType::SpecialOps => {
                            self.military_count += 1;
                        },
                    }
                }
            }
        }
    }
    
    pub fn count_buildings(&mut self, buildings: &Query<(Entity, &mut ProductionQueue, &Building, &RTSUnit), With<Building>>, player_id: u8) {
        for (_, _, building, unit) in buildings.iter() {
            if unit.player_id == player_id {
                match building.building_type {
                    BuildingType::Nursery => self.houses += 1,
                    BuildingType::WarriorChamber => self.barracks += 1,
                    _ => {}
                }
            }
        }
    }
    
    fn reset(&mut self) {
        self.villager_count = 0;
        self.military_count = 0;
        // Don't reset building counts as they're counted separately
    }
}

// Helper function to spawn AI players with reduced parameters
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
        decision_timer: Timer::from_seconds(3.0, TimerMode::Repeating),
        build_order_index: 0,
    });
    
    spawn_starting_units(commands, meshes, materials, player_id, base_position);
    
    info!("Spawned AI Player {} ({:?}) at {:?}", player_id, ai_type, base_position);
}

fn spawn_starting_units(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_id: u8,
    base_position: Vec3,
) {
    // Spawn AI town center
    crate::entities::rts_entities::RTSEntityFactory::spawn_queen_chamber(
        commands, meshes, materials, base_position, player_id, None
    );
    
    // Spawn starting villagers
    for i in 0..3 {
        use crate::constants::ai::*;
        let villager_pos = base_position + Vec3::new(i as f32 * AI_SPAWN_RANGE, 0.0, AI_SPAWN_RANGE);
        crate::entities::rts_entities::RTSEntityFactory::spawn_worker_ant(
            commands, meshes, materials, villager_pos, player_id, rand::random()
        );
    }
}