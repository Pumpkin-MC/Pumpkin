/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxolotlVariant {
    Lucy,
    Wild,
    Gold,
    Cyan,
    Blue,
}
impl AxolotlVariant {
    pub const ALL: &'static [Self] = &[Self::Lucy, Self::Wild, Self::Gold, Self::Cyan, Self::Blue];
    pub const DEFAULT: Self = Self::Lucy;
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::Lucy => 0,
            Self::Wild => 1,
            Self::Gold => 2,
            Self::Cyan => 3,
            Self::Blue => 4,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Lucy => "lucy",
            Self::Wild => "wild",
            Self::Gold => "gold",
            Self::Cyan => "cyan",
            Self::Blue => "blue",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "lucy" => Some(Self::Lucy),
            "wild" => Some(Self::Wild),
            "gold" => Some(Self::Gold),
            "cyan" => Some(Self::Cyan),
            "blue" => Some(Self::Blue),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::Lucy,
            1 => Self::Wild,
            2 => Self::Gold,
            3 => Self::Cyan,
            4 => Self::Blue,
            _ => Self::Lucy,
        }
    }
    #[must_use]
    pub const fn common(&self) -> bool {
        match self {
            Self::Lucy => true,
            Self::Wild => true,
            Self::Gold => true,
            Self::Cyan => true,
            Self::Blue => false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FoxVariant {
    Red,
    Snow,
}
impl FoxVariant {
    pub const ALL: &'static [Self] = &[Self::Red, Self::Snow];
    pub const DEFAULT: Self = Self::Red;
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::Red => 0,
            Self::Snow => 1,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Snow => "snow",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "red" => Some(Self::Red),
            "snow" => Some(Self::Snow),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::Red,
            1 => Self::Snow,
            _ => Self::Red,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HorseMarkings {
    None,
    White,
    WhiteField,
    WhiteDots,
    BlackDots,
}
impl HorseMarkings {
    pub const ALL: &'static [Self] = &[
        Self::None,
        Self::White,
        Self::WhiteField,
        Self::WhiteDots,
        Self::BlackDots,
    ];
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::None => 0,
            Self::White => 1,
            Self::WhiteField => 2,
            Self::WhiteDots => 3,
            Self::BlackDots => 4,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::White => "white",
            Self::WhiteField => "white_field",
            Self::WhiteDots => "white_dots",
            Self::BlackDots => "black_dots",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "none" => Some(Self::None),
            "white" => Some(Self::White),
            "white_field" => Some(Self::WhiteField),
            "white_dots" => Some(Self::WhiteDots),
            "black_dots" => Some(Self::BlackDots),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::None,
            1 => Self::White,
            2 => Self::WhiteField,
            3 => Self::WhiteDots,
            4 => Self::BlackDots,
            _ => Self::ALL[id.rem_euclid(5) as usize],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HorseVariant {
    White,
    Creamy,
    Chestnut,
    Brown,
    Black,
    Gray,
    DarkBrown,
}
impl HorseVariant {
    pub const ALL: &'static [Self] = &[
        Self::White,
        Self::Creamy,
        Self::Chestnut,
        Self::Brown,
        Self::Black,
        Self::Gray,
        Self::DarkBrown,
    ];
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::White => 0,
            Self::Creamy => 1,
            Self::Chestnut => 2,
            Self::Brown => 3,
            Self::Black => 4,
            Self::Gray => 5,
            Self::DarkBrown => 6,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Creamy => "creamy",
            Self::Chestnut => "chestnut",
            Self::Brown => "brown",
            Self::Black => "black",
            Self::Gray => "gray",
            Self::DarkBrown => "dark_brown",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "white" => Some(Self::White),
            "creamy" => Some(Self::Creamy),
            "chestnut" => Some(Self::Chestnut),
            "brown" => Some(Self::Brown),
            "black" => Some(Self::Black),
            "gray" => Some(Self::Gray),
            "dark_brown" => Some(Self::DarkBrown),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::White,
            1 => Self::Creamy,
            2 => Self::Chestnut,
            3 => Self::Brown,
            4 => Self::Black,
            5 => Self::Gray,
            6 => Self::DarkBrown,
            _ => Self::ALL[id.rem_euclid(7) as usize],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LlamaVariant {
    Creamy,
    White,
    Brown,
    Gray,
}
impl LlamaVariant {
    pub const ALL: &'static [Self] = &[Self::Creamy, Self::White, Self::Brown, Self::Gray];
    pub const DEFAULT: Self = Self::Creamy;
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::Creamy => 0,
            Self::White => 1,
            Self::Brown => 2,
            Self::Gray => 3,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Creamy => "creamy",
            Self::White => "white",
            Self::Brown => "brown",
            Self::Gray => "gray",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "creamy" => Some(Self::Creamy),
            "white" => Some(Self::White),
            "brown" => Some(Self::Brown),
            "gray" => Some(Self::Gray),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::Creamy,
            1 => Self::White,
            2 => Self::Brown,
            3 => Self::Gray,
            _ => {
                if id < 0 {
                    Self::Creamy
                } else {
                    Self::Gray
                }
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MooshroomVariant {
    Red,
    Brown,
}
impl MooshroomVariant {
    pub const ALL: &'static [Self] = &[Self::Red, Self::Brown];
    pub const DEFAULT: Self = Self::Red;
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::Red => 0,
            Self::Brown => 1,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Brown => "brown",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "red" => Some(Self::Red),
            "brown" => Some(Self::Brown),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::Red,
            1 => Self::Brown,
            _ => {
                if id < 0 {
                    Self::Red
                } else {
                    Self::Brown
                }
            }
        }
    }
    #[must_use]
    pub const fn mushroom(&self) -> &'static str {
        match self {
            Self::Red => "minecraft:red_mushroom",
            Self::Brown => "minecraft:brown_mushroom",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PandaGene {
    Normal,
    Lazy,
    Worried,
    Playful,
    Brown,
    Weak,
    Aggressive,
}
impl PandaGene {
    pub const ALL: &'static [Self] = &[
        Self::Normal,
        Self::Lazy,
        Self::Worried,
        Self::Playful,
        Self::Brown,
        Self::Weak,
        Self::Aggressive,
    ];
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::Normal => 0,
            Self::Lazy => 1,
            Self::Worried => 2,
            Self::Playful => 3,
            Self::Brown => 4,
            Self::Weak => 5,
            Self::Aggressive => 6,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Lazy => "lazy",
            Self::Worried => "worried",
            Self::Playful => "playful",
            Self::Brown => "brown",
            Self::Weak => "weak",
            Self::Aggressive => "aggressive",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "normal" => Some(Self::Normal),
            "lazy" => Some(Self::Lazy),
            "worried" => Some(Self::Worried),
            "playful" => Some(Self::Playful),
            "brown" => Some(Self::Brown),
            "weak" => Some(Self::Weak),
            "aggressive" => Some(Self::Aggressive),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::Normal,
            1 => Self::Lazy,
            2 => Self::Worried,
            3 => Self::Playful,
            4 => Self::Brown,
            5 => Self::Weak,
            6 => Self::Aggressive,
            _ => Self::Normal,
        }
    }
    #[must_use]
    pub const fn recessive(&self) -> bool {
        match self {
            Self::Normal => false,
            Self::Lazy => false,
            Self::Worried => false,
            Self::Playful => false,
            Self::Brown => true,
            Self::Weak => true,
            Self::Aggressive => false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParrotVariant {
    RedBlue,
    Blue,
    Green,
    YellowBlue,
    Gray,
}
impl ParrotVariant {
    pub const ALL: &'static [Self] = &[
        Self::RedBlue,
        Self::Blue,
        Self::Green,
        Self::YellowBlue,
        Self::Gray,
    ];
    pub const DEFAULT: Self = Self::RedBlue;
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::RedBlue => 0,
            Self::Blue => 1,
            Self::Green => 2,
            Self::YellowBlue => 3,
            Self::Gray => 4,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::RedBlue => "red_blue",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::YellowBlue => "yellow_blue",
            Self::Gray => "gray",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "red_blue" => Some(Self::RedBlue),
            "blue" => Some(Self::Blue),
            "green" => Some(Self::Green),
            "yellow_blue" => Some(Self::YellowBlue),
            "gray" => Some(Self::Gray),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::RedBlue,
            1 => Self::Blue,
            2 => Self::Green,
            3 => Self::YellowBlue,
            4 => Self::Gray,
            _ => {
                if id < 0 {
                    Self::RedBlue
                } else {
                    Self::Gray
                }
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RabbitVariant {
    Brown,
    White,
    Black,
    WhiteSplotched,
    Gold,
    Salt,
    Evil,
}
impl RabbitVariant {
    pub const ALL: &'static [Self] = &[
        Self::Brown,
        Self::White,
        Self::Black,
        Self::WhiteSplotched,
        Self::Gold,
        Self::Salt,
        Self::Evil,
    ];
    pub const DEFAULT: Self = Self::Brown;
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::Brown => 0,
            Self::White => 1,
            Self::Black => 2,
            Self::WhiteSplotched => 3,
            Self::Gold => 4,
            Self::Salt => 5,
            Self::Evil => 99,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Brown => "brown",
            Self::White => "white",
            Self::Black => "black",
            Self::WhiteSplotched => "white_splotched",
            Self::Gold => "gold",
            Self::Salt => "salt",
            Self::Evil => "evil",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "brown" => Some(Self::Brown),
            "white" => Some(Self::White),
            "black" => Some(Self::Black),
            "white_splotched" => Some(Self::WhiteSplotched),
            "gold" => Some(Self::Gold),
            "salt" => Some(Self::Salt),
            "evil" => Some(Self::Evil),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::Brown,
            1 => Self::White,
            2 => Self::Black,
            3 => Self::WhiteSplotched,
            4 => Self::Gold,
            5 => Self::Salt,
            99 => Self::Evil,
            _ => Self::Brown,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SalmonSize {
    Small,
    Medium,
    Large,
}
impl SalmonSize {
    pub const ALL: &'static [Self] = &[Self::Small, Self::Medium, Self::Large];
    pub const DEFAULT: Self = Self::Medium;
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::Small => 0,
            Self::Medium => 1,
            Self::Large => 2,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "small" => Some(Self::Small),
            "medium" => Some(Self::Medium),
            "large" => Some(Self::Large),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::Small,
            1 => Self::Medium,
            2 => Self::Large,
            _ => {
                if id < 0 {
                    Self::Small
                } else {
                    Self::Large
                }
            }
        }
    }
    #[must_use]
    pub const fn bounding_box_scale(&self) -> f32 {
        match self {
            Self::Small => 0.5,
            Self::Medium => 1.0,
            Self::Large => 1.5,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TropicalFishPattern {
    Kob,
    Flopper,
    Sunstreak,
    Stripey,
    Snooper,
    Glitter,
    Dasher,
    Blockfish,
    Brinely,
    Betty,
    Spotty,
    Clayfish,
}
impl TropicalFishPattern {
    pub const ALL: &'static [Self] = &[
        Self::Kob,
        Self::Flopper,
        Self::Sunstreak,
        Self::Stripey,
        Self::Snooper,
        Self::Glitter,
        Self::Dasher,
        Self::Blockfish,
        Self::Brinely,
        Self::Betty,
        Self::Spotty,
        Self::Clayfish,
    ];
    pub const DEFAULT: Self = Self::Kob;
    #[must_use]
    pub const fn id(&self) -> i32 {
        match self {
            Self::Kob => 0,
            Self::Flopper => 1,
            Self::Sunstreak => 256,
            Self::Stripey => 257,
            Self::Snooper => 512,
            Self::Glitter => 513,
            Self::Dasher => 768,
            Self::Blockfish => 769,
            Self::Brinely => 1024,
            Self::Betty => 1025,
            Self::Spotty => 1280,
            Self::Clayfish => 1281,
        }
    }
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Kob => "kob",
            Self::Flopper => "flopper",
            Self::Sunstreak => "sunstreak",
            Self::Stripey => "stripey",
            Self::Snooper => "snooper",
            Self::Glitter => "glitter",
            Self::Dasher => "dasher",
            Self::Blockfish => "blockfish",
            Self::Brinely => "brinely",
            Self::Betty => "betty",
            Self::Spotty => "spotty",
            Self::Clayfish => "clayfish",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "kob" => Some(Self::Kob),
            "flopper" => Some(Self::Flopper),
            "sunstreak" => Some(Self::Sunstreak),
            "stripey" => Some(Self::Stripey),
            "snooper" => Some(Self::Snooper),
            "glitter" => Some(Self::Glitter),
            "dasher" => Some(Self::Dasher),
            "blockfish" => Some(Self::Blockfish),
            "brinely" => Some(Self::Brinely),
            "betty" => Some(Self::Betty),
            "spotty" => Some(Self::Spotty),
            "clayfish" => Some(Self::Clayfish),
            _ => None,
        }
    }
    #[must_use]
    pub const fn by_id(id: i32) -> Self {
        match id {
            0 => Self::Kob,
            1 => Self::Flopper,
            256 => Self::Sunstreak,
            257 => Self::Stripey,
            512 => Self::Snooper,
            513 => Self::Glitter,
            768 => Self::Dasher,
            769 => Self::Blockfish,
            1024 => Self::Brinely,
            1025 => Self::Betty,
            1280 => Self::Spotty,
            1281 => Self::Clayfish,
            _ => Self::Kob,
        }
    }
}
pub const TROPICAL_FISH_COMMON_VARIANTS: &[(
    TropicalFishPattern,
    crate::dye_color::DyeColor,
    crate::dye_color::DyeColor,
)] = &[
    (
        TropicalFishPattern::Stripey,
        crate::dye_color::DyeColor::Orange,
        crate::dye_color::DyeColor::Gray,
    ),
    (
        TropicalFishPattern::Flopper,
        crate::dye_color::DyeColor::Gray,
        crate::dye_color::DyeColor::Gray,
    ),
    (
        TropicalFishPattern::Flopper,
        crate::dye_color::DyeColor::Gray,
        crate::dye_color::DyeColor::Blue,
    ),
    (
        TropicalFishPattern::Clayfish,
        crate::dye_color::DyeColor::White,
        crate::dye_color::DyeColor::Gray,
    ),
    (
        TropicalFishPattern::Sunstreak,
        crate::dye_color::DyeColor::Blue,
        crate::dye_color::DyeColor::Gray,
    ),
    (
        TropicalFishPattern::Kob,
        crate::dye_color::DyeColor::Orange,
        crate::dye_color::DyeColor::White,
    ),
    (
        TropicalFishPattern::Spotty,
        crate::dye_color::DyeColor::Pink,
        crate::dye_color::DyeColor::LightBlue,
    ),
    (
        TropicalFishPattern::Blockfish,
        crate::dye_color::DyeColor::Purple,
        crate::dye_color::DyeColor::Yellow,
    ),
    (
        TropicalFishPattern::Clayfish,
        crate::dye_color::DyeColor::White,
        crate::dye_color::DyeColor::Red,
    ),
    (
        TropicalFishPattern::Spotty,
        crate::dye_color::DyeColor::White,
        crate::dye_color::DyeColor::Yellow,
    ),
    (
        TropicalFishPattern::Glitter,
        crate::dye_color::DyeColor::White,
        crate::dye_color::DyeColor::Gray,
    ),
    (
        TropicalFishPattern::Clayfish,
        crate::dye_color::DyeColor::White,
        crate::dye_color::DyeColor::Orange,
    ),
    (
        TropicalFishPattern::Dasher,
        crate::dye_color::DyeColor::Cyan,
        crate::dye_color::DyeColor::Pink,
    ),
    (
        TropicalFishPattern::Brinely,
        crate::dye_color::DyeColor::Lime,
        crate::dye_color::DyeColor::LightBlue,
    ),
    (
        TropicalFishPattern::Betty,
        crate::dye_color::DyeColor::Red,
        crate::dye_color::DyeColor::White,
    ),
    (
        TropicalFishPattern::Snooper,
        crate::dye_color::DyeColor::Gray,
        crate::dye_color::DyeColor::Red,
    ),
    (
        TropicalFishPattern::Blockfish,
        crate::dye_color::DyeColor::Red,
        crate::dye_color::DyeColor::White,
    ),
    (
        TropicalFishPattern::Flopper,
        crate::dye_color::DyeColor::White,
        crate::dye_color::DyeColor::Yellow,
    ),
    (
        TropicalFishPattern::Kob,
        crate::dye_color::DyeColor::Red,
        crate::dye_color::DyeColor::White,
    ),
    (
        TropicalFishPattern::Sunstreak,
        crate::dye_color::DyeColor::Gray,
        crate::dye_color::DyeColor::White,
    ),
    (
        TropicalFishPattern::Dasher,
        crate::dye_color::DyeColor::Cyan,
        crate::dye_color::DyeColor::Yellow,
    ),
    (
        TropicalFishPattern::Flopper,
        crate::dye_color::DyeColor::Yellow,
        crate::dye_color::DyeColor::Yellow,
    ),
];
