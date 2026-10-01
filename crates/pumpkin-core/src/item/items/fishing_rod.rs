use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::enchantment::EnchantmentHelper;
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::fishing_bobber::FishingBobberEntity;
use crate::item::{ItemBehaviour, ItemMetadata, hand_holding};
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::Hand;

pub struct FishingRodItem;

impl ItemMetadata for FishingRodItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::FISHING_ROD.id])
    }
}

/// Vanilla `FishingRodItem.use` randomises the bobber sound pitch as
/// `0.4 / (random * 0.4 + 0.8)`, so the throw and retrieve sounds vary around 0.4.
fn bobber_sound_pitch(random: f32) -> f32 {
    0.4 / (random * 0.4 + 0.8)
}

impl ItemBehaviour for FishingRodItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        let hand = hand_holding(player, item);
        self.normal_use_with_hand(item, player, yaw, pitch, hand);
    }

    fn normal_use_with_rotation(&self, item: &Item, player: &Player, yaw: f32, pitch: f32) {
        let hand = hand_holding(player, item);
        self.normal_use_with_hand(item, player, yaw, pitch, hand);
    }

    /// Vanilla `FishingRodItem.use`: cast when no bobber is out, otherwise reel the current one in.
    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        yaw: f32,
        pitch: f32,
        hand: Hand,
    ) {
        let world = player.world();
        let inventory = player.inventory();
        let used_item = inventory.get_stack_in_hand(hand);
        let equipment_slot = EquipmentSlot::from_hand(hand);

        let bobber_id = player.fishing_bobber.load(Ordering::Relaxed);

        if bobber_id == -1 {
            // Cast
            let sound_pitch = bobber_sound_pitch(rand::random());
            world.play_sound_fine(
                Sound::EntityFishingBobberThrow,
                SoundCategory::Neutral,
                &player.position(),
                0.5,
                sound_pitch,
            );

            let wait_time_reduction =
                (EnchantmentHelper::modify_fishing_time_reduction(&used_item, 0.0) * 20.0) as i32;
            let luck_bonus = EnchantmentHelper::modify_fishing_luck_bonus(&used_item, 0.0) as i32;

            let bobber_entity = Entity::new(
                world.clone(),
                player.position(),
                &EntityType::FISHING_BOBBER,
            );
            let bobber = FishingBobberEntity::new(
                bobber_entity,
                player,
                yaw,
                pitch,
                luck_bonus,
                wait_time_reduction,
            );

            player
                .fishing_bobber
                .store(bobber.entity.entity_id, Ordering::Relaxed);

            let bobber_arc: Arc<FishingBobberEntity> = Arc::new(bobber);
            world.spawn_entity(bobber_arc);

            // Vanilla `ItemStack.causeUseVibration(player, ITEM_INTERACT_START)`.
            world.emit_game_event(GameEvent::ItemInteractStart.name(), player.position());
        } else {
            // Reel in
            if let Some(bobber_base) = world.get_entity_by_id(bobber_id) {
                if let Some(bobber) = bobber_base.cast_any().downcast_ref::<FishingBobberEntity>() {
                    let damage = bobber.reel_in(player, &used_item, hand);
                    if damage > 0 {
                        player.damage_item_in_slot(&equipment_slot, damage);
                    }
                }
                bobber_base.get_entity().remove();
            }
            player.fishing_bobber.store(-1, Ordering::Relaxed);

            let sound_pitch = bobber_sound_pitch(rand::random());
            world.play_sound_fine(
                Sound::EntityFishingBobberRetrieve,
                SoundCategory::Neutral,
                &player.position(),
                1.0,
                sound_pitch,
            );

            // Vanilla `ItemStack.causeUseVibration(player, ITEM_INTERACT_FINISH)`.
            world.emit_game_event(GameEvent::ItemInteractFinish.name(), player.position());
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
