use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct UIIcons {
    // Building icons
    pub nursery_icon: Handle<Image>,
    pub warrior_chamber_icon: Handle<Image>,
    pub hunter_chamber_icon: Handle<Image>,
    pub fungal_garden_icon: Handle<Image>,
    pub wood_processor_icon: Handle<Image>,
    pub mineral_processor_icon: Handle<Image>,
    
    // Unit icons
    pub worker_icon: Handle<Image>,
    pub soldier_icon: Handle<Image>,
    pub hunter_icon: Handle<Image>,
    
    // Resource icons
    pub nectar_icon: Handle<Image>,
    pub chitin_icon: Handle<Image>,
    pub minerals_icon: Handle<Image>,
    pub pheromones_icon: Handle<Image>,
    pub population_icon: Handle<Image>,
    
    pub icons_loaded: bool,
}

pub fn load_ui_icons(
    mut ui_icons: ResMut<UIIcons>,
    asset_server: Res<AssetServer>,
) {
    info!("Loading UI icons from game-icons collection...");
    
    // Building icons using appropriate themed icons
    ui_icons.nursery_icon = asset_server.load("icons/ffffff/000000/1x1/sbed/hive.svg");
    ui_icons.warrior_chamber_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/artificial-hive.svg");
    ui_icons.hunter_chamber_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/spider-web.svg");
    ui_icons.fungal_garden_icon = asset_server.load("icons/ffffff/000000/1x1/caro-asercion/water-mill.svg");
    ui_icons.wood_processor_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/gear-hammer.svg");
    ui_icons.mineral_processor_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/stone-sphere.svg");  // Using stone icon for mineral processing
    
    // Unit icons using insect-themed icons
    ui_icons.worker_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/bee.svg");
    ui_icons.soldier_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/centipede.svg");
    ui_icons.hunter_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/dragonfly.svg");
    
    // Resource icons using thematic representations
    ui_icons.nectar_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/honeycomb.svg");
    ui_icons.chitin_icon = asset_server.load("icons/ffffff/000000/1x1/sbed/claw.svg");
    ui_icons.minerals_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/stone-block.svg");
    ui_icons.pheromones_icon = asset_server.load("icons/ffffff/000000/1x1/willdabeast/gold-bar.svg");
    ui_icons.population_icon = asset_server.load("icons/ffffff/000000/1x1/lorc/all-for-one.svg");
    
    ui_icons.icons_loaded = true;
    info!("UI icons loaded successfully");
}