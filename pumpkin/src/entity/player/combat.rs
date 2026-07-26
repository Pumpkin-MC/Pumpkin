use super::Player;
use super::statistics;
use crate::entity::EntityBase;
use crate::entity::combat::{self, AttackType, player_attack_sound};
use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::Enchantment;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::AttributeModifiersImpl;
use pumpkin_data::data_component_impl::EnchantmentsImpl;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::data_component_impl::Operation;
use pumpkin_data::data_component_impl::ToolImpl;
use pumpkin_data::data_component_impl::WeaponImpl;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Animation;
use pumpkin_protocol::java::client::play::CSetPlayerInventory;
use pumpkin_util::GameMode;
use pumpkin_util::Hand;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::inventory::Inventory;
use std::sync::Arc;
use std::sync::atomic::Ordering;

impl Player {
    #[expect(clippy::too_many_lines)]
    pub async fn attack(&self, victim: Arc<dyn EntityBase>) {
        let world = self.world();
        let server = world.server.upgrade().unwrap();
        let victim_entity = victim.get_entity();
        let attacker_entity = &self.living_entity.entity;
        let config = &server.advanced_config.pvp;

        let inventory = self.inventory();
        let item_stack = inventory.held_item();

        let base_damage = self
            .living_entity
            .get_attribute_value(&Attributes::ATTACK_DAMAGE);
        let base_attack_speed = 4.0;

        let mut damage_multiplier = 1.0;
        let mut add_damage = 0.0;
        let mut add_speed = 0.0;
        let mut extra_ench_damage = 0.0;
        let mut knockback_level = 0u32;

        {
            let stack = item_stack.lock().await;
            if let Some(modifiers) = stack.get_data_component::<AttributeModifiersImpl>() {
                for item_mod in modifiers.attribute_modifiers.iter() {
                    if item_mod.operation == Operation::AddValue {
                        if item_mod.id == "minecraft:base_attack_damage" {
                            add_damage = item_mod.amount;
                        } else if item_mod.id == "minecraft:base_attack_speed" {
                            add_speed = item_mod.amount;
                        }
                    }
                }
            }
            if let Some(enchantments) = stack.get_data_component::<EnchantmentsImpl>() {
                for (enchantment, level) in enchantments.enchantment.iter() {
                    if **enchantment == Enchantment::SHARPNESS {
                        extra_ench_damage += 0.5 * f64::from(*level) + 0.5;
                    } else if **enchantment == Enchantment::SMITE {
                        let target_type = victim_entity.entity_type.id;
                        let is_undead = target_type == EntityType::ZOMBIE.id
                            || target_type == EntityType::DROWNED.id
                            || target_type == EntityType::HUSK.id
                            || target_type == EntityType::ZOMBIE_VILLAGER.id
                            || target_type == EntityType::ZOMBIFIED_PIGLIN.id
                            || target_type == EntityType::SKELETON.id
                            || target_type == EntityType::BOGGED.id
                            || target_type == EntityType::PARCHED.id
                            || target_type == EntityType::WITHER_SKELETON.id
                            || target_type == EntityType::STRAY.id
                            || target_type == EntityType::PHANTOM.id
                            || target_type == EntityType::WITHER.id
                            || target_type == EntityType::ZOMBIE_HORSE.id
                            || target_type == EntityType::SKELETON_HORSE.id;
                        if is_undead {
                            extra_ench_damage += 2.5 * f64::from(*level);
                        }
                    } else if **enchantment == Enchantment::BANE_OF_ARTHROPODS {
                        let target_type = victim_entity.entity_type.id;
                        let is_arthropod = target_type == EntityType::SPIDER.id
                            || target_type == EntityType::CAVE_SPIDER.id
                            || target_type == EntityType::SILVERFISH.id
                            || target_type == EntityType::ENDERMITE.id
                            || target_type == EntityType::BEE.id;
                        if is_arthropod {
                            extra_ench_damage += 2.5 * f64::from(*level);
                        }
                    } else if **enchantment == Enchantment::KNOCKBACK {
                        knockback_level = *level as u32;
                    }
                }
            }
        }

        let attack_speed = base_attack_speed + add_speed;

        let attack_cooldown_progress = self.get_attack_cooldown_progress(
            f64::from(server.basic_config.tps),
            0.5,
            attack_speed,
        );
        self.last_attacked_ticks.store(0, Ordering::Relaxed);

        // Only reduce attack damage if in cooldown
        // TODO: Enchantments are reduced in the same way, just without the square.
        if attack_cooldown_progress < 1.0 {
            damage_multiplier = attack_cooldown_progress.powi(2).mul_add(0.8, 0.2);
        }

        // Modify the added damage based on the multiplier.
        let mut damage = base_damage + add_damage * damage_multiplier;
        damage += extra_ench_damage * attack_cooldown_progress;

        if let Some(strength) = self
            .living_entity
            .get_effect(&pumpkin_data::effect::StatusEffect::STRENGTH)
            .await
        {
            damage += 3.0 * (f64::from(strength.amplifier) + 1.0);
        }
        if let Some(weakness) = self
            .living_entity
            .get_effect(&pumpkin_data::effect::StatusEffect::WEAKNESS)
            .await
        {
            damage -= 4.0 * (f64::from(weakness.amplifier) + 1.0);
        }
        damage = damage.max(0.0);

        let pos = victim_entity.pos.load();
        let attack_type = AttackType::new(self, attack_cooldown_progress as f32).await;

        if matches!(attack_type, AttackType::Critical) {
            damage *= 1.5;
        }

        let is_mace_smash = matches!(attack_type, AttackType::MaceSmash);
        if is_mace_smash {
            let fall_distance = self.living_entity.fall_distance.load();
            damage += 1.5 * f64::from(fall_distance);
        }

        if !victim
            .damage_with_context(
                &*victim,
                damage as f32,
                if is_mace_smash {
                    DamageType::MACE_SMASH
                } else {
                    DamageType::PLAYER_ATTACK
                },
                None,
                Some(self),
                Some(self),
            )
            .await
        {
            world.play_sound_fine(
                Sound::EntityPlayerAttackNodamage,
                SoundCategory::Players,
                &self.living_entity.entity.pos.load(),
                0.5,
                1.0,
            );
            return;
        }

        if damage >= 100.0 {
            self.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::DealtOverkillDamage).await;
        }

        if let Some(enchantments) = item_stack
            .lock()
            .await
            .get_data_component::<EnchantmentsImpl>()
        {
            for (enchantment, level) in enchantments.enchantment.iter() {
                if **enchantment == Enchantment::FIRE_ASPECT {
                    victim_entity.set_on_fire_for_ticks(*level as u32 * 80);
                }
            }
        }

        if is_mace_smash {
            let fall_distance = self.living_entity.fall_distance.load();
            self.living_entity.fall_distance.store(0.0);
            world.play_sound(
                if fall_distance > 5.0 {
                    Sound::ItemMaceSmashGroundHeavy
                } else {
                    Sound::ItemMaceSmashGround
                },
                SoundCategory::Players,
                &pos,
            );
        }

        player_attack_sound(&pos, &world, attack_type).await;

        self.living_entity.last_attacking_id.store(
            victim_entity.entity_id,
            std::sync::atomic::Ordering::Relaxed,
        );
        self.living_entity.last_attack_time.store(
            self.living_entity
                .entity
                .age
                .load(std::sync::atomic::Ordering::Relaxed),
            std::sync::atomic::Ordering::Relaxed,
        );

        if victim.get_living_entity().is_some() {
            let mut knockback_strength = 1.0 + f64::from(knockback_level);
            match attack_type {
                AttackType::Knockback => knockback_strength += 1.0,
                AttackType::Sweeping => {
                    combat::spawn_sweep_particle(attacker_entity, &world, &pos);

                    let mut sweep_damage = 1.0;
                    if let Some(enchantments) = item_stack
                        .lock()
                        .await
                        .get_data_component::<EnchantmentsImpl>()
                    {
                        for (enchantment, level) in enchantments.enchantment.iter() {
                            if **enchantment == Enchantment::SWEEPING_EDGE {
                                sweep_damage +=
                                    damage as f32 * (*level as f32 / (*level as f32 + 1.0));
                            }
                        }
                    }

                    let search_box = BoundingBox::new(
                        Vector3::new(pos.x - 1.0, pos.y - 0.5, pos.z - 1.0),
                        Vector3::new(pos.x + 1.0, pos.y + 0.5, pos.z + 1.0),
                    );
                    let victims = world.get_all_at_box(&search_box);
                    for other_victim in victims {
                        if other_victim.get_entity().entity_id != victim_entity.entity_id
                            && other_victim.get_entity().entity_id != attacker_entity.entity_id
                        {
                            other_victim
                                .damage_with_context(
                                    other_victim.as_ref(),
                                    sweep_damage,
                                    DamageType::PLAYER_ATTACK,
                                    None,
                                    Some(self),
                                    Some(self),
                                )
                                .await;
                        }
                    }
                }
                _ => {}
            }
            if config.knockback {
                // Vanilla LivingEntity.takeKnockback: strength *= (1 - knockbackResistance).
                // Must apply here — bare Entity has no attributes (golem KB=1.0 → unmovable).
                let strength = if let Some(living) = victim.get_living_entity() {
                    let kb_res = living
                        .get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE)
                        .clamp(0.0, 1.0);
                    knockback_strength * (1.0 - kb_res)
                } else {
                    knockback_strength
                };
                if strength > 0.0 {
                    combat::handle_knockback(attacker_entity, victim_entity, strength);
                }
            }
        }

        // NOTE: TOCTOU race condition in single-player context.
        // The weapon cost is computed (cost = 1 or 2) with item_stack locked, then damage_held_item
        // re-acquires the lock. In async multi-task scenarios, another task could theoretically
        // swap the held item between these operations, causing the cost to apply to the wrong item.
        // Mitigation options (in priority order):
        // 1. Create damage_held_item_with_lock(&self, item_stack: MutexGuard, amount) variant
        //    to hold the lock across both computation and application.
        // 2. Refactor compute cost as a closure: damage_held_item(self, |stack| -> i32 { ... })
        // 3. In practice, single-player scenarios are safe (this is not multiplayer). Document
        //    as a known limitation if refactoring is deemed too invasive.
        self.damage_held_item({
            let stack = item_stack.lock().await;
            Self::combat_weapon_durability_cost(&stack)
        })
        .await;

        if config.swing {}
    }

    /// Returns the durability cost for using the held item as a weapon in combat.
    /// Derived from the `Weapon` data component: items without it (e.g. shears, tools
    /// not designed for combat) take no durability damage on attack.
    /// Items with the component use its `item_damage_per_attack` value (default 1;
    /// axes, pickaxes, shovels, and hoes carry a value of 2).
    fn combat_weapon_durability_cost(stack: &ItemStack) -> i32 {
        stack
            .get_data_component::<WeaponImpl>()
            .map_or(0, |w| w.item_damage_per_attack as i32)
    }

    pub async fn sync_hand_slot(&self, slot_index: usize, stack: ItemStack) {
        self.enqueue_slot_set_packet(&CSetPlayerInventory::new(
            (slot_index as i32).into(),
            &ItemStackSerializer::from(stack.clone()),
        ))
        .await;

        if slot_index == self.inventory.get_selected_slot() as usize {
            self.living_entity
                .send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, stack)]);
        } else if slot_index == PlayerInventory::OFF_HAND_SLOT {
            self.living_entity
                .send_equipment_changes(&[(EquipmentSlot::OFF_HAND, stack)]);
        }
    }

    /// Applies `amount` durability damage to the item in `slot`.
    /// Broadcasts an [`EntityStatus`] break event and syncs the slot if the item is destroyed.
    pub async fn damage_item_in_slot(&self, slot: &EquipmentSlot, amount: i32) -> bool {
        if matches!(
            self.gamemode.load(),
            GameMode::Creative | GameMode::Spectator
        ) {
            return false;
        }

        // Direct PlayerInventory slot indices (matches build_equipment_slots).
        let slot_index: usize = match slot {
            EquipmentSlot::MainHand(_) => self.inventory.get_selected_slot() as usize,
            EquipmentSlot::OffHand(_) => PlayerInventory::OFF_HAND_SLOT, // 40
            EquipmentSlot::Feet(_) => 36,
            EquipmentSlot::Legs(_) => 37,
            EquipmentSlot::Chest(_) => 38,
            EquipmentSlot::Head(_) => 39,
            // Players do not have Body or Saddle equipment slots;
            // these are only used by non-player entities (e.g. horses).
            EquipmentSlot::Body(_) | EquipmentSlot::Saddle(_) => return false,
        };

        let stack_arc = self.inventory.get_stack(slot_index).await;

        let updated = {
            let mut stack = stack_arc.lock().await;
            let result = stack.damage_item(amount);
            (result != pumpkin_data::item_stack::DamageResult::Untouched)
                .then_some((result, stack.clone()))
        };

        if let Some((result, updated_stack)) = updated {
            // Send the break status before clearing the slot so the client can
            // use the item texture for break particles.
            if result == pumpkin_data::item_stack::DamageResult::Broken {
                self.increment_stat(
                    statistics::StatisticCategory::Broken,
                    updated_stack.item.id as i32,
                    1,
                )
                .await;
                self.world().send_entity_status(
                    &self.living_entity.entity,
                    crate::entity::equipment_break_status(slot),
                );
            }

            self.enqueue_slot_set_packet(&CSetPlayerInventory::new(
                (slot_index as i32).into(),
                &ItemStackSerializer::from(updated_stack.clone()),
            ))
            .await;

            self.living_entity
                .send_equipment_changes(&[(slot.clone(), updated_stack)]);

            return true;
        }

        false
    }

    /// Convenience wrapper – damages the currently held (main-hand) item.
    pub async fn damage_held_item(&self, amount: i32) -> bool {
        self.damage_item_in_slot(&EquipmentSlot::MAIN_HAND, amount)
            .await
    }

    pub async fn apply_tool_damage_for_block_break(&self, state: &BlockState) {
        if matches!(
            self.gamemode.load(),
            GameMode::Creative | GameMode::Spectator
        ) {
            return;
        }

        if state.hardness <= 0.0 {
            return;
        }

        let damage = {
            let stack = self.inventory.held_item();
            let stack = stack.lock().await;
            stack
                .get_data_component::<ToolImpl>()
                .map_or(0, |tool| tool.damage_per_block as i32)
        };

        if damage > 0 {
            self.damage_held_item(damage).await;
        }
    }

    pub fn get_attack_cooldown_progress(&self, tps: f64, base_time: f64, attack_speed: f64) -> f64 {
        let x = f64::from(self.last_attacked_ticks.load(Ordering::Acquire)) + base_time;

        let progress_per_tick = tps / attack_speed;
        let progress = x / progress_per_tick;
        progress.clamp(0.0, 1.0)
    }

    pub async fn can_harvest(&self, state: &BlockState, block: &'static Block) -> bool {
        !state.tool_required()
            || self
                .inventory
                .held_item()
                .lock()
                .await
                .is_correct_for_drops(block)
    }

    pub async fn get_mining_speed(&self, block: &'static Block) -> f32 {
        let mut speed = self.inventory.held_item().lock().await.get_speed(block);
        // Haste
        if self.living_entity.has_effect(&StatusEffect::HASTE).await
            || self
                .living_entity
                .has_effect(&StatusEffect::CONDUIT_POWER)
                .await
        {
            speed *= ((self.get_haste_amplifier().await + 1) as f32).mul_add(0.2, 1.0);
        }
        // Fatigue
        if let Some(fatigue) = self
            .living_entity
            .get_effect(&StatusEffect::MINING_FATIGUE)
            .await
        {
            let fatigue_speed = match fatigue.amplifier {
                0 => 0.3,
                1 => 0.09,
                2 => 0.0027,
                _ => 8.1E-4,
            };
            speed *= fatigue_speed;
        }
        // TODO: Handle when in water
        if !self.living_entity.entity.on_ground.load(Ordering::Relaxed) {
            speed /= 5.0;
        }
        speed
    }

    async fn get_haste_amplifier(&self) -> u32 {
        let mut i = 0;
        let mut j = 0;
        if let Some(effect) = self.living_entity.get_effect(&StatusEffect::HASTE).await {
            i = effect.amplifier;
        }
        if let Some(effect) = self
            .living_entity
            .get_effect(&StatusEffect::CONDUIT_POWER)
            .await
        {
            j = effect.amplifier;
        }
        u32::from(i.max(j))
    }

    /// Swing the hand of the player
    pub async fn swing_hand(&self, hand: Hand, all: bool) {
        let world = self.world();
        let entity_id = self.entity_id();

        let animation = match hand {
            Hand::Right => Animation::SwingMainArm,
            Hand::Left => Animation::SwingOffhand,
        };

        let je_packet = pumpkin_protocol::java::client::play::CEntityAnimation::new(
            VarInt(entity_id),
            animation,
        );

        let be_packet = pumpkin_protocol::bedrock::server::animate::SAnimate {
            action: pumpkin_protocol::bedrock::server::animate::AnimateAction::SwingArm,
            runtime_entity_id: pumpkin_protocol::codec::var_ulong::VarULong(entity_id as u64),
            data: 0.0,
            swing_source: None,
        };

        if all {
            world.broadcast_editioned(&je_packet, &be_packet).await;
        } else {
            world
                .broadcast_packet_except_editioned(&[self.gameprofile.id], &je_packet, &be_packet)
                .await;
        }
    }

    /// Start using an item (e.g. drawing a bow)
    pub fn start_using_item(&self, hand: Hand) {
        self.using_item.store(true, Ordering::Relaxed);
        self.item_use_start_time
            .store(self.tick_counter.load(Ordering::Relaxed), Ordering::Relaxed);
        self.using_hand.store(Some(hand));
    }

    /// Stop using an item
    pub fn stop_using_item(&self) {
        self.using_item.store(false, Ordering::Relaxed);
        self.using_hand.store(None);
    }

    /// Get the number of ticks the item has been in use
    pub fn get_item_use_ticks(&self) -> i32 {
        if !self.using_item.load(Ordering::Relaxed) {
            return 0;
        }
        self.tick_counter.load(Ordering::Relaxed) - self.item_use_start_time.load(Ordering::Relaxed)
    }
}
