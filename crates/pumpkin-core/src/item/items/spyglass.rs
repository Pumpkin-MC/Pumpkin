use std::any::Any;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::Hand;

pub struct SpyglassItem;

impl ItemMetadata for SpyglassItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::SPYGLASS.id])
    }
}

impl ItemBehaviour for SpyglassItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        self.normal_use_with_hand(item, player, 0.0, 0.0, Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        player.world().play_sound_fine(
            Sound::ItemSpyglassUse,
            SoundCategory::Players,
            &player.position(),
            1.0,
            1.0,
        );
        player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
        let looking_at = player
            .get_target_entity(&player.world(), 128.0)
            .map(|e| format!("minecraft:{}", e.get_entity().entity_type.resource_name));
        player.trigger_advancement(
            crate::entity::player::advancement::trigger::AdvancementTrigger::UsingItem {
                item_id: "minecraft:spyglass".to_string(),
                looking_at,
            },
        );
        let stack = player.inventory().get_stack_in_hand(hand);
        player
            .living_entity
            .set_active_hand(hand, stack, Self::USE_DURATION);
    }

    fn on_stopped_using(&self, _stack: &pumpkin_data::item_stack::ItemStack, player: &Player) {
        player.world().play_sound_fine(
            Sound::ItemSpyglassStopUsing,
            SoundCategory::Players,
            &player.position(),
            1.0,
            1.0,
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
