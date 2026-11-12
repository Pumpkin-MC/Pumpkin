use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::{CustomDataImpl, InstrumentImpl};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::instrument::Instrument;
use pumpkin_data::item::Item;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::Hand;

pub struct GoatHornItem;

impl ItemMetadata for GoatHornItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::GOAT_HORN.id])
    }
}

impl ItemBehaviour for GoatHornItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        let stack = player.inventory().get_stack_in_hand(hand);
        let Some(instrument) = stack
            .get_data_component::<InstrumentImpl>()
            .and_then(|inst| Instrument::from_name(&inst.instrument))
            .or_else(|| {
                stack
                    .get_data_component::<CustomDataImpl>()
                    .and_then(|custom| {
                        custom
                            .data
                            .get_string("instrument")
                            .or_else(|| custom.data.get_string("Instrument"))
                    })
                    .and_then(Instrument::from_name)
            })
        else {
            return;
        };

        let duration = i32::try_from(instrument.use_duration_ticks()).unwrap_or(i32::MAX);
        let volume = instrument.range() / 16.0;
        let world = player.world();
        let pos = player.position();
        // Mirrors InstrumentItem#use: cooldown matches the instrument's use duration.
        // The client identifies the default group by the item's namespaced id, so
        // normalize the component value the same way or a datapack group like
        // "minecraft:goat_horn" would get a different server key than the default.
        let cooldown_group = stack
            .get_use_cooldown()
            .and_then(|c| c.cooldown_group.clone())
            .map_or_else(
                || format!("minecraft:{}", item.registry_key),
                |group| {
                    let name = group
                        .rsplit_once(':')
                        .map_or(group.as_str(), |(_, name)| name);
                    format!("minecraft:{name}")
                },
            );
        if player.is_on_cooldown(&cooldown_group) {
            return;
        }

        player.living_entity.set_active_hand(hand, stack, duration);
        world.play_sound_raw(
            instrument.sound() as u16,
            SoundCategory::Records,
            &pos,
            volume,
            1.0,
        );
        world.emit_game_event(GameEvent::InstrumentPlay.name(), pos);
        if duration > 0 {
            player.start_cooldown(cooldown_group, duration);
        }
        player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
    }

    fn get_use_duration(&self) -> i32 {
        Instrument::PonderGoatHorn.use_duration_ticks() as i32
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GoatHornItem {
    pub const USE_DURATION: i32 = Instrument::PonderGoatHorn.use_duration_ticks() as i32;
}
