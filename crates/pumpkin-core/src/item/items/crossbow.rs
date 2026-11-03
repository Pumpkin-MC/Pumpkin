use std::any::Any;
use std::sync::atomic::Ordering;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::items::projectile_weapon::ProjectileWeaponItem;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{ChargedProjectilesImpl, EnchantmentsImpl, EquipmentSlot};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::{GameMode, Hand};

pub struct CrossbowItem;

impl ItemMetadata for CrossbowItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::CROSSBOW.id])
    }
}

impl ItemBehaviour for CrossbowItem {
    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        let stack = player.inventory().get_stack_in_hand(hand);

        // Every crossbow carries a ChargedProjectiles component by default, so its mere
        // presence does not mean the crossbow is loaded. Vanilla checks the list is also
        // non-empty (CrossbowItem.java:68).
        if Self::is_charged(&stack) {
            Self::fire_projectiles(player, hand);
            return;
        }

        let has_projectiles = player.find_crossbow_projectile().is_some();
        if !has_projectiles && player.gamemode.load() != GameMode::Creative {
            return;
        }

        player
            .living_entity
            .set_active_hand(hand, stack, Self::USE_DURATION);
    }

    fn on_use_tick(&self, _stack: &ItemStack, player: &Player, remaining_use_ticks: i32) {
        let hand = (*player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner))
        .unwrap_or(Hand::Right);
        let mut stack = player.inventory().get_stack_in_hand(hand);
        if stack.item.id != Item::CROSSBOW.id {
            return;
        }

        let charge_time =
            crate::enchantment::EnchantmentHelper::modify_crossbow_charge_time(&stack, 25).max(1);
        let use_ticks = Self::USE_DURATION - remaining_use_ticks;
        let prev_percent = (use_ticks - 1) as f32 / charge_time as f32;
        let tick_percent = use_ticks as f32 / charge_time as f32;

        if prev_percent < 0.2 && tick_percent >= 0.2 {
            let start_sound = match Self::quick_charge_level(&stack) {
                1 => Sound::ItemCrossbowQuickCharge1,
                2 => Sound::ItemCrossbowQuickCharge2,
                3.. => Sound::ItemCrossbowQuickCharge3,
                _ => Sound::ItemCrossbowLoadingStart,
            };
            player.world().play_sound_fine(
                start_sound,
                SoundCategory::Players,
                &player.position(),
                0.5,
                1.0,
            );
        }

        if prev_percent < 0.5 && tick_percent >= 0.5 && Self::quick_charge_level(&stack) == 0 {
            player.world().play_sound_fine(
                Sound::ItemCrossbowLoadingMiddle,
                SoundCategory::Players,
                &player.position(),
                0.5,
                1.0,
            );
        }

        if tick_percent >= 1.0
            && !Self::is_charged(&stack)
            && Self::try_load_projectiles(player, hand, &mut stack)
        {
            player.world().play_sound_fine(
                Sound::ItemCrossbowLoadingEnd,
                SoundCategory::Players,
                &player.position(),
                1.0,
                1.0 / (rand::random::<f32>() * 0.5 + 1.0) + 0.2,
            );
        }
    }

    fn on_stopped_using(&self, _stack: &ItemStack, player: &Player) {
        let use_ticks = player.living_entity.item_use_time.load(Ordering::Relaxed);
        let use_ticks = Self::USE_DURATION - use_ticks;

        let hand = (*player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner))
        .unwrap_or(Hand::Right);
        let mut stack = player.inventory().get_stack_in_hand(hand);
        let charge_time =
            crate::enchantment::EnchantmentHelper::modify_crossbow_charge_time(&stack, 25).max(1);

        let f = (use_ticks as f32 / charge_time as f32).min(1.0);
        if f >= 1.0
            && !Self::is_charged(&stack)
            && Self::try_load_projectiles(player, hand, &mut stack)
        {
            player.world().play_sound(
                Sound::ItemCrossbowLoadingEnd,
                SoundCategory::Players,
                &player.position(),
            );
        }
        player.living_entity.clear_active_hand();
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CrossbowItem {
    pub const USE_DURATION: i32 = 72000;
    pub const ARROW_POWER: f32 = 3.15;
    pub const FIREWORK_POWER: f32 = 1.6;

    #[must_use]
    pub fn is_charged(stack: &ItemStack) -> bool {
        stack
            .get_data_component::<ChargedProjectilesImpl>()
            .is_some_and(|charged| !charged.projectiles.is_empty())
    }

    fn quick_charge_level(stack: &ItemStack) -> i32 {
        stack
            .get_data_component::<EnchantmentsImpl>()
            .and_then(|enchantments| {
                enchantments
                    .enchantment
                    .iter()
                    .find(|(e, _)| **e == pumpkin_data::Enchantment::QUICK_CHARGE)
                    .map(|(_, level)| *level)
            })
            .unwrap_or(0)
    }

    fn try_load_projectiles(player: &Player, hand: Hand, stack: &mut ItemStack) -> bool {
        let arrow_slot = player.find_crossbow_projectile();
        let gamemode = player.gamemode.load();
        let is_creative = gamemode == GameMode::Creative;

        if arrow_slot.is_none() && !is_creative {
            return false;
        }

        let projectile = arrow_slot.map_or_else(
            || ItemStack::new(1, &Item::ARROW),
            |slot| player.inventory().get_slot(slot).copy_with_count(1),
        );

        let drawn = ProjectileWeaponItem::draw(stack, &projectile, is_creative);
        if drawn.is_empty() {
            return false;
        }

        let mut charged_nbts = Vec::with_capacity(drawn.len());
        for item in drawn {
            let mut arrow_nbt = pumpkin_nbt::compound::NbtCompound::new();
            item.write_item_stack(&mut arrow_nbt);
            charged_nbts.push(arrow_nbt);
        }

        stack
            .patch
            .retain(|(id, _)| *id != DataComponent::ChargedProjectiles);
        stack.patch.push((
            DataComponent::ChargedProjectiles,
            Some(Box::new(ChargedProjectilesImpl {
                projectiles: charged_nbts,
            })),
        ));
        player.inventory().set_stack_in_hand(hand, stack.clone());

        if let Some(slot) = arrow_slot
            && !is_creative
        {
            player.consume_arrow(slot);
        }

        true
    }

    fn fire_projectiles(player: &Player, hand: Hand) {
        let mut held = player.inventory().get_stack_in_hand(hand);
        let charged_opt = held.get_data_component::<ChargedProjectilesImpl>().cloned();

        if let Some(charged) = charged_opt {
            let mut projectiles = Vec::new();
            for projectile_nbt in charged.projectiles {
                if let Some(projectile) = ItemStack::read_item_stack(&projectile_nbt) {
                    projectiles.push(projectile);
                }
            }

            if !projectiles.is_empty() {
                let world = player.world();
                world.play_sound(
                    Sound::ItemCrossbowShoot,
                    SoundCategory::Players,
                    &player.position(),
                );

                let has_firework = projectiles
                    .iter()
                    .any(|p| p.item.id == Item::FIREWORK_ROCKET.id);
                let power = if has_firework {
                    Self::FIREWORK_POWER
                } else {
                    Self::ARROW_POWER
                };

                let is_creative = player.gamemode.load() == GameMode::Creative;
                ProjectileWeaponItem::shoot_projectiles(
                    &world,
                    player.get_entity(),
                    &held,
                    &projectiles,
                    power,
                    1.0,
                    true,
                    is_creative,
                );

                player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::ShotCrossbow {
                        item_id: held.item.registry_key.to_string(),
                    },
                );

                let durability_use: i32 = projectiles
                    .iter()
                    .map(|p| {
                        if p.item.id == Item::FIREWORK_ROCKET.id {
                            3
                        } else {
                            1
                        }
                    })
                    .sum();

                held.patch
                    .retain(|(id, _)| *id != DataComponent::ChargedProjectiles);
                player.inventory().set_stack_in_hand(hand, held.clone());
                let slot = if hand == Hand::Left {
                    EquipmentSlot::OFF_HAND
                } else {
                    EquipmentSlot::MAIN_HAND
                };
                player.damage_item_in_slot(&slot, durability_use);
                player.increment_stat(StatisticCategory::Used, i32::from(held.item.id), 1);
            }
        }
    }
}
