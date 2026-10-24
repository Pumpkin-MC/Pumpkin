use std::any::Any;
use std::sync::Arc;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};

pub struct SaddleItem;

impl ItemMetadata for SaddleItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::SADDLE.id])
    }
}

impl ItemBehaviour for SaddleItem {
    fn use_on_entity(&self, item: &mut ItemStack, player: &Player, entity: Arc<dyn EntityBase>) {
        if let Some(mob) = entity.get_mob()
            && mob.can_be_saddled()
            && !mob.is_saddled()
        {
            mob.set_saddled(true);
            let ent = entity.get_entity();
            let sound = if ent.entity_type == &pumpkin_data::entity::EntityType::STRIDER {
                Sound::EntityStriderSaddle
            } else {
                Sound::EntityPigSaddle
            };
            player
                .world()
                .play_sound(sound, SoundCategory::Neutral, &ent.pos.load());
            item.decrement_unless_creative(player.gamemode.load(), 1);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
