use crate::core::components::*;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct AIPlayer {
    pub player_id: u8,
    pub ai_type: AIType,
    pub decision_timer: Timer,
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
                        UnitType::WorkerAnt | UnitType::TermiteWorker => self.villager_count += 1,
                        UnitType::SoldierAnt
                        | UnitType::BeetleKnight
                        | UnitType::SpearMantis
                        | UnitType::ScoutAnt
                        | UnitType::BatteringBeetle
                        | UnitType::AcidSpitter
                        | UnitType::DragonFly
                        | UnitType::DefenderBug
                        | UnitType::EliteSpider
                        | UnitType::HoneyBee
                        | UnitType::Scorpion
                        | UnitType::SpiderHunter
                        | UnitType::WolfSpider
                        | UnitType::Ladybug
                        | UnitType::LadybugScout
                        | UnitType::Housefly
                        | UnitType::TermiteWarrior
                        | UnitType::LegBeetle
                        | UnitType::Stinkbug => {
                            self.military_count += 1;
                        }
                    }
                }
            }
        }
    }

    pub fn count_buildings(
        &mut self,
        buildings: &Query<(Entity, &mut ProductionQueue, &Building, &RTSUnit), With<Building>>,
        player_id: u8,
    ) {
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
