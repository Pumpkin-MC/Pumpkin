use crate::{codec::var_long::VarLong, serial::PacketWrite};

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
