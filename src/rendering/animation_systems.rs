use bevy::prelude::*;
use crate::core::components::*;
use crate::rendering::model_loader::UseGLBModel;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                add_missing_animation_controllers, // Add this first
                animation_state_manager,
                update_animations,
                find_animation_players,
                start_idle_animations, // Start animations for newly found players
                animation_debug_system,
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
    animation_graphs: Res<Assets<AnimationGraph>>,
    animation_clips: Res<Assets<AnimationClip>>,
) {
    for event in animation_events.read() {
        if let Ok(mut controller) = controllers.get_mut(event.entity) {
            // Update animation state
            controller.previous_state = controller.current_state.clone();
            controller.current_state = event.new_state.clone();

            // Try to play the animation if we have a player
            if let Some(player_entity) = controller.animation_player {
                if let Ok(mut player) = animation_players.get_mut(player_entity) {
                    if let Some(animation_name) = get_animation_name_for_state(&controller.clips, &event.new_state) {
                        // Try to find and play the animation
                        // In Bevy 0.15, we need to search the animation graph for the clip
                        let mut found_animation = false;

                        // Get the animation graph if available
                        if let Some(graph_handle) = player.animation_graph() {
                            if let Some(graph) = animation_graphs.get(graph_handle) {
                                // Search for animation by name
                                for (node_index, node) in graph.iter_nodes_with_indices() {
                                    if let Some(clip_handle) = node.clip {
                                        if let Some(clip) = animation_clips.get(clip_handle) {
                                            // Check if the clip name matches (simplified name matching)
                                            // Play the first animation found - proper name matching would require more info
                                            player.play(node_index).repeat();
                                            found_animation = true;
                                            debug!("Playing animation {:?} for state {:?} on entity {:?}",
                                                  node_index, event.new_state, event.entity);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if !found_animation {
                            // Fallback: just play the first available animation
                            if let Some(graph_handle) = player.animation_graph() {
                                if let Some(graph) = animation_graphs.get(graph_handle) {
                                    if let Some((first_node, _)) = graph.iter_nodes_with_indices().next() {
                                        player.play(first_node).repeat();
                                        debug!("Playing first available animation for state {:?} on entity {:?}",
                                              event.new_state, event.entity);
                                    }
                                }
                            }
                        }
                    } else {
                        // No specific animation for this state, play first available
                        if let Some(graph_handle) = player.animation_graph() {
                            if let Some(graph) = animation_graphs.get(graph_handle) {
                                if let Some((first_node, _)) = graph.iter_nodes_with_indices().next() {
                                    // Only play if not already playing to avoid restart loops
                                    if !player.is_playing_animation() {
                                        player.play(first_node).repeat();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// System to find animation players for controllers
// This waits for GLB scene instantiation to complete before searching
pub fn find_animation_players(
    mut controllers: Query<(Entity, &mut UnitAnimationController), Without<AnimationPlayer>>,
    animation_players: Query<Entity, With<AnimationPlayer>>,
    children: Query<&Children>,
    parents: Query<&Parent>,
    scene_roots: Query<&SceneRoot>,
) {
    for (controller_entity, mut controller) in controllers.iter_mut() {
        if controller.animation_player.is_none() {
            // Check if this entity has a SceneRoot (GLB model)
            if let Ok(_scene_root) = scene_roots.get(controller_entity) {
                // For GLB scenes, wait until the entity has children before searching
                // This indicates the scene has been instantiated
                if children.get(controller_entity).is_err() {
                    // No children yet, scene still loading
                    continue;
                }
                
                // Scene is ready (has children), now search for animation players
                if let Some(player) = search_recursive_for_player(
                    controller_entity, 
                    &children, 
                    &animation_players, 
                    0
                ) {
                    controller.animation_player = Some(player);
                    debug!("Found AnimationPlayer for GLB scene controller {:?} -> {:?}", 
                          controller_entity, player);
                }
            } else {
                // Non-GLB entity, use simpler search
                if let Some(player) = search_simple_for_player(
                    controller_entity,
                    &children,
                    &parents,
                    &animation_players,
                ) {
                    controller.animation_player = Some(player);
                    debug!("Found AnimationPlayer for non-GLB controller {:?} -> {:?}", 
                          controller_entity, player);
                }
            }
        }
    }
}

// Recursive search for animation players in GLB scene hierarchies
fn search_recursive_for_player(
    entity: Entity,
    children: &Query<&Children>,
    animation_players: &Query<Entity, With<AnimationPlayer>>,
    depth: usize,
) -> Option<Entity> {
    if depth > 8 { return None; } // Prevent infinite recursion, deeper limit for GLB scenes
    
    // Check if this entity is an animation player
    if animation_players.get(entity).is_ok() {
        return Some(entity);
    }
    
    // Search children
    if let Ok(children_list) = children.get(entity) {
        for &child in children_list.iter() {
            if let Some(player) = search_recursive_for_player(child, children, animation_players, depth + 1) {
                return Some(player);
            }
        }
    }
    
    None
}

// Simple search for animation players in non-GLB entities
fn search_simple_for_player(
    entity: Entity,
    children: &Query<&Children>,
    parents: &Query<&Parent>,
    animation_players: &Query<Entity, With<AnimationPlayer>>,
) -> Option<Entity> {
    // Check direct children
    if let Ok(children_list) = children.get(entity) {
        for &child in children_list.iter() {
            if animation_players.get(child).is_ok() {
                return Some(child);
            }
        }
    }
    
    // Check siblings
    if let Ok(parent) = parents.get(entity) {
        if let Ok(siblings) = children.get(parent.get()) {
            for &sibling in siblings.iter() {
                if animation_players.get(sibling).is_ok() {
                    return Some(sibling);
                }
            }
        }
    }
    
    None
}

// System to start idle animations for units that just got their animation player assigned
pub fn start_idle_animations(
    mut controllers: Query<(Entity, &mut UnitAnimationController), Changed<UnitAnimationController>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animation_graphs: Res<Assets<AnimationGraph>>,
) {
    for (entity, controller) in controllers.iter_mut() {
        // If we just got an animation player assigned, start the idle animation
        if let Some(player_entity) = controller.animation_player {
            if let Ok(mut player) = animation_players.get_mut(player_entity) {
                // Only start if not already playing
                if !player.is_playing_animation() {
                    if let Some(graph_handle) = player.animation_graph() {
                        if let Some(graph) = animation_graphs.get(graph_handle) {
                            // Play the first animation on repeat
                            if let Some((first_node, _)) = graph.iter_nodes_with_indices().next() {
                                player.play(first_node).repeat();
                                debug!("Started idle animation for newly assigned player on entity {:?}", entity);
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
    use crate::rendering::model_loader::get_unit_insect_model;
    
    let model_type = get_unit_insect_model(unit_type);
    
    match model_type {
        crate::rendering::model_loader::InsectModelType::Beetle => {
            // Black ox beetle has the most complete animation set - use exact names from animation summary
            UnitAnimationClips {
                idle: Some("AS_BlackOxBeetle_Idle_NC_01_SK_BlackOxBeetle01".to_string()), // Non-combat idle
                walking: Some("AS_BlackOxBeetle_Walk_Forward_01_SK_BlackOxBeetle01".to_string()), // Walk forward
                running: Some("AS_BlackOxBeetle_Run_Forward_01_SK_BlackOxBeetle01".to_string()), // Run forward
                attacking: Some("AS_BlackOxBeetle_Attack_Basic_SK_BlackOxBeetle01".to_string()), // Basic attack
                defending: Some("AS_BlackOxBeetle_Idle_Combat_01_SK_BlackOxBeetle01".to_string()), // Combat idle
                taking_damage: None, // No flinch animation found in summary
                death: Some("AS_BlackOxBeetle_Death_01_SK_BlackOxBeetle01".to_string()), // Death
                special: vec![
                    "AS_BlackOxBeetle_Attack_Spin_SK_BlackOxBeetle01".to_string(), // Spin attack
                    "AS_BlackOxBeetle_Attack_Thrash_SK_BlackOxBeetle01".to_string(), // Thrash attack
                    "AS_BlackOxBeetle_Attack_RockFling_SK_BlackOxBeetle01".to_string(), // Rock fling
                    "as_blackoxbeetle_jump_SK_BlackOxBeetle01".to_string(), // Jump
                ],
            }
        },
        crate::rendering::model_loader::InsectModelType::WolfSpider => {
            // Wolf spider has good movement animations - use exact names from animation summary
            UnitAnimationClips {
                idle: Some("Wolf Spider Armature|Spider walking".to_string()), // Use walking as idle
                walking: Some("Wolf Spider Armature|Spider walking".to_string()), // Spider walking
                running: Some("Wolf Spider Armature|Spider running".to_string()), // Spider running
                attacking: None, // No attack animation available
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![
                    "Wolf Spider Armature|Spider walking fast".to_string(), // Walking fast
                    "Wolf Spider Armature|Spider walking backword".to_string(), // Walking backward
                    "Wolf Spider Armature|Spider walk and turn left".to_string(), // Turn left
                    "Wolf Spider Armature|Spider walk and turn right".to_string(), // Turn right
                ],
            }
        },
        crate::rendering::model_loader::InsectModelType::Scorpion => {
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
        crate::rendering::model_loader::InsectModelType::Bee => {
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
        crate::rendering::model_loader::InsectModelType::Spider => {
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
        crate::rendering::model_loader::InsectModelType::QueenFacedBug => {
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
        crate::rendering::model_loader::InsectModelType::ApisMellifera => {
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
        crate::rendering::model_loader::InsectModelType::Meganeura => {
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
        crate::rendering::model_loader::InsectModelType::AnimatedSpider => {
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
        crate::rendering::model_loader::InsectModelType::RhinoBeetle => {
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
        crate::rendering::model_loader::InsectModelType::Hornet => {
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
        crate::rendering::model_loader::InsectModelType::Fourmi => {
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
        crate::rendering::model_loader::InsectModelType::CairnsBirdwing => {
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
        crate::rendering::model_loader::InsectModelType::LadybugLowpoly => {
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
        crate::rendering::model_loader::InsectModelType::RolyPoly => {
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
        crate::rendering::model_loader::InsectModelType::DragonFly => {
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
        crate::rendering::model_loader::InsectModelType::Mushrooms => {
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
        // Environment objects - all static, no animations
        crate::rendering::model_loader::InsectModelType::Mushrooms |
        crate::rendering::model_loader::InsectModelType::Grass |
        crate::rendering::model_loader::InsectModelType::Grass2 |
        crate::rendering::model_loader::InsectModelType::Hive |
        crate::rendering::model_loader::InsectModelType::StickShelter |
        crate::rendering::model_loader::InsectModelType::WoodStick |
        crate::rendering::model_loader::InsectModelType::SimpleGrassChunks |
        crate::rendering::model_loader::InsectModelType::CherryBlossomTree |
        crate::rendering::model_loader::InsectModelType::PineCone |
        crate::rendering::model_loader::InsectModelType::PlantsAssetSet |
        crate::rendering::model_loader::InsectModelType::BeechFern |
        crate::rendering::model_loader::InsectModelType::TreesPack |
        crate::rendering::model_loader::InsectModelType::RiverRock |
        crate::rendering::model_loader::InsectModelType::SmallRocks |
        // Building objects - all static, no animations
        crate::rendering::model_loader::InsectModelType::Anthill => {
            UnitAnimationClips {
                idle: None, // Static environment objects
                walking: None,
                running: None,
                attacking: None,
                defending: None,
                taking_damage: None,
                death: None,
                special: vec![],
            }
        },
        
        // Defensive catch-all for any new model types not explicitly handled
        _ => {
            warn!("Animation clips not specifically defined for model type: {:?}. Using default static configuration.", model_type);
            UnitAnimationClips {
                idle: None, // Default: no animations (static)
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

// System to retroactively add animation controllers to units that don't have them
pub fn add_missing_animation_controllers(
    mut commands: Commands,
    units_without_controllers: Query<(Entity, &RTSUnit), (Without<UnitAnimationController>, With<RTSUnit>)>,
) {
    for (entity, unit) in units_without_controllers.iter() {
        // Only add animation controller to units with a specific type
        let Some(unit_type) = &unit.unit_type else {
            continue;
        };
        
        // Create animation clips specific to this unit type
        let clips = create_animation_clips_for_unit(unit_type);
        
        let animation_controller = UnitAnimationController {
            current_state: AnimationState::Idle,
            previous_state: AnimationState::Idle,
            animation_player: None, // Will be populated by find_animation_players system
            clips,
        };
        
        // Add the animation controller to the entity
        commands.entity(entity).insert(animation_controller);
        debug!("Retroactively added animation controller to unit {:?} (entity {:?})", unit_type, entity);
    }
}

// Debug system to check animation status
pub fn animation_debug_system(
    animation_controllers: Query<(Entity, &UnitAnimationController, Option<&RTSUnit>)>,
    animation_players: Query<&AnimationPlayer>,
    all_rts_units: Query<Entity, With<RTSUnit>>, // Check all RTS units
    all_glb_models: Query<Entity, With<UseGLBModel>>, // Check all GLB models
    children: Query<&Children>,
    names: Query<&Name>,
    scene_roots: Query<Entity, With<SceneRoot>>,
    mut logged_entities: Local<std::collections::HashSet<Entity>>,
    mut run_count: Local<u32>,
) {
    *run_count += 1;
    
    // Log system is running every 300 frames (about once per 5 seconds)
    if *run_count % 300 == 1 {
        let controller_count = animation_controllers.iter().count();
        let player_count = animation_players.iter().count();
        let unit_count = all_rts_units.iter().count();
        let glb_count = all_glb_models.iter().count();
        let scene_count = scene_roots.iter().count();
        info!("Animation debug: {} controllers, {} players, {} total RTSUnits, {} GLB models, {} scene roots (frame {})", 
              controller_count, player_count, unit_count, glb_count, scene_count, *run_count);
        
        // Every 30 seconds, do a deep hierarchy analysis
        if *run_count % 1800 == 1 {
            info!("=== DEEP HIERARCHY ANALYSIS ===");
            
            // Find all units with SceneRoot and examine their children
            for (controller_entity, controller, unit_opt) in animation_controllers.iter().take(3) {
                let unit_type = unit_opt.and_then(|u| u.unit_type.as_ref());
                info!("Analyzing entity {:?} (unit: {:?})", controller_entity, unit_type);
                
                // Check if this entity has children
                if let Ok(children_list) = children.get(controller_entity) {
                    info!("  Entity has {} children", children_list.len());
                    
                    // Examine each child
                    for &child in children_list.iter() {
                        let child_name = names.get(child).map(|n| n.as_str()).unwrap_or("unnamed");
                        let has_animation_player = animation_players.get(child).is_ok();
                        let has_children = children.get(child).map(|c| c.len()).unwrap_or(0);
                        
                        info!("    Child {:?} '{}' - AnimPlayer: {}, Children: {}", 
                              child, child_name, has_animation_player, has_children);
                        
                        // Check grandchildren too
                        if let Ok(grandchildren) = children.get(child) {
                            for &grandchild in grandchildren.iter() {
                                let grandchild_name = names.get(grandchild).map(|n| n.as_str()).unwrap_or("unnamed");
                                let grandchild_has_player = animation_players.get(grandchild).is_ok();
                                
                                info!("      Grandchild {:?} '{}' - AnimPlayer: {}", 
                                      grandchild, grandchild_name, grandchild_has_player);
                            }
                        }
                    }
                } else {
                    info!("  Entity has no children");
                }
            }
            
            info!("=== END HIERARCHY ANALYSIS ===");
        }
    }
    
    for (entity, controller, unit_opt) in animation_controllers.iter() {
        // Only log each entity once to avoid spam
        if !logged_entities.contains(&entity) {
            if let Some(player_entity) = controller.animation_player {
                if animation_players.get(player_entity).is_ok() {
                    let unit_type = unit_opt.and_then(|u| u.unit_type.as_ref());
                    info!("✓ Animation controller found for entity {:?} (unit: {:?}), player entity: {:?}", 
                          entity, unit_type, player_entity);
                } else {
                    info!("✗ Animation controller for entity {:?} references invalid player entity {:?}", 
                          entity, player_entity);
                }
            } else {
                let unit_type = unit_opt.and_then(|u| u.unit_type.as_ref());
                info!("⚠ Animation controller for entity {:?} (unit: {:?}) has no animation player assigned", 
                      entity, unit_type);
            }
            logged_entities.insert(entity);
        }
    }
}