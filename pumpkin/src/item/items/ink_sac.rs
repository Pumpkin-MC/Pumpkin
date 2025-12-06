use std::sync::{Arc, atomic::Ordering};

use pumpkin_data::item::Item;
use pumpkin_world::block::entities::{BlockEntity, sign::Text};

use crate::{
    block::{UseWithItemArgs, registry::BlockActionResult},
    item::{ItemBehaviour, ItemMetadata},
};

pub struct InkSacItem;

impl ItemMetadata for InkSacItem {
    fn ids() -> Box<[u16]> {
        [Item::INK_SAC.id].into()
    }
}

impl ItemBehaviour for InkSacItem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl InkSacItem {
    pub async fn apply_to_sign(
        &self,
        args: &UseWithItemArgs<'_>,
        block_entity: &Arc<dyn BlockEntity>,
        text: &Text,
    ) -> BlockActionResult {
        let changed = text.has_glowing_text.swap(false, Ordering::Relaxed);

        if !changed {
            return BlockActionResult::PassToDefaultBlockAction;
        }

        args.world.update_block_entity(block_entity).await;
        args.world
            .play_block_sound(
                pumpkin_data::sound::Sound::ItemInkSacUse,
                pumpkin_data::sound::SoundCategory::Blocks,
                *args.position,
            )
            .await;
        BlockActionResult::Success
    }
}
