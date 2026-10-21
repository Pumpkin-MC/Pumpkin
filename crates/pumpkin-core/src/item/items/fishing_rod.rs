use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::player::Player;
use crate::entity::projectile::fishing_bobber::FishingBobberEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::Hand;

pub struct FishingRodItem;

impl ItemMetadata for FishingRodItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::FISHING_ROD.id])
    }
}

impl ItemBehaviour for FishingRodItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        let world = player.world();
        let bobber_id = player.fishing_bobber.load(Ordering::Relaxed);
        let rod = player.inventory().get_stack_in_hand(hand);

        if bobber_id == -1 {
            // Cast
            world.play_sound(
                Sound::EntityFishingBobberThrow,
                SoundCategory::Neutral,
                &player.position(),
            );

            let time_reduction =
                (crate::enchantment::helper::EnchantmentHelper::modify_fishing_time_reduction(
                    &rod, 0.0,
                ) * 20.0) as i32;
            let bobber_entity = Entity::new(
                world.clone(),
                player.position(),
                &EntityType::FISHING_BOBBER,
            );
            let bobber = FishingBobberEntity::new_with_rod(bobber_entity, player, time_reduction);

            let look_vec = player.living_entity.get_looking_vector();
            bobber
                .entity
                .velocity
                .store(look_vec.multiply(1.5, 1.5, 1.5));

            player
                .fishing_bobber
                .store(bobber.entity.entity_id, Ordering::Relaxed);

            let bobber_arc: Arc<FishingBobberEntity> = Arc::new(bobber);
            world.spawn_entity(bobber_arc);
        } else {
            // Reel in
            let mut dmg = 0;
            if let Some(bobber_base) = world.get_entity_by_id(bobber_id) {
                if let Some(bobber) = bobber_base.cast_any().downcast_ref::<FishingBobberEntity>() {
                    dmg = bobber.reel_in(player, &rod);
                }
                bobber_base.get_entity().remove();
            }
            player.fishing_bobber.store(-1, Ordering::Relaxed);

            if dmg > 0 {
                player.damage_held_item(dmg);
            }

            world.play_sound(
                Sound::EntityFishingBobberRetrieve,
                SoundCategory::Neutral,
                &player.position(),
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
