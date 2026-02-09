/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy)]
pub enum SoundCategory {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
    Ui,
}
impl SoundCategory {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "master" => Some(Self::Master),
            "music" => Some(Self::Music),
            "records" => Some(Self::Records),
            "weather" => Some(Self::Weather),
            "blocks" => Some(Self::Blocks),
            "hostile" => Some(Self::Hostile),
            "neutral" => Some(Self::Neutral),
            "players" => Some(Self::Players),
            "ambient" => Some(Self::Ambient),
            "voice" => Some(Self::Voice),
            "ui" => Some(Self::Ui),
            _ => None,
        }
    }
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Master => "master",
            Self::Music => "music",
            Self::Records => "records",
            Self::Weather => "weather",
            Self::Blocks => "blocks",
            Self::Hostile => "hostile",
            Self::Neutral => "neutral",
            Self::Players => "players",
            Self::Ambient => "ambient",
            Self::Voice => "voice",
            Self::Ui => "ui",
        }
    }
}
