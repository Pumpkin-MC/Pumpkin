use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::text::TextComponent;

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

        if world.dimension.has_fixed_time || !world.dimension.has_skylight {
            let msg = TextComponent::text(
                "§cKim đồng hồ quay loạn xạ không xác định được thời gian!",
            );
            player.send_system_message_raw(&msg, true);
        } else {
            let time_of_day = world
                .level_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .time_of_day;
            let world_time = time_of_day.rem_euclid(24000);
            let hours = ((world_time / 1000) + 6) % 24;
            let minutes = ((world_time % 1000) * 60) / 1000;
            let day = (time_of_day / 24000).max(0) + 1;
            let msg = TextComponent::text(format!(
                "§eNgày {day} - Thời gian: {hours:02}:{minutes:02}"
            ));
            player.send_system_message_raw(&msg, true);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
