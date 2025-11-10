use bevy::prelude::*;
use bevy_animation::graph::AnimationNodeIndex;
use bevy_animation::prelude::{AnimationGraph, AnimationGraphHandle};
use crate::core::components::*;
use crate::rendering::model_loader::UseGLBModel;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                add_missing_animation_controllers, // Add this first
                setup_glb_animations, // NEW: Set up animations for GLB models
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
    pub animation_node_index: Option<AnimationNodeIndex>,
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
// Runs every frame to ensure smooth animation transitions
pub fn animation_state_manager(
    mut animation_events: EventWriter<AnimationStateChangeEvent>,
    // All units with animation controllers
    units: Query<(Entity, &Movement, &Combat, Option<&RTSHealth>, &UnitAnimationController), With<RTSUnit>>,
) {
    for (entity, movement, combat, health, controller) in units.iter() {
        // Determine the appropriate animation state based on unit behavior
        let new_state = determine_animation_state(movement, combat, health, &controller);

        // Send animation change event if state changed
        if controller.current_state != new_state {
            let force = matches!(new_state, AnimationState::Death);
            animation_events.send(AnimationStateChangeEvent {
                entity,
                new_state,
                force,
            });
        }
    }
}

fn determine_animation_state(
    movement: &Movement,
    combat: &Combat,
    health: Option<&RTSHealth>,
    _controller: &UnitAnimationController,
) -> AnimationState {
    // Priority order: Death > Attacking > Moving > Idle

    // Check if dead
    if let Some(health) = health {
        if health.current <= 0.0 {
            return AnimationState::Death;
        }
    }

    // Check if attacking
    if combat.is_attacking {
        return AnimationState::Attacking;
    }

    // Check movement state
    let velocity = movement.current_velocity.length();
    if velocity > 0.1 {
        if velocity > movement.max_speed * 0.7 {
            AnimationState::Running
        } else {
            AnimationState::Walking
        }
    } else {
        AnimationState::Idle
    }
}

// System to handle animation updates
pub fn update_animations(
    mut animation_events: EventReader<AnimationStateChangeEvent>,
    mut controllers: Query<&mut UnitAnimationController>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for event in animation_events.read() {
        if let Ok(mut controller) = controllers.get_mut(event.entity) {
            // Update animation state
            controller.previous_state = controller.current_state.clone();
            controller.current_state = event.new_state.clone();

            // Try to play the animation if we have a player
            if let Some(player_entity) = controller.animation_player {
                if let Ok(mut player) = animation_players.get_mut(player_entity) {
                    // Use the stored node index from the animation graph
                    // For now, all GLB models use the same animation (first clip)
                    // TODO: Map animation states to specific animation indices when models have multiple animations
                    if let Some(node_index) = controller.animation_node_index {
                        player.play(node_index).repeat();

                        debug!("🎬 Animation state changed: {:?} → {:?} for entity {:?}",
                              controller.previous_state, event.new_state, event.entity);
                    } else {
                        debug!("No animation node index stored for entity {:?}", event.entity);
                    }
                } else {
                    warn!("AnimationPlayer entity {:?} not found for controller on entity {:?}",
                          player_entity, event.entity);
                }
            } else {
                debug!("No AnimationPlayer assigned to controller on entity {:?}", event.entity);
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
) {
    for (entity, controller) in controllers.iter_mut() {
        // If we just got an animation player assigned, start the idle animation
        if let Some(player_entity) = controller.animation_player {
            if let Ok(mut player) = animation_players.get_mut(player_entity) {
                // Use the stored node index if available, otherwise fall back to index 0
                let node_index = controller.animation_node_index.unwrap_or(AnimationNodeIndex::new(0));
                player.play(node_index).repeat();
                debug!("Started idle animation for newly assigned player on entity {:?}", entity);
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
        crate::rendering::model_loader::InsectModelType::Grass |
        crate::rendering::model_loader::InsectModelType::Grass2 |
        crate::rendering::model_loader::InsectModelType::Hive |
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
            animation_node_index: None, // Will be populated by setup_glb_animations system
            clips,
        };
        
        // Add the animation controller to the entity
        commands.entity(entity).insert(animation_controller);
        debug!("Retroactively added animation controller to unit {:?} (entity {:?})", unit_type, entity);
    }
}

// System to set up animations for GLB models
// In Bevy 0.15, GLB animations are loaded automatically, but AnimationPlayer might be on child entities
pub fn setup_glb_animations(
    mut glb_models: Query<(Entity, &SceneRoot, &mut UnitAnimationController), Without<AnimationPlayerSearched>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut commands: Commands,
    children: Query<&Children>,
    asset_server: Res<AssetServer>,
) {
    for (entity, scene_root, mut controller) in glb_models.iter_mut() {
        // Check if scene has children (indicating it's loaded)
        if children.get(entity).is_err() {
            continue;
        }

        // Mark this entity as searched so we don't search again
        commands.entity(entity).insert(AnimationPlayerSearched);

        // Search for AnimationPlayer in the entity hierarchy
        if let Some(player_entity) = search_for_animation_player_entity(entity, &children, &animation_players) {
            // Store the AnimationPlayer entity in the controller
            controller.animation_player = Some(player_entity);

            // Create AnimationGraph from the first animation clip in the GLB
            // GLB animations are indexed as #Animation0, #Animation1, etc.
            if let Some(scene_path) = asset_server.get_path(scene_root.0.id()) {
                let animation_path = format!("{}#Animation0", scene_path.path().display());
                let animation_clip: Handle<bevy::animation::AnimationClip> =
                    asset_server.load(&animation_path);

                // Create graph from clip
                let (graph, node_index) = AnimationGraph::from_clip(animation_clip);
                let graph_handle = animation_graphs.add(graph);

                // Store the node index in the controller for later use
                controller.animation_node_index = Some(node_index);

                // Insert the graph handle on the AnimationPlayer entity
                commands.entity(player_entity).insert(
                    AnimationGraphHandle(graph_handle)
                );

                // Start playing animation immediately
                if let Ok(mut player) = animation_players.get_mut(player_entity) {
                    player.play(node_index).repeat();
                    info!("✓ Created AnimationGraph, assigned to player {:?} on entity {:?}, and started animation", player_entity, entity);
                }
            } else {
                warn!("Could not get asset path for scene {:?}", scene_root.0.id());
            }
        } else {
            // Some GLB models might not have animations, that's okay
            debug!("No AnimationPlayer found for GLB model entity {:?} (model may not have animations)", entity);
        }
    }
}

// Mark component to track that we've searched for an animation player
#[derive(Component)]
struct AnimationPlayerSearched;

// Helper function to recursively search for AnimationPlayer entity
fn search_for_animation_player_entity(
    entity: Entity,
    children: &Query<&Children>,
    animation_players: &Query<&mut AnimationPlayer>,
) -> Option<Entity> {
    search_for_animation_player_recursive(entity, children, animation_players, 0)
}

fn search_for_animation_player_recursive(
    entity: Entity,
    children: &Query<&Children>,
    animation_players: &Query<&mut AnimationPlayer>,
    depth: usize,
) -> Option<Entity> {
    if depth > 10 { return None; } // Prevent infinite recursion

    // Check if this entity has AnimationPlayer
    if animation_players.get(entity).is_ok() {
        return Some(entity);
    }

    // Search children
    if let Ok(children_list) = children.get(entity) {
        for &child in children_list.iter() {
            if let Some(player) = search_for_animation_player_recursive(child, children, animation_players, depth + 1) {
                return Some(player);
            }
        }
    }

    None
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