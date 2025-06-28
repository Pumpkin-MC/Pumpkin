use pumpkin_data::packet::clientbound::PLAY_RESPAWN;
use pumpkin_macros::packet;
use pumpkin_util::{math::position::BlockPos, resource_location::ResourceLocation};
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_RESPAWN)]
pub struct CRespawn {
    dimension_type: VarInt,
    dimension_name: ResourceLocation,
    hashed_seed: i64,
    game_mode: u8,
    previous_gamemode: i8,
    debug: bool,
    is_flat: bool,
    death_dimension_name: Option<(ResourceLocation, BlockPos)>,
    portal_cooldown: VarInt,
    sealevel: VarInt,
    data_kept: u8,
}

impl CRespawn {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        dimension_type: VarInt,
        dimension_name: ResourceLocation,
        hashed_seed: i64,
        game_mode: u8,
        previous_gamemode: i8,
        debug: bool,
        is_flat: bool,
        death_dimension_name: Option<(ResourceLocation, BlockPos)>,
        portal_cooldown: VarInt,
        sealevel: VarInt,
        data_kept: u8,
    ) -> Self {
        Self {
            dimension_type,
            dimension_name,
            hashed_seed,
            game_mode,
            previous_gamemode,
            debug,
            is_flat,
            death_dimension_name,
            portal_cooldown,
            sealevel,
            data_kept,
        }
    }
}
