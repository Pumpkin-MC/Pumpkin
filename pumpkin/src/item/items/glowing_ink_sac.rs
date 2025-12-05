use std::sync::{Arc, atomic::Ordering};

use pumpkin_data::item::Item;
use pumpkin_world::block::entities::{BlockEntity, sign::Text};

use crate::{
    block::{UseWithItemArgs, registry::BlockActionResult},
    item::{ItemBehaviour, ItemMetadata},
};

pub struct GlowingInkSacItem;

impl ItemMetadata for GlowingInkSacItem {
    fn ids() -> Box<[u16]> {
        [Item::GLOW_INK_SAC.id].into()
    }
}

impl ItemBehaviour for GlowingInkSacItem {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl GlowingInkSacItem {
    pub async fn apply_to_sign(
        &self,
        args: &UseWithItemArgs<'_>,
        block_entity: &Arc<dyn BlockEntity>,
        text: &Text,
    ) -> BlockActionResult {
        let changed = !text.has_glowing_text.swap(true, Ordering::Relaxed);

        if !changed {
            return BlockActionResult::PassToDefaultBlockAction;
        }

        args.world.update_block_entity(block_entity).await;
        args.world
            .play_block_sound(
                pumpkin_data::sound::Sound::ItemGlowInkSacUse,
                pumpkin_data::sound::SoundCategory::Blocks,
                *args.position,
            )
            .await;
        BlockActionResult::Success
    }
}
