use super::{LivingEntity, bypasses_armor_durability};
use crate::entity::player::statistics::{CustomStatistic, StatisticCategory};
use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage};
use crate::server::Server;
use pumpkin_data::Enchantment;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::{
    AttributeModifiersImpl, EnchantmentsImpl, EquipmentSlot, FoodImpl,
};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::{DamageResult, ItemStack};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_protocol::bedrock::server::actor_event::{ActorEventType, SActorEvent};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::java::client::play::{CDamageEvent, CEntityStatus, CHurtAnimation};
use pumpkin_util::GameMode;
use pumpkin_util::Hand;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;
use std::sync::atomic::Ordering::{self, Relaxed};
use tracing::info;

impl EntityBase for LivingEntity {
    #[allow(clippy::too_many_lines)]
    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let mut amount = amount;

            // Check invulnerability before applying damage
            if self.entity.is_invulnerable_to(&damage_type).await {
                return false;
            }

            if self.entity.removed.load(Ordering::Relaxed) || self.entity.is_removed() {
                return false; // Already removed (despawn / unload race)
            }

            if self.health.load() <= 0.0 || self.dead.load(Relaxed) {
                return false; // Dying or dead
            }

            if amount < 0.0 {
                return false;
            }

            let world = self.entity.world.load();
            let is_fire_damage = damage_type == DamageType::IN_FIRE
                || damage_type == DamageType::ON_FIRE
                || damage_type == DamageType::LAVA
                || damage_type == DamageType::HOT_FLOOR;

            // Fire damage can be prevented by either game rules or fire resistance
            if is_fire_damage {
                // Check game rule for fire damage (only for players)
                if self.entity.entity_type == &EntityType::PLAYER
                    && !world.level_info.load().game_rules.fire_damage
                {
                    return false;
                }

                // Check for fire resistance effect
                if self.has_effect(&StatusEffect::FIRE_RESISTANCE).await {
                    return false;
                }
            }

            // Vanilla parity: entities in FREEZE_HURTS_EXTRA_TYPES take 5x freezing damage.
            if damage_type == DamageType::FREEZE
                && self
                    .entity
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_FREEZE_HURTS_EXTRA_TYPES)
            {
                amount *= 5.0;
            }

            // These damage types bypass the hurt cooldown and death protection
            let bypasses_cooldown_protection =
                damage_type == DamageType::GENERIC_KILL || damage_type == DamageType::OUT_OF_WORLD;

            let mut damage_after_armor = amount;
            if !bypasses_armor_durability(&damage_type) {
                let mut armor = 0.0f32;
                let mut toughness = 0.0f32;
                {
                    let equipment_lock = self.entity_equipment.lock().await;
                    for slot in [
                        EquipmentSlot::HEAD,
                        EquipmentSlot::CHEST,
                        EquipmentSlot::LEGS,
                        EquipmentSlot::FEET,
                    ] {
                        let stack_arc = equipment_lock.get(&slot);
                        let stack = stack_arc.lock().await;
                        if !stack.is_empty()
                            && let Some(modifiers) =
                                stack.get_data_component::<AttributeModifiersImpl>()
                        {
                            for modifier in modifiers.attribute_modifiers.iter() {
                                if modifier.r#type == &Attributes::ARMOR {
                                    armor += modifier.amount as f32;
                                } else if modifier.r#type == &Attributes::ARMOR_TOUGHNESS {
                                    toughness += modifier.amount as f32;
                                }
                            }
                        }
                    }
                }
                let value = 2.0f32 + toughness / 4.0;
                let clamped_armor = (armor - damage_after_armor / value)
                    .max(armor / 5.0)
                    .min(20.0);
                damage_after_armor *= 1.0 - clamped_armor / 25.0;
            }

            let mut damage_after_enchantments = damage_after_armor;
            if damage_type != DamageType::OUT_OF_WORLD {
                let mut epf = 0i32;
                {
                    let equipment_lock = self.entity_equipment.lock().await;
                    for slot in [
                        EquipmentSlot::HEAD,
                        EquipmentSlot::CHEST,
                        EquipmentSlot::LEGS,
                        EquipmentSlot::FEET,
                    ] {
                        let stack_arc = equipment_lock.get(&slot);
                        let stack = stack_arc.lock().await;
                        if !stack.is_empty()
                            && let Some(enchantments) =
                                stack.get_data_component::<EnchantmentsImpl>()
                        {
                            for (enchantment, level) in enchantments.enchantment.iter() {
                                let mut factor = 0;
                                let enc = *enchantment;
                                if enc == &Enchantment::PROTECTION {
                                    if damage_type != DamageType::DROWN
                                        && damage_type != DamageType::STARVE
                                        && damage_type != DamageType::GENERIC_KILL
                                    {
                                        factor = *level;
                                    }
                                } else if enc == &Enchantment::FIRE_PROTECTION {
                                    if is_fire_damage {
                                        factor = *level * 2;
                                    }
                                } else if enc == &Enchantment::BLAST_PROTECTION {
                                    if damage_type == DamageType::EXPLOSION
                                        || damage_type == DamageType::PLAYER_EXPLOSION
                                    {
                                        factor = *level * 2;
                                    }
                                } else if enc == &Enchantment::PROJECTILE_PROTECTION {
                                    if damage_type == DamageType::ARROW
                                        || damage_type == DamageType::MOB_PROJECTILE
                                        || damage_type == DamageType::THROWN
                                    {
                                        factor = (*level) * 2;
                                    }
                                } else if enc == &Enchantment::FEATHER_FALLING
                                    && damage_type == DamageType::FALL
                                {
                                    factor = (*level) * 4;
                                }
                                epf += factor;
                            }
                        }
                    }
                }
                epf = epf.min(20);
                if epf > 0 {
                    damage_after_enchantments *= 1.0 - (epf as f32 * 0.04);
                }
            }

            // Apply Resistance effect reduction (20% per level), excluding bypasses_cooldown_protection and starvation damage
            let resistance_reduction =
                if !bypasses_cooldown_protection && damage_type != DamageType::STARVE {
                    self.get_effect(&StatusEffect::RESISTANCE)
                        .await
                        .map_or(0.0, |e| 0.2 * (e.amplifier + 1) as f32)
                } else {
                    0.0
                };

            // Total damage after reductions
            let effective_amount = damage_after_enchantments * (1.0 - resistance_reduction);

            if resistance_reduction > 0.0 {
                let resisted = damage_after_enchantments * resistance_reduction;
                if let Some(player) = caller.get_player() {
                    player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageResisted as i32,
                            (resisted * 10.0) as i32,
                        )
                        .await;
                }
                if let Some(attacker_player) = cause.and_then(|c| c.get_player()) {
                    attacker_player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageDealtResisted as i32,
                            (resisted * 10.0) as i32,
                        )
                        .await;
                }
            }

            // Check for shield blocking
            if self.is_blocking().await
                && !damage_type.has_tag(&tag::DamageType::MINECRAFT_BYPASSES_SHIELD)
                && let Some(pos) = position
            {
                let player_pos = self.entity.pos.load();
                let look_vec = Vector3::rotation_vector(0.0, self.entity.yaw.load() as f64);
                let mut source_to_player = (player_pos - pos).normalize();
                source_to_player.y = 0.0;

                if source_to_player.dot(&look_vec) < 0.0 {
                    world.play_sound(Sound::ItemShieldBlock, SoundCategory::Players, &player_pos);

                    if let Some(player) = caller.get_player() {
                        player
                            .increment_stat(
                                StatisticCategory::Custom,
                                CustomStatistic::DamageBlockedByShield as i32,
                                (effective_amount * 10.0) as i32,
                            )
                            .await;

                        player.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::DeflectedDamage).await;
                    }

                    if let Some(attacker_player) = cause.and_then(|c| c.get_player()) {
                        let held_item = attacker_player.inventory().held_item();
                        let is_axe = held_item.lock().await.is_axe();
                        if is_axe {
                            let mut disable_chance = 0.25;
                            let is_sprinting = attacker_player
                                .living_entity
                                .entity
                                .sprinting
                                .load(Ordering::Relaxed);
                            if is_sprinting {
                                disable_chance = 1.0;
                            }

                            if rand::random::<f32>() < disable_chance
                                && let Some(victim_player) = caller.get_player()
                            {
                                victim_player
                                    .start_cooldown("minecraft:shield".to_string(), 100)
                                    .await;
                                self.clear_active_hand().await;

                                world.broadcast_packet_all(&CEntityStatus::new(
                                    self.entity.entity_id,
                                    30,
                                ));
                            }
                        }
                    }

                    let active_hand = self.active_hand.lock().await;
                    if let Some(hand) = *active_hand {
                        let slot = if hand == Hand::Left {
                            EquipmentSlot::MAIN_HAND
                        } else {
                            EquipmentSlot::OFF_HAND
                        };

                        let equipment_lock = self.entity_equipment.lock().await;
                        let stack_arc = equipment_lock.get(&slot);
                        let mut stack = stack_arc.lock().await;

                        // Vanilla shield blocks_attacks item_damage: threshold
                        // 3.0, base 1.0, factor 1.0 — hits under 3 damage cost
                        // no durability.
                        if amount < 3.0 {
                            return false;
                        }
                        let durability_damage = (1.0 + amount).floor() as i32;
                        if stack.damage_item(durability_damage) == DamageResult::Broken {
                            if let Some(player) = caller.get_player() {
                                player
                                    .increment_stat(
                                        StatisticCategory::Broken,
                                        stack.item.id as i32,
                                        1,
                                    )
                                    .await;
                            }
                            world.send_entity_status(
                                &self.entity,
                                crate::entity::equipment_break_status(&slot),
                            );
                            *stack = ItemStack::EMPTY.clone();
                            let broken_stack = stack.clone();
                            drop(stack);
                            drop(stack_arc);
                            drop(equipment_lock);

                            self.send_equipment_changes(&[(slot, broken_stack)]);
                            self.clear_active_hand().await;
                        }
                    }

                    return false;
                }
            }

            // Apply hurt cooldown logic
            let last_damage = self.last_damage_taken.load();
            let (damage_amount, play_sound) =
                if self.hurt_cooldown.load(Relaxed) > 10 && !bypasses_cooldown_protection {
                    if effective_amount <= last_damage {
                        return false;
                    }
                    (effective_amount - last_damage, false)
                } else {
                    self.hurt_cooldown.store(20, Relaxed);
                    (effective_amount, true)
                };

            // Finalize state
            self.last_damage_taken.store(amount);
            let damage_amount = damage_amount.max(0.0);

            let config = &world.server.upgrade().unwrap().advanced_config.pvp;

            if config.hurt_animation {
                let entity_id = self.entity.entity_id;
                let hurt_yaw = source.map_or(0.0, |source| {
                    let src = source.get_entity().pos.load();
                    let tgt = self.entity.pos.load();
                    (src.z - tgt.z).atan2(src.x - tgt.x).to_degrees() as f32
                        - self.entity.yaw.load()
                });
                let hurt_event = SActorEvent {
                    entity_runtime_id: VarLong(entity_id as i64),
                    event_type: ActorEventType::Hurt,
                    event_data: VarInt(0),
                    fire_at_position: None,
                };
                world
                    .broadcast_editioned(
                        &CHurtAnimation::new(VarInt(entity_id), hurt_yaw),
                        &hurt_event,
                    )
                    .await;
            }

            world.broadcast_packet_all(&CDamageEvent::new(
                self.entity.entity_id.into(),
                damage_type.id.into(),
                source.map(|e| e.get_entity().entity_id.into()),
                cause.map(|e| e.get_entity().entity_id.into()),
                position,
            ));

            // Try to spawn infested silverfish
            self.try_spawn_infested_silverfish().await;

            if play_sound {
                // Vanilla LivingEntity.playHurtSound: category from entity sound source,
                // volume 1.0, pitch ~1.0 ± 0.2.
                let pitch = {
                    use rand::RngExt;
                    let mut rng = rand::rng();
                    1.0 + (rng.random::<f32>() - rng.random::<f32>()) * 0.2
                };
                world.play_sound_fine(
                    self.hurt_sound(),
                    Self::sound_category_for_entity(self.entity.entity_type),
                    &self.entity.pos.load(),
                    1.0,
                    pitch,
                );

                if let Some(source) = source {
                    // Vanilla / Paper / Leaves: IronGolem.doHurtTarget fully overrides
                    // Mob.doHurtTarget and does NOT apply horizontal knockback here.
                    // It only adds vertical motion after a successful hit (see try_attack).
                    // Applying generic KB for golem attacks would be non-vanilla.
                    let attacker_is_iron_golem = source.get_entity().entity_type.id
                        == pumpkin_data::entity::EntityType::IRON_GOLEM.id;
                    if !attacker_is_iron_golem {
                        // LivingEntity.takeKnockback: strength *= 1 - knockbackResistance
                        let kb_res = self
                            .get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE)
                            .clamp(0.0, 1.0);
                        let strength = 0.4 * (1.0 - kb_res);
                        if strength > 0.0 {
                            let source_pos = source.get_entity().pos.load();
                            let target_pos = self.entity.pos.load();
                            let dx = source_pos.x - target_pos.x;
                            let dz = source_pos.z - target_pos.z;
                            self.entity.apply_knockback(strength, dx, dz);
                            self.entity.send_velocity();
                        }
                    }
                }
            }

            // Always record the attacker as soon as we accept a hit (even if
            // absorption eats all HP damage). RevengeGoal needs this so a near
            // mob hitting us steals focus from a far opportunistic target.
            if let Some(attacker) = cause.or(source) {
                self.last_attacker_id
                    .store(attacker.get_entity().entity_id, Relaxed);
                self.last_attacked_time
                    .store(self.entity.age.load(Relaxed), Relaxed);
            }

            // Consume absorption first, then apply remaining damage to health
            let mut remaining = damage_amount;
            let current_abs = self.absorption.load();
            if current_abs > 0.0 {
                let absorbed = current_abs.min(remaining);
                if let Some(player) = caller.get_player() {
                    player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageAbsorbed as i32,
                            (absorbed * 10.0) as i32,
                        )
                        .await;
                }

                if let Some(attacker_player) = cause.and_then(|c| c.get_player()) {
                    attacker_player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageDealtAbsorbed as i32,
                            (absorbed * 10.0) as i32,
                        )
                        .await;
                }

                if current_abs >= remaining {
                    let new_abs = current_abs - remaining;
                    self.set_absorption(new_abs).await;
                    remaining = 0.0;
                } else {
                    remaining -= current_abs;
                    self.set_absorption(0.0).await;
                }
            }

            // Apply remaining damage to health (clamped)
            let max_h = self.get_max_health();
            let health_before = self.health.load();
            let new_health = health_before - remaining;
            let clamped_health = new_health.max(0.0).min(max_h);
            if remaining > 0.0 {
                self.set_health(clamped_health);

                // Statistics updates
                if let Some(player) = caller.get_player() {
                    player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageTaken as i32,
                            (remaining * 10.0) as i32,
                        )
                        .await;
                }

                if let Some(attacker_player) = cause.and_then(|c| c.get_player()) {
                    attacker_player
                        .increment_stat(
                            StatisticCategory::Custom,
                            CustomStatistic::DamageDealt as i32,
                            (remaining * 10.0) as i32,
                        )
                        .await;
                }
            }

            // Check if the entity died and isn't protected by a death protection mechanic (ex. totem of undying)
            if clamped_health <= 0.0 {
                let protected =
                    !bypasses_cooldown_protection && self.try_use_death_protector(caller).await;
                if !protected {
                    if pumpkin_config::development_mode() {
                        info!(
                            entity = self.entity.entity_type.resource_name,
                            entity_id = self.entity.entity_id,
                            damage_type = damage_type.message_id,
                            source = source.map_or("none", |entity| entity
                                .get_entity()
                                .entity_type
                                .resource_name),
                            cause = cause.map_or("none", |entity| entity
                                .get_entity()
                                .entity_type
                                .resource_name),
                            raw_damage = amount,
                            effective_damage = effective_amount,
                            applied_damage = remaining,
                            health_before,
                            health_after = clamped_health,
                            "lethal damage"
                        );
                    }
                    if self
                        .try_convert_villager_on_zombie_kill(source, cause)
                        .await
                    {
                        return true;
                    }
                    self.on_death(damage_type, source, cause).await;
                }
            }

            // Armor durability is based on incoming raw damage, not post-absorption remaining.
            // Armor loses floor(raw_damage / 4) durability, minimum 1.
            // Not applied when the source is in `#minecraft:bypasses_armor`.
            if damage_amount > 0.0 && !bypasses_armor_durability(&damage_type) {
                self.damage_armor_items(caller, damage_amount).await;
            }

            true
        })
    }

    fn tick_in_void<'a>(&'a self, dyn_self: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            dyn_self
                .damage(dyn_self, 4.0, DamageType::OUT_OF_WORLD)
                .await;
        })
    }

    fn get_gravity(&self) -> f64 {
        self.get_attribute_value(&Attributes::GRAVITY)
    }

    #[allow(clippy::too_many_lines)]
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.entity.tick(caller, server).await;

            // Only tick movement if the entity is alive. This prevents a dead "corpse"
            // from continuing to be simulated (accumulating fall_distance/velocity).
            // We allow movement during death animation (20 ticks) so knockback is applied.
            let is_alive = !self.dead.load(Relaxed) && self.health.load() > 0.0;
            let in_death_animation =
                self.health.load() <= 0.0 && self.death_time.load(Relaxed) < 20;
            if is_alive || (in_death_animation && self.entity.entity_type != &EntityType::PLAYER) {
                self.tick_movement(server, caller).await;
                // Vanilla-like order: freeze logic runs after movement/collisions.
                self.entity.tick_frozen(caller.as_ref()).await;
            }

            // TODO
            let player = caller.get_player();
            let is_player = player.is_some();

            if !is_player {
                self.entity.send_pos_rot();
            }

            // Fetch supporting blocks for players or other entities
            let supporting_pos = caller.get_player().map_or_else(
                || self.entity.get_supporting_block_pos(),
                crate::entity::player::Player::get_supporting_block_pos,
            );

            // Notify the block under the entity each tick if a supporting block position is found
            if let Some(supporting) = supporting_pos {
                let world = self.entity.world.load();
                let (block, state) = world.get_block_and_state(&supporting);

                world
                    .block_registry
                    .on_entity_step(
                        block,
                        &world,
                        caller.as_ref() as &dyn EntityBase,
                        &supporting,
                        state,
                        false,
                    )
                    .await;

                // Check slightly below supporting_pos for additional supporting blocks (blocks under carpets and the like)
                if !block.is_solid() {
                    let below_supporting = supporting.down();
                    let (below_block, below_state) = world.get_block_and_state(&below_supporting);

                    // If block is not air, notify it as well
                    world
                        .block_registry
                        .on_entity_step(
                            below_block,
                            &world,
                            caller.as_ref() as &dyn EntityBase,
                            &below_supporting,
                            below_state,
                            true, // below supporting block
                        )
                        .await;
                }
            }

            self.tick_effects().await;

            // Current active item
            {
                let item_in_use = self.item_in_use.lock().await.clone();
                if let Some(item) = item_in_use.as_ref()
                    && self.item_use_time.fetch_sub(1, Ordering::Relaxed) <= 0
                {
                    // Consume item
                    let mut is_potion = false;
                    if let Some(food) = item.get_data_component::<FoodImpl>()
                        && let Some(player) = caller.get_player()
                    {
                        player
                            .hunger_manager
                            .eat(player, food.nutrition as u8, food.saturation)
                            .await;

                        // Special food effects
                        if item.item == &Item::GOLDEN_APPLE {
                            self.add_effect(pumpkin_data::potion::Effect {
                                effect_type: &pumpkin_data::effect::StatusEffect::REGENERATION,
                                amplifier: 1,
                                duration: 100,
                                ambient: false,
                                show_particles: true,
                                show_icon: true,
                                blend: false,
                            })
                            .await;
                            self.add_effect(pumpkin_data::potion::Effect {
                                effect_type: &pumpkin_data::effect::StatusEffect::ABSORPTION,
                                amplifier: 0,
                                duration: 2400,
                                ambient: false,
                                show_particles: true,
                                show_icon: true,
                                blend: false,
                            })
                            .await;
                        } else if item.item == &Item::ENCHANTED_GOLDEN_APPLE {
                            self.add_effect(pumpkin_data::potion::Effect {
                                effect_type: &pumpkin_data::effect::StatusEffect::REGENERATION,
                                amplifier: 1,
                                duration: 400,
                                ambient: false,
                                show_particles: true,
                                show_icon: true,
                                blend: false,
                            })
                            .await;
                            self.add_effect(pumpkin_data::potion::Effect {
                                effect_type: &pumpkin_data::effect::StatusEffect::ABSORPTION,
                                amplifier: 3,
                                duration: 2400,
                                ambient: false,
                                show_particles: true,
                                show_icon: true,
                                blend: false,
                            })
                            .await;
                            self.add_effect(pumpkin_data::potion::Effect {
                                effect_type: &pumpkin_data::effect::StatusEffect::RESISTANCE,
                                amplifier: 0,
                                duration: 6000,
                                ambient: false,
                                show_particles: true,
                                show_icon: true,
                                blend: false,
                            })
                            .await;
                            self.add_effect(pumpkin_data::potion::Effect {
                                effect_type: &pumpkin_data::effect::StatusEffect::FIRE_RESISTANCE,
                                amplifier: 0,
                                duration: 6000,
                                ambient: false,
                                show_particles: true,
                                show_icon: true,
                                blend: false,
                            })
                            .await;
                        }
                    }

                    // Handle potion consumption
                    if item.get_data_component::<pumpkin_data::data_component_impl::PotionContentsImpl>().is_some() {
                        let effects = crate::item::potion::PotionContents::read_potion_effects(item);
                        crate::item::potion::PotionContents::apply_effects_to(self, effects, 1.0, crate::item::potion::PotionApplicationSource::Normal).await;
                        is_potion = true;
                    }

                    if let Some(player) = caller.get_player() {
                        player
                            .trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::ConsumeItem {
                                item_id: format!("minecraft:{}", item.item.registry_key),
                            })
                            .await;

                        // Prefer modifying the exact stack that matches the consumed item:
                        // 1) selected hotbar (held_item)
                        // 2) off-hand
                        // 3) fallback to active_hand if the above didn't match
                        let mut handled = false;

                        // Check main hand (hotbar selected)
                        let held_arc = player.inventory.held_item();
                        {
                            let mut held_lock = held_arc.lock().await;
                            if held_lock.are_items_and_components_equal(item) {
                                if is_potion {
                                    if player.gamemode.load() != GameMode::Creative {
                                        held_lock.decrement(1);
                                        if held_lock.is_empty() {
                                            *held_lock = ItemStack::new(1, &Item::GLASS_BOTTLE);
                                        }
                                    }
                                } else {
                                    held_lock.decrement_unless_creative(player.gamemode.load(), 1);
                                }
                                handled = true;
                            }
                        }

                        if !handled {
                            // Check off-hand
                            let off_arc = player.inventory.off_hand_item().await;
                            let mut off_lock = off_arc.lock().await;
                            if off_lock.are_items_and_components_equal(item) {
                                if is_potion {
                                    if player.gamemode.load() != GameMode::Creative {
                                        off_lock.decrement(1);
                                        if off_lock.is_empty() {
                                            *off_lock = ItemStack::new(1, &Item::GLASS_BOTTLE);
                                        }
                                    }
                                } else {
                                    off_lock.decrement_unless_creative(player.gamemode.load(), 1);
                                }

                                handled = true;
                            }
                        }

                        if !handled {
                            // Use stored active_hand (as a fallback)
                            let active_hand = *self.active_hand.lock().await;
                            let hand_to_modify = active_hand.unwrap_or(Hand::Right);
                            let item_stack = self
                                .get_stack_in_hand(caller.as_ref(), hand_to_modify)
                                .await;
                            let mut item_lock = item_stack.lock().await;

                            if is_potion {
                                if player.gamemode.load() != GameMode::Creative {
                                    item_lock.decrement(1);
                                    if item_lock.is_empty() {
                                        *item_lock = ItemStack::new(1, &Item::GLASS_BOTTLE);
                                    }
                                }
                            } else {
                                item_lock.decrement_unless_creative(player.gamemode.load(), 1);
                            }
                        }

                        if let Some(cooldown) = item.get_use_cooldown() {
                            let group = cooldown
                                .cooldown_group
                                .clone()
                                .unwrap_or_else(|| item.item.registry_key.to_string());
                            player
                                .start_cooldown(group, (cooldown.seconds * 20.0) as i32)
                                .await;
                        }
                    }

                    self.clear_active_hand().await;
                }
            }

            if self.hurt_cooldown.load(Relaxed) > 0 {
                self.hurt_cooldown.fetch_sub(1, Relaxed);
            }
            if self.health.load() <= 0.0 {
                let time = self.death_time.fetch_add(1, Relaxed);
                // Only send death particles once (on the exact tick death_time reaches 20)
                // and then remove the entity, preventing entity_event spam.
                if time == 20 && !self.entity.removed.swap(true, Ordering::Relaxed) {
                    self.entity
                        .removal_reason
                        .store(Some(crate::entity::RemovalReason::Killed));
                    self.entity
                        .world
                        .load()
                        .send_entity_status(&self.entity, EntityStatus::Death);
                    self.entity.remove().await;
                }
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(self)
    }

    fn is_pushable(&self) -> bool {
        self.health.load() > 0.0 && !self.dead.load(Relaxed)
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
}
