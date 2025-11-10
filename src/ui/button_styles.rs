use bevy::prelude::*;
use crate::components::*;

// Button style definitions to reduce duplication
pub struct ButtonStyle {
    pub normal: Color,
    pub hover: Color,
    pub pressed: Color,
    #[allow(dead_code)]
    pub disabled: Color,
}

impl ButtonStyle {
    pub const BUILDING_AFFORDABLE: Self = Self {
        normal: Color::srgba(0.2, 0.2, 0.2, 0.8),
        hover: Color::srgba(0.3, 0.7, 0.3, 0.8),
        pressed: Color::srgba(0.2, 0.8, 0.2, 0.8),
        disabled: Color::srgba(0.4, 0.2, 0.2, 0.8),
    };

    pub const BUILDING_UNAFFORDABLE: Self = Self {
        normal: Color::srgba(0.4, 0.2, 0.2, 0.8),
        hover: Color::srgba(0.7, 0.3, 0.3, 0.8),
        pressed: Color::srgba(0.8, 0.2, 0.2, 0.8),
        disabled: Color::srgba(0.4, 0.2, 0.2, 0.8),
    };

    pub const UNIT_BUTTON: Self = Self {
        normal: Color::srgba(0.3, 0.3, 0.3, 0.8),
        hover: Color::srgba(0.4, 0.4, 0.4, 0.8),
        pressed: Color::srgba(0.2, 0.2, 0.2, 0.8),
        disabled: Color::srgba(0.2, 0.2, 0.2, 0.4),
    };
}

pub fn create_building_button_with_icon(
    parent: &mut ChildBuilder,
    building_type: BuildingType,
    icon_handle: Handle<Image>,
    name: &str,
    cost: Vec<(ResourceType, f32)>,
) {
    use crate::constants::ui::*;
    
    parent.spawn((
        Button,
        Node {
            width: Val::Px(BUILDING_BUTTON_SIZE),
            height: Val::Px(BUILDING_BUTTON_SIZE),
            border: UiRect::all(Val::Px(BUILDING_BUTTON_BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(ButtonStyle::BUILDING_AFFORDABLE.normal),
        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        BuildingButton { building_type, cost },
    )).with_children(|parent| {
        // Icon image
        parent.spawn((
            ImageNode::new(icon_handle),
            Node {
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                ..default()
            },
        ));
        // Building name
        parent.spawn((
            Text::new(name),
            TextFont {
                font_size: BUILDING_BUTTON_TEXT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

pub fn create_unit_button_with_icon(
    parent: &mut ChildBuilder,
    unit_type: UnitType,
    icon_handle: Handle<Image>,
    name: &str,
    cost: Vec<(ResourceType, f32)>,
    building_type: BuildingType,
) {
    use crate::constants::ui::*;
    
    parent.spawn((
        Button,
        Node {
            width: Val::Percent(48.0), // Make buttons thinner for two-column layout
            height: Val::Px(UNIT_BUTTON_HEIGHT),
            border: UiRect::all(Val::Px(UNIT_BUTTON_BORDER)),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(UNIT_BUTTON_PADDING)),
            column_gap: Val::Px(4.0), // Reduce gap between icon and text
            ..default()
        },
        BackgroundColor(UNIT_BUTTON_COLOR),
        BorderColor(UNIT_BORDER_COLOR),
        UnitButton { unit_type, cost, building_type },
    )).with_children(|parent| {
        // Unit icon
        parent.spawn((
            ImageNode::new(icon_handle),
            Node {
                width: Val::Px(20.0), // Smaller icon for compact layout
                height: Val::Px(20.0),
                ..default()
            },
        ));
        // Unit name
        parent.spawn((
            Text::new(name),
            TextFont {
                font_size: UNIT_BUTTON_TEXT_SIZE - 2.0, // Smaller text to fit better
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

#[derive(Component)]
pub struct BuildingButton {
    pub building_type: BuildingType,
    pub cost: Vec<(ResourceType, f32)>,
}

#[derive(Component)]
pub struct UnitButton {
    pub unit_type: UnitType,
    pub cost: Vec<(ResourceType, f32)>,
    #[allow(dead_code)]
    pub building_type: BuildingType,
}