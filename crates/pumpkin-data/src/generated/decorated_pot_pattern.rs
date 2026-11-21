/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
pub enum DecoratedPotPattern {
    Angler,
    Archer,
    ArmsUp,
    Blade,
    Brewer,
    Burn,
    Danger,
    Explorer,
    Flow,
    Friend,
    Guster,
    Heart,
    Heartbreak,
    Howl,
    Miner,
    Mourner,
    Plenty,
    Prize,
    Scrape,
    Sheaf,
    Shelter,
    Skull,
    Snort,
}
impl DecoratedPotPattern {
    #[doc = "Returns the decorated pot pattern from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:angler" | "angler" => Some(Self::Angler),
            "minecraft:archer" | "archer" => Some(Self::Archer),
            "minecraft:arms_up" | "arms_up" => Some(Self::ArmsUp),
            "minecraft:blade" | "blade" => Some(Self::Blade),
            "minecraft:brewer" | "brewer" => Some(Self::Brewer),
            "minecraft:burn" | "burn" => Some(Self::Burn),
            "minecraft:danger" | "danger" => Some(Self::Danger),
            "minecraft:explorer" | "explorer" => Some(Self::Explorer),
            "minecraft:flow" | "flow" => Some(Self::Flow),
            "minecraft:friend" | "friend" => Some(Self::Friend),
            "minecraft:guster" | "guster" => Some(Self::Guster),
            "minecraft:heart" | "heart" => Some(Self::Heart),
            "minecraft:heartbreak" | "heartbreak" => Some(Self::Heartbreak),
            "minecraft:howl" | "howl" => Some(Self::Howl),
            "minecraft:miner" | "miner" => Some(Self::Miner),
            "minecraft:mourner" | "mourner" => Some(Self::Mourner),
            "minecraft:plenty" | "plenty" => Some(Self::Plenty),
            "minecraft:prize" | "prize" => Some(Self::Prize),
            "minecraft:scrape" | "scrape" => Some(Self::Scrape),
            "minecraft:sheaf" | "sheaf" => Some(Self::Sheaf),
            "minecraft:shelter" | "shelter" => Some(Self::Shelter),
            "minecraft:skull" | "skull" => Some(Self::Skull),
            "minecraft:snort" | "snort" => Some(Self::Snort),
            _ => None,
        }
    }
    #[doc = "Returns the numeric ID of the decorated pot pattern in the synced registry."]
    #[must_use]
    pub const fn id(&self) -> u32 {
        *self as u32
    }
    #[doc = "Returns the bare string name of the decorated pot pattern."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Angler => "angler",
            Self::Archer => "archer",
            Self::ArmsUp => "arms_up",
            Self::Blade => "blade",
            Self::Brewer => "brewer",
            Self::Burn => "burn",
            Self::Danger => "danger",
            Self::Explorer => "explorer",
            Self::Flow => "flow",
            Self::Friend => "friend",
            Self::Guster => "guster",
            Self::Heart => "heart",
            Self::Heartbreak => "heartbreak",
            Self::Howl => "howl",
            Self::Miner => "miner",
            Self::Mourner => "mourner",
            Self::Plenty => "plenty",
            Self::Prize => "prize",
            Self::Scrape => "scrape",
            Self::Sheaf => "sheaf",
            Self::Shelter => "shelter",
            Self::Skull => "skull",
            Self::Snort => "snort",
        }
    }
    #[doc = "Returns the fully-qualified asset id of the decorated pot pattern."]
    #[must_use]
    pub const fn asset_id(&self) -> &'static str {
        match self {
            Self::Angler => "minecraft:angler_pottery_pattern",
            Self::Archer => "minecraft:archer_pottery_pattern",
            Self::ArmsUp => "minecraft:arms_up_pottery_pattern",
            Self::Blade => "minecraft:blade_pottery_pattern",
            Self::Brewer => "minecraft:brewer_pottery_pattern",
            Self::Burn => "minecraft:burn_pottery_pattern",
            Self::Danger => "minecraft:danger_pottery_pattern",
            Self::Explorer => "minecraft:explorer_pottery_pattern",
            Self::Flow => "minecraft:flow_pottery_pattern",
            Self::Friend => "minecraft:friend_pottery_pattern",
            Self::Guster => "minecraft:guster_pottery_pattern",
            Self::Heart => "minecraft:heart_pottery_pattern",
            Self::Heartbreak => "minecraft:heartbreak_pottery_pattern",
            Self::Howl => "minecraft:howl_pottery_pattern",
            Self::Miner => "minecraft:miner_pottery_pattern",
            Self::Mourner => "minecraft:mourner_pottery_pattern",
            Self::Plenty => "minecraft:plenty_pottery_pattern",
            Self::Prize => "minecraft:prize_pottery_pattern",
            Self::Scrape => "minecraft:scrape_pottery_pattern",
            Self::Sheaf => "minecraft:sheaf_pottery_pattern",
            Self::Shelter => "minecraft:shelter_pottery_pattern",
            Self::Skull => "minecraft:skull_pottery_pattern",
            Self::Snort => "minecraft:snort_pottery_pattern",
        }
    }
    #[doc = "Returns all vanilla decorated pot patterns."]
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Angler,
            Self::Archer,
            Self::ArmsUp,
            Self::Blade,
            Self::Brewer,
            Self::Burn,
            Self::Danger,
            Self::Explorer,
            Self::Flow,
            Self::Friend,
            Self::Guster,
            Self::Heart,
            Self::Heartbreak,
            Self::Howl,
            Self::Miner,
            Self::Mourner,
            Self::Plenty,
            Self::Prize,
            Self::Scrape,
            Self::Sheaf,
            Self::Shelter,
            Self::Skull,
            Self::Snort,
        ]
    }
}
