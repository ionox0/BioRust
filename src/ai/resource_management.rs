use bevy::prelude::*;
use crate::resources::AIResources;

pub fn ai_resource_management_system(
    mut ai_resources: ResMut<AIResources>,
    time: Res<Time>,
) {
    use crate::constants::ai::*;
    
    // Simple passive resource income for AI players
    for (_, resources) in ai_resources.resources.iter_mut() {
        resources.nectar += AI_FOOD_RATE * time.delta_secs();
        resources.chitin += AI_WOOD_RATE * time.delta_secs();
        resources.minerals += AI_STONE_RATE * time.delta_secs();
        resources.pheromones += AI_GOLD_RATE * time.delta_secs();
    }
}