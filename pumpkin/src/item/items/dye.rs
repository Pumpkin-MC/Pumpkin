use std::sync::Arc;

use pumpkin_data::tag;
use pumpkin_util::GameMode;
use pumpkin_world::block::entities::{
    BlockEntity,
    sign::{DyeColor, Text},
};

use crate::{
    block::{UseWithItemArgs, registry::BlockActionResult},
    entity::player::Player,
    item::{ItemBehaviour, ItemMetadata},
};

pub struct DyeItem;

impl ItemMetadata for DyeItem {
    fn ids() -> Box<[u16]> {
        tag::Item::C_DYES.1.to_vec().into_boxed_slice()
    }
}

impl ItemBehaviour for DyeItem {
    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DyeItem {
    pub async fn apply_to_sign(
        &self,
        args: &UseWithItemArgs<'_>,
        block_entity: &Arc<dyn BlockEntity>,
        text: &Text,
        color_name: &str,
    ) -> BlockActionResult {
        let dye_color = DyeColor::from(color_name);

        text.set_color(dye_color);

        args.world.update_block_entity(block_entity).await;
        args.world
            .play_block_sound(
                pumpkin_data::sound::Sound::ItemDyeUse,
                pumpkin_data::sound::SoundCategory::Blocks,
                *args.position,
            )
            .await;
        BlockActionResult::Success
    }
}
