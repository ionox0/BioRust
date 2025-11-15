//! Team-aware building panel that only shows units available to the current team

use crate::core::components::*;
use crate::ui::building_panel::*;
use crate::ui::icons::UIIcons;
use bevy::prelude::*;

/// System to update building panel based on player team
pub fn update_team_building_panel(
    mut commands: Commands,
    player_teams: Query<&PlayerTeam, (With<PlayerTeam>, Added<PlayerTeam>)>,
    building_panels: Query<Entity, With<BuildingPanel>>,
    ui_icons: Res<UIIcons>,
    game_costs: Res<crate::core::resources::GameCosts>,
) {
    // Find the human player's team (player_id == 1)
    if let Some(player_team) = player_teams
        .iter()
        .find(|team| team.player_id == 1)
    {
        info!("Updating building panel for team: {:?}", player_team.team_type);

        // Remove existing building panel
        for entity in building_panels.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Create new team-specific building panel
        setup_team_building_panel(&mut commands, &ui_icons, &game_costs, &player_team.team_type);
    }
}

/// Setup team-specific building panel
fn setup_team_building_panel(
    commands: &mut Commands,
    ui_icons: &UIIcons,
    game_costs: &crate::core::resources::GameCosts,
    team_type: &TeamType,
) {
    let team_roster = team_type.get_unit_roster();
    
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(180.0),
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
            BuildingPanel,
        ))
        .with_children(|parent| {
            // Buildings section
            setup_team_buildings_section(parent, ui_icons, game_costs);
            
            // Team-specific units section  
            setup_team_units_section(parent, ui_icons, game_costs, &team_roster, team_type);
        });
}

fn setup_team_buildings_section(
    parent: &mut ChildBuilder,
    ui_icons: &UIIcons,
    game_costs: &crate::core::resources::GameCosts,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Buildings header
            parent.spawn((
                Text::new("Buildings"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            // Buildings grid
            parent
                .spawn((
                    Node {
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
                        column_gap: Val::Px(2.0),
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    let get_building_cost = |building_type: BuildingType| {
                        game_costs
                            .building_costs
                            .get(&building_type)
                            .cloned()
                            .unwrap_or_default()
                    };

                    // Essential buildings for all teams
                    let buildings = vec![
                        (
                            BuildingType::Nursery,
                            ui_icons.nursery_icon.clone(),
                            "House",
                            get_building_cost(BuildingType::Nursery),
                        ),
                        (
                            BuildingType::WarriorChamber,
                            ui_icons.warrior_chamber_icon.clone(),
                            "Barracks",
                            get_building_cost(BuildingType::WarriorChamber),
                        ),
                    ];

                    for (building_type, icon_handle, name, cost) in buildings {
                        crate::ui::button_styles::create_building_button_with_icon(
                            parent, building_type, icon_handle, name, cost,
                        );
                    }
                });
        });
}

fn setup_team_units_section(
    parent: &mut ChildBuilder,
    ui_icons: &UIIcons,
    game_costs: &crate::core::resources::GameCosts,
    team_roster: &[UnitType],
    team_type: &TeamType,
) {
    parent
        .spawn((
            Node {
                width: Val::Auto,
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Team name header
            parent.spawn((
                Text::new(format!("{} Units", team_type.display_name())),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            // Units grid - show up to 12 units in 4x3 grid
            parent
                .spawn((
                    Node {
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::flex(6, 1.0), // 6 columns for better layout
                        column_gap: Val::Px(2.0),
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    let get_unit_cost = |unit_type: UnitType| {
                        game_costs
                            .unit_costs
                            .get(&unit_type)
                            .cloned()
                            .unwrap_or_default()
                    };

                    // Create buttons for team-specific units
                    for (_i, unit_type) in team_roster.iter().enumerate().take(12) {
                        // Limit to 12 units to fit in UI
                        let (icon_handle, display_name) = get_unit_icon_and_name(unit_type, ui_icons);
                        let building_type = get_unit_building_requirement(unit_type);
                        let cost = get_unit_cost(unit_type.clone());
                        
                        crate::ui::button_styles::create_unit_button_with_icon(
                            parent,
                            unit_type.clone(),
                            icon_handle,
                            display_name,
                            cost,
                            building_type,
                        );
                    }
                });
        });
}

/// Get appropriate icon and display name for unit type
fn get_unit_icon_and_name(unit_type: &UnitType, ui_icons: &UIIcons) -> (Handle<Image>, &'static str) {
    match unit_type {
        // Worker units
        UnitType::WorkerAnt | UnitType::DungBeetle | UnitType::Silverfish | 
        UnitType::Aphids | UnitType::Mites | UnitType::Honeybees |
        UnitType::BlackAnt | UnitType::WorkerFourmi => 
            (ui_icons.worker_icon.clone(), "Worker"),
            
        // Combat units
        UnitType::SoldierAnt | UnitType::StagBeetle | UnitType::WidowSpider |
        UnitType::CommonMantis | UnitType::RedAnt | UnitType::FireAnt |
        UnitType::SoldierFourmi | UnitType::Hornets | UnitType::Wasps => 
            (ui_icons.soldier_icon.clone(), "Soldier"),
            
        // Hunter/Scout units  
        UnitType::ScoutAnt | UnitType::DaddyLongLegs | UnitType::Fleas |
        UnitType::Grasshoppers | UnitType::Firefly | UnitType::DragonFlies |
        UnitType::Damselfly => 
            (ui_icons.hunter_icon.clone(), "Hunter"),
            
        // Default to soldier icon for other units
        _ => (ui_icons.soldier_icon.clone(), "Unit"),
    }
}

/// Get short display name for unit
fn get_unit_display_name(unit_type: &UnitType) -> &str {
    match unit_type {
        UnitType::WorkerAnt => "Worker",
        UnitType::SoldierAnt => "Soldier", 
        UnitType::ScoutAnt => "Scout",
        UnitType::BeetleKnight => "Knight",
        UnitType::SpearMantis => "Mantis",
        UnitType::DragonFly => "Dragon",
        UnitType::HoneyBee => "Bee",
        UnitType::Scorpion => "Scorpion",
        UnitType::Housefly => "Fly",
        UnitType::DefenderBug => "Defender",
        UnitType::BatteringBeetle => "Battering",
        UnitType::AcidSpitter => "Acid",
        UnitType::EliteSpider => "Elite",
        UnitType::SpiderHunter => "Hunter",
        UnitType::WolfSpider => "Wolf",
        UnitType::TermiteWorker => "T.Worker",
        UnitType::TermiteWarrior => "T.Warrior", 
        UnitType::LegBeetle => "L.Beetle",
        UnitType::Stinkbug => "Stinkbug",
        
        // New team units - shortened names for UI
        UnitType::StagBeetle => "Stag",
        UnitType::DungBeetle => "Dung",
        UnitType::RhinoBeetle => "Rhino",
        UnitType::StinkBeetle => "S.Beetle",
        UnitType::JewelBug => "Jewel",
        UnitType::CommonMantis => "C.Mantis",
        UnitType::OrchidMantis => "Orchid",
        UnitType::RedAnt => "Red Ant",
        UnitType::BlackAnt => "Black",
        UnitType::FireAnt => "Fire",
        UnitType::SoldierFourmi => "S.Fourmi",
        UnitType::WorkerFourmi => "W.Fourmi",
        UnitType::Pillbug => "Pillbug",
        UnitType::Silverfish => "Silver",
        UnitType::Woodlouse => "Woodlouse",
        UnitType::SandFleas => "S.Fleas",
        UnitType::Aphids => "Aphids",
        UnitType::Mites => "Mites",
        UnitType::Ticks => "Ticks",
        UnitType::Fleas => "Fleas",
        UnitType::Lice => "Lice",
        UnitType::Moths => "Moths",
        UnitType::Caterpillars => "Larvae",
        UnitType::PeacockMoth => "Peacock",
        UnitType::WidowSpider => "Widow",
        UnitType::WolfSpiderVariant => "W.Spider",
        UnitType::Tarantula => "Tarantula",
        UnitType::DaddyLongLegs => "D.Legs",
        UnitType::HouseflyVariant => "H.Fly",
        UnitType::Horsefly => "Horsefly",
        UnitType::Firefly => "Firefly",
        UnitType::DragonFlies => "Dragons",
        UnitType::Damselfly => "Damsel",
        UnitType::Hornets => "Hornets",
        UnitType::Wasps => "Wasps",
        UnitType::Bumblebees => "Bumbles",
        UnitType::Honeybees => "Honey",
        UnitType::MurderHornet => "Murder",
        UnitType::Earwigs => "Earwigs",
        UnitType::ScorpionVariant => "S.Scorpion",
        UnitType::StickBugs => "Sticks",
        UnitType::LeafBugs => "Leaves",
        UnitType::Cicadas => "Cicadas",
        UnitType::Grasshoppers => "Hoppers",
        UnitType::Cockroaches => "Roaches",
    }
}

/// Get the building requirement for a unit type  
fn get_unit_building_requirement(unit_type: &UnitType) -> BuildingType {
    match unit_type {
        // Workers and basic units from Queen
        UnitType::WorkerAnt | UnitType::DungBeetle | UnitType::Silverfish |
        UnitType::BlackAnt | UnitType::WorkerFourmi => 
            BuildingType::Queen,
            
        // Flying and scout units from Nursery
        UnitType::HoneyBee | UnitType::Honeybees | UnitType::DragonFly |
        UnitType::DragonFlies | UnitType::Moths | UnitType::PeacockMoth |
        UnitType::Firefly | UnitType::Damselfly | UnitType::Housefly |
        UnitType::HouseflyVariant | UnitType::Horsefly | UnitType::ScoutAnt |
        UnitType::Aphids | UnitType::Mites | UnitType::Fleas => 
            BuildingType::Nursery,
            
        // Military units from WarriorChamber  
        _ => BuildingType::WarriorChamber,
    }
}