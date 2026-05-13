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
use pumpkin_data::entity::EntityType;
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
                    if **enchantment == pumpkin_data::Enchantment::QUICK_CHARGE {
                        charge_time -= 5 * level;
                    }
                }
            }
            drop(stack);
            charge_time = charge_time.max(0);

            if use_ticks >= charge_time {
                let arrow_slot = player.find_arrow().await;
                if let Some(slot) = arrow_slot {
                    let inventory = player.inventory();
                    let mut stack = held.lock().await;

                    let arrow_stack_arc = inventory.get_stack(slot).await;
                    let arrow_stack = arrow_stack_arc.lock().await;
                    let mut arrow_nbt = pumpkin_nbt::compound::NbtCompound::new();
                    arrow_stack.write_item_stack(&mut arrow_nbt);
                    drop(arrow_stack);

                    stack.patch.push((
                        DataComponent::ChargedProjectiles,
                        Some(Box::new(ChargedProjectilesImpl {
                            projectiles: vec![arrow_nbt],
                        })),
                    ));

                    if player.gamemode.load() != GameMode::Creative {
                        player.consume_arrow(slot).await;
                    }

                    player
                        .world()
                        .play_sound(
                            Sound::ItemCrossbowLoadingEnd,
                            SoundCategory::Players,
                            &player.position(),
                        )
                        .await;
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

impl CrossbowItem {
    async fn fire_projectiles(player: &Player, held: &Arc<Mutex<ItemStack>>) {
        let mut stack = held.lock().await;
        let projectiles = stack
            .get_data_component::<ChargedProjectilesImpl>()
            .cloned();

        if let Some(charged) = projectiles {
            let has_multishot =
                stack
                    .get_data_component::<EnchantmentsImpl>()
                    .is_some_and(|enchantments| {
                        enchantments
                            .enchantment
                            .iter()
                            .any(|(e, _)| **e == pumpkin_data::Enchantment::MULTISHOT)
                    });

            let world = player.world();
            world
                .play_sound(
                    Sound::ItemCrossbowShoot,
                    SoundCategory::Players,
                    &player.position(),
                )
                .await;

            let yaw = player.living_entity.entity.yaw.load();
            let pitch = player.living_entity.entity.pitch.load();

            for _ in charged.projectiles {
                let yaws = if has_multishot {
                    vec![yaw - 10.0, yaw, yaw + 10.0]
                } else {
                    vec![yaw]
                };

                for t_yaw in yaws {
                    let arrow_entity =
                        Entity::new(world.clone(), player.position(), &EntityType::ARROW);
                    let pickup = if player.gamemode.load() == GameMode::Creative {
                        ArrowPickup::CreativeOnly
                    } else {
                        ArrowPickup::Allowed
                    };

                    let arrow =
                        ArrowEntity::new_shot(arrow_entity, &player.living_entity.entity, pickup)
                            .await;
                    arrow.set_velocity_from_rotation(pitch, t_yaw, 0.0, 3.15, 1.0);
                    let arrow_arc: Arc<dyn EntityBase> = Arc::new(arrow);
                    world.spawn_entity(arrow_arc).await;
                }
            }

            stack
                .patch
                .retain(|(id, _)| *id != DataComponent::ChargedProjectiles);
            player.damage_held_item(1).await;
        }
    }
}
