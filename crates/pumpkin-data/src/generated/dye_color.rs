/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    #[default]
    Black,
}
impl DyeColor {
    #[must_use]
    pub const fn id(&self) -> u8 {
        match self {
            Self::White => 0u8,
            Self::Orange => 1u8,
            Self::Magenta => 2u8,
            Self::LightBlue => 3u8,
            Self::Yellow => 4u8,
            Self::Lime => 5u8,
            Self::Pink => 6u8,
            Self::Gray => 7u8,
            Self::LightGray => 8u8,
            Self::Cyan => 9u8,
            Self::Purple => 10u8,
            Self::Blue => 11u8,
            Self::Brown => 12u8,
            Self::Green => 13u8,
            Self::Red => 14u8,
            Self::Black => 15u8,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Orange => "orange",
            Self::Magenta => "magenta",
            Self::LightBlue => "light_blue",
            Self::Yellow => "yellow",
            Self::Lime => "lime",
            Self::Pink => "pink",
            Self::Gray => "gray",
            Self::LightGray => "light_gray",
            Self::Cyan => "cyan",
            Self::Purple => "purple",
            Self::Blue => "blue",
            Self::Brown => "brown",
            Self::Green => "green",
            Self::Red => "red",
            Self::Black => "black",
        }
    }
    #[must_use]
    pub const fn map_color_id(&self) -> u8 {
        match self {
            Self::White => 8u8,
            Self::Orange => 15u8,
            Self::Magenta => 16u8,
            Self::LightBlue => 17u8,
            Self::Yellow => 18u8,
            Self::Lime => 19u8,
            Self::Pink => 20u8,
            Self::Gray => 21u8,
            Self::LightGray => 22u8,
            Self::Cyan => 23u8,
            Self::Purple => 24u8,
            Self::Blue => 25u8,
            Self::Brown => 26u8,
            Self::Green => 27u8,
            Self::Red => 28u8,
            Self::Black => 29u8,
        }
    }
    #[must_use]
    pub const fn terracotta_color_id(&self) -> u8 {
        match self {
            Self::White => 36u8,
            Self::Orange => 37u8,
            Self::Magenta => 38u8,
            Self::LightBlue => 39u8,
            Self::Yellow => 40u8,
            Self::Lime => 41u8,
            Self::Pink => 42u8,
            Self::Gray => 43u8,
            Self::LightGray => 44u8,
            Self::Cyan => 45u8,
            Self::Purple => 46u8,
            Self::Blue => 47u8,
            Self::Brown => 48u8,
            Self::Green => 49u8,
            Self::Red => 50u8,
            Self::Black => 51u8,
        }
    }
    #[must_use]
    pub const fn texture_diffuse_color(&self) -> u32 {
        match self {
            Self::White => 16383998u32,
            Self::Orange => 16351261u32,
            Self::Magenta => 13061821u32,
            Self::LightBlue => 3847130u32,
            Self::Yellow => 16701501u32,
            Self::Lime => 8439583u32,
            Self::Pink => 15961002u32,
            Self::Gray => 4673362u32,
            Self::LightGray => 10329495u32,
            Self::Cyan => 1481884u32,
            Self::Purple => 8991416u32,
            Self::Blue => 3949738u32,
            Self::Brown => 8606770u32,
            Self::Green => 6192150u32,
            Self::Red => 11546150u32,
            Self::Black => 1908001u32,
        }
    }
    #[must_use]
    pub const fn firework_color(&self) -> u32 {
        match self {
            Self::White => 15790320u32,
            Self::Orange => 15435844u32,
            Self::Magenta => 12801229u32,
            Self::LightBlue => 6719955u32,
            Self::Yellow => 14602026u32,
            Self::Lime => 4312372u32,
            Self::Pink => 14188952u32,
            Self::Gray => 4408131u32,
            Self::LightGray => 11250603u32,
            Self::Cyan => 2651799u32,
            Self::Purple => 8073150u32,
            Self::Blue => 2437522u32,
            Self::Brown => 5320730u32,
            Self::Green => 3887386u32,
            Self::Red => 11743532u32,
            Self::Black => 1973019u32,
        }
    }
    #[must_use]
    pub const fn text_color(&self) -> u32 {
        match self {
            Self::White => 16777215u32,
            Self::Orange => 16738335u32,
            Self::Magenta => 16711935u32,
            Self::LightBlue => 10141901u32,
            Self::Yellow => 16776960u32,
            Self::Lime => 12582656u32,
            Self::Pink => 16738740u32,
            Self::Gray => 8421504u32,
            Self::LightGray => 13882323u32,
            Self::Cyan => 65535u32,
            Self::Purple => 10494192u32,
            Self::Blue => 255u32,
            Self::Brown => 9127187u32,
            Self::Green => 65280u32,
            Self::Red => 16711680u32,
            Self::Black => 0u32,
        }
    }
    #[must_use]
    pub const fn by_id(id: u8) -> Option<Self> {
        match id {
            0u8 => Some(Self::White),
            1u8 => Some(Self::Orange),
            2u8 => Some(Self::Magenta),
            3u8 => Some(Self::LightBlue),
            4u8 => Some(Self::Yellow),
            5u8 => Some(Self::Lime),
            6u8 => Some(Self::Pink),
            7u8 => Some(Self::Gray),
            8u8 => Some(Self::LightGray),
            9u8 => Some(Self::Cyan),
            10u8 => Some(Self::Purple),
            11u8 => Some(Self::Blue),
            12u8 => Some(Self::Brown),
            13u8 => Some(Self::Green),
            14u8 => Some(Self::Red),
            15u8 => Some(Self::Black),
            _ => None,
        }
    }
    #[must_use]
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "white" => Some(Self::White),
            "orange" => Some(Self::Orange),
            "magenta" => Some(Self::Magenta),
            "light_blue" => Some(Self::LightBlue),
            "yellow" => Some(Self::Yellow),
            "lime" => Some(Self::Lime),
            "pink" => Some(Self::Pink),
            "gray" => Some(Self::Gray),
            "light_gray" => Some(Self::LightGray),
            "cyan" => Some(Self::Cyan),
            "purple" => Some(Self::Purple),
            "blue" => Some(Self::Blue),
            "brown" => Some(Self::Brown),
            "green" => Some(Self::Green),
            "red" => Some(Self::Red),
            "black" => Some(Self::Black),
            _ => None,
        }
    }
}
impl From<DyeColor> for String {
    fn from(value: DyeColor) -> Self {
        value.name().to_string()
    }
}
impl From<&str> for DyeColor {
    fn from(s: &str) -> Self {
        DyeColor::by_name(s).unwrap_or_default()
    }
}
impl From<i8> for DyeColor {
    fn from(s: i8) -> Self {
        if s >= 0 {
            DyeColor::by_id(s as u8).unwrap_or_default()
        } else {
            DyeColor::default()
        }
    }
}
impl From<u8> for DyeColor {
    fn from(s: u8) -> Self {
        DyeColor::by_id(s).unwrap_or_default()
    }
}
