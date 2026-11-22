/* This file is generated. Do not edit manually. */
use crate::sound::Sound;
#[doc = r" A vanilla wood type, shared by fence gates and signs."]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WoodType {
    pub name: &'static str,
    pub set_type: &'static crate::block_set_type::BlockSetType,
    #[doc = r" The vanilla `SoundType` constant name."]
    pub sound_type: &'static str,
    #[doc = r" The vanilla `SoundType` constant name for hanging signs."]
    pub hanging_sign_sound_type: &'static str,
    pub fence_gate_close: Sound,
    pub fence_gate_open: Sound,
}
impl WoodType {
    pub const OAK: WoodType = WoodType {
        name: "oak",
        set_type: &crate::block_set_type::BlockSetType::OAK,
        sound_type: "WOOD",
        hanging_sign_sound_type: "HANGING_SIGN",
        fence_gate_close: Sound::BlockFenceGateClose,
        fence_gate_open: Sound::BlockFenceGateOpen,
    };
    pub const SPRUCE: WoodType = WoodType {
        name: "spruce",
        set_type: &crate::block_set_type::BlockSetType::SPRUCE,
        sound_type: "WOOD",
        hanging_sign_sound_type: "HANGING_SIGN",
        fence_gate_close: Sound::BlockFenceGateClose,
        fence_gate_open: Sound::BlockFenceGateOpen,
    };
    pub const BIRCH: WoodType = WoodType {
        name: "birch",
        set_type: &crate::block_set_type::BlockSetType::BIRCH,
        sound_type: "WOOD",
        hanging_sign_sound_type: "HANGING_SIGN",
        fence_gate_close: Sound::BlockFenceGateClose,
        fence_gate_open: Sound::BlockFenceGateOpen,
    };
    pub const ACACIA: WoodType = WoodType {
        name: "acacia",
        set_type: &crate::block_set_type::BlockSetType::ACACIA,
        sound_type: "WOOD",
        hanging_sign_sound_type: "HANGING_SIGN",
        fence_gate_close: Sound::BlockFenceGateClose,
        fence_gate_open: Sound::BlockFenceGateOpen,
    };
    pub const CHERRY: WoodType = WoodType {
        name: "cherry",
        set_type: &crate::block_set_type::BlockSetType::CHERRY,
        sound_type: "CHERRY_WOOD",
        hanging_sign_sound_type: "CHERRY_WOOD_HANGING_SIGN",
        fence_gate_close: Sound::BlockCherryWoodFenceGateClose,
        fence_gate_open: Sound::BlockCherryWoodFenceGateOpen,
    };
    pub const JUNGLE: WoodType = WoodType {
        name: "jungle",
        set_type: &crate::block_set_type::BlockSetType::JUNGLE,
        sound_type: "WOOD",
        hanging_sign_sound_type: "HANGING_SIGN",
        fence_gate_close: Sound::BlockFenceGateClose,
        fence_gate_open: Sound::BlockFenceGateOpen,
    };
    pub const DARK_OAK: WoodType = WoodType {
        name: "dark_oak",
        set_type: &crate::block_set_type::BlockSetType::DARK_OAK,
        sound_type: "WOOD",
        hanging_sign_sound_type: "HANGING_SIGN",
        fence_gate_close: Sound::BlockFenceGateClose,
        fence_gate_open: Sound::BlockFenceGateOpen,
    };
    pub const PALE_OAK: WoodType = WoodType {
        name: "pale_oak",
        set_type: &crate::block_set_type::BlockSetType::PALE_OAK,
        sound_type: "WOOD",
        hanging_sign_sound_type: "HANGING_SIGN",
        fence_gate_close: Sound::BlockFenceGateClose,
        fence_gate_open: Sound::BlockFenceGateOpen,
    };
    pub const POPLAR: WoodType = WoodType {
        name: "poplar",
        set_type: &crate::block_set_type::BlockSetType::POPLAR,
        sound_type: "WOOD",
        hanging_sign_sound_type: "HANGING_SIGN",
        fence_gate_close: Sound::BlockFenceGateClose,
        fence_gate_open: Sound::BlockFenceGateOpen,
    };
    pub const CRIMSON: WoodType = WoodType {
        name: "crimson",
        set_type: &crate::block_set_type::BlockSetType::CRIMSON,
        sound_type: "NETHER_WOOD",
        hanging_sign_sound_type: "NETHER_WOOD_HANGING_SIGN",
        fence_gate_close: Sound::BlockNetherWoodFenceGateClose,
        fence_gate_open: Sound::BlockNetherWoodFenceGateOpen,
    };
    pub const WARPED: WoodType = WoodType {
        name: "warped",
        set_type: &crate::block_set_type::BlockSetType::WARPED,
        sound_type: "NETHER_WOOD",
        hanging_sign_sound_type: "NETHER_WOOD_HANGING_SIGN",
        fence_gate_close: Sound::BlockNetherWoodFenceGateClose,
        fence_gate_open: Sound::BlockNetherWoodFenceGateOpen,
    };
    pub const MANGROVE: WoodType = WoodType {
        name: "mangrove",
        set_type: &crate::block_set_type::BlockSetType::MANGROVE,
        sound_type: "WOOD",
        hanging_sign_sound_type: "HANGING_SIGN",
        fence_gate_close: Sound::BlockFenceGateClose,
        fence_gate_open: Sound::BlockFenceGateOpen,
    };
    pub const BAMBOO: WoodType = WoodType {
        name: "bamboo",
        set_type: &crate::block_set_type::BlockSetType::BAMBOO,
        sound_type: "BAMBOO_WOOD",
        hanging_sign_sound_type: "BAMBOO_WOOD_HANGING_SIGN",
        fence_gate_close: Sound::BlockBambooWoodFenceGateClose,
        fence_gate_open: Sound::BlockBambooWoodFenceGateOpen,
    };
}
