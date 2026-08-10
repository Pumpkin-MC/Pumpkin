use std::io::{Error, Write};

use crate::{codec::var_long::VarLong, serial::PacketWrite};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BuildPlatform {
    Unknown = -1,
    Google = 1,
    Ios = 2,
    Osx = 3,
    Amazon = 4,
    GearVr = 5,
    Hololens = 6,
    Uwp = 7,
    Win32 = 8,
    Dedicated = 9,
    TvOs = 10,
    Sony = 11,
    Nx = 12,
    Xbox = 13,
    WindowsPhone = 14,
    Linux = 15,
}

impl PacketWrite for BuildPlatform {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as i32).write(writer)
    }
}

#[derive(Default, Clone, PacketWrite)]
pub struct AbilityLayer {
    pub serialized_layer: u16,
    pub abilities_set: u32,
    pub ability_value: u32,
    pub fly_speed: f32,
    pub vertical_fly_speed: f32,
    pub walk_speed: f32,
}

#[derive(Default, Clone, PacketWrite)]
pub struct EntityLink {
    pub ridden_unique_id: VarLong,
    pub rider_unique_id: VarLong,
    pub link_type: u8,
    pub immediate: bool,
    pub rider_initiated: bool,
    pub vehicle_angular_velocity: f32,
}
