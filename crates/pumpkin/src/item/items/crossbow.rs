use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

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

pub struct CrossbowItem;

impl ItemMetadata for CrossbowItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::CROSSBOW.id])
    }
}

impl ItemBehaviour for CrossbowItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        let inventory = player.inventory();
        let stack = inventory.held_item();

        // Every crossbow carries a ChargedProjectiles component by default, so its mere
        // presence does not mean the crossbow is loaded. Vanilla checks the list is also
        // non-empty (CrossbowItem.java:68).
        if stack
            .get_data_component::<ChargedProjectilesImpl>()
            .is_some_and(|charged| !charged.projectiles.is_empty())
        {
            Self::fire_projectiles(player);
            return;
        }

        let has_arrows = player.find_arrow().is_some();
        if !has_arrows && player.gamemode.load() != GameMode::Creative {
            return;
        }

        player
            .living_entity
            .set_active_hand(pumpkin_util::Hand::Right, stack, 72000);
    }

    fn on_stopped_using(&self, _stack: &ItemStack, player: &Player) {
        let use_ticks = player.living_entity.item_use_time.load(Ordering::Relaxed);
        let use_ticks = 72000 - use_ticks;

        let mut charge_time = 25;
        let mut stack = player.inventory().held_item();

        if let Some(enchantments) = stack.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                if **enchantment == pumpkin_data::Enchantment::QUICK_CHARGE {
                    charge_time -= 5 * level;
                }
            }
        }
        charge_time = charge_time.max(0);

        if use_ticks >= charge_time {
            let arrow_slot = player.find_arrow();
            let (arrow_nbt_wrapper, slot) = arrow_slot.map_or_else(
                || {
                    if player.gamemode.load() == GameMode::Creative {
                        let mut arrow_nbt = pumpkin_nbt::compound::NbtCompound::new();
                        let arrow_stack = ItemStack::new(1, &Item::ARROW);
                        arrow_stack.write_item_stack(&mut arrow_nbt);

                        (Some(arrow_nbt), 0)
                    } else {
                        (None, 0)
                    }
                },
                |slot| {
                    let inventory = player.inventory();

                    let arrow_stack = inventory.get_slot(slot);
                    let mut arrow_nbt = pumpkin_nbt::compound::NbtCompound::new();
                    arrow_stack
                        .copy_with_count(1)
                        .write_item_stack(&mut arrow_nbt);
                    (Some(arrow_nbt), slot)
                },
            );
            if let Some(arrow_nbt) = arrow_nbt_wrapper {
                stack.patch.push((
                    DataComponent::ChargedProjectiles,
                    Some(Box::new(ChargedProjectilesImpl {
                        projectiles: vec![arrow_nbt],
                    })),
                ));
                player.inventory().set_held_item(stack);

                if player.gamemode.load() != GameMode::Creative {
                    player.consume_arrow(slot);
                }

                player.world().play_sound(
                    Sound::ItemCrossbowLoadingEnd,
                    SoundCategory::Players,
                    &player.position(),
                );
            }
        }
        player.living_entity.clear_active_hand();
    }

    fn get_use_duration(&self) -> i32 {
        72000
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CrossbowItem {
    fn fire_projectiles(player: &Player) {
        let mut held = player.inventory().held_item();
        let projectiles = held.get_data_component::<ChargedProjectilesImpl>().cloned();
        let has_multishot =
            held.get_data_component::<EnchantmentsImpl>()
                .is_some_and(|enchantments| {
                    enchantments
                        .enchantment
                        .iter()
                        .any(|(e, _)| **e == pumpkin_data::Enchantment::MULTISHOT)
                });

        if let Some(charged) = projectiles {
            let world = player.world();
            world.play_sound(
                Sound::ItemCrossbowShoot,
                SoundCategory::Players,
                &player.position(),
            );

            let (yaw, pitch) = player.rotation();

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
                    world.spawn_entity(arrow_arc);
                }
            }

            held.patch
                .retain(|(id, _)| *id != DataComponent::ChargedProjectiles);
            player.inventory().set_held_item(held);
            player.damage_held_item(1);
        }
    }
}
