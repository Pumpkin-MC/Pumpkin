use super::LivingEntity;
use crate::entity::EntityBase;
use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::mob::Mob;
use crate::entity::mob::equipment::DEFAULT_EQUIPMENT_DROP_CHANCE;
use crate::entity::mob::zombie::zombie_villager::ZombieVillagerEntity;
use crate::entity::passive::villager::VillagerEntity;
use crate::entity::player::statistics::{CustomStatistic, StatisticCategory};
use crate::world::loot::{LootContextParameters, LootTableExt};
use pumpkin_data::damage::{DamageType, DeathMessageType};
use pumpkin_data::data_component_impl::{DeathProtectionImpl, EquipmentSlot, EquippableImpl};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityPose, EntityStatus, EntityType};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Enchantment, translation};
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetPlayerInventory;
use pumpkin_util::Hand;
use pumpkin_util::difficulty::Difficulty;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use rand::RngExt;
use std::mem;
use std::sync::Arc;
use std::sync::atomic::Ordering::{self, Relaxed};
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

impl LivingEntity {
    pub async fn get_death_message(
        dyn_self: &dyn EntityBase,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> TextComponent {
        match damage_type.death_message_type {
            DeathMessageType::Default => {
                if let Some(cause) = cause
                    && source.is_some()
                {
                    TextComponent::translate_cross(
                        format!("death.attack.{}.player", damage_type.message_id),
                        format!("death.attack.{}.player", damage_type.message_id),
                        [
                            dyn_self.get_display_name().await,
                            cause.get_display_name().await,
                        ],
                    )
                } else {
                    TextComponent::translate_cross(
                        format!("death.attack.{}", damage_type.message_id),
                        format!("death.attack.{}", damage_type.message_id),
                        [dyn_self.get_display_name().await],
                    )
                }
            }
            DeathMessageType::FallVariants => {
                //TODO
                TextComponent::translate_cross(
                    translation::java::DEATH_FELL_ACCIDENT_GENERIC,
                    translation::bedrock::DEATH_FELL_ACCIDENT_GENERIC,
                    [dyn_self.get_display_name().await],
                )
            }
            DeathMessageType::IntentionalGameDesign => TextComponent::text("[")
                .add_child(TextComponent::translate_cross(
                    format!("death.attack.{}.message", damage_type.message_id),
                    format!("death.attack.{}.message", damage_type.message_id),
                    [dyn_self.get_display_name().await],
                ))
                .add_child(TextComponent::text("]")),
        }
    }

    pub(super) async fn try_convert_villager_on_zombie_kill(
        &self,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        if self.entity.entity_type != &EntityType::VILLAGER {
            return false;
        }

        let Some(killer) = cause.or(source) else {
            return false;
        };
        let killer_type = killer.get_entity().entity_type.id;
        if killer_type != EntityType::ZOMBIE.id
            && killer_type != EntityType::HUSK.id
            && killer_type != EntityType::DROWNED.id
            && killer_type != EntityType::ZOMBIE_VILLAGER.id
            && killer_type != EntityType::ZOMBIFIED_PIGLIN.id
        {
            return false;
        }

        let world = self.entity.world.load().clone();
        let converts = match world.level_info.load().difficulty {
            Difficulty::Hard => true,
            Difficulty::Normal => rand::random(),
            Difficulty::Peaceful | Difficulty::Easy => false,
        };
        if !converts {
            return false;
        }

        let Some(victim) = world.get_entity_by_id(self.entity.entity_id) else {
            return false;
        };
        let Some(villager) = victim.cast_any().downcast_ref::<VillagerEntity>() else {
            return false;
        };

        // Same death latch as on_death: exactly one killer thread processes
        // this villager. Losing the race means another lethal hit is already
        // converting or running the death path — skip both here.
        if self
            .dead
            .compare_exchange(false, true, Relaxed, Relaxed)
            .is_err()
        {
            return true;
        }

        let source_entity = villager.get_entity();
        let custom_name = source_entity.custom_name.load().as_ref().clone();
        let custom_name_visible = source_entity.custom_name_visible.load(Ordering::Relaxed);
        let converted = ZombieVillagerEntity::from_villager(villager).await;
        let converted_entity = converted.get_entity();
        let converted_base: Arc<dyn EntityBase> = converted.clone();
        // Vanilla emits the infection event at the killing zombie's position.
        let block_pos = killer.get_entity().block_pos.load();

        // Conversion replaces the villager before normal death handling, so it
        // neither drops villager loot nor awards experience.
        world.remove_entity(victim.as_ref()).await;
        world.broadcast_entity_spawn(&converted_base);
        converted.mob_init_data_tracker().await;
        world.add_entity_silent(converted_base).await;

        if let Some(custom_name) = custom_name {
            converted_entity.set_custom_name(custom_name);
        }
        if custom_name_visible {
            converted_entity.set_custom_name_visible(true);
        }
        world.sync_world_event(WorldEvent::SoundZombieInfected, block_pos, 0);
        true
    }

    pub async fn on_death(
        &self,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        let world = self.entity.world.load();
        // Entity may already be removed (despawn race / concurrent tick). Never
        // panicking the whole server on death — soft-skip if not in world map.
        let Some(dyn_self) = world.get_entity_by_id(self.entity.entity_id) else {
            let pos = self.entity.pos.load();
            warn!(
                entity_id = self.entity.entity_id,
                entity_uuid = %self.entity.entity_uuid,
                entity_type = self.entity.entity_type.resource_name,
                x = pos.x,
                y = pos.y,
                z = pos.z,
                health = self.health.load(),
                death_time = self.death_time.load(Relaxed),
                age = self.entity.age.load(Relaxed),
                dead = self.dead.load(Relaxed),
                removed = self.entity.removed.load(Ordering::Relaxed),
                removal_reason = ?self.entity.removal_reason.load(),
                damage_type = damage_type.message_id,
                source_id = source.map(|s| s.get_entity().entity_id),
                source_type = source.map(|s| s.get_entity().entity_type.resource_name),
                cause_id = cause.map(|c| c.get_entity().entity_id),
                cause_type = cause.map(|c| c.get_entity().entity_type.resource_name),
                world_entities = world.entities.len(),
                "on_death: entity already removed from world; skipping death handling \
                 (likely concurrent despawn/remove during parallel entity tick)"
            );
            let _ = self.dead.compare_exchange(false, true, Relaxed, Relaxed);
            // Ensure removed flag is set so further concurrent ticks bail out.
            if !self.entity.removed.swap(true, Ordering::Relaxed) {
                self.entity
                    .removal_reason
                    .store(Some(crate::entity::RemovalReason::Discarded));
            }
            return;
        };
        if self
            .dead
            .compare_exchange(false, true, Relaxed, Relaxed)
            .is_ok()
        {
            world
                .emit_vibration(
                    crate::world::vibrations::Vibration::EntityDie,
                    self.entity.pos.load(),
                )
                .await;
            self.movement_input.store(Vector3::default());
            self.jumping.store(false, Relaxed);

            // Statistics updates
            self.update_death_stats(&*dyn_self, cause).await;

            // Plays the death sound
            world.send_entity_status(&self.entity, EntityStatus::Death);
            let looting_level;
            let tool = if let Some(cause_ent) = cause {
                if let Some(player) = cause_ent
                    .cast_any()
                    .downcast_ref::<crate::entity::player::Player>()
                {
                    let hand_stack = player
                        .inventory
                        .get_stack_in_hand(pumpkin_util::Hand::Right)
                        .await;
                    let stack_guard = hand_stack.lock().await;
                    looting_level = stack_guard
                        .get_enchantment_level(&Enchantment::LOOTING)
                        .max(0) as u32;
                    (stack_guard.item_count > 0).then(|| stack_guard.clone())
                } else {
                    looting_level = 0;
                    None
                }
            } else {
                looting_level = 0;
                None
            };

            let is_raining = world.is_raining().await;
            let is_thundering = world.is_thundering().await;

            let params = LootContextParameters {
                killed_by_player: cause.map(|c| c.get_entity().entity_type == &EntityType::PLAYER),
                this_entity: Some(self.entity.entity_type),
                killer_entity: cause.map(|c| c.get_entity().entity_type),
                direct_killer_entity: source.map(|s| s.get_entity().entity_type),
                position: Some(self.entity.pos.load()),
                world_time: world.level_info.load().day_time as u64,
                damage_type: Some(damage_type),
                tool,
                is_raining: Some(is_raining),
                is_thundering: Some(is_thundering),
                is_on_fire: Some(
                    self.entity
                        .fire_ticks
                        .load(std::sync::atomic::Ordering::Relaxed)
                        > 0,
                ),
                ..Default::default()
            };

            // Drop loot
            self.drop_loot(params.clone()).await;

            // Award experience
            if params.killed_by_player.unwrap_or(false)
                && world.level_info.load().game_rules.mob_drops
            {
                let amount = dyn_self.get_experience_reward(cause);
                if amount > 0 {
                    ExperienceOrbEntity::spawn(&world, self.entity.pos.load(), amount).await;
                }
            }
            self.entity.pose.store(EntityPose::Dying);

            self.drop_equipment(looting_level).await;

            // Broadcast death message if it's a player and the gamerule is enabled
            self.broadcast_death_message(&*dyn_self, damage_type, source, cause)
                .await;

            self.reset_effects_and_attributes().await;
        }
    }

    async fn drop_equipment(&self, looting_level: u32) {
        let world = self.entity.world.load();
        let block_pos = self.entity.block_pos.load();

        let drop_chances = self.equipment_drop_chances.lock().await;

        let slots_to_drop: Vec<EquipmentSlot> = {
            let mut slots: Vec<_> = self.equipment_slots.values().cloned().collect();
            slots.push(EquipmentSlot::MAIN_HAND);
            slots
        };

        for slot in &slots_to_drop {
            let mut chance = drop_chances
                .get(slot)
                .copied()
                .unwrap_or(DEFAULT_EQUIPMENT_DROP_CHANCE);
            // Vanilla approximation: EnchantmentHelper.processEquipmentDropChance
            // adds lootingLevel * 0.01 to the per-slot equipment drop chance.
            chance += looting_level as f32 * 0.01;
            chance = chance.min(1.0);
            if rand::random::<f32>() >= chance {
                continue;
            }
            let mut item = {
                let q = self.entity_equipment.lock().await;
                let item_arc = q.get(slot);
                let mut item_lock = item_arc.lock().await;
                mem::replace(&mut *item_lock, ItemStack::EMPTY.clone())
            };
            if item.is_empty() {
                continue;
            }
            // Vanilla approximation: Mob.dropCustomDeathLoot applies random
            // damage to dropped equipment using two chained random calls:
            // setDamageValue(maxDamage - random.nextInt(1 + random.nextInt(max(maxDamage - 3, 1))))
            if let Some(max_damage) = item.get_max_damage() {
                let mut rng = rand::rng();
                let inner = rng.random_range(0..(max_damage - 3).max(1));
                let outer = rng.random_range(0..=inner);
                item.set_damage((max_damage - outer).max(0));
            }
            world.drop_stack(&block_pos, item).await;
        }
    }

    async fn broadcast_death_message(
        &self,
        dyn_self: &dyn EntityBase,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        let world = self.entity.world.load();
        let show_death_messages = { world.level_info.load().game_rules.show_death_messages };
        if self.entity.entity_type == &EntityType::PLAYER && show_death_messages {
            if let Some(player) = dyn_self.get_player() {
                info!(
                    player = %player.gameprofile.name,
                    damage_type = damage_type.message_id,
                    "Player died"
                );
            }
            //TODO: KillCredit
            let death_message = Self::get_death_message(dyn_self, damage_type, source, cause).await;
            if let Some(server) = world.server.upgrade() {
                for player in server.get_all_players() {
                    player.send_system_message(&death_message).await;
                }
            }
        }
    }

    async fn update_death_stats(&self, dyn_self: &dyn EntityBase, cause: Option<&dyn EntityBase>) {
        if let Some(victim_player) = dyn_self.get_player() {
            victim_player
                .increment_stat(StatisticCategory::Custom, CustomStatistic::Deaths as i32, 1)
                .await;
            victim_player
                .set_stat(
                    StatisticCategory::Custom,
                    CustomStatistic::TimeSinceDeath as i32,
                    0,
                )
                .await;
            if let Some(killer_entity) = cause.map(EntityBase::get_entity) {
                victim_player
                    .increment_stat(
                        StatisticCategory::KilledBy,
                        killer_entity.entity_type.id as i32,
                        1,
                    )
                    .await;
            }
        }

        if let Some(killer_player) = cause.and_then(|c| c.get_player()) {
            if dyn_self.get_player().is_some() {
                killer_player
                    .increment_stat(
                        StatisticCategory::Custom,
                        CustomStatistic::PlayerKills as i32,
                        1,
                    )
                    .await;
            } else {
                killer_player
                    .increment_stat(
                        StatisticCategory::Custom,
                        CustomStatistic::MobKills as i32,
                        1,
                    )
                    .await;

                let resource_name = self.entity.entity_type.resource_name;
                let criterion_key = format!("minecraft:{resource_name}");
                killer_player
                    .trigger_advancement(
                        crate::entity::player::advancement::trigger::AdvancementTrigger::PlayerKilledEntity {
                            entity_type_resource: criterion_key,
                        }
                    )
                    .await;

                if resource_name == "skeleton" {
                    let distance_sq = killer_player
                        .position()
                        .squared_distance_to_vec(&self.entity.pos.load());
                    if distance_sq >= 2500.0 {
                        killer_player.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::SniperDuel).await;
                    }
                }

                if resource_name == "phantom" {
                    killer_player.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::TwoBirdsOneArrow).await;
                }

                let held_item = killer_player.inventory().held_item();
                let is_crossbow = {
                    let lock = held_item.lock().await;
                    lock.item.registry_key == "crossbow"
                };
                if is_crossbow {
                    killer_player.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::Arbalistic).await;
                }
            }
            killer_player
                .increment_stat(
                    StatisticCategory::Killed,
                    self.entity.entity_type.id as i32,
                    1,
                )
                .await;
        }
    }

    async fn drop_loot(&self, params: LootContextParameters) {
        if let Some(loot_table) = &self.get_entity().entity_type.loot_table {
            let pos = self.entity.block_pos.load();
            for stack in loot_table.get_loot(params) {
                self.entity.world.load().drop_stack(&pos, stack).await;
            }
        }
    }

    /// Tries to use a totem of undying from the entity's hands. If successful, applies the totem effects and returns true.
    pub(super) async fn try_use_death_protector(&self, caller: &dyn EntityBase) -> bool {
        for hand in Hand::all() {
            let stack = self.get_stack_in_hand(caller, hand).await;
            let mut stack = stack.lock().await;

            // Clear the stack and use the totem of undying
            if stack.get_data_component::<DeathProtectionImpl>().is_some() {
                stack.clear();
                self.set_health(1.0);
                self.entity
                    .world
                    .load()
                    .send_entity_status(&self.entity, EntityStatus::ProtectedFromDeath);

                // Set Absorption, Regeneration, and Fire Resistance effects
                self.add_effect(Effect {
                    effect_type: &StatusEffect::ABSORPTION,
                    duration: 100,
                    amplifier: 1,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;
                self.add_effect(Effect {
                    effect_type: &StatusEffect::REGENERATION,
                    duration: 900,
                    amplifier: 1,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;
                self.add_effect(Effect {
                    effect_type: &StatusEffect::FIRE_RESISTANCE,
                    duration: 800,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;

                return true;
            }
        }

        false
    }

    pub(super) async fn damage_armor_items(&self, caller: &dyn EntityBase, damage_amount: f32) {
        // Formula: armor loses floor(incoming_damage / 4) durability, minimum 1.
        let armor_damage = (damage_amount / 4.0).floor().max(1.0) as i32;
        let mut equipment_updates = Vec::new();

        // TODO: Falling anvil/stalactite should only damage the helmet slot.
        // TODO: Implement DAMAGE_RESISTANT component checks (e.g. netherite vs fire).

        let armor_slots: Vec<(usize, Arc<Mutex<ItemStack>>, EquipmentSlot)> = {
            let equipment_lock = self.entity_equipment.lock().await;
            self.equipment_slots
                .iter()
                .filter(|(_, slot)| slot.is_armor_slot())
                .map(|(index, slot)| (*index, equipment_lock.get(slot), slot.clone()))
                .collect()
        };

        for (slot_index, equipment, slot) in armor_slots {
            let (slot_result, updated_stack_opt) = {
                let mut stack = equipment.lock().await;
                if stack.is_empty() {
                    (pumpkin_data::item_stack::DamageResult::Untouched, None)
                } else {
                    // Items without `EquippableImpl` component take damage freely.
                    // Items with `damage_on_hurt: false` (e.g. elytra) are exempt from armor hit durability.
                    // PERF: Component lookup runs O(1) per armor slot (max 4 per hit). Caching
                    // at the item type level could optimize, but belongs in a broader caching pass.
                    let takes_damage = stack
                        .get_data_component::<EquippableImpl>()
                        .is_none_or(|equippable| equippable.damage_on_hurt);

                    if takes_damage {
                        // Base armor durability damage.
                        let result = stack.damage_item(armor_damage);
                        let changed = result != pumpkin_data::item_stack::DamageResult::Untouched;
                        (result, changed.then_some(stack.clone()))
                    } else {
                        // Equippable items can opt out of on-hurt durability loss (e.g. elytra).
                        (pumpkin_data::item_stack::DamageResult::Untouched, None)
                    }
                }
            };

            if let Some(updated_stack) = updated_stack_opt {
                // Broadcast break status before clearing the slot.
                if slot_result == pumpkin_data::item_stack::DamageResult::Broken {
                    let world = self.entity.world.load();
                    world.send_entity_status(
                        &self.entity,
                        crate::entity::equipment_break_status(&slot),
                    );
                }
                equipment_updates.push((slot.clone(), updated_stack.clone()));
                if let Some(player) = caller.get_player() {
                    player
                        .enqueue_slot_set_packet(&CSetPlayerInventory::new(
                            (slot_index as i32).into(),
                            &ItemStackSerializer::from(updated_stack),
                        ))
                        .await;
                }
            }
        }

        if !equipment_updates.is_empty() {
            self.send_equipment_changes(&equipment_updates);
        }
    }

    /// Try to spawn silverfish when this entity is infested and hurt.
    pub(super) async fn try_spawn_infested_silverfish(&self) {
        if !self.has_effect(&StatusEffect::INFESTED).await {
            return;
        }

        // Wither, ender dragon and silverfish are immune
        if self.entity.entity_type == &EntityType::WITHER
            || self.entity.entity_type == &EntityType::ENDER_DRAGON
            || self.entity.entity_type == &EntityType::SILVERFISH
        {
            return;
        }

        let world = self.entity.world.load();

        // 10% chance
        if rand::rng().random::<f32>() <= 0.1 {
            let count = rand::rng().random_range(1..3);
            for _ in 0..count {
                // Spawn at center of entity
                let bbox = self.entity.bounding_box.load();
                let center = Vector3::new(
                    f64::midpoint(bbox.min.x, bbox.max.x),
                    f64::midpoint(bbox.min.y, bbox.max.y),
                    f64::midpoint(bbox.min.z, bbox.max.z),
                );

                // Random direction
                let yaw_rad = self.entity.yaw.load().to_radians() as f64;
                let random_angle = rand::rng().random::<f64>() * std::f64::consts::PI
                    - std::f64::consts::FRAC_PI_2;
                let angle = yaw_rad + random_angle;
                let speed = 0.3f64;
                let dx = -angle.sin() * speed;
                let dz = angle.cos() * speed;
                let dy = 0.1f64;

                // Spawn
                let silver = crate::entity::r#type::from_type(
                    &EntityType::SILVERFISH,
                    center,
                    &world,
                    Uuid::new_v4(),
                );

                silver.get_entity().set_pos(center);
                silver.get_entity().velocity.store(Vector3::new(dx, dy, dz));

                world.spawn_entity(silver).await;

                // Play sound
                world.play_sound(Sound::EntitySilverfishHurt, SoundCategory::Players, &center);
            }
        }
    }
}
