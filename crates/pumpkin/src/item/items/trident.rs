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
use pumpkin_data::sound::Sound;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::inventory::Inventory;

pub struct TridentItem;

impl ItemMetadata for TridentItem {
    fn ids() -> Box<[u16]> {
        [Item::TRIDENT.id].into()
    }
}

impl ItemBehaviour for TridentItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let inventory = player.inventory();
            let stack = inventory.held_item().await;

            player
                .living_entity
                .set_active_hand(pumpkin_util::Hand::Right, stack, 72000)
                .await;
        })
    }

    fn on_stopped_using<'a>(
        &'a self,
        _stack: &'a ItemStack,
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
            let stack_guard = player.inventory().held_item().await;

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

            if riptide_level > 0 {
                let in_water = world.get_block_state(&player.position().to_block_pos()).id
                    == pumpkin_data::Block::WATER.default_state.id;
                if !in_water {
                    player.living_entity.clear_active_hand().await;
                    return;
                }

                let f = f64::from(riptide_level);
                let (yaw, pitch) = player.rotation();
                let f_yaw = f32::to_radians(yaw);
                let f_pitch = f32::to_radians(pitch);

                let vx = f64::from(-f32::sin(f_yaw) * f32::cos(f_pitch));
                let vy = f64::from(-f32::sin(f_pitch));
                let vz = f64::from(f32::cos(f_yaw) * f32::cos(f_pitch));

                let sq = (vx * vx + vy * vy + vz * vz).sqrt();
                if sq > 0.0 {
                    let mult = (1.0 + f * 0.75) / sq;
                    player.living_entity.entity.velocity.store(Vector3::new(
                        vx * mult,
                        vy * mult,
                        vz * mult,
                    ));
                }

                player.damage_held_item(1).await;
                player.living_entity.clear_active_hand().await;
                return;
            }

            // Normal throw - spawn thrown trident
            let (yaw, pitch) = player.rotation();
            let entity = Entity::new(world.clone(), player.position(), &EntityType::TRIDENT);
            let trident = TridentEntity::new_shot(
                entity,
                player.get_entity(),
                stack_guard.clone(),
                ArrowPickup::Allowed,
            );
            trident.set_velocity_from_rotation(pitch, yaw, 0.0, 2.5, 1.0);
            world.spawn_entity(Arc::new(trident)).await;

            world.play_sound(
                Sound::ItemTridentThrow,
                pumpkin_data::sound::SoundCategory::Players,
                &player.position(),
            );

            if player.gamemode.load() != GameMode::Creative {
                let inventory = player.inventory();
                let selected_slot = inventory.get_selected_slot() as usize;

                let main_hand_item = inventory.get_stack(selected_slot).await;
                if main_hand_item.item.id == Item::TRIDENT.id {
                    inventory
                        .set_stack(selected_slot, ItemStack::EMPTY.clone())
                        .await;
                    player
                        .sync_hand_slot(selected_slot, ItemStack::EMPTY.clone())
                        .await;
                } else {
                    let off_hand_slot =
                        pumpkin_inventory::player::player_inventory::PlayerInventory::OFF_HAND_SLOT;
                    let off_hand_item = inventory.get_stack(off_hand_slot).await;
                    if off_hand_item.item.id == Item::TRIDENT.id {
                        inventory
                            .set_stack(off_hand_slot, ItemStack::EMPTY.clone())
                            .await;
                        player
                            .sync_hand_slot(off_hand_slot, ItemStack::EMPTY.clone())
                            .await;
                    }
                }
            }

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
