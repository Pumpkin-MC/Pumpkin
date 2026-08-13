use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::entity::projectile::arrow::ArrowPickup;
use crate::entity::projectile::trident::TridentEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;

pub struct TridentItem;

impl ItemMetadata for TridentItem {
    fn ids() -> Box<[u16]> {
        [Item::TRIDENT.id].into()
    }
}

impl ItemBehaviour for TridentItem {
    fn normal_use_in_hand<'a>(
        &'a self,
        _item: &'a Item,
        stack: &'a ItemStack,
        hand: pumpkin_util::Hand,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let riptide_level = stack.get_enchantment_level(&pumpkin_data::Enchantment::RIPTIDE);
            if stack
                .get_max_damage()
                .is_some_and(|max| stack.get_damage() + 1 >= max)
                || (riptide_level > 0 && !player.is_in_water_or_rain().await)
            {
                return;
            }

            player
                .living_entity
                .set_active_hand(hand, stack.clone(), 72000)
                .await;
        })
    }

    #[allow(clippy::too_many_lines)]
    fn on_stopped_using<'a>(
        &'a self,
        stack: &'a ItemStack,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let use_ticks = player
                .living_entity
                .item_use_time
                .load(std::sync::atomic::Ordering::Relaxed);
            let use_ticks = 72000 - use_ticks;

            if use_ticks < 10 {
                return;
            }

            let world = player.world();
            if stack
                .get_max_damage()
                .is_some_and(|max| stack.get_damage() + 1 >= max)
            {
                return;
            }
            let stack_guard = stack.clone();

            // Check Riptide level
            let mut riptide_level = 0u32;
            if let Some(enchantments) = stack_guard
                .get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>(
            ) {
                for (enchantment, level) in enchantments.enchantment.iter() {
                    if **enchantment == pumpkin_data::Enchantment::RIPTIDE {
                        riptide_level = *level as u32;
                    }
                }
            }

            let hand = (*player.living_entity.active_hand.lock().await)
                .unwrap_or(pumpkin_util::Hand::Right);
            if riptide_level > 0 {
                if !player.is_in_water_or_rain().await || player.get_entity().has_vehicle().await {
                    return;
                }

                let (yaw, pitch) = player.rotation();
                let look_vec = Vector3::rotation_vector(pitch as f64, yaw as f64);
                let speed = f64::from(riptide_level.saturating_sub(1)).mul_add(0.75, 1.5);
                let launch_velocity = look_vec.multiply(speed, speed, speed);

                player.get_entity().add_velocity(launch_velocity);
                let mut spin_stack = player.inventory().get_stack_in_hand(hand).await;
                if player.gamemode.load() != GameMode::Creative {
                    let _ = spin_stack.damage_item(1);
                    player
                        .inventory()
                        .set_stack_in_hand(hand, spin_stack.clone())
                        .await;
                    let slot_index = match hand {
                        pumpkin_util::Hand::Right => {
                            player.inventory().get_selected_slot() as usize
                        }
                        pumpkin_util::Hand::Left => {
                            pumpkin_inventory::player::player_inventory::PlayerInventory::OFF_HAND_SLOT
                        }
                    };
                    player.sync_hand_slot(slot_index, spin_stack.clone()).await;
                }
                player
                    .living_entity
                    .start_auto_spin_attack(20, 8.0, spin_stack, hand)
                    .await;
                if player
                    .get_entity()
                    .on_ground
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    player.get_entity().move_pos(Vector3::new(0.0, 1.2, 0.0));
                }

                let sound = match riptide_level {
                    1 => Sound::ItemTridentRiptide1,
                    2 => Sound::ItemTridentRiptide2,
                    _ => Sound::ItemTridentRiptide3,
                };
                world.play_sound(sound, SoundCategory::Players, &player.position());

                player.living_entity.clear_active_hand().await;
                return;
            }

            // Normal throw - spawn thrown trident
            let mut inventory_stack = player.inventory().get_stack_in_hand(hand).await;
            if player.gamemode.load() != GameMode::Creative {
                let _ = inventory_stack.damage_item(1);
            }
            let thrown_stack = inventory_stack.split_unless_creative(player.gamemode.load(), 1);
            if inventory_stack.is_empty() {
                inventory_stack.clear();
            }
            player
                .inventory()
                .set_stack_in_hand(hand, inventory_stack.clone())
                .await;
            let slot_index = match hand {
                pumpkin_util::Hand::Right => player.inventory().get_selected_slot() as usize,
                pumpkin_util::Hand::Left => {
                    pumpkin_inventory::player::player_inventory::PlayerInventory::OFF_HAND_SLOT
                }
            };
            player.sync_hand_slot(slot_index, inventory_stack).await;
            let (yaw, pitch) = player.rotation();
            let entity = Entity::new(world.clone(), player.position(), &EntityType::TRIDENT);
            let trident = TridentEntity::new_shot(
                entity,
                player.get_entity(),
                thrown_stack,
                if player.gamemode.load() == GameMode::Creative {
                    ArrowPickup::CreativeOnly
                } else {
                    ArrowPickup::Allowed
                },
            );
            trident.set_velocity_from_rotation(pitch, yaw, 0.0, 2.5, 1.0);
            world.spawn_entity(Arc::new(trident)).await;

            world.play_sound(
                Sound::ItemTridentThrow,
                pumpkin_data::sound::SoundCategory::Players,
                &player.position(),
            );

            player.living_entity.clear_active_hand().await;
        })
    }

    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }

    fn get_use_duration(&self) -> i32 {
        72000
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
