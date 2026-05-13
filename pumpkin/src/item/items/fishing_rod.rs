use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::player::Player;
use crate::entity::projectile::fishing_bobber::FishingBobberEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};

pub struct FishingRodItem;

impl ItemMetadata for FishingRodItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::FISHING_ROD.id])
    }
}

impl ItemBehaviour for FishingRodItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let bobber_id = player.fishing_bobber.load(Ordering::Relaxed);

            if bobber_id == -1 {
                // Cast
                world
                    .play_sound(
                        Sound::EntityFishingBobberThrow,
                        SoundCategory::Neutral,
                        &player.position(),
                    )
                    .await;

                let bobber_entity = Entity::new(
                    world.clone(),
                    player.position(),
                    &EntityType::FISHING_BOBBER,
                );
                let bobber = FishingBobberEntity::new(bobber_entity, player);

                let look_vec = player.living_entity.get_looking_vector();
                bobber
                    .entity
                    .velocity
                    .store(look_vec.multiply(1.5, 1.5, 1.5));

                player
                    .fishing_bobber
                    .store(bobber.entity.entity_id, Ordering::Relaxed);

                let bobber_arc: Arc<FishingBobberEntity> = Arc::new(bobber);
                world.spawn_entity(bobber_arc).await;
            } else {
                // Reel in
                if let Some(bobber_base) = world.get_entity_by_id(bobber_id) {
                    if let Some(bobber) =
                        bobber_base.cast_any().downcast_ref::<FishingBobberEntity>()
                    {
                        let _result = bobber.reel_in(player).await;
                        // TODO: give items
                    }
                    bobber_base.get_entity().remove().await;
                }
                player.fishing_bobber.store(-1, Ordering::Relaxed);

                world
                    .play_sound(
                        Sound::EntityFishingBobberRetrieve,
                        SoundCategory::Neutral,
                        &player.position(),
                    )
                    .await;
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
