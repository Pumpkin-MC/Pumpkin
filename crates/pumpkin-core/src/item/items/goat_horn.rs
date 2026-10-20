use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::{CustomDataImpl, InstrumentImpl, UseCooldownImpl};
use pumpkin_data::instrument::Instrument;
use pumpkin_data::item::Item;
use pumpkin_data::sound::SoundCategory;
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
        let instrument = stack
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
            .unwrap_or(Instrument::PonderGoatHorn);

        let duration = instrument.use_duration_ticks() as i32;
        let volume = instrument.range() / 16.0;
        let world = player.world();
        let pos = player.position();

        player
            .living_entity
            .set_active_hand(hand, stack.clone(), duration);
        world.play_sound_raw(
            instrument.sound() as u16,
            SoundCategory::Records,
            &pos,
            volume,
            1.0,
        );
        world.emit_game_event("instrument_play", pos);
        let (cooldown_key, cooldown_ticks) =
            stack.get_data_component::<UseCooldownImpl>().map_or_else(
                || (item.registry_key.to_string(), duration),
                |cooldown| {
                    (
                        cooldown
                            .cooldown_group
                            .clone()
                            .unwrap_or_else(|| item.registry_key.to_string()),
                        if cooldown.seconds > 0.0 {
                            (cooldown.seconds * 20.0).floor() as i32
                        } else {
                            duration
                        },
                    )
                },
            );
        if cooldown_ticks > 0 {
            player.start_cooldown(cooldown_key, cooldown_ticks);
        }
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
