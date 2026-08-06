use std::any::Any;
use std::future::Future;
use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::Hand;

pub struct SpyglassItem;

impl ItemMetadata for SpyglassItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::SPYGLASS.id])
    }
}

impl SpyglassItem {
    /// Vanilla `SpyglassItem.USE_DURATION`.
    const USE_DURATION: i32 = 1200;

    /// Vanilla `SpyglassItem#use`: `player.playSound(SPYGLASS_USE, 1.0F, 1.0F)`,
    /// which resolves to `Player#playSound` -> `level().playSound(this, ...)`,
    /// i.e. audible to everyone nearby except the acting player.
    fn play_use_sound(player: &Player) {
        player.world().play_sound_expect(
            player,
            Sound::ItemSpyglassUse,
            SoundCategory::Players,
            &player.position(),
        );
    }

    /// Vanilla `SpyglassItem#stopUsing`: `entity.playSound(SPYGLASS_STOP_USING, 1.0F, 1.0F)`.
    fn play_stop_using_sound(player: &Player) {
        player.world().play_sound_expect(
            player,
            Sound::ItemSpyglassStopUsing,
            SoundCategory::Players,
            &player.position(),
        );
    }
}

impl ItemBehaviour for SpyglassItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            Self::play_use_sound(player);
            player
                .increment_stat(StatisticCategory::Used, Item::SPYGLASS.id as i32, 1)
                .await;

            let held = player.inventory().held_item();
            let stack = held.lock().await.clone();
            player
                .living_entity
                .set_active_hand(Hand::Right, stack, Self::USE_DURATION)
                .await;
        })
    }

    fn on_stopped_using<'a>(
        &'a self,
        _stack: &'a ItemStack,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            Self::play_stop_using_sound(player);
        })
    }

    /// Vanilla `SpyglassItem#finishUsingItem`: same `stopUsing` call as
    /// `releaseUsing`. `on_use_tick` runs before the tick loop's completion
    /// check with the pre-decrement remaining ticks, so `0` here is the
    /// natural-completion tick.
    fn on_use_tick<'a>(
        &'a self,
        _stack: &'a ItemStack,
        player: &'a Player,
        remaining_use_ticks: i32,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if remaining_use_ticks == 0 {
                Self::play_stop_using_sound(player);
            }
        })
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::SpyglassItem;
    use crate::item::{ItemBehaviour, ItemMetadata};
    use pumpkin_data::item::Item;

    #[test]
    fn use_duration_matches_vanilla() {
        assert_eq!(SpyglassItem.get_use_duration(), 1200);
    }

    #[test]
    fn ids_resolve_to_spyglass() {
        assert_eq!(&*SpyglassItem::ids(), [Item::SPYGLASS.id]);
    }
}
