//! Specialized State Management Systems
//! 
//! This module provides state management using specialized components
//! instead of a single monolithic EntityState component.

use crate::core::components::*;
use bevy::prelude::*;

pub struct EntityStatePlugin;

impl Plugin for EntityStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_gathering_state_to_gatherers,
                add_combat_state_to_fighters,
                update_gathering_states,
                update_building_states, 
                update_movement_states,
                update_combat_states,
                sync_animations_with_specialized_states,
                cleanup_dead_units,
            )
                .chain(),
        )
        .add_event::<GatheringStateChangeEvent>()
        .add_event::<BuildingStateChangeEvent>()
        .add_event::<MovementStateChangeEvent>()
        .add_event::<CombatStateChangeEvent>();
    }
}

// State change events for each specialized component
#[derive(Event, Debug)]
pub struct GatheringStateChangeEvent {
    pub entity: Entity,
    pub old_state: GatheringStateType,
    pub new_state: GatheringStateType,
}

#[derive(Event, Debug)]
pub struct BuildingStateChangeEvent {
    pub entity: Entity,
    pub old_state: BuildingStateType,
    pub new_state: BuildingStateType,
}

#[derive(Event, Debug)]
pub struct MovementStateChangeEvent {
    pub entity: Entity,
    pub old_state: MovementStateType,
    pub new_state: MovementStateType,
}

#[derive(Event, Debug)]
pub struct CombatStateChangeEvent {
    pub entity: Entity,
    pub old_state: CombatStateType,
    pub new_state: CombatStateType,
}

/// Update gathering state component based on ResourceGatherer data
pub fn update_gathering_states(
    mut gathering_query: Query<(Entity, &mut GatheringState, &ResourceGatherer, Option<&Movement>)>,
    _commands: Commands,
    mut events: EventWriter<GatheringStateChangeEvent>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    for (entity, mut gathering_state, gatherer, movement) in gathering_query.iter_mut() {
        let old_state = gathering_state.state.clone();
        let new_state = determine_gathering_state(gatherer, movement);
        
        if new_state != old_state {
            gathering_state.state = new_state.clone();
            gathering_state.last_state_change = current_time;
            
            events.send(GatheringStateChangeEvent {
                entity,
                old_state,
                new_state,
            });
        }
    }
    
    // Add GatheringState to units with ResourceGatherer but no state
    // This is handled by the add_gathering_state_to_gatherers system
}

/// Add gathering state to resource gatherers that don't have it
pub fn add_gathering_state_to_gatherers(
    mut commands: Commands,
    gatherers_without_state: Query<Entity, (With<ResourceGatherer>, Without<GatheringState>)>,
) {
    for entity in gatherers_without_state.iter() {
        commands.entity(entity).insert(GatheringState::default());
    }
}

/// Add combat state to units with Combat component that don't have it (only auto-attacking units)
pub fn add_combat_state_to_fighters(
    mut commands: Commands,
    fighters_without_state: Query<(Entity, &Combat), Without<CombatState>>,
) {
    for (entity, combat) in fighters_without_state.iter() {
        // Only add combat state to units that auto-attack (not economic units like workers)
        if combat.auto_attack {
            commands.entity(entity).insert(CombatState::default());
        }
    }
}

/// Determine gathering state based on ResourceGatherer component
fn determine_gathering_state(
    gatherer: &ResourceGatherer,
    movement: Option<&Movement>,
) -> GatheringStateType {
    // If carrying resources
    if gatherer.carried_amount > 0.0 {
        // Should return if at capacity or no target resource
        if gatherer.carried_amount >= gatherer.capacity || gatherer.target_resource.is_none() {
            // Check if actually moving to return
            if let Some(movement) = movement {
                if movement.target_position.is_some() {
                    return GatheringStateType::ReturningToBase;
                }
            }
            // Has resources but not moving = delivering
            return GatheringStateType::DeliveringResources;
        }
    }
    
    // If has target resource
    if gatherer.target_resource.is_some() {
        // Check if moving to resource or at resource
        if let Some(movement) = movement {
            if movement.target_position.is_some() {
                return GatheringStateType::MovingToResource;
            }
        }
        // At resource location, gathering
        return GatheringStateType::Gathering;
    }
    
    // Default to moving to resource (looking for work)
    GatheringStateType::MovingToResource
}

/// Update building states (placeholder for now)
pub fn update_building_states(
    mut building_query: Query<(Entity, &mut BuildingState)>,
    _events: EventWriter<BuildingStateChangeEvent>,
    time: Res<Time>,
) {
    // Placeholder - implement based on actual construction system
    let _current_time = time.elapsed_secs();
    
    for (_entity, _building_state) in building_query.iter_mut() {
        // TODO: Implement building state logic when construction system is available
    }
}

/// Update movement states (placeholder for now) 
pub fn update_movement_states(
    mut movement_query: Query<(Entity, &mut MovementState, &Movement)>,
    _events: EventWriter<MovementStateChangeEvent>,
    time: Res<Time>,
) {
    let _current_time = time.elapsed_secs();
    
    for (_entity, _movement_state, _movement) in movement_query.iter_mut() {
        // TODO: Implement movement state logic for following, patrolling, etc.
    }
}

/// Update combat states based on Combat component data
pub fn update_combat_states(
    mut combat_query: Query<(Entity, &mut CombatState, &Combat, &Transform, Option<&Movement>)>,
    targets_query: Query<&Transform, (With<RTSHealth>, Without<DeathEvent>)>,
    mut _events: EventWriter<CombatStateChangeEvent>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    for (_entity, mut combat_state, combat, transform, movement) in combat_query.iter_mut() {
        let old_state = combat_state.state.clone();
        let new_state = determine_combat_state(combat, transform, movement, &targets_query);
        
        if new_state != old_state {
            combat_state.state = new_state.clone();
            combat_state.last_state_change = current_time;
            
            // Update target position and entity references
            if let Some(target_entity) = combat.target {
                if let Ok(target_transform) = targets_query.get(target_entity) {
                    combat_state.target_entity = Some(target_entity);
                    combat_state.target_position = Some(target_transform.translation);
                } else {
                    combat_state.target_entity = None;
                    combat_state.target_position = None;
                }
            } else {
                combat_state.target_entity = None;
                combat_state.target_position = None;
            }
        }
    }
}

/// Determine combat state based on Combat component and related data
fn determine_combat_state(
    combat: &Combat,
    transform: &Transform,
    movement: Option<&Movement>,
    targets_query: &Query<&Transform, (With<RTSHealth>, Without<DeathEvent>)>,
) -> CombatStateType {
    // If no target, unit is idle
    let Some(target_entity) = combat.target else {
        return CombatStateType::Idle;
    };
    
    // If target doesn't exist, unit should be idle
    let Ok(target_transform) = targets_query.get(target_entity) else {
        return CombatStateType::Idle;
    };
    
    let distance = transform.translation.distance(target_transform.translation);
    
    // If actively attacking (within range and attacking), in combat
    if distance <= combat.attack_range && combat.is_attacking {
        return CombatStateType::InCombat;
    }
    
    // If within range but not attacking (cooling down), still in combat
    if distance <= combat.attack_range {
        return CombatStateType::InCombat;
    }
    
    // If moving towards target (has movement target), moving to attack
    if let Some(movement) = movement {
        if movement.target_position.is_some() {
            return CombatStateType::MovingToAttack;
        }
    }
    
    // If has target but not moving, consider it moving to combat
    CombatStateType::MovingToCombat
}

/// Sync animations with specialized states
pub fn sync_animations_with_specialized_states(
    animation_controllers: Query<(
        Entity,
        &crate::rendering::animation_systems::UnitAnimationController,
    )>,
    gathering_states: Query<&GatheringState>,
    building_states: Query<&BuildingState>, 
    movement_states: Query<&MovementState>,
    combat_states: Query<&CombatState>,
    health_query: Query<&RTSHealth>,
    movement_query: Query<&Movement>,
    mut animation_events: EventWriter<
        crate::rendering::animation_systems::AnimationStateChangeEvent,
    >,
) {
    
    for (entity, controller) in animation_controllers.iter() {
        let desired_animation = determine_animation_from_states(
            entity,
            &gathering_states,
            &building_states,
            &movement_states,
            &combat_states,
            &health_query,
            &movement_query,
        );
        
        if controller.current_state != desired_animation {
            animation_events.send(
                crate::rendering::animation_systems::AnimationStateChangeEvent {
                    entity,
                    new_state: desired_animation,
                    force: false,
                },
            );
        }
    }
}

/// Determine animation based on specialized state components (priority order)
fn determine_animation_from_states(
    entity: Entity,
    gathering_states: &Query<&GatheringState>,
    _building_states: &Query<&BuildingState>,
    _movement_states: &Query<&MovementState>,
    combat_states: &Query<&CombatState>,
    health_query: &Query<&RTSHealth>,
    movement_query: &Query<&Movement>,
) -> crate::rendering::animation_systems::AnimationState {
    
    // 1. Check for death (highest priority)
    if let Ok(health) = health_query.get(entity) {
        if health.current <= 0.0 {
            return crate::rendering::animation_systems::AnimationState::Death;
        }
    }
    
    // 2. Check for combat
    if let Ok(combat_state) = combat_states.get(entity) {
        if matches!(combat_state.state, CombatStateType::InCombat) {
            return crate::rendering::animation_systems::AnimationState::Attacking;
        }
    }
    
    // 3. Check for gathering
    if let Ok(gathering_state) = gathering_states.get(entity) {
        match gathering_state.state {
            GatheringStateType::Gathering => return crate::rendering::animation_systems::AnimationState::Special,
            GatheringStateType::MovingToResource | GatheringStateType::ReturningToBase => {
                return crate::rendering::animation_systems::AnimationState::Walking;
            }
            GatheringStateType::DeliveringResources => return crate::rendering::animation_systems::AnimationState::Idle,
        }
    }
    
    // 4. Check for movement
    if let Ok(movement) = movement_query.get(entity) {
        if movement.target_position.is_some() {
            return crate::rendering::animation_systems::AnimationState::Walking;
        }
    }
    
    // 5. Default to idle
    crate::rendering::animation_systems::AnimationState::Idle
}

/// Clean up dead units
pub fn cleanup_dead_units(
    mut commands: Commands,
    health_query: Query<(Entity, &RTSHealth), Without<Dying>>,
) {
    for (entity, health) in health_query.iter() {
        if health.current <= 0.0 {
            if let Some(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.insert(Dying);
            }
        }
    }
}

// Legacy compatibility functions (deprecated)

/// Get a simplified state representation for debugging
#[allow(dead_code)]
pub fn get_entity_activity(entity: Entity, world: &World) -> &'static str {
    if world.get::<GatheringState>(entity).is_some() {
        "Gathering"
    } else if world.get::<BuildingState>(entity).is_some() {
        "Building"
    } else if world.get::<CombatState>(entity).is_some() {
        "Combat"
    } else if world.get::<Movement>(entity).and_then(|m| m.target_position).is_some() {
        "Moving"
    } else {
        "Idle"
    }
}