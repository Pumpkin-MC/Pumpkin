use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};

pub struct ClockItem;

impl ItemMetadata for ClockItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::CLOCK.id])
    }
}

impl ItemBehaviour for ClockItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        let world = player.world();
        world.play_sound(
            Sound::UiButtonClick,
            SoundCategory::Players,
            &player.position(),
        );
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
