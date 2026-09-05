use std::sync::Arc;

use crate::block::entities::{
    BlockEntity,
    sign::{DyeColor, Text},
};
use pumpkin_data::tag;
use pumpkin_util::GameMode;

use crate::{
    block::{UseWithItemArgs, registry::BlockActionResult},
    entity::player::Player,
    item::{ItemBehaviour, ItemMetadata},
};

use crate::entity::EntityBase;
use pumpkin_data::item_stack::ItemStack;

pub struct DyeItem;

impl ItemMetadata for DyeItem {
    fn ids() -> Box<[u16]> {
        tag::Item::C_DYES.1.into()
    }
}

impl ItemBehaviour for DyeItem {
    fn use_on_entity(&self, item: &mut ItemStack, player: &Player, entity: Arc<dyn EntityBase>) {
        if let Some(sheep) = entity
            .cast_any()
            .downcast_ref::<crate::entity::passive::sheep::SheepEntity>()
            && let Some(color) =
                crate::entity::passive::animal::get_dye_color_from_item(item.get_item())
            && !sheep.is_sheared()
            && color != sheep.get_color()
        {
            sheep.set_color(color);
            let ent = entity.get_entity();
            let world = ent.world.load();
            world.play_sound(
                pumpkin_data::sound::Sound::ItemDyeUse,
                pumpkin_data::sound::SoundCategory::Players,
                &ent.pos.load(),
            );
            item.decrement_unless_creative(player.gamemode.load(), 1);
        }
    }

    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
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
        let dye_color = DyeColor::by_name(color_name).unwrap_or_default();
        if text.get_color() == dye_color {
            return BlockActionResult::PassToDefaultBlockAction;
        }

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
