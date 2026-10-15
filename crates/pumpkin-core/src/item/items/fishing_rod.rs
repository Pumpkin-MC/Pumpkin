use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::player::Player;
use crate::entity::projectile::fishing_bobber::FishingBobberEntity;
use crate::entity::Entity;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::EquipmentSlot;
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

        let (held_stack, eq_slot) = if hand == Hand::Right {
            (player.inventory().held_item(), EquipmentSlot::MAIN_HAND)
        } else {
            (player.inventory().off_hand_item(), EquipmentSlot::OFF_HAND)
        };

        if bobber_id == -1 {
            // Cast
            let pitch_rand = 0.4 / (rand::random_range(0.0f32..0.4f32) + 0.8);
            world.play_sound_fine(
                Sound::EntityFishingBobberThrow,
                SoundCategory::Neutral,
                &player.position(),
                0.5,
                pitch_rand,
            );

            let lure_level = held_stack.get_enchantment_level(&pumpkin_data::Enchantment::LURE);
            let luck_level =
                held_stack.get_enchantment_level(&pumpkin_data::Enchantment::LUCK_OF_THE_SEA);
            let lure_speed = lure_level * 100;
            let luck = luck_level;

            let bobber_entity = Entity::new(
                world.clone(),
                player.position(),
                &EntityType::FISHING_BOBBER,
            );
            let bobber = FishingBobberEntity::new(bobber_entity, player, luck, lure_speed);

            player
                .fishing_bobber
                .store(bobber.entity.entity_id, Ordering::Relaxed);

            let bobber_arc: Arc<FishingBobberEntity> = Arc::new(bobber);
            world.spawn_entity(bobber_arc);

            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Used,
                Item::FISHING_ROD.id as i32,
                1,
            );
        } else {
            // Reel in
            let mut rod_damage = 0;
            if let Some(bobber_base) = world.get_entity_by_id(bobber_id) {
                if let Some(bobber) = bobber_base.cast_any().downcast_ref::<FishingBobberEntity>() {
                    rod_damage = bobber.reel_in(player);
                }
                bobber_base.get_entity().remove();
            }
            player.fishing_bobber.store(-1, Ordering::Relaxed);

            if rod_damage > 0 {
                player.damage_item_in_slot(&eq_slot, rod_damage);
            }

            let pitch_rand = 0.4 / (rand::random_range(0.0f32..0.4f32) + 0.8);
            world.play_sound_fine(
                Sound::EntityFishingBobberRetrieve,
                SoundCategory::Neutral,
                &player.position(),
                1.0,
                pitch_rand,
            );
        }
    }

    fn normal_use(&self, item: &Item, player: &Player) {
        self.normal_use_with_hand(item, player, 0.0, 0.0, Hand::Right);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
