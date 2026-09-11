/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapColor {
    pub id: u8,
    pub name: &'static str,
    pub col: u32,
    pub rgb: (u8, u8, u8),
}
impl MapColor {
    pub const NONE_0: MapColor = MapColor {
        id: 0u8,
        name: "none",
        col: 0u32,
        rgb: (0u8, 0u8, 0u8),
    };
    pub const GRASS: MapColor = MapColor {
        id: 1u8,
        name: "grass",
        col: 8368696u32,
        rgb: (127u8, 178u8, 56u8),
    };
    pub const SAND: MapColor = MapColor {
        id: 2u8,
        name: "sand",
        col: 16247203u32,
        rgb: (247u8, 233u8, 163u8),
    };
    pub const WOOL: MapColor = MapColor {
        id: 3u8,
        name: "wool",
        col: 13092807u32,
        rgb: (199u8, 199u8, 199u8),
    };
    pub const FIRE: MapColor = MapColor {
        id: 4u8,
        name: "fire",
        col: 16711680u32,
        rgb: (255u8, 0u8, 0u8),
    };
    pub const ICE: MapColor = MapColor {
        id: 5u8,
        name: "ice",
        col: 10526975u32,
        rgb: (160u8, 160u8, 255u8),
    };
    pub const METAL: MapColor = MapColor {
        id: 6u8,
        name: "metal",
        col: 10987431u32,
        rgb: (167u8, 167u8, 167u8),
    };
    pub const PLANT: MapColor = MapColor {
        id: 7u8,
        name: "plant",
        col: 31744u32,
        rgb: (0u8, 124u8, 0u8),
    };
    pub const SNOW: MapColor = MapColor {
        id: 8u8,
        name: "snow",
        col: 16777215u32,
        rgb: (255u8, 255u8, 255u8),
    };
    pub const CLAY: MapColor = MapColor {
        id: 9u8,
        name: "clay",
        col: 10791096u32,
        rgb: (164u8, 168u8, 184u8),
    };
    pub const DIRT: MapColor = MapColor {
        id: 10u8,
        name: "dirt",
        col: 9923917u32,
        rgb: (151u8, 109u8, 77u8),
    };
    pub const STONE: MapColor = MapColor {
        id: 11u8,
        name: "stone",
        col: 7368816u32,
        rgb: (112u8, 112u8, 112u8),
    };
    pub const WATER: MapColor = MapColor {
        id: 12u8,
        name: "water",
        col: 4210943u32,
        rgb: (64u8, 64u8, 255u8),
    };
    pub const WOOD: MapColor = MapColor {
        id: 13u8,
        name: "wood",
        col: 9402184u32,
        rgb: (143u8, 119u8, 72u8),
    };
    pub const QUARTZ: MapColor = MapColor {
        id: 14u8,
        name: "quartz",
        col: 16776437u32,
        rgb: (255u8, 252u8, 245u8),
    };
    pub const COLOR_ORANGE: MapColor = MapColor {
        id: 15u8,
        name: "color_orange",
        col: 14188339u32,
        rgb: (216u8, 127u8, 51u8),
    };
    pub const COLOR_MAGENTA: MapColor = MapColor {
        id: 16u8,
        name: "color_magenta",
        col: 11685080u32,
        rgb: (178u8, 76u8, 216u8),
    };
    pub const COLOR_LIGHT_BLUE: MapColor = MapColor {
        id: 17u8,
        name: "color_light_blue",
        col: 6724056u32,
        rgb: (102u8, 153u8, 216u8),
    };
    pub const COLOR_YELLOW: MapColor = MapColor {
        id: 18u8,
        name: "color_yellow",
        col: 15066419u32,
        rgb: (229u8, 229u8, 51u8),
    };
    pub const COLOR_LIGHT_GREEN: MapColor = MapColor {
        id: 19u8,
        name: "color_light_green",
        col: 8375321u32,
        rgb: (127u8, 204u8, 25u8),
    };
    pub const COLOR_PINK: MapColor = MapColor {
        id: 20u8,
        name: "color_pink",
        col: 15892389u32,
        rgb: (242u8, 127u8, 165u8),
    };
    pub const COLOR_GRAY: MapColor = MapColor {
        id: 21u8,
        name: "color_gray",
        col: 5000268u32,
        rgb: (76u8, 76u8, 76u8),
    };
    pub const COLOR_LIGHT_GRAY: MapColor = MapColor {
        id: 22u8,
        name: "color_light_gray",
        col: 10066329u32,
        rgb: (153u8, 153u8, 153u8),
    };
    pub const COLOR_CYAN: MapColor = MapColor {
        id: 23u8,
        name: "color_cyan",
        col: 5013401u32,
        rgb: (76u8, 127u8, 153u8),
    };
    pub const COLOR_PURPLE: MapColor = MapColor {
        id: 24u8,
        name: "color_purple",
        col: 8339378u32,
        rgb: (127u8, 63u8, 178u8),
    };
    pub const COLOR_BLUE: MapColor = MapColor {
        id: 25u8,
        name: "color_blue",
        col: 3361970u32,
        rgb: (51u8, 76u8, 178u8),
    };
    pub const COLOR_BROWN: MapColor = MapColor {
        id: 26u8,
        name: "color_brown",
        col: 6704179u32,
        rgb: (102u8, 76u8, 51u8),
    };
    pub const COLOR_GREEN: MapColor = MapColor {
        id: 27u8,
        name: "color_green",
        col: 6717235u32,
        rgb: (102u8, 127u8, 51u8),
    };
    pub const COLOR_RED: MapColor = MapColor {
        id: 28u8,
        name: "color_red",
        col: 10040115u32,
        rgb: (153u8, 51u8, 51u8),
    };
    pub const COLOR_BLACK: MapColor = MapColor {
        id: 29u8,
        name: "color_black",
        col: 1644825u32,
        rgb: (25u8, 25u8, 25u8),
    };
    pub const GOLD: MapColor = MapColor {
        id: 30u8,
        name: "gold",
        col: 16445005u32,
        rgb: (250u8, 238u8, 77u8),
    };
    pub const DIAMOND: MapColor = MapColor {
        id: 31u8,
        name: "diamond",
        col: 6085589u32,
        rgb: (92u8, 219u8, 213u8),
    };
    pub const LAPIS: MapColor = MapColor {
        id: 32u8,
        name: "lapis",
        col: 4882687u32,
        rgb: (74u8, 128u8, 255u8),
    };
    pub const EMERALD: MapColor = MapColor {
        id: 33u8,
        name: "emerald",
        col: 55610u32,
        rgb: (0u8, 217u8, 58u8),
    };
    pub const PODZOL: MapColor = MapColor {
        id: 34u8,
        name: "podzol",
        col: 8476209u32,
        rgb: (129u8, 86u8, 49u8),
    };
    pub const NETHER: MapColor = MapColor {
        id: 35u8,
        name: "nether",
        col: 7340544u32,
        rgb: (112u8, 2u8, 0u8),
    };
    pub const TERRACOTTA_WHITE: MapColor = MapColor {
        id: 36u8,
        name: "terracotta_white",
        col: 13742497u32,
        rgb: (209u8, 177u8, 161u8),
    };
    pub const TERRACOTTA_ORANGE: MapColor = MapColor {
        id: 37u8,
        name: "terracotta_orange",
        col: 10441252u32,
        rgb: (159u8, 82u8, 36u8),
    };
    pub const TERRACOTTA_MAGENTA: MapColor = MapColor {
        id: 38u8,
        name: "terracotta_magenta",
        col: 9787244u32,
        rgb: (149u8, 87u8, 108u8),
    };
    pub const TERRACOTTA_LIGHT_BLUE: MapColor = MapColor {
        id: 39u8,
        name: "terracotta_light_blue",
        col: 7367818u32,
        rgb: (112u8, 108u8, 138u8),
    };
    pub const TERRACOTTA_YELLOW: MapColor = MapColor {
        id: 40u8,
        name: "terracotta_yellow",
        col: 12223780u32,
        rgb: (186u8, 133u8, 36u8),
    };
    pub const TERRACOTTA_LIGHT_GREEN: MapColor = MapColor {
        id: 41u8,
        name: "terracotta_light_green",
        col: 6780213u32,
        rgb: (103u8, 117u8, 53u8),
    };
    pub const TERRACOTTA_PINK: MapColor = MapColor {
        id: 42u8,
        name: "terracotta_pink",
        col: 10505550u32,
        rgb: (160u8, 77u8, 78u8),
    };
    pub const TERRACOTTA_GRAY: MapColor = MapColor {
        id: 43u8,
        name: "terracotta_gray",
        col: 3746083u32,
        rgb: (57u8, 41u8, 35u8),
    };
    pub const TERRACOTTA_LIGHT_GRAY: MapColor = MapColor {
        id: 44u8,
        name: "terracotta_light_gray",
        col: 8874850u32,
        rgb: (135u8, 107u8, 98u8),
    };
    pub const TERRACOTTA_CYAN: MapColor = MapColor {
        id: 45u8,
        name: "terracotta_cyan",
        col: 5725276u32,
        rgb: (87u8, 92u8, 92u8),
    };
    pub const TERRACOTTA_PURPLE: MapColor = MapColor {
        id: 46u8,
        name: "terracotta_purple",
        col: 8014168u32,
        rgb: (122u8, 73u8, 88u8),
    };
    pub const TERRACOTTA_BLUE: MapColor = MapColor {
        id: 47u8,
        name: "terracotta_blue",
        col: 4996700u32,
        rgb: (76u8, 62u8, 92u8),
    };
    pub const TERRACOTTA_BROWN: MapColor = MapColor {
        id: 48u8,
        name: "terracotta_brown",
        col: 4993571u32,
        rgb: (76u8, 50u8, 35u8),
    };
    pub const TERRACOTTA_GREEN: MapColor = MapColor {
        id: 49u8,
        name: "terracotta_green",
        col: 5001770u32,
        rgb: (76u8, 82u8, 42u8),
    };
    pub const TERRACOTTA_RED: MapColor = MapColor {
        id: 50u8,
        name: "terracotta_red",
        col: 9321518u32,
        rgb: (142u8, 60u8, 46u8),
    };
    pub const TERRACOTTA_BLACK: MapColor = MapColor {
        id: 51u8,
        name: "terracotta_black",
        col: 2430480u32,
        rgb: (37u8, 22u8, 16u8),
    };
    pub const CRIMSON_NYLIUM: MapColor = MapColor {
        id: 52u8,
        name: "crimson_nylium",
        col: 12398641u32,
        rgb: (189u8, 48u8, 49u8),
    };
    pub const CRIMSON_STEM: MapColor = MapColor {
        id: 53u8,
        name: "crimson_stem",
        col: 9715553u32,
        rgb: (148u8, 63u8, 97u8),
    };
    pub const CRIMSON_HYPHAE: MapColor = MapColor {
        id: 54u8,
        name: "crimson_hyphae",
        col: 6035741u32,
        rgb: (92u8, 25u8, 29u8),
    };
    pub const WARPED_NYLIUM: MapColor = MapColor {
        id: 55u8,
        name: "warped_nylium",
        col: 1474182u32,
        rgb: (22u8, 126u8, 134u8),
    };
    pub const WARPED_STEM: MapColor = MapColor {
        id: 56u8,
        name: "warped_stem",
        col: 3837580u32,
        rgb: (58u8, 142u8, 140u8),
    };
    pub const WARPED_HYPHAE: MapColor = MapColor {
        id: 57u8,
        name: "warped_hyphae",
        col: 5647422u32,
        rgb: (86u8, 44u8, 62u8),
    };
    pub const WARPED_WART_BLOCK: MapColor = MapColor {
        id: 58u8,
        name: "warped_wart_block",
        col: 1356933u32,
        rgb: (20u8, 180u8, 133u8),
    };
    pub const DEEPSLATE: MapColor = MapColor {
        id: 59u8,
        name: "deepslate",
        col: 6579300u32,
        rgb: (100u8, 100u8, 100u8),
    };
    pub const RAW_IRON: MapColor = MapColor {
        id: 60u8,
        name: "raw_iron",
        col: 14200723u32,
        rgb: (216u8, 175u8, 147u8),
    };
    pub const GLOW_LICHEN: MapColor = MapColor {
        id: 61u8,
        name: "glow_lichen",
        col: 8365974u32,
        rgb: (127u8, 167u8, 150u8),
    };
    pub const NONE_62: MapColor = MapColor {
        id: 62u8,
        name: "none",
        col: 0u32,
        rgb: (0u8, 0u8, 0u8),
    };
    pub const NONE_63: MapColor = MapColor {
        id: 63u8,
        name: "none",
        col: 0u32,
        rgb: (0u8, 0u8, 0u8),
    };
    pub const ALL: &'static [MapColor] = &[
        MapColor::NONE_0,
        MapColor::GRASS,
        MapColor::SAND,
        MapColor::WOOL,
        MapColor::FIRE,
        MapColor::ICE,
        MapColor::METAL,
        MapColor::PLANT,
        MapColor::SNOW,
        MapColor::CLAY,
        MapColor::DIRT,
        MapColor::STONE,
        MapColor::WATER,
        MapColor::WOOD,
        MapColor::QUARTZ,
        MapColor::COLOR_ORANGE,
        MapColor::COLOR_MAGENTA,
        MapColor::COLOR_LIGHT_BLUE,
        MapColor::COLOR_YELLOW,
        MapColor::COLOR_LIGHT_GREEN,
        MapColor::COLOR_PINK,
        MapColor::COLOR_GRAY,
        MapColor::COLOR_LIGHT_GRAY,
        MapColor::COLOR_CYAN,
        MapColor::COLOR_PURPLE,
        MapColor::COLOR_BLUE,
        MapColor::COLOR_BROWN,
        MapColor::COLOR_GREEN,
        MapColor::COLOR_RED,
        MapColor::COLOR_BLACK,
        MapColor::GOLD,
        MapColor::DIAMOND,
        MapColor::LAPIS,
        MapColor::EMERALD,
        MapColor::PODZOL,
        MapColor::NETHER,
        MapColor::TERRACOTTA_WHITE,
        MapColor::TERRACOTTA_ORANGE,
        MapColor::TERRACOTTA_MAGENTA,
        MapColor::TERRACOTTA_LIGHT_BLUE,
        MapColor::TERRACOTTA_YELLOW,
        MapColor::TERRACOTTA_LIGHT_GREEN,
        MapColor::TERRACOTTA_PINK,
        MapColor::TERRACOTTA_GRAY,
        MapColor::TERRACOTTA_LIGHT_GRAY,
        MapColor::TERRACOTTA_CYAN,
        MapColor::TERRACOTTA_PURPLE,
        MapColor::TERRACOTTA_BLUE,
        MapColor::TERRACOTTA_BROWN,
        MapColor::TERRACOTTA_GREEN,
        MapColor::TERRACOTTA_RED,
        MapColor::TERRACOTTA_BLACK,
        MapColor::CRIMSON_NYLIUM,
        MapColor::CRIMSON_STEM,
        MapColor::CRIMSON_HYPHAE,
        MapColor::WARPED_NYLIUM,
        MapColor::WARPED_STEM,
        MapColor::WARPED_HYPHAE,
        MapColor::WARPED_WART_BLOCK,
        MapColor::DEEPSLATE,
        MapColor::RAW_IRON,
        MapColor::GLOW_LICHEN,
        MapColor::NONE_62,
        MapColor::NONE_63,
    ];
    #[must_use]
    pub const fn from_id(id: u8) -> Option<&'static MapColor> {
        if (id as usize) < Self::ALL.len() {
            Some(&Self::ALL[id as usize])
        } else {
            None
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<&'static MapColor> {
        match name {
            "none" => Some(&Self::NONE_0),
            "grass" => Some(&Self::GRASS),
            "sand" => Some(&Self::SAND),
            "wool" => Some(&Self::WOOL),
            "fire" => Some(&Self::FIRE),
            "ice" => Some(&Self::ICE),
            "metal" => Some(&Self::METAL),
            "plant" => Some(&Self::PLANT),
            "snow" => Some(&Self::SNOW),
            "clay" => Some(&Self::CLAY),
            "dirt" => Some(&Self::DIRT),
            "stone" => Some(&Self::STONE),
            "water" => Some(&Self::WATER),
            "wood" => Some(&Self::WOOD),
            "quartz" => Some(&Self::QUARTZ),
            "color_orange" => Some(&Self::COLOR_ORANGE),
            "color_magenta" => Some(&Self::COLOR_MAGENTA),
            "color_light_blue" => Some(&Self::COLOR_LIGHT_BLUE),
            "color_yellow" => Some(&Self::COLOR_YELLOW),
            "color_light_green" => Some(&Self::COLOR_LIGHT_GREEN),
            "color_pink" => Some(&Self::COLOR_PINK),
            "color_gray" => Some(&Self::COLOR_GRAY),
            "color_light_gray" => Some(&Self::COLOR_LIGHT_GRAY),
            "color_cyan" => Some(&Self::COLOR_CYAN),
            "color_purple" => Some(&Self::COLOR_PURPLE),
            "color_blue" => Some(&Self::COLOR_BLUE),
            "color_brown" => Some(&Self::COLOR_BROWN),
            "color_green" => Some(&Self::COLOR_GREEN),
            "color_red" => Some(&Self::COLOR_RED),
            "color_black" => Some(&Self::COLOR_BLACK),
            "gold" => Some(&Self::GOLD),
            "diamond" => Some(&Self::DIAMOND),
            "lapis" => Some(&Self::LAPIS),
            "emerald" => Some(&Self::EMERALD),
            "podzol" => Some(&Self::PODZOL),
            "nether" => Some(&Self::NETHER),
            "terracotta_white" => Some(&Self::TERRACOTTA_WHITE),
            "terracotta_orange" => Some(&Self::TERRACOTTA_ORANGE),
            "terracotta_magenta" => Some(&Self::TERRACOTTA_MAGENTA),
            "terracotta_light_blue" => Some(&Self::TERRACOTTA_LIGHT_BLUE),
            "terracotta_yellow" => Some(&Self::TERRACOTTA_YELLOW),
            "terracotta_light_green" => Some(&Self::TERRACOTTA_LIGHT_GREEN),
            "terracotta_pink" => Some(&Self::TERRACOTTA_PINK),
            "terracotta_gray" => Some(&Self::TERRACOTTA_GRAY),
            "terracotta_light_gray" => Some(&Self::TERRACOTTA_LIGHT_GRAY),
            "terracotta_cyan" => Some(&Self::TERRACOTTA_CYAN),
            "terracotta_purple" => Some(&Self::TERRACOTTA_PURPLE),
            "terracotta_blue" => Some(&Self::TERRACOTTA_BLUE),
            "terracotta_brown" => Some(&Self::TERRACOTTA_BROWN),
            "terracotta_green" => Some(&Self::TERRACOTTA_GREEN),
            "terracotta_red" => Some(&Self::TERRACOTTA_RED),
            "terracotta_black" => Some(&Self::TERRACOTTA_BLACK),
            "crimson_nylium" => Some(&Self::CRIMSON_NYLIUM),
            "crimson_stem" => Some(&Self::CRIMSON_STEM),
            "crimson_hyphae" => Some(&Self::CRIMSON_HYPHAE),
            "warped_nylium" => Some(&Self::WARPED_NYLIUM),
            "warped_stem" => Some(&Self::WARPED_STEM),
            "warped_hyphae" => Some(&Self::WARPED_HYPHAE),
            "warped_wart_block" => Some(&Self::WARPED_WART_BLOCK),
            "deepslate" => Some(&Self::DEEPSLATE),
            "raw_iron" => Some(&Self::RAW_IRON),
            "glow_lichen" => Some(&Self::GLOW_LICHEN),
            _ => None,
        }
    }
}
