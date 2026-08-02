use std::pin::Pin;
use std::sync::Arc;

use crate::block::entities::{
    BlockEntity,
    sign::{DyeColor, Text},
};
use crate::entity::EntityBase;
use crate::entity::passive::sheep::SheepEntity;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag;
use pumpkin_util::GameMode;

use crate::{
    block::{UseWithItemArgs, registry::BlockActionResult},
    entity::player::Player,
    item::{ItemBehaviour, ItemMetadata},
};

pub struct DyeItem;

impl ItemMetadata for DyeItem {
    fn ids() -> Box<[u16]> {
        tag::Item::C_DYES.1.into()
    }
}

impl ItemBehaviour for DyeItem {
    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }

    // Vanilla DyeItem.java#interactLivingEntity: only sheep, only alive, only unsheared,
    // no-op (no consume) if already the target color.
    fn use_on_entity<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        entity: Arc<dyn EntityBase>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let Some(sheep) = entity.cast_any().downcast_ref::<SheepEntity>() else {
                return;
            };
            if sheep.is_sheared() {
                return;
            }
            let Some(color_name) = item.item.registry_key.strip_suffix("_dye") else {
                return;
            };
            let new_color = DyeColor::from(color_name) as u8;
            if new_color == sheep.get_color() {
                return;
            }

            let world = entity.get_entity().world.load();
            world.play_sound(
                Sound::ItemDyeUse,
                SoundCategory::Players,
                &entity.get_entity().pos.load(),
            );
            sheep.set_color(new_color);
            item.decrement_unless_creative(player.gamemode.load(), 1);
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DyeItem {
    pub fn apply_to_sign(
        &self,
        args: &UseWithItemArgs<'_>,
        block_entity: &Arc<dyn BlockEntity>,
        text: &Text,
        color_name: &str,
    ) -> BlockActionResult {
        let dye_color = DyeColor::from(color_name);

        text.set_color(dye_color);

        args.world.update_block_entity(block_entity);
        args.world.play_block_sound(
            pumpkin_data::sound::Sound::ItemDyeUse,
            pumpkin_data::sound::SoundCategory::Blocks,
            *args.position,
        );
        BlockActionResult::Success
    }
}
