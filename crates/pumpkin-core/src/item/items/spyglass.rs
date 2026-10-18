use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};

pub struct SpyglassItem;

impl ItemMetadata for SpyglassItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::SPYGLASS.id])
    }
}

impl ItemBehaviour for SpyglassItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        player.world().play_sound(
            Sound::ItemSpyglassUse,
            SoundCategory::Players,
            &player.position(),
        );
        let stack = player.inventory().held_item();
        player
            .living_entity
            .set_active_hand(pumpkin_util::Hand::Right, stack, Self::USE_DURATION);
    }

    fn on_stopped_using(&self, _stack: &pumpkin_data::item_stack::ItemStack, player: &Player) {
        player.world().play_sound(
            Sound::ItemSpyglassStopUsing,
            SoundCategory::Players,
            &player.position(),
        );
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SpyglassItem {
    pub const USE_DURATION: i32 = 1200;
}
