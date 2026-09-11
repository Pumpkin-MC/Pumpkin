/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapDecorationType {
    pub id: u32,
    pub name: &'static str,
    pub asset_name: &'static str,
    pub show_on_item_frame: bool,
    pub map_color: i32,
    pub exploration_map_element: bool,
    pub track_count: bool,
}
impl MapDecorationType {
    pub const PLAYER: MapDecorationType = MapDecorationType {
        id: 0u32,
        name: "player",
        asset_name: "player",
        show_on_item_frame: false,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const FRAME: MapDecorationType = MapDecorationType {
        id: 1u32,
        name: "frame",
        asset_name: "frame",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const RED_MARKER: MapDecorationType = MapDecorationType {
        id: 2u32,
        name: "red_marker",
        asset_name: "red_marker",
        show_on_item_frame: false,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BLUE_MARKER: MapDecorationType = MapDecorationType {
        id: 3u32,
        name: "blue_marker",
        asset_name: "blue_marker",
        show_on_item_frame: false,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const TARGET_X: MapDecorationType = MapDecorationType {
        id: 4u32,
        name: "target_x",
        asset_name: "target_x",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: false,
    };
    pub const TARGET_POINT: MapDecorationType = MapDecorationType {
        id: 5u32,
        name: "target_point",
        asset_name: "target_point",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: false,
    };
    pub const PLAYER_OFF_MAP: MapDecorationType = MapDecorationType {
        id: 6u32,
        name: "player_off_map",
        asset_name: "player_off_map",
        show_on_item_frame: false,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const PLAYER_OFF_LIMITS: MapDecorationType = MapDecorationType {
        id: 7u32,
        name: "player_off_limits",
        asset_name: "player_off_limits",
        show_on_item_frame: false,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const MANSION: MapDecorationType = MapDecorationType {
        id: 8u32,
        name: "mansion",
        asset_name: "woodland_mansion",
        show_on_item_frame: true,
        map_color: 5393476i32,
        exploration_map_element: true,
        track_count: false,
    };
    pub const MONUMENT: MapDecorationType = MapDecorationType {
        id: 9u32,
        name: "monument",
        asset_name: "ocean_monument",
        show_on_item_frame: true,
        map_color: 3830373i32,
        exploration_map_element: true,
        track_count: false,
    };
    pub const BANNER_WHITE: MapDecorationType = MapDecorationType {
        id: 10u32,
        name: "banner_white",
        asset_name: "white_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_ORANGE: MapDecorationType = MapDecorationType {
        id: 11u32,
        name: "banner_orange",
        asset_name: "orange_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_MAGENTA: MapDecorationType = MapDecorationType {
        id: 12u32,
        name: "banner_magenta",
        asset_name: "magenta_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_LIGHT_BLUE: MapDecorationType = MapDecorationType {
        id: 13u32,
        name: "banner_light_blue",
        asset_name: "light_blue_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_YELLOW: MapDecorationType = MapDecorationType {
        id: 14u32,
        name: "banner_yellow",
        asset_name: "yellow_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_LIME: MapDecorationType = MapDecorationType {
        id: 15u32,
        name: "banner_lime",
        asset_name: "lime_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_PINK: MapDecorationType = MapDecorationType {
        id: 16u32,
        name: "banner_pink",
        asset_name: "pink_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_GRAY: MapDecorationType = MapDecorationType {
        id: 17u32,
        name: "banner_gray",
        asset_name: "gray_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_LIGHT_GRAY: MapDecorationType = MapDecorationType {
        id: 18u32,
        name: "banner_light_gray",
        asset_name: "light_gray_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_CYAN: MapDecorationType = MapDecorationType {
        id: 19u32,
        name: "banner_cyan",
        asset_name: "cyan_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_PURPLE: MapDecorationType = MapDecorationType {
        id: 20u32,
        name: "banner_purple",
        asset_name: "purple_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_BLUE: MapDecorationType = MapDecorationType {
        id: 21u32,
        name: "banner_blue",
        asset_name: "blue_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_BROWN: MapDecorationType = MapDecorationType {
        id: 22u32,
        name: "banner_brown",
        asset_name: "brown_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_GREEN: MapDecorationType = MapDecorationType {
        id: 23u32,
        name: "banner_green",
        asset_name: "green_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_RED: MapDecorationType = MapDecorationType {
        id: 24u32,
        name: "banner_red",
        asset_name: "red_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const BANNER_BLACK: MapDecorationType = MapDecorationType {
        id: 25u32,
        name: "banner_black",
        asset_name: "black_banner",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: true,
    };
    pub const RED_X: MapDecorationType = MapDecorationType {
        id: 26u32,
        name: "red_x",
        asset_name: "red_x",
        show_on_item_frame: true,
        map_color: -1i32,
        exploration_map_element: false,
        track_count: false,
    };
    pub const VILLAGE_DESERT: MapDecorationType = MapDecorationType {
        id: 27u32,
        name: "village_desert",
        asset_name: "desert_village",
        show_on_item_frame: true,
        map_color: 10066329i32,
        exploration_map_element: true,
        track_count: false,
    };
    pub const VILLAGE_PLAINS: MapDecorationType = MapDecorationType {
        id: 28u32,
        name: "village_plains",
        asset_name: "plains_village",
        show_on_item_frame: true,
        map_color: 10066329i32,
        exploration_map_element: true,
        track_count: false,
    };
    pub const VILLAGE_SAVANNA: MapDecorationType = MapDecorationType {
        id: 29u32,
        name: "village_savanna",
        asset_name: "savanna_village",
        show_on_item_frame: true,
        map_color: 10066329i32,
        exploration_map_element: true,
        track_count: false,
    };
    pub const VILLAGE_SNOWY: MapDecorationType = MapDecorationType {
        id: 30u32,
        name: "village_snowy",
        asset_name: "snowy_village",
        show_on_item_frame: true,
        map_color: 10066329i32,
        exploration_map_element: true,
        track_count: false,
    };
    pub const VILLAGE_TAIGA: MapDecorationType = MapDecorationType {
        id: 31u32,
        name: "village_taiga",
        asset_name: "taiga_village",
        show_on_item_frame: true,
        map_color: 10066329i32,
        exploration_map_element: true,
        track_count: false,
    };
    pub const JUNGLE_TEMPLE: MapDecorationType = MapDecorationType {
        id: 32u32,
        name: "jungle_temple",
        asset_name: "jungle_temple",
        show_on_item_frame: true,
        map_color: 10066329i32,
        exploration_map_element: true,
        track_count: false,
    };
    pub const SWAMP_HUT: MapDecorationType = MapDecorationType {
        id: 33u32,
        name: "swamp_hut",
        asset_name: "swamp_hut",
        show_on_item_frame: true,
        map_color: 10066329i32,
        exploration_map_element: true,
        track_count: false,
    };
    pub const TRIAL_CHAMBERS: MapDecorationType = MapDecorationType {
        id: 34u32,
        name: "trial_chambers",
        asset_name: "trial_chambers",
        show_on_item_frame: true,
        map_color: 12741452i32,
        exploration_map_element: true,
        track_count: false,
    };
    pub const ALL: &'static [MapDecorationType] = &[
        MapDecorationType::PLAYER,
        MapDecorationType::FRAME,
        MapDecorationType::RED_MARKER,
        MapDecorationType::BLUE_MARKER,
        MapDecorationType::TARGET_X,
        MapDecorationType::TARGET_POINT,
        MapDecorationType::PLAYER_OFF_MAP,
        MapDecorationType::PLAYER_OFF_LIMITS,
        MapDecorationType::MANSION,
        MapDecorationType::MONUMENT,
        MapDecorationType::BANNER_WHITE,
        MapDecorationType::BANNER_ORANGE,
        MapDecorationType::BANNER_MAGENTA,
        MapDecorationType::BANNER_LIGHT_BLUE,
        MapDecorationType::BANNER_YELLOW,
        MapDecorationType::BANNER_LIME,
        MapDecorationType::BANNER_PINK,
        MapDecorationType::BANNER_GRAY,
        MapDecorationType::BANNER_LIGHT_GRAY,
        MapDecorationType::BANNER_CYAN,
        MapDecorationType::BANNER_PURPLE,
        MapDecorationType::BANNER_BLUE,
        MapDecorationType::BANNER_BROWN,
        MapDecorationType::BANNER_GREEN,
        MapDecorationType::BANNER_RED,
        MapDecorationType::BANNER_BLACK,
        MapDecorationType::RED_X,
        MapDecorationType::VILLAGE_DESERT,
        MapDecorationType::VILLAGE_PLAINS,
        MapDecorationType::VILLAGE_SAVANNA,
        MapDecorationType::VILLAGE_SNOWY,
        MapDecorationType::VILLAGE_TAIGA,
        MapDecorationType::JUNGLE_TEMPLE,
        MapDecorationType::SWAMP_HUT,
        MapDecorationType::TRIAL_CHAMBERS,
    ];
    #[must_use]
    pub const fn from_id(id: u32) -> Option<&'static MapDecorationType> {
        if (id as usize) < Self::ALL.len() {
            Some(&Self::ALL[id as usize])
        } else {
            None
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<&'static MapDecorationType> {
        match name {
            "player" => Some(&Self::PLAYER),
            "frame" => Some(&Self::FRAME),
            "red_marker" => Some(&Self::RED_MARKER),
            "blue_marker" => Some(&Self::BLUE_MARKER),
            "target_x" => Some(&Self::TARGET_X),
            "target_point" => Some(&Self::TARGET_POINT),
            "player_off_map" => Some(&Self::PLAYER_OFF_MAP),
            "player_off_limits" => Some(&Self::PLAYER_OFF_LIMITS),
            "mansion" => Some(&Self::MANSION),
            "monument" => Some(&Self::MONUMENT),
            "banner_white" => Some(&Self::BANNER_WHITE),
            "banner_orange" => Some(&Self::BANNER_ORANGE),
            "banner_magenta" => Some(&Self::BANNER_MAGENTA),
            "banner_light_blue" => Some(&Self::BANNER_LIGHT_BLUE),
            "banner_yellow" => Some(&Self::BANNER_YELLOW),
            "banner_lime" => Some(&Self::BANNER_LIME),
            "banner_pink" => Some(&Self::BANNER_PINK),
            "banner_gray" => Some(&Self::BANNER_GRAY),
            "banner_light_gray" => Some(&Self::BANNER_LIGHT_GRAY),
            "banner_cyan" => Some(&Self::BANNER_CYAN),
            "banner_purple" => Some(&Self::BANNER_PURPLE),
            "banner_blue" => Some(&Self::BANNER_BLUE),
            "banner_brown" => Some(&Self::BANNER_BROWN),
            "banner_green" => Some(&Self::BANNER_GREEN),
            "banner_red" => Some(&Self::BANNER_RED),
            "banner_black" => Some(&Self::BANNER_BLACK),
            "red_x" => Some(&Self::RED_X),
            "village_desert" => Some(&Self::VILLAGE_DESERT),
            "village_plains" => Some(&Self::VILLAGE_PLAINS),
            "village_savanna" => Some(&Self::VILLAGE_SAVANNA),
            "village_snowy" => Some(&Self::VILLAGE_SNOWY),
            "village_taiga" => Some(&Self::VILLAGE_TAIGA),
            "jungle_temple" => Some(&Self::JUNGLE_TEMPLE),
            "swamp_hut" => Some(&Self::SWAMP_HUT),
            "trial_chambers" => Some(&Self::TRIAL_CHAMBERS),
            _ => None,
        }
    }
}
