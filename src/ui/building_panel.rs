use bevy::prelude::*;
use crate::components::*;
use crate::ui::{
    resource_display::{PlayerResources, setup_resource_display},
    button_styles::{create_building_button_with_icon, create_unit_button_with_icon, ButtonStyle, BuildingButton, UnitButton},
    placement::{can_afford_building, BuildingPlacement, PlacementStatusText},
    icons::UIIcons,
};

#[derive(Component)]
pub struct BuildingPanel;

#[derive(Component)]
pub struct ProductionQueueDisplay;

pub fn setup_building_ui(
    mut commands: Commands,
    ui_icons: Res<UIIcons>,
) {
    use crate::constants::resources::*;
    
    // Main UI root
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        GlobalZIndex(100),
    )).with_children(|parent| {
        setup_resource_display(parent, &ui_icons);
        setup_building_panel(parent, &ui_icons);
    });

    // Initialize player resources (will sync with main PlayerResources)
    commands.insert_resource(PlayerResources {
        nectar: STARTING_NECTAR,
        chitin: STARTING_CHITIN,
        minerals: STARTING_MINERALS,
        pheromones: STARTING_PHEROMONES,
        population_used: 0,
        population_limit: STARTING_POPULATION_LIMIT,
    });
}

fn setup_building_panel(parent: &mut ChildBuilder, ui_icons: &UIIcons) {
    
    // Bottom panel - Building interface
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(150.0),
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect::all(Val::Px(10.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
        BorderColor(Color::srgb(0.4, 0.4, 0.4)),
        BuildingPanel,
    )).with_children(|parent| {
        setup_buildings_section(parent, ui_icons);
        setup_units_section(parent, ui_icons);
    });
}

fn setup_buildings_section(parent: &mut ChildBuilder, ui_icons: &UIIcons) {
    
    parent.spawn((
        Node {
            width: Val::Percent(70.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Buildings"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        
        // Instructions
        parent.spawn((
            Text::new("Click building, then click terrain to place"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
        ));
        
        // Placement status indicator
        parent.spawn((
            Text::new(""),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.2, 0.8, 0.2)),
            Node {
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
            PlacementStatusText,
        ));
        
        // Building buttons grid
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(80.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(10.0),
                row_gap: Val::Px(10.0),
                ..default()
            },
        )).with_children(|parent| {
            create_building_buttons(parent, ui_icons);
        });
    });
}

fn create_building_buttons(parent: &mut ChildBuilder, ui_icons: &UIIcons) {
    use crate::constants::resources::*;
    
    let building_data = [
        (BuildingType::Nursery, ui_icons.nursery_icon.clone(), "Nursery", vec![(ResourceType::Chitin, NURSERY_CHITIN_COST)]),
        (BuildingType::WarriorChamber, ui_icons.warrior_chamber_icon.clone(), "Warriors", vec![(ResourceType::Chitin, WARRIOR_CHAMBER_CHITIN_COST), (ResourceType::Minerals, WARRIOR_CHAMBER_MINERALS_COST)]),
        (BuildingType::HunterChamber, ui_icons.hunter_chamber_icon.clone(), "Hunters", vec![(ResourceType::Chitin, HUNTER_CHAMBER_CHITIN_COST)]),
        (BuildingType::WoodProcessor, ui_icons.wood_processor_icon.clone(), "Processor", vec![(ResourceType::Chitin, WOOD_PROCESSOR_CHITIN_COST)]),
        (BuildingType::MineralProcessor, ui_icons.mineral_processor_icon.clone(), "Mineral Processor", vec![(ResourceType::Chitin, MINERAL_PROCESSOR_WOOD_COST)]),
        (BuildingType::FungalGarden, ui_icons.fungal_garden_icon.clone(), "Garden", vec![(ResourceType::Chitin, FUNGAL_GARDEN_CHITIN_COST)]),
    ];
    
    for (building_type, icon, name, cost) in building_data {
        create_building_button_with_icon(parent, building_type, icon, name, cost);
    }
}

fn setup_units_section(parent: &mut ChildBuilder, ui_icons: &UIIcons) {
    
    parent.spawn((
        Node {
            width: Val::Percent(30.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Units"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(80.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            ProductionQueueDisplay,
        )).with_children(|parent| {
            create_unit_buttons(parent, ui_icons);
        });
    });
}

fn create_unit_buttons(parent: &mut ChildBuilder, ui_icons: &UIIcons) {
    use crate::constants::resources::*;
    
    let unit_data = [
        (UnitType::WorkerAnt, ui_icons.worker_icon.clone(), "Worker", vec![(ResourceType::Nectar, WORKER_ANT_NECTAR_COST)], BuildingType::Queen),
        (UnitType::SoldierAnt, ui_icons.soldier_icon.clone(), "Soldier", vec![(ResourceType::Nectar, SOLDIER_ANT_NECTAR_COST), (ResourceType::Pheromones, SOLDIER_ANT_PHEROMONES_COST)], BuildingType::WarriorChamber),
        (UnitType::HunterWasp, ui_icons.hunter_icon.clone(), "Hunter", vec![(ResourceType::Chitin, HUNTER_WASP_CHITIN_COST), (ResourceType::Pheromones, HUNTER_WASP_PHEROMONES_COST)], BuildingType::HunterChamber),
    ];
    
    for (unit_type, icon, name, cost, building_type) in unit_data {
        create_unit_button_with_icon(parent, unit_type, icon, name, cost, building_type);
    }
}

pub fn handle_building_panel_interactions(
    mut interaction_query: Query<(&Interaction, &BuildingButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut unit_interaction_query: Query<(&Interaction, &UnitButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>, Without<BuildingButton>)>,
    mut placement: ResMut<BuildingPlacement>,
    player_resources: Res<PlayerResources>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    handle_building_button_interactions(&mut interaction_query, &mut placement, &player_resources);
    handle_unit_button_interactions(&mut unit_interaction_query, &player_resources, &mut commands, &mut meshes, &mut materials);
}

fn handle_building_button_interactions(
    interaction_query: &mut Query<(&Interaction, &BuildingButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    placement: &mut BuildingPlacement,
    player_resources: &PlayerResources,
) {
    for (interaction, building_button, mut background_color) in interaction_query.iter_mut() {
        let can_afford = can_afford_building(&building_button.cost, player_resources);
        let is_selected = placement.active_building.as_ref() == Some(&building_button.building_type);
        
        let style = if can_afford { ButtonStyle::BUILDING_AFFORDABLE } else { ButtonStyle::BUILDING_UNAFFORDABLE };
        
        match *interaction {
            Interaction::Pressed => {
                if can_afford {
                    placement.active_building = Some(building_button.building_type.clone());
                    info!("Selected building: {:?} - Click on terrain to place, ESC to cancel", building_button.building_type);
                    *background_color = style.pressed.into();
                } else {
                    info!("Cannot afford building: {:?}", building_button.building_type);
                    *background_color = style.pressed.into();
                }
            }
            Interaction::Hovered => {
                *background_color = style.hover.into();
            }
            Interaction::None => {
                if is_selected {
                    *background_color = Color::srgba(0.2, 0.6, 0.2, 0.8).into(); // Keep selected buildings highlighted
                } else {
                    *background_color = style.normal.into();
                }
            }
        }
    }
}

fn handle_unit_button_interactions(
    unit_interaction_query: &mut Query<(&Interaction, &UnitButton, &mut BackgroundColor), (Changed<Interaction>, With<Button>, Without<BuildingButton>)>,
    player_resources: &PlayerResources,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    for (interaction, unit_button, mut background_color) in unit_interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if can_afford_unit(&unit_button.cost, player_resources) {
                    spawn_unit_from_building(commands, meshes, materials, unit_button.unit_type.clone());
                    info!("Producing unit: {:?}", unit_button.unit_type);
                } else {
                    info!("Cannot afford unit: {:?}", unit_button.unit_type);
                }
            }
            Interaction::Hovered => {
                *background_color = ButtonStyle::UNIT_BUTTON.hover.into();
            }
            Interaction::None => {
                *background_color = ButtonStyle::UNIT_BUTTON.normal.into();
            }
        }
    }
}

fn can_afford_unit(cost: &[(ResourceType, f32)], resources: &PlayerResources) -> bool {
    can_afford_building(cost, resources) && resources.population_used < resources.population_limit
}

fn spawn_unit_from_building(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    unit_type: UnitType,
) {
    use crate::constants::combat::*;
    let x = rand::random::<f32>() * UNIT_SPAWN_RANGE - UNIT_SPAWN_OFFSET;
    let z = rand::random::<f32>() * UNIT_SPAWN_RANGE - UNIT_SPAWN_OFFSET;
    let spawn_position = Vec3::new(x, 1.0, z);

    match unit_type {
        UnitType::WorkerAnt => {
            crate::rts_entities::RTSEntityFactory::spawn_villager(
                commands, meshes, materials, spawn_position, 1, rand::random()
            );
        }
        UnitType::SoldierAnt => {
            crate::combat_systems::create_combat_unit(
                commands, meshes, materials, spawn_position, 1, UnitType::SoldierAnt
            );
        }
        UnitType::HunterWasp => {
            crate::combat_systems::create_combat_unit(
                commands, meshes, materials, spawn_position, 1, UnitType::HunterWasp
            );
        }
        _ => {}
    }
}

pub fn update_production_queue_display() {
    // TODO: Update production queue display based on selected buildings
}

pub fn building_hotkeys_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut placement: ResMut<BuildingPlacement>,
    player_resources: Res<PlayerResources>,
) {
    use crate::constants::{hotkeys::*, resources::*};
    
    let hotkey_buildings = [
        (BUILD_WARRIOR_CHAMBER, BuildingType::WarriorChamber, vec![(ResourceType::Chitin, WARRIOR_CHAMBER_CHITIN_COST), (ResourceType::Minerals, WARRIOR_CHAMBER_MINERALS_COST)]),
        (BUILD_NURSERY, BuildingType::Nursery, vec![(ResourceType::Chitin, NURSERY_CHITIN_COST)]),
        (BUILD_FUNGAL_GARDEN, BuildingType::FungalGarden, vec![(ResourceType::Chitin, FUNGAL_GARDEN_CHITIN_COST)]),
    ];
    
    for (key, building_type, cost) in hotkey_buildings {
        if keyboard.just_pressed(key) && can_afford_building(&cost, &player_resources) {
            placement.active_building = Some(building_type);
            break;
        }
    }
}