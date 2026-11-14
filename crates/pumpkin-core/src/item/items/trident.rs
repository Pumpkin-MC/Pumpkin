use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::enchantment::EnchantmentHelper;
use crate::entity::player::Player;
use crate::entity::projectile::arrow::ArrowPickup;
use crate::entity::projectile::trident::TridentEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::{EnchantmentsImpl, EquipmentSlot};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::{DamageResult, ItemStack};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::{GameMode, Hand};

pub struct TridentItem;

impl ItemMetadata for TridentItem {
    fn ids() -> Box<[u16]> {
        [Item::TRIDENT.id].into()
    }
}

impl TridentItem {
    const THROW_THRESHOLD_TIME: i32 = 10;
    const BASE_DAMAGE: f32 = 8.0;
    const PROJECTILE_SHOOT_POWER: f32 = 2.5;
    const USE_DURATION: i32 = 72000;
    const SPIN_ATTACK_TICKS: i32 = 20;

    /// Mirrors `EnchantmentHelper.pickHighestLevel(itemStack, EnchantmentEffectComponents.TRIDENT_SOUND)`.
    fn trident_sound(stack: &ItemStack) -> Sound {
        let riptide_level = stack
            .get_data_component::<EnchantmentsImpl>()
            .and_then(|enchantments| {
                enchantments
                    .enchantment
                    .iter()
                    .filter(|(e, _)| **e == pumpkin_data::Enchantment::RIPTIDE)
                    .map(|(_, level)| *level)
                    .max()
            })
            .unwrap_or(0);
        match riptide_level {
            i32::MIN..=0 => Sound::ItemTridentThrow,
            1 => Sound::ItemTridentRiptide1,
            2 => Sound::ItemTridentRiptide2,
            3..=i32::MAX => Sound::ItemTridentRiptide3,
        }
    }

    fn throw_trident(
        player: &Player,
        mut stack: ItemStack,
        slot_index: usize,
        is_creative: bool,
        sound: Sound,
    ) {
        let world = player.world();
        let thrown_stack = stack.copy_with_count(1);
        if !is_creative {
            stack.decrement(1);
            let remaining_stack = if stack.is_empty() {
                ItemStack::EMPTY.clone()
            } else {
                stack
            };
            player
                .inventory()
                .set_slot(slot_index, remaining_stack.clone());
            player.sync_hand_slot(slot_index, remaining_stack);
        }

        let (yaw, pitch) = player.rotation();
        let entity = Entity::new(world.clone(), player.position(), &EntityType::TRIDENT);
        let pickup = if is_creative {
            ArrowPickup::CreativeOnly
        } else {
            ArrowPickup::Allowed
        };
        let trident =
            TridentEntity::new_shot(entity, player.get_entity(), thrown_stack.clone(), pickup);
        trident.shoot_from_rotation(
            player.get_entity(),
            pitch,
            yaw,
            0.0,
            Self::PROJECTILE_SHOOT_POWER,
            1.0,
        );
        let trident_pos = trident.get_entity().pos.load();
        let trident_arc = Arc::new(trident);
        world.spawn_entity(trident_arc.clone());
        trident_arc.apply_on_projectile_spawned(&thrown_stack);

        world.play_sound_raw(sound as u16, SoundCategory::Players, &trident_pos, 1.0, 1.0);
    }

    fn activate_riptide(
        player: &Player,
        stack: ItemStack,
        slot: EquipmentSlot,
        slot_index: usize,
        damage_result: DamageResult,
        riptide_strength: f32,
        sound: Sound,
    ) {
        if damage_result != DamageResult::Untouched {
            player.inventory().set_slot(slot_index, stack.clone());
            player.sync_hand_slot(slot_index, stack.clone());
        }

        let (yaw, pitch) = player.rotation();
        let y_rot = yaw.to_radians();
        let x_rot = pitch.to_radians();
        let mut xd = -y_rot.sin() * x_rot.cos();
        let mut yd = -x_rot.sin();
        let mut zd = y_rot.cos() * x_rot.cos();
        let dist = (xd * xd + yd * yd + zd * zd).sqrt();
        if dist > 0.0 {
            let scale = riptide_strength / dist;
            xd *= scale;
            yd *= scale;
            zd *= scale;
            // Java clients apply their own Riptide push and +1.2 Y lift locally; `push_by`
            // syncs velocity to watchers on Java and sends `SetActorMotion` on Bedrock.
            player
                .get_entity()
                .push_by(Vector3::new(f64::from(xd), f64::from(yd), f64::from(zd)));
        }
        player.start_auto_spin_attack(
            Self::SPIN_ATTACK_TICKS,
            Self::BASE_DAMAGE,
            stack,
            Some(slot),
        );
        player.world().play_sound_raw(
            sound as u16,
            SoundCategory::Players,
            &player.position(),
            1.0,
            1.0,
        );
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
        if stack.next_damage_will_break() {
            return;
        }
        if EnchantmentHelper::modify_trident_spin_attack_strength(&stack, 0.0) > 0.0
            && !player.get_entity().is_in_water_or_rain()
        {
            return;
        }

        player
            .living_entity
            .set_active_hand(hand, stack, Self::USE_DURATION);
    }

    fn on_stopped_using(&self, _stack: &ItemStack, player: &Player) {
        let remaining_time = player.living_entity.item_use_time.load(Ordering::Relaxed);
        let time_held = Self::USE_DURATION - remaining_time;
        if time_held < Self::THROW_THRESHOLD_TIME {
            return;
        }

        let hand = (*player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner))
        .unwrap_or(Hand::Right);
        let inventory = player.inventory();
        let (slot, slot_index) = if hand == Hand::Left {
            (EquipmentSlot::OFF_HAND, PlayerInventory::OFF_HAND_SLOT)
        } else {
            (
                EquipmentSlot::MAIN_HAND,
                inventory.get_selected_slot() as usize,
            )
        };

        let mut stack = inventory.get_stack_in_hand(hand);
        if stack.is_empty() || stack.item.id != Item::TRIDENT.id {
            player.living_entity.clear_active_hand();
            return;
        }

        let riptide_strength = EnchantmentHelper::modify_trident_spin_attack_strength(&stack, 0.0);
        if riptide_strength > 0.0
            && (!player.get_entity().is_in_water_or_rain() || player.get_entity().has_vehicle())
        {
            player.living_entity.clear_active_hand();
            return;
        }

        if stack.next_damage_will_break() {
            player.living_entity.clear_active_hand();
            return;
        }

        let world = player.world();
        if riptide_strength > 0.0
            && let Some(player_arc) = world.get_player_by_uuid(player.gameprofile.id)
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

        let sound = Self::trident_sound(&stack);
        player.increment_stat(StatisticCategory::Used, i32::from(Item::TRIDENT.id), 1);
        let is_creative = player.gamemode.load() == GameMode::Creative;
        let damage_result = player.hurt_item_stack_without_breaking(&mut stack, 1);

        if riptide_strength == 0.0 {
            Self::throw_trident(player, stack, slot_index, is_creative, sound);
        } else {
            Self::activate_riptide(
                player,
                stack,
                slot,
                slot_index,
                damage_result,
                riptide_strength,
                sound,
            );
        }

        player.living_entity.clear_active_hand();
    }

    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
