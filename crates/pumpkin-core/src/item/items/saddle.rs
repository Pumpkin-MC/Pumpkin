use std::any::Any;
use std::sync::Arc;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::game_event::GameEvent;
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
        let ent = entity.get_entity();
        if ent.is_alive()
            && let Some(mob) = entity.get_mob()
            && mob.can_be_saddled()
            && !mob.is_saddled()
            && !mob
                .as_ageable()
                .is_some_and(crate::entity::ageable::AgeableMob::is_baby)
        {
            mob.set_saddled(true);
            let sound = if ent.entity_type == &pumpkin_data::entity::EntityType::STRIDER {
                Sound::EntityStriderSaddle
            } else {
                Sound::EntityPigSaddle
            };
            let pos = ent.pos.load();
            let world = player.world();
            world.play_sound(sound, SoundCategory::Neutral, &pos);
            world.emit_game_event(GameEvent::Equip.name(), pos);
            item.decrement_unless_creative(player.gamemode.load(), 1);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
