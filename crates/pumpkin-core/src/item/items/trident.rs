use std::any::Any;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::entity::projectile::arrow::ArrowPickup;
use crate::entity::projectile::trident::TridentEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::{EnchantmentsImpl, EquipmentSlot};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::{GameMode, Hand};

pub struct TridentItem;

impl ItemMetadata for TridentItem {
    fn ids() -> Box<[u16]> {
        [Item::TRIDENT.id].into()
    }
}

impl TridentItem {
    fn riptide_level(stack: &ItemStack) -> u32 {
        stack
            .get_data_component::<EnchantmentsImpl>()
            .and_then(|enchantments| {
                enchantments
                    .enchantment
                    .iter()
                    .find(|(e, _)| **e == pumpkin_data::Enchantment::RIPTIDE)
                    .map(|(_, level)| *level as u32)
            })
            .unwrap_or(0)
    }

    fn next_damage_will_break(stack: &ItemStack) -> bool {
        !stack.is_unbreakable()
            && stack
                .get_max_damage()
                .is_some_and(|max| stack.get_damage() >= max - 1)
    }

    fn perform_riptide(player: &Player, hand: Hand, riptide_level: u32) {
        if !player.get_entity().is_in_water_or_rain() || player.get_entity().has_vehicle() {
            player.living_entity.clear_active_hand();
            return;
        }

        let world = player.world();
        if let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
            && let Some(server) = world.server.upgrade()
        {
            let mut event =
                crate::plugin::api::events::player::player_riptide::PlayerRiptideEvent {
                    player: player_arc,
                    item_name: "trident".to_string(),
                    cancelled: false,
                };
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                player.living_entity.clear_active_hand();
                return;
            }
        }

        player.increment_stat(StatisticCategory::Used, i32::from(Item::TRIDENT.id), 1);

        let riptide_strength = f64::from(0.75 * riptide_level as f32 + 0.75);
        let (yaw, pitch) = player.rotation();
        let f_yaw = f32::to_radians(yaw);
        let f_pitch = f32::to_radians(pitch);

        let vx = f64::from(-f32::sin(f_yaw) * f32::cos(f_pitch));
        let vy = f64::from(-f32::sin(f_pitch));
        let vz = f64::from(f32::cos(f_yaw) * f32::cos(f_pitch));

        let sq = (vx * vx + vy * vy + vz * vz).sqrt();
        if sq > 0.0 {
            let mult = riptide_strength / sq;
            player.living_entity.entity.velocity.store(Vector3::new(
                vx * mult,
                vy * mult,
                vz * mult,
            ));
            player.get_entity().send_velocity();
        }

        let sound = match riptide_level {
            1 => Sound::ItemTridentRiptide1,
            2 => Sound::ItemTridentRiptide2,
            _ => Sound::ItemTridentRiptide3,
        };
        world.play_sound(sound, SoundCategory::Players, &player.position());

        let slot = if hand == Hand::Left {
            EquipmentSlot::OFF_HAND
        } else {
            EquipmentSlot::MAIN_HAND
        };
        player.damage_item_in_slot(&slot, 1);
        player.living_entity.clear_active_hand();
    }
}

impl ItemBehaviour for TridentItem {
    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        let stack = player.inventory().get_stack_in_hand(hand);
        if Self::next_damage_will_break(&stack) {
            return;
        }
        if Self::riptide_level(&stack) > 0 && !player.get_entity().is_in_water_or_rain() {
            return;
        }

        player.living_entity.set_active_hand(hand, stack, 72000);
    }

    fn on_stopped_using(&self, _stack: &ItemStack, player: &Player) {
        let use_ticks = player
            .living_entity
            .item_use_time
            .load(std::sync::atomic::Ordering::Relaxed);
        let use_ticks = 72000 - use_ticks;

        if use_ticks < 10 {
            return;
        }

        let hand = (*player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner))
        .unwrap_or(Hand::Right);
        let world = player.world();
        let mut stack_guard = player.inventory().get_stack_in_hand(hand);

        if Self::next_damage_will_break(&stack_guard) {
            player.living_entity.clear_active_hand();
            return;
        }

        let riptide_level = Self::riptide_level(&stack_guard);

        if riptide_level > 0 {
            Self::perform_riptide(player, hand, riptide_level);
            return;
        }

        player.increment_stat(StatisticCategory::Used, i32::from(Item::TRIDENT.id), 1);
        let is_creative = player.gamemode.load() == GameMode::Creative;
        if !is_creative {
            let _ = player.damage_item_stack(&mut stack_guard, 1);
        }

        // Normal throw - spawn thrown trident
        let (yaw, pitch) = player.rotation();
        let entity = Entity::new(world.clone(), player.position(), &EntityType::TRIDENT);
        let pickup = if is_creative {
            ArrowPickup::CreativeOnly
        } else {
            ArrowPickup::Allowed
        };
        let trident = TridentEntity::new_shot(
            entity,
            player.get_entity(),
            stack_guard.copy_with_count(1),
            pickup,
        );
        trident.set_velocity_from_rotation(pitch, yaw, 0.0, 2.5, 1.0);
        trident.apply_on_projectile_spawned(&stack_guard);
        world.spawn_entity(Arc::new(trident));

        world.play_sound(
            Sound::ItemTridentThrow,
            SoundCategory::Players,
            &player.position(),
        );

        if !is_creative {
            let inventory = player.inventory();
            let slot_index = if hand == Hand::Left {
                pumpkin_inventory::player::player_inventory::PlayerInventory::OFF_HAND_SLOT
            } else {
                inventory.get_selected_slot() as usize
            };
            inventory.set_slot(slot_index, ItemStack::EMPTY.clone());
            player.sync_hand_slot(slot_index, ItemStack::EMPTY.clone());
        }

        player.living_entity.clear_active_hand();
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
