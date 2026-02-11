/* This file is generated. Do not edit manually. */
#[derive(Debug, Clone, Copy)]
pub struct Dimension {
    pub id: u8,
    pub minecraft_name: &'static str,
    pub fixed_time: Option<i64>,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub coordinate_scale: f64,
    pub min_y: i32,
    pub height: i32,
    pub logical_height: i32,
    pub infiniburn: &'static str,
    pub ambient_light: f32,
}
impl Dimension {
    pub const OVERWORLD: Self = Self {
        id: 0u8,
        minecraft_name: "minecraft:overworld",
        fixed_time: None,
        has_skylight: true,
        has_ceiling: false,
        coordinate_scale: 1f64,
        min_y: -64i32,
        height: 384i32,
        logical_height: 384i32,
        infiniburn: "#minecraft:infiniburn_overworld",
        ambient_light: 0f32,
    };
    pub const OVERWORLD_CAVES: Self = Self {
        id: 1u8,
        minecraft_name: "minecraft:overworld_caves",
        fixed_time: None,
        has_skylight: true,
        has_ceiling: true,
        coordinate_scale: 1f64,
        min_y: -64i32,
        height: 384i32,
        logical_height: 384i32,
        infiniburn: "#minecraft:infiniburn_overworld",
        ambient_light: 0f32,
    };
    pub const THE_END: Self = Self {
        id: 2u8,
        minecraft_name: "minecraft:the_end",
        fixed_time: None,
        has_skylight: true,
        has_ceiling: false,
        coordinate_scale: 1f64,
        min_y: 0i32,
        height: 256i32,
        logical_height: 256i32,
        infiniburn: "#minecraft:infiniburn_end",
        ambient_light: 0.25f32,
    };
    pub const THE_NETHER: Self = Self {
        id: 3u8,
        minecraft_name: "minecraft:the_nether",
        fixed_time: None,
        has_skylight: false,
        has_ceiling: true,
        coordinate_scale: 8f64,
        min_y: 0i32,
        height: 256i32,
        logical_height: 128i32,
        infiniburn: "#minecraft:infiniburn_nether",
        ambient_light: 0.1f32,
    };
    pub fn from_name(name: &str) -> Option<&'static Self> {
        match name {
            "minecraft:overworld" => Some(&Self::OVERWORLD),
            "minecraft:overworld_caves" => Some(&Self::OVERWORLD_CAVES),
            "minecraft:the_end" => Some(&Self::THE_END),
            "minecraft:the_nether" => Some(&Self::THE_NETHER),
            _ => None,
        }
    }
}
impl PartialEq for Dimension {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Dimension {}
