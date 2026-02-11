/* This file is generated. Do not edit manually. */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum JukeboxSong {
    Id11,
    Id13,
    Id5,
    Blocks,
    Cat,
    Chirp,
    Creator,
    CreatorMusicBox,
    Far,
    LavaChicken,
    Mall,
    Mellohi,
    Otherside,
    Pigstep,
    Precipice,
    Relic,
    Stal,
    Strad,
    Tears,
    Wait,
    Ward,
}
impl JukeboxSong {
    #[doc = r" Returns the JukeboxSong from the string name (e.g., 'pigstep')."]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "11" => Some(Self::Id11),
            "13" => Some(Self::Id13),
            "5" => Some(Self::Id5),
            "blocks" => Some(Self::Blocks),
            "cat" => Some(Self::Cat),
            "chirp" => Some(Self::Chirp),
            "creator" => Some(Self::Creator),
            "creator_music_box" => Some(Self::CreatorMusicBox),
            "far" => Some(Self::Far),
            "lava_chicken" => Some(Self::LavaChicken),
            "mall" => Some(Self::Mall),
            "mellohi" => Some(Self::Mellohi),
            "otherside" => Some(Self::Otherside),
            "pigstep" => Some(Self::Pigstep),
            "precipice" => Some(Self::Precipice),
            "relic" => Some(Self::Relic),
            "stal" => Some(Self::Stal),
            "strad" => Some(Self::Strad),
            "tears" => Some(Self::Tears),
            "wait" => Some(Self::Wait),
            "ward" => Some(Self::Ward),
            _ => None,
        }
    }
    #[doc = r" Returns the string name of the song."]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Id11 => "11",
            Self::Id13 => "13",
            Self::Id5 => "5",
            Self::Blocks => "blocks",
            Self::Cat => "cat",
            Self::Chirp => "chirp",
            Self::Creator => "creator",
            Self::CreatorMusicBox => "creator_music_box",
            Self::Far => "far",
            Self::LavaChicken => "lava_chicken",
            Self::Mall => "mall",
            Self::Mellohi => "mellohi",
            Self::Otherside => "otherside",
            Self::Pigstep => "pigstep",
            Self::Precipice => "precipice",
            Self::Relic => "relic",
            Self::Stal => "stal",
            Self::Strad => "strad",
            Self::Tears => "tears",
            Self::Wait => "wait",
            Self::Ward => "ward",
        }
    }
    #[doc = r" Returns the numeric ID associated with the song."]
    pub const fn get_id(&self) -> u32 {
        match self {
            Self::Id11 => 0u32,
            Self::Id13 => 1u32,
            Self::Id5 => 2u32,
            Self::Blocks => 3u32,
            Self::Cat => 4u32,
            Self::Chirp => 5u32,
            Self::Creator => 6u32,
            Self::CreatorMusicBox => 7u32,
            Self::Far => 8u32,
            Self::LavaChicken => 9u32,
            Self::Mall => 10u32,
            Self::Mellohi => 11u32,
            Self::Otherside => 12u32,
            Self::Pigstep => 13u32,
            Self::Precipice => 14u32,
            Self::Relic => 15u32,
            Self::Stal => 16u32,
            Self::Strad => 17u32,
            Self::Tears => 18u32,
            Self::Wait => 19u32,
            Self::Ward => 20u32,
        }
    }
    #[doc = r" Returns the comparator output value (0-15) for this song."]
    pub const fn comparator_output(&self) -> u8 {
        match self {
            Self::Id11 => 11u8,
            Self::Id13 => 1u8,
            Self::Id5 => 15u8,
            Self::Blocks => 3u8,
            Self::Cat => 2u8,
            Self::Chirp => 4u8,
            Self::Creator => 12u8,
            Self::CreatorMusicBox => 11u8,
            Self::Far => 5u8,
            Self::LavaChicken => 9u8,
            Self::Mall => 6u8,
            Self::Mellohi => 7u8,
            Self::Otherside => 14u8,
            Self::Pigstep => 13u8,
            Self::Precipice => 13u8,
            Self::Relic => 14u8,
            Self::Stal => 8u8,
            Self::Strad => 9u8,
            Self::Tears => 10u8,
            Self::Wait => 12u8,
            Self::Ward => 10u8,
        }
    }
    #[doc = r" Returns the song length in seconds."]
    pub const fn length_in_seconds(&self) -> u32 {
        match self {
            Self::Id11 => 71u32,
            Self::Id13 => 178u32,
            Self::Id5 => 178u32,
            Self::Blocks => 345u32,
            Self::Cat => 185u32,
            Self::Chirp => 185u32,
            Self::Creator => 176u32,
            Self::CreatorMusicBox => 73u32,
            Self::Far => 174u32,
            Self::LavaChicken => 134u32,
            Self::Mall => 197u32,
            Self::Mellohi => 96u32,
            Self::Otherside => 195u32,
            Self::Pigstep => 149u32,
            Self::Precipice => 299u32,
            Self::Relic => 218u32,
            Self::Stal => 150u32,
            Self::Strad => 188u32,
            Self::Tears => 175u32,
            Self::Wait => 238u32,
            Self::Ward => 251u32,
        }
    }
    #[doc = r" Returns the song length in ticks (20 ticks per second)."]
    pub const fn length_in_ticks(&self) -> u64 {
        self.length_in_seconds() as u64 * 20
    }
}
