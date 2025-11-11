#![allow(dead_code)] // Allow unused state functionality for future features
use bevy::prelude::*;
use crate::core::components::*;

pub struct EntityStatePlugin;

impl Plugin for EntityStatePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                add_entity_state_to_units,
                update_entity_states,
                sync_animation_with_entity_state,
            ).chain())
            .add_event::<EntityStateChangeEvent>();
    }
}

/// Event fired when an entity changes state
#[derive(Event, Debug)]
pub struct EntityStateChangeEvent {
    pub entity: Entity,
    pub old_state: UnitState,
    pub new_state: UnitState,
}

/// System that updates entity states based on their current activities
pub fn update_entity_states(
    mut commands: Commands,
    mut entity_query: Query<(Entity, &mut EntityState, Option<&Movement>, Option<&Combat>, Option<&RTSHealth>, Option<&ResourceGatherer>), Without<Dying>>,
    mut state_events: EventWriter<EntityStateChangeEvent>,
    time: Res<Time>,
) {
    for (entity, mut entity_state, movement, combat, health, gatherer) in entity_query.iter_mut() {
        entity_state.state_timer += time.delta_secs();
        
        let old_state = entity_state.current.clone();
        let new_state = determine_entity_state(movement, combat, health, gatherer);
        
        if new_state != old_state {
            entity_state.previous = old_state.clone();
            entity_state.current = new_state.clone();
            entity_state.state_timer = 0.0;
            
            // Fire state change event
            state_events.send(EntityStateChangeEvent {
                entity,
                old_state,
                new_state: new_state.clone(),
            });
            
            // Handle special state transitions
            match new_state {
                UnitState::Dead => {
                    // Ensure the entity is marked as dying, but check if entity still exists first
                    if let Some(mut entity_commands) = commands.get_entity(entity) {
                        entity_commands.insert(Dying);
                    }
                },
                _ => {},
            }
        }
    }
}

/// Determine what state an entity should be in based on its components
fn determine_entity_state(
    movement: Option<&Movement>, 
    combat: Option<&Combat>, 
    health: Option<&RTSHealth>,
    gatherer: Option<&ResourceGatherer>
) -> UnitState {
    // Check for death first (highest priority)
    if let Some(health) = health {
        if health.current <= 0.0 {
            return UnitState::Dead;
        }
    }
    
    // Check for combat (high priority)
    if let Some(combat) = combat {
        if combat.is_attacking || combat.target.is_some() {
            return UnitState::Fighting;
        }
    }

    // Check for resource gathering/delivery
    if let Some(gatherer) = gatherer {
        // Check if returning to base with resources
        // Unit is returning if: carrying resources AND (at capacity OR no target resource) AND actually moving
        if gatherer.carried_amount > 0.0 {
            let should_be_returning = gatherer.carried_amount >= gatherer.capacity ||
                                     gatherer.target_resource.is_none();

            if should_be_returning {
                // Only mark as "returning" if actually moving toward dropoff
                if let Some(movement) = movement {
                    if movement.target_position.is_some() {
                        return UnitState::ReturningWithResources;
                    }
                }
                // Has resources and should return, but no movement = waiting idle for building
                // Fall through to Idle state below
                return UnitState::Idle;
            }
        }

        // Check if actively gathering at a resource
        // This includes traveling TO the resource and gathering AT the resource
        if gatherer.target_resource.is_some() {
            return UnitState::Gathering;
        }
    }

    // Check for movement
    if let Some(movement) = movement {
        if movement.target_position.is_some() {
            // Determine if this is fast or slow movement
            return UnitState::Moving;
        }
    }
    
    // Default to idle
    UnitState::Idle
}

/// System that synchronizes animation states with entity states
pub fn sync_animation_with_entity_state(
    animation_controllers: Query<(Entity, &crate::rendering::animation_systems::UnitAnimationController)>,
    entity_states: Query<&EntityState>,
    mut animation_events: EventWriter<crate::rendering::animation_systems::AnimationStateChangeEvent>,
) {
    use crate::rendering::animation_systems::AnimationState;
    
    for (entity, controller) in animation_controllers.iter() {
        if let Ok(entity_state) = entity_states.get(entity) {
            let desired_animation = match entity_state.current {
                UnitState::Idle => AnimationState::Idle,
                UnitState::Moving => {
                    // Could differentiate between walking and running based on speed
                    AnimationState::Walking
                },
                UnitState::Fighting => AnimationState::Attacking,
                UnitState::Gathering => AnimationState::Special, // Use special animation for gathering
                UnitState::ReturningWithResources => AnimationState::Walking, // Walking back with resources
                UnitState::Building => AnimationState::Special,  // Use special animation for building
                UnitState::Dead => AnimationState::Death,
                UnitState::Following => AnimationState::Walking,
                UnitState::Patrolling => AnimationState::Walking,
                UnitState::Guarding => AnimationState::Idle,
            };
            
            if controller.current_state != desired_animation {
                animation_events.send(crate::rendering::animation_systems::AnimationStateChangeEvent {
                    entity,
                    new_state: desired_animation,
                    force: false, // Use smooth transitions for entity state changes
                });
            }
        }
    }
}

/// Helper function to add EntityState component to entities that don't have it
pub fn add_entity_state_to_units(
    mut commands: Commands,
    units_without_state: Query<Entity, (With<RTSUnit>, Without<EntityState>)>,
) {
    for entity in units_without_state.iter() {
        commands.entity(entity).insert(EntityState::default());
    }
}

/// Helper function to get the current state of an entity
pub fn get_entity_state(entity: Entity, states: &Query<&EntityState>) -> Option<UnitState> {
    states.get(entity).ok().map(|state| state.current.clone())
}

/// Helper function to check if an entity is in a specific state
pub fn is_entity_in_state(entity: Entity, target_state: UnitState, states: &Query<&EntityState>) -> bool {
    if let Ok(state) = states.get(entity) {
        state.current == target_state
    } else {
        false
    }
}