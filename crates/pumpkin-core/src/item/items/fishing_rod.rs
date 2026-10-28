use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::fishing_bobber::FishingBobberEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::Hand;
use pumpkin_util::math::vector3::Vector3;

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

    fn normal_use_with_hand(&self, item: &Item, player: &Player, yaw: f32, pitch: f32, hand: Hand) {
        let world = player.world();
        let bobber_id = player.fishing_bobber.load(Ordering::Relaxed);
        let rod = player.inventory().get_stack_in_hand(hand);

        if bobber_id == -1 {
            // Cast
            world.play_sound_fine(
                Sound::EntityFishingBobberThrow,
                SoundCategory::Neutral,
                &player.position(),
                0.5,
                0.4 / (rand::random::<f32>() * 0.4 + 0.8),
            );

            let time_reduction =
                (crate::enchantment::helper::EnchantmentHelper::modify_fishing_time_reduction(
                    &rod, 0.0,
                ) * 20.0) as i32;
            let player_pos = player.position();
            let y_angle = -yaw.to_radians() - std::f32::consts::PI;
            let x_angle = -pitch.to_radians();
            let y_cos = y_angle.cos();
            let y_sin = y_angle.sin();
            let x_cos = -x_angle.cos();
            let x_sin = x_angle.sin();
            let spawn_pos = Vector3::new(
                player_pos.x - f64::from(y_sin) * 0.3,
                player.eye_position().y,
                player_pos.z - f64::from(y_cos) * 0.3,
            );
            let bobber_entity = Entity::new(world.clone(), spawn_pos, &EntityType::FISHING_BOBBER);
            let bobber = FishingBobberEntity::new_with_rod(bobber_entity, player, time_reduction);
            bobber.entity.pos.store(spawn_pos);

            let dir = Vector3::new(
                f64::from(-y_sin),
                f64::from((-(x_sin / x_cos)).clamp(-5.0, 5.0)),
                f64::from(-y_cos),
            );
            let dist = dir.length();
            let triangle = || 0.5 + 0.010_336_5 * (rand::random::<f64>() - rand::random::<f64>());
            let velocity = dir.multiply(
                0.6 / dist + triangle(),
                0.6 / dist + triangle(),
                0.6 / dist + triangle(),
            );
            bobber.entity.velocity.store(velocity);

            player
                .fishing_bobber
                .store(bobber.entity.entity_id, Ordering::Relaxed);

            let bobber_arc: Arc<FishingBobberEntity> = Arc::new(bobber);
            world.spawn_entity(bobber_arc);

            player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
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
                let slot = if hand == Hand::Left {
                    EquipmentSlot::OFF_HAND
                } else {
                    EquipmentSlot::MAIN_HAND
                };
                player.damage_item_in_slot(&slot, dmg);
            }

            world.play_sound_fine(
                Sound::EntityFishingBobberRetrieve,
                SoundCategory::Neutral,
                &player.position(),
                1.0,
                0.4 / (rand::random::<f32>() * 0.4 + 0.8),
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
