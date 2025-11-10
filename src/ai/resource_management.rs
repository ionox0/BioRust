use bevy::prelude::*;
use crate::core::resources::AIResources;

pub fn ai_resource_management_system(
    _ai_resources: ResMut<AIResources>,
    _time: Res<Time>,
) {
    // DISABLED: AI now must gather resources like the player, no passive income
    // AI workers will collect resources and return them to base buildings

    // Previous passive income code removed - AI must work for resources!
}