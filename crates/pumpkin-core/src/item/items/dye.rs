use std::sync::Arc;

use crate::block::entities::{BlockEntity, sign::Text};
use pumpkin_data::data_component_impl::DyeImpl;
use pumpkin_data::tag;

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
        let Some(color) = item.get_data_component::<DyeImpl>().map(|d| d.color.id()) else {
            return;
        };
        if let Some(sheep) = entity
            .cast_any()
            .downcast_ref::<crate::entity::passive::sheep::SheepEntity>()
            && entity.get_entity().is_alive()
            && !sheep.is_sheared()
            && color != sheep.get_color()
        {
            let ent = entity.get_entity();
            ent.world.load().play_sound_expect(
                player,
                pumpkin_data::sound::Sound::ItemDyeUse,
                pumpkin_data::sound::SoundCategory::Players,
                &ent.pos.load(),
            );
            sheep.set_color(color);
            item.decrement_unless_creative(player.gamemode.load(), 1);
        }
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
    ) -> BlockActionResult {
        if !text.has_message(args.player.is_text_filtering_enabled()) {
            return BlockActionResult::PassToDefaultBlockAction;
        }
        let Some(dye) = args.item_stack.get_data_component::<DyeImpl>() else {
            return BlockActionResult::PassToDefaultBlockAction;
        };
        let dye_color = dye.color;
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
