// java/net/minecraft/world/level/block/state/properties/BlockSetType.java

use pumpkin_data::sound::Sound;

#[derive(Copy, Clone)]
pub enum PressurePlateSensitivity {
    Everything,
    Mobs,
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct BlockSetType {
    // pub name: &'static str, unused
    pub can_open_by_hand: bool,
    pub can_open_by_wind_charge: bool,
    pub can_button_be_activated_by_arrows: bool,
    pub pressure_plate_sensitivity: PressurePlateSensitivity,
    // TODO pub soundType: SoundType,
    pub door_close: Sound,               //java SoundEvent
    pub door_open: Sound,                //java SoundEvent
    pub trapdoor_close: Sound,           //java SoundEvent
    pub trapdoor_open: Sound,            //java SoundEvent
    pub pressure_plate_click_off: Sound, //java SoundEvent
    pub pressure_plate_click_on: Sound,  //java SoundEvent
    pub button_click_off: Sound,         //java SoundEvent
    pub button_click_on: Sound,          //java SoundEvent
}

#[allow(dead_code)]
impl BlockSetType {
    pub const IRON: Self = Self {
        // name: "iron",
        can_open_by_hand: false,
        can_open_by_wind_charge: false,
        can_button_be_activated_by_arrows: false,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.IRON,
        door_close: Sound::BlockIronDoorClose,
        door_open: Sound::BlockIronDoorOpen,
        trapdoor_close: Sound::BlockIronTrapdoorClose,
        trapdoor_open: Sound::BlockIronTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockMetalPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockMetalPressurePlateClickOn,
        button_click_off: Sound::BlockStoneButtonClickOff,
        button_click_on: Sound::BlockStoneButtonClickOn,
    };

    pub const COPPER: Self = Self {
        // name: "copper",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: false,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.COPPER,
        door_close: Sound::BlockCopperDoorClose,
        door_open: Sound::BlockCopperDoorOpen,
        trapdoor_close: Sound::BlockCopperTrapdoorClose,
        trapdoor_open: Sound::BlockCopperTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockMetalPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockMetalPressurePlateClickOn,
        button_click_off: Sound::BlockStoneButtonClickOff,
        button_click_on: Sound::BlockStoneButtonClickOn,
    };

    pub const GOLD: Self = Self {
        // name: "gold",
        can_open_by_hand: false,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: false,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.METAL,
        door_close: Sound::BlockIronDoorClose,
        door_open: Sound::BlockIronDoorOpen,
        trapdoor_close: Sound::BlockIronTrapdoorClose,
        trapdoor_open: Sound::BlockIronTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockMetalPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockMetalPressurePlateClickOn,
        button_click_off: Sound::BlockStoneButtonClickOff,
        button_click_on: Sound::BlockStoneButtonClickOn,
    };

    pub const STONE: Self = Self {
        // name: "stone",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: false,
        pressure_plate_sensitivity: PressurePlateSensitivity::Mobs,
        // soundType: SoundType.STONE,
        door_close: Sound::BlockIronDoorClose,
        door_open: Sound::BlockIronDoorOpen,
        trapdoor_close: Sound::BlockIronTrapdoorClose,
        trapdoor_open: Sound::BlockIronTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockStonePressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockStonePressurePlateClickOn,
        button_click_off: Sound::BlockStoneButtonClickOff,
        button_click_on: Sound::BlockStoneButtonClickOn,
    };

    pub const POLISHED_BLACKSTONE: Self = Self {
        // name: "polished_blackstone",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: false,
        pressure_plate_sensitivity: PressurePlateSensitivity::Mobs,
        // soundType: SoundType.STONE,
        door_close: Sound::BlockIronDoorClose,
        door_open: Sound::BlockIronDoorOpen,
        trapdoor_close: Sound::BlockIronTrapdoorClose,
        trapdoor_open: Sound::BlockIronTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockStonePressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockStonePressurePlateClickOn,
        button_click_off: Sound::BlockStoneButtonClickOff,
        button_click_on: Sound::BlockStoneButtonClickOn,
    };

    pub const OAK: Self = Self {
        // name: "oak",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.WOOD,
        door_close: Sound::BlockWoodenDoorClose,
        door_open: Sound::BlockWoodenDoorOpen,
        trapdoor_close: Sound::BlockWoodenTrapdoorClose,
        trapdoor_open: Sound::BlockWoodenTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockWoodenPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockWoodenPressurePlateClickOn,
        button_click_off: Sound::BlockWoodenButtonClickOff,
        button_click_on: Sound::BlockWoodenButtonClickOn,
    };

    pub const SPRUCE: Self = Self {
        // name: "spruce",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.WOOD,
        door_close: Sound::BlockWoodenDoorClose,
        door_open: Sound::BlockWoodenDoorOpen,
        trapdoor_close: Sound::BlockWoodenTrapdoorClose,
        trapdoor_open: Sound::BlockWoodenTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockWoodenPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockWoodenPressurePlateClickOn,
        button_click_off: Sound::BlockWoodenButtonClickOff,
        button_click_on: Sound::BlockWoodenButtonClickOn,
    };

    pub const BIRCH: Self = Self {
        // name: "birch",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.WOOD,
        door_close: Sound::BlockWoodenDoorClose,
        door_open: Sound::BlockWoodenDoorOpen,
        trapdoor_close: Sound::BlockWoodenTrapdoorClose,
        trapdoor_open: Sound::BlockWoodenTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockWoodenPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockWoodenPressurePlateClickOn,
        button_click_off: Sound::BlockWoodenButtonClickOff,
        button_click_on: Sound::BlockWoodenButtonClickOn,
    };

    pub const ACACIA: Self = Self {
        // name: "acacia",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.WOOD,
        door_close: Sound::BlockWoodenDoorClose,
        door_open: Sound::BlockWoodenDoorOpen,
        trapdoor_close: Sound::BlockWoodenTrapdoorClose,
        trapdoor_open: Sound::BlockWoodenTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockWoodenPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockWoodenPressurePlateClickOn,
        button_click_off: Sound::BlockWoodenButtonClickOff,
        button_click_on: Sound::BlockWoodenButtonClickOn,
    };

    pub const CHERRY: Self = Self {
        // name: "cherry",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.CHERRY_WOOD,
        door_close: Sound::BlockCherryWoodDoorClose,
        door_open: Sound::BlockCherryWoodDoorOpen,
        trapdoor_close: Sound::BlockCherryWoodTrapdoorClose,
        trapdoor_open: Sound::BlockCherryWoodTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockCherryWoodPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockCherryWoodPressurePlateClickOn,
        button_click_off: Sound::BlockCherryWoodButtonClickOff,
        button_click_on: Sound::BlockCherryWoodButtonClickOn,
    };

    pub const JUNGLE: Self = Self {
        // name: "jungle",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.WOOD,
        door_close: Sound::BlockWoodenDoorClose,
        door_open: Sound::BlockWoodenDoorOpen,
        trapdoor_close: Sound::BlockWoodenTrapdoorClose,
        trapdoor_open: Sound::BlockWoodenTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockWoodenPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockWoodenPressurePlateClickOn,
        button_click_off: Sound::BlockWoodenButtonClickOff,
        button_click_on: Sound::BlockWoodenButtonClickOn,
    };

    pub const DARK_OAK: Self = Self {
        // name: "dark_oak",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.WOOD,
        door_close: Sound::BlockWoodenDoorClose,
        door_open: Sound::BlockWoodenDoorOpen,
        trapdoor_close: Sound::BlockWoodenTrapdoorClose,
        trapdoor_open: Sound::BlockWoodenTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockWoodenPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockWoodenPressurePlateClickOn,
        button_click_off: Sound::BlockWoodenButtonClickOff,
        button_click_on: Sound::BlockWoodenButtonClickOn,
    };

    pub const PALE_OAK: Self = Self {
        // name: "pale_oak",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.WOOD,
        door_close: Sound::BlockWoodenDoorClose,
        door_open: Sound::BlockWoodenDoorOpen,
        trapdoor_close: Sound::BlockWoodenTrapdoorClose,
        trapdoor_open: Sound::BlockWoodenTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockWoodenPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockWoodenPressurePlateClickOn,
        button_click_off: Sound::BlockWoodenButtonClickOff,
        button_click_on: Sound::BlockWoodenButtonClickOn,
    };

    pub const CRIMSON: Self = Self {
        // name: "crimson",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.NETHER_WOOD,
        door_close: Sound::BlockNetherWoodDoorClose,
        door_open: Sound::BlockNetherWoodDoorOpen,
        trapdoor_close: Sound::BlockNetherWoodTrapdoorClose,
        trapdoor_open: Sound::BlockNetherWoodTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockNetherWoodPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockNetherWoodPressurePlateClickOn,
        button_click_off: Sound::BlockNetherWoodButtonClickOff,
        button_click_on: Sound::BlockNetherWoodButtonClickOn,
    };

    pub const WARPED: Self = Self {
        // name: "warped",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.NETHER_WOOD,
        door_close: Sound::BlockNetherWoodDoorClose,
        door_open: Sound::BlockNetherWoodDoorOpen,
        trapdoor_close: Sound::BlockNetherWoodTrapdoorClose,
        trapdoor_open: Sound::BlockNetherWoodTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockNetherWoodPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockNetherWoodPressurePlateClickOn,
        button_click_off: Sound::BlockNetherWoodButtonClickOff,
        button_click_on: Sound::BlockNetherWoodButtonClickOn,
    };

    pub const MANGROVE: Self = Self {
        // name: "mangrove",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.WOOD,
        door_close: Sound::BlockWoodenDoorClose,
        door_open: Sound::BlockWoodenDoorOpen,
        trapdoor_close: Sound::BlockWoodenTrapdoorClose,
        trapdoor_open: Sound::BlockWoodenTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockWoodenPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockWoodenPressurePlateClickOn,
        button_click_off: Sound::BlockWoodenButtonClickOff,
        button_click_on: Sound::BlockWoodenButtonClickOn,
    };

    pub const BAMBOO: Self = Self {
        // name: "bamboo",
        can_open_by_hand: true,
        can_open_by_wind_charge: true,
        can_button_be_activated_by_arrows: true,
        pressure_plate_sensitivity: PressurePlateSensitivity::Everything,
        // soundType: SoundType.BAMBOO_WOOD,
        door_close: Sound::BlockBambooWoodDoorClose,
        door_open: Sound::BlockBambooWoodDoorOpen,
        trapdoor_close: Sound::BlockBambooWoodTrapdoorClose,
        trapdoor_open: Sound::BlockBambooWoodTrapdoorOpen,
        pressure_plate_click_off: Sound::BlockBambooWoodPressurePlateClickOff,
        pressure_plate_click_on: Sound::BlockBambooWoodPressurePlateClickOn,
        button_click_off: Sound::BlockBambooWoodButtonClickOff,
        button_click_on: Sound::BlockBambooWoodButtonClickOn,
    };
}
