use bevy::prelude::*;
use crate::components::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                animation_state_manager,
                update_animations,
                find_animation_players,
            ).chain())
            .add_event::<AnimationStateChangeEvent>();
    }
}

#[derive(Component, Debug, Clone)]
pub struct UnitAnimationController {
    pub current_state: AnimationState,
    pub previous_state: AnimationState,
    pub animation_player: Option<Entity>,
    pub clips: UnitAnimationClips,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationState {
    Idle,
    Walking,
    Running,
    Attacking,
    Defending,
    TakingDamage,
    Death,
    Special, // For unit-specific animations like flying, eating, etc.
}

#[derive(Debug, Clone)]
pub struct UnitAnimationClips {
    pub idle: Option<String>,
    pub walking: Option<String>,
    pub running: Option<String>,
    pub attacking: Option<String>,
    pub defending: Option<String>,
    pub taking_damage: Option<String>,
    pub death: Option<String>,
    pub special: Vec<String>,
}

impl Default for UnitAnimationClips {
    fn default() -> Self {
        Self {
            idle: None,
            walking: None,
            running: None,
            attacking: None,
            defending: None,
            taking_damage: None,
            death: None,
            special: Vec::new(),
        }
    }
}

#[derive(Event, Debug)]
pub struct AnimationStateChangeEvent {
    pub entity: Entity,
    pub new_state: AnimationState,
    #[allow(dead_code)]
    pub force: bool, // Force immediate transition without blending
}

// System to manage animation state changes based on unit behavior
pub fn animation_state_manager(
    mut animation_events: EventWriter<AnimationStateChangeEvent>,
    // Units with movement
    moving_units: Query<
        (Entity, &Movement, &UnitAnimationController),
        (With<RTSUnit>, Changed<Movement>)
    >,
    // Units in combat
    combat_units: Query<
        (Entity, &Combat, &UnitAnimationController),
        (With<RTSUnit>, Changed<Combat>)
    >,
    // Units taking damage
    damaged_units: Query<
        (Entity, &RTSHealth, &UnitAnimationController),
        (With<RTSUnit>, Changed<RTSHealth>)
    >,
) {
    // Handle movement state changes
    for (entity, movement, controller) in moving_units.iter() {
        let new_state = if movement.current_velocity.length() > 0.1 {
            if movement.current_velocity.length() > movement.max_speed * 0.7 {
                AnimationState::Running
            } else {
                AnimationState::Walking
            }
        } else {
            AnimationState::Idle
        };

        if controller.current_state != new_state {
            animation_events.send(AnimationStateChangeEvent {
                entity,
                new_state,
                force: false,
            });
        }
    }

    // Handle combat state changes
    for (entity, combat, controller) in combat_units.iter() {
        if combat.is_attacking && controller.current_state != AnimationState::Attacking {
            animation_events.send(AnimationStateChangeEvent {
                entity,
                new_state: AnimationState::Attacking,
                force: false,
            });
        }
    }

    // Handle damage state changes
    for (entity, health, controller) in damaged_units.iter() {
        if health.current <= 0.0 && controller.current_state != AnimationState::Death {
            animation_events.send(AnimationStateChangeEvent {
                entity,
                new_state: AnimationState::Death,
                force: true, // Death animations should be immediate
            });
        }
    }
}

// System to handle animation updates
pub fn update_animations(
    mut animation_events: EventReader<AnimationStateChangeEvent>,
    mut controllers: Query<&mut UnitAnimationController>,
    mut animation_players: Query<&mut AnimationPlayer>,
    _scenes: Res<Assets<Scene>>,
) {
    for event in animation_events.read() {
        if let Ok(mut controller) = controllers.get_mut(event.entity) {
            // Update animation state
            controller.previous_state = controller.current_state.clone();
            controller.current_state = event.new_state.clone();
            
            // Try to play the animation if we have a player
            if let Some(player_entity) = controller.animation_player {
                if let Ok(_player) = animation_players.get_mut(player_entity) {
                    if let Some(_animation_name) = get_animation_name_for_state(&controller.clips, &event.new_state) {
                        // For now, just log that we would play the animation
                        // In a full implementation, you'd need to set up the animation graph properly
                        info!("Would play animation '{}' for state {:?} on entity {:?}", 
                              _animation_name, event.new_state, event.entity);
                    }
                }
            }
        }
    }
}

// System to find animation players for controllers
pub fn find_animation_players(
    mut controllers: Query<(Entity, &mut UnitAnimationController), Without<AnimationPlayer>>,
    animation_players: Query<Entity, With<AnimationPlayer>>,
    children: Query<&Children>,
    parents: Query<&Parent>,
) {
    for (controller_entity, mut controller) in controllers.iter_mut() {
        if controller.animation_player.is_none() {
            // Try to find animation player in children
            if let Ok(children) = children.get(controller_entity) {
                for &child in children.iter() {
                    if animation_players.get(child).is_ok() {
                        controller.animation_player = Some(child);
                        info!("Found AnimationPlayer child for controller {:?}", controller_entity);
                        break;
                    }
                }
            }
            
            // Try to find animation player in siblings (common with GLB imports)
            if controller.animation_player.is_none() {
                if let Ok(parent) = parents.get(controller_entity) {
                    if let Ok(siblings) = children.get(parent.get()) {
                        for &sibling in siblings.iter() {
                            if animation_players.get(sibling).is_ok() {
                                controller.animation_player = Some(sibling);
                                info!("Found AnimationPlayer sibling for controller {:?}", controller_entity);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

// Helper function to get animation name for a state
fn get_animation_name_for_state<'a>(clips: &'a UnitAnimationClips, state: &'a AnimationState) -> Option<&'a String> {
    match state {
        AnimationState::Idle => clips.idle.as_ref(),
        AnimationState::Walking => clips.walking.as_ref(),
        AnimationState::Running => clips.running.as_ref(),
        AnimationState::Attacking => clips.attacking.as_ref(),
        AnimationState::Defending => clips.defending.as_ref(),
        AnimationState::TakingDamage => clips.taking_damage.as_ref(),
        AnimationState::Death => clips.death.as_ref(),
        AnimationState::Special => clips.special.get(0),
    }
}

// Helper function to create animation clips based on unit type
pub fn create_animation_clips_for_unit(
    unit_type: &UnitType,
) -> UnitAnimationClips {
    use crate::model_loader::get_unit_insect_model;
    
    let model_type = get_unit_insect_model(unit_type);
    
    match model_type {
        crate::model_loader::InsectModelType::Beetle => {
            // Black ox beetle has the most complete animation set
            UnitAnimationClips {
                idle: Some("AS_BlackOxBeetle_Idle_SK_BlackOxBeetle01".to_string()), // Idle
                walking: Some("AS_BlackOxBeetle_Walk_Forward_SK_BlackOxBeetle01".to_string()), // Walk forward
                running: Some("AS_BlackOxBeetle_Run_Forward_SK_BlackOxBeetle01".to_string()), // Run forward
                attacking: Some("AS_BlackOxBeetle_Attack_Basic_SK_BlackOxBeetle01".to_string()), // Basic attack
                defending: Some("AS_BlackOxBeetle_CombatIdle_SK_BlackOxBeetle01".to_string()), // Combat idle
                taking_damage: Some("AS_BlackOxBeetle_Flinch_Front_SK_BlackOxBeetle01".to_string()), // Flinch from front
                death: Some("AS_BlackOxBeetle_Death_SK_BlackOxBeetle01".to_string()), // Death
                special: vec![
                    "AS_BlackOxBeetle_Attack_Spin_SK_BlackOxBeetle01".to_string(), // Spin attack
                    "AS_BlackOxBeetle_Attack_Thrash_SK_BlackOxBeetle01".to_string(), // Thrash attack
                    "AS_BlackOxBeetle_Attack_RockFling_SK_BlackOxBeetle01".to_string(), // Rock fling
                ],
            }
        },
        crate::model_loader::InsectModelType::WolfSpider => {
            // Wolf spider has good movement animations
            UnitAnimationClips {
                idle: Some("Walking".to_string()), // Walking (as idle)
                walking: Some("Walking".to_string()), // Spider walking
                running: Some("Running".to_string()), // Spider running
                attacking: None, // No attack animation available
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![
                    "Walking_fast".to_string(), // Walking fast
                    "Walking_backward".to_string(), // Walking backward
                    "Turn_left_while_walking".to_string(), // Turn left
                    "Turn_right_while_walking".to_string(), // Turn right
                ],
            }
        },
        crate::model_loader::InsectModelType::Scorpion => {
            // Scorpion has essential combat animations
            UnitAnimationClips {
                idle: Some("Idle".to_string()), // Idle
                walking: Some("Walk".to_string()), // Walk
                running: Some("Walk".to_string()), // Use walk for running too
                attacking: Some("Area Attack".to_string()), // Area Attack
                defending: Some("Defend".to_string()), // Defend
                taking_damage: None,
                death: Some("Death".to_string()), // Death
                special: vec![],
            }
        },
        crate::model_loader::InsectModelType::Bee => {
            // Bee has unique flying animations
            UnitAnimationClips {
                idle: Some("_bee_idle".to_string()), // Bee idle
                walking: Some("_bee_hover".to_string()), // Bee hover
                running: Some("_bee_hover".to_string()), // Bee hover (faster)
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![
                    "_bee_take_off_and_land".to_string(), // Take off and land
                ],
            }
        },
        crate::model_loader::InsectModelType::Spider => {
            // Simple spider with very limited animations (needs improvement)
            UnitAnimationClips {
                idle: None,
                walking: None,
                running: None,
                attacking: Some("SpiderSmall_Attack".to_string()), // Single attack animation
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![],
            }
        },
        crate::model_loader::InsectModelType::Ladybug => {
            // Ladybug with basic movement (WARNING: Contains placeholder animations)
            UnitAnimationClips {
                idle: Some("LADYBUGAction".to_string()), // Only valid ladybug animation
                walking: Some("LADYBUGAction".to_string()), // Reuse for walking
                running: Some("LADYBUGAction".to_string()), // Reuse for running
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![
                    // Note: Other animations in this model are placeholders (CubeAction, etc.)
                ],
            }
        },
        crate::model_loader::InsectModelType::QueenFacedBug => {
            // Queen faced bug - placeholder animations for now
            UnitAnimationClips {
                idle: None, // No specific animations defined yet
                walking: None,
                running: None,
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![],
            }
        },
        crate::model_loader::InsectModelType::ApisMellifera => {
            // Apis mellifera - high-quality honey bee with professional animations
            UnitAnimationClips {
                idle: Some("_bee_idle".to_string()), // Reuse bee idle
                walking: Some("_bee_hover".to_string()), // Reuse bee hover
                running: Some("_bee_hover".to_string()), // Reuse bee hover (faster)
                attacking: None, // To be defined when available
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![
                    "_bee_take_off_and_land".to_string(), // Take off and land
                ],
            }
        },
        crate::model_loader::InsectModelType::Meganeura => {
            // Meganeura - ancient dragonfly with flight animations
            UnitAnimationClips {
                idle: Some("_bee_hover".to_string()), // Hover like a bee
                walking: Some("_bee_hover".to_string()),
                running: Some("_bee_hover".to_string()),
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec!["_bee_take_off_and_land".to_string()],
            }
        },
        crate::model_loader::InsectModelType::AnimatedSpider => {
            // Animated spider - enhanced spider animations
            UnitAnimationClips {
                idle: Some("SpiderSmall_Attack".to_string()),
                walking: Some("SpiderSmall_Attack".to_string()), // Reuse attack as movement
                running: Some("SpiderSmall_Attack".to_string()),
                attacking: Some("SpiderSmall_Attack".to_string()),
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![],
            }
        },
        crate::model_loader::InsectModelType::RhinoBeetle => {
            // Rhino beetle - heavy armored unit
            UnitAnimationClips {
                idle: Some("AS_BlackOxBeetle_Idle_SK_BlackOxBeetle01".to_string()),
                walking: Some("AS_BlackOxBeetle_Walk_Forward_SK_BlackOxBeetle01".to_string()),
                running: Some("AS_BlackOxBeetle_Run_Forward_SK_BlackOxBeetle01".to_string()),
                attacking: Some("AS_BlackOxBeetle_Attack_Basic_SK_BlackOxBeetle01".to_string()),
                defending: Some("AS_BlackOxBeetle_CombatIdle_SK_BlackOxBeetle01".to_string()),
                taking_damage: Some("AS_BlackOxBeetle_Flinch_Front_SK_BlackOxBeetle01".to_string()),
                death: Some("AS_BlackOxBeetle_Death_SK_BlackOxBeetle01".to_string()),
                special: vec![
                    "AS_BlackOxBeetle_Attack_Spin_SK_BlackOxBeetle01".to_string(),
                    "AS_BlackOxBeetle_Attack_Thrash_SK_BlackOxBeetle01".to_string(),
                ],
            }
        },
        crate::model_loader::InsectModelType::Hornet => {
            // Hornet - aggressive flying unit
            UnitAnimationClips {
                idle: Some("_bee_idle".to_string()),
                walking: Some("_bee_hover".to_string()),
                running: Some("_bee_hover".to_string()),
                attacking: None, // To be defined
                defending: None,
                taking_damage: None,
                death: None,
                special: vec!["_bee_take_off_and_land".to_string()],
            }
        },
        crate::model_loader::InsectModelType::Fourmi => {
            // Fourmi - French ant, perfect worker
            UnitAnimationClips {
                idle: None, // Simple static model
                walking: None,
                running: None,
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![],
            }
        },
        crate::model_loader::InsectModelType::CairnsBirdwing => {
            // Cairns birdwing butterfly - scout unit
            UnitAnimationClips {
                idle: Some("_bee_hover".to_string()), // Hover like flying insect
                walking: Some("_bee_hover".to_string()),
                running: Some("_bee_hover".to_string()),
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec!["_bee_take_off_and_land".to_string()],
            }
        },
        crate::model_loader::InsectModelType::LadybugLowpoly => {
            // Low-poly ladybug alternative
            UnitAnimationClips {
                idle: Some("LADYBUGAction".to_string()),
                walking: Some("LADYBUGAction".to_string()),
                running: Some("LADYBUGAction".to_string()),
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![],
            }
        },
        crate::model_loader::InsectModelType::RolyPoly => {
            // Roly poly pill bug - defensive unit
            UnitAnimationClips {
                idle: None, // Static defensive posture
                walking: None,
                running: None,
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![], // Could add rolling animation
            }
        },
        crate::model_loader::InsectModelType::MysteryModel => {
            // Mystery model - unknown capabilities
            UnitAnimationClips {
                idle: None,
                walking: None,
                running: None,
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![],
            }
        },
        crate::model_loader::InsectModelType::Mushrooms => {
            // Mushrooms - environment objects (static)
            UnitAnimationClips {
                idle: None, // Static environment object
                walking: None,
                running: None,
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![],
            }
        },
    }
}