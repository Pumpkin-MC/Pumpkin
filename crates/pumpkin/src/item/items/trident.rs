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
            let held = inventory.held_item();
            let stack = held.lock().await.clone();

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
            let held = player.inventory().held_item();
            let stack_guard = held.lock().await.clone();

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
                let is_touching_water = player
                    .living_entity
                    .entity
                    .touching_water
                    .load(std::sync::atomic::Ordering::Relaxed);
                let is_raining = world.is_raining().await;

                if is_touching_water || is_raining {
                    let (yaw, pitch) = player.rotation();
                    let look_vec = Vector3::rotation_vector(pitch as f64, yaw as f64);
                    let speed = f64::from(riptide_level).mul_add(0.75, 1.5);
                    let launch_velocity = look_vec.multiply(speed, speed, speed);

                    player.get_entity().set_velocity(launch_velocity);
                    player.get_entity().send_velocity();

                    let sound = match riptide_level {
                        1 => Sound::ItemTridentRiptide1,
                        2 => Sound::ItemTridentRiptide2,
                        _ => Sound::ItemTridentRiptide3,
                    };
                    world.play_sound(sound, SoundCategory::Players, &player.position());

                    player.living_entity.clear_active_hand().await;

                    if player.gamemode.load() != GameMode::Creative {
                        player.damage_held_item(1).await;
                    }
                    return;
                }
            }

            // Normal throw
            let entity = Entity::new(world.clone(), player.position(), &EntityType::TRIDENT);
            let pickup = if player.gamemode.load() == GameMode::Creative {
                ArrowPickup::CreativeOnly
            } else {
                ArrowPickup::Allowed
            };

            let trident_entity =
                TridentEntity::new_shot(entity, player.get_entity(), stack_guard, pickup);

            let (yaw, pitch) = player.rotation();
            trident_entity.set_velocity_from_rotation(pitch, yaw, 0.0, 2.5, 1.0);

            let trident_arc: Arc<dyn EntityBase> = Arc::new(trident_entity);
            world.spawn_entity(trident_arc).await;

            world.play_sound(
                Sound::ItemTridentThrow,
                SoundCategory::Players,
                &player.position(),
            );

            if player.gamemode.load() != GameMode::Creative {
                let inventory = player.inventory();
                let selected_slot = inventory.get_selected_slot() as usize;

                let main_hand_item = inventory.get_stack(selected_slot).await;
                let mut stack_lock = main_hand_item.lock().await;
                if stack_lock.item.id == Item::TRIDENT.id {
                    *stack_lock = ItemStack::EMPTY.clone();
                    player
                        .sync_hand_slot(selected_slot, ItemStack::EMPTY.clone())
                        .await;
                } else {
                    let off_hand_slot =
                        pumpkin_inventory::player::player_inventory::PlayerInventory::OFF_HAND_SLOT;
                    let off_hand_item = inventory.get_stack(off_hand_slot).await;
                    let mut off_stack_lock = off_hand_item.lock().await;
                    if off_stack_lock.item.id == Item::TRIDENT.id {
                        *off_stack_lock = ItemStack::EMPTY.clone();
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
