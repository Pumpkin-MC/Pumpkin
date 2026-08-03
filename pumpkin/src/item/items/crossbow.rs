use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;

use crate::entity::player::Player;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{ChargedProjectilesImpl, EnchantmentsImpl};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::GameMode;
use pumpkin_world::inventory::Inventory;

pub struct CrossbowItem;

impl ItemMetadata for CrossbowItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::CROSSBOW.id])
    }
}

impl ItemBehaviour for CrossbowItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let inventory = player.inventory();
            let held = inventory.held_item();
            let stack = held.lock().await.clone();

            if stack
                .get_data_component::<ChargedProjectilesImpl>()
                .is_some()
            {
                Self::fire_projectiles(player, &held).await;
                return;
            }

            let has_arrows = player.find_arrow().await.is_some();
            if !has_arrows && player.gamemode.load() != GameMode::Creative {
                return;
            }

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
            let use_ticks = player.living_entity.item_use_time.load(Ordering::Relaxed);
            let use_ticks = 72000 - use_ticks;

            let mut charge_time = 25;
            let held = player.inventory().held_item();
            let stack = held.lock().await;

            if let Some(enchantments) = stack.get_data_component::<EnchantmentsImpl>() {
                for (enchantment, level) in enchantments.enchantment.iter() {
                    for effect in crate::enchantment::effects_for(enchantment) {
                        if let crate::enchantment::EnchantmentEffect::CrossbowChargeTime(value) =
                            effect
                        {
                            // quick_charge.json's crossbow_charge_time is in seconds; the base
                            // 25-tick (1.25s) charge time here is already in ticks.
                            charge_time += (value.calculate(*level) * 20.0) as i32;
                        }
                    }
                }
            }
            drop(stack);
            charge_time = charge_time.max(0);

            if use_ticks >= charge_time {
                let arrow_slot = player.find_arrow().await;
                let mut stack = held.lock().await;
                let (arrow_nbt_wrapper, slot) = {
                    if let Some(slot) = arrow_slot {
                        let inventory = player.inventory();

                        let arrow_stack_arc = inventory.get_stack(slot).await;
                        let arrow_stack = arrow_stack_arc.lock().await;
                        let mut arrow_nbt = pumpkin_nbt::compound::NbtCompound::new();
                        arrow_stack
                            .copy_with_count(1)
                            .write_item_stack(&mut arrow_nbt);
                        drop(arrow_stack);
                        (Some(arrow_nbt), slot)
                    } else if player.gamemode.load() == GameMode::Creative {
                        let mut arrow_nbt = pumpkin_nbt::compound::NbtCompound::new();
                        let arrow_stack = ItemStack::new(1, &Item::ARROW);
                        arrow_stack.write_item_stack(&mut arrow_nbt);
                        drop(arrow_stack);

                        (Some(arrow_nbt), 0)
                    } else {
                        (None, 0)
                    }
                };
                if let Some(arrow_nbt) = arrow_nbt_wrapper {
                    stack.patch.push((
                        DataComponent::ChargedProjectiles,
                        Some(Box::new(ChargedProjectilesImpl {
                            projectiles: vec![arrow_nbt],
                        })),
                    ));
                    let updated_stack = stack.clone();
                    drop(stack);

                    if player.gamemode.load() != GameMode::Creative {
                        player.consume_arrow(slot).await;
                    }

                    player
                        .sync_hand_slot(
                            player.inventory.get_selected_slot() as usize,
                            updated_stack,
                        )
                        .await;

                    // Vanilla `CrossbowItem#onUseTick`: volume 1.0, pitch
                    // 1.0F / (random.nextFloat() * 0.5F + 1.0F) + 0.2F.
                    player.world().play_sound_fine(
                        Sound::ItemCrossbowLoadingEnd,
                        SoundCategory::Players,
                        &player.position(),
                        1.0,
                        1.0 / rand::random::<f32>().mul_add(0.5, 1.0) + 0.2,
                    );
                }
            }
            player.living_entity.clear_active_hand().await;
        })
    }

    fn get_use_duration(&self) -> i32 {
        72000
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Vanilla `CrossbowItem#getShotPitch` / `#getRandomShotPitch`: the first shot is always 1.0,
/// later shots alternate between a high (0.63) and a low (0.43) offset.
fn crossbow_shot_pitch(index: u32, random_value: f32) -> f32 {
    if index == 0 {
        return 1.0;
    }
    let range_decider = if index & 1 == 1 { 0.63 } else { 0.43 };
    1.0 / random_value.mul_add(0.5, 1.8) + range_decider
}

impl CrossbowItem {
    async fn fire_projectiles(player: &Player, held: &Arc<Mutex<ItemStack>>) {
        let (projectiles, has_multishot) = {
            let stack = held.lock().await;
            let projectiles = stack
                .get_data_component::<ChargedProjectilesImpl>()
                .cloned();
            let has_multishot =
                stack
                    .get_data_component::<EnchantmentsImpl>()
                    .is_some_and(|enchantments| {
                        enchantments
                            .enchantment
                            .iter()
                            .any(|(e, _)| **e == pumpkin_data::Enchantment::MULTISHOT)
                    });
            (projectiles, has_multishot)
        };

        if let Some(charged) = projectiles {
            let world = player.world();
            let (yaw, pitch) = player.rotation();
            let mut shot_index = 0u32;

            for projectile_nbt in charged.projectiles {
                let Some(projectile) = ItemStack::read_item_stack(&projectile_nbt) else {
                    continue;
                };
                let yaws = if has_multishot {
                    vec![yaw - 10.0, yaw, yaw + 10.0]
                } else {
                    vec![yaw]
                };

                for t_yaw in yaws {
                    let arrow_entity = Entity::new(
                        world.clone(),
                        player.position(),
                        ArrowEntity::entity_type_for_item(projectile.item),
                    );
                    let pickup = if player.gamemode.load() == GameMode::Creative {
                        ArrowPickup::CreativeOnly
                    } else {
                        ArrowPickup::Allowed
                    };

                    let arrow = ArrowEntity::new_shot(
                        arrow_entity,
                        player.get_entity(),
                        &projectile,
                        pickup,
                    );
                    arrow.set_velocity_from_rotation(pitch, t_yaw, 0.0, 3.15, 1.0);
                    let arrow_arc: Arc<dyn EntityBase> = Arc::new(arrow);
                    world.spawn_entity(arrow_arc).await;

                    // Vanilla `CrossbowItem#shootProjectile` plays CROSSBOW_SHOOT once per fired
                    // projectile at volume 1.0 with `getShotPitch(random, index)`.
                    world.play_sound_fine(
                        Sound::ItemCrossbowShoot,
                        SoundCategory::Players,
                        &player.position(),
                        1.0,
                        crossbow_shot_pitch(shot_index, rand::random()),
                    );
                    shot_index += 1;
                }
            }

            held.lock()
                .await
                .patch
                .retain(|(id, _)| *id != DataComponent::ChargedProjectiles);
            player.damage_held_item(1).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::crossbow_shot_pitch;

    #[test]
    fn crossbow_shot_pitch_matches_vanilla_shot_indices() {
        assert!((crossbow_shot_pitch(0, 0.5) - 1.0).abs() < 1e-6);
        assert!((crossbow_shot_pitch(1, 0.0) - (1.0 / 1.8 + 0.63)).abs() < 1e-6);
        assert!((crossbow_shot_pitch(2, 0.0) - (1.0 / 1.8 + 0.43)).abs() < 1e-6);
    }
}
