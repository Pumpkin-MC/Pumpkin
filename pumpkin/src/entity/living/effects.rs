use super::LivingEntity;
use crate::entity::EntityBase;
use crate::entity::attributes::{AttributeInstance, Modifier, ModifierOperation};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::Operation;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::potion::Effect;
use pumpkin_data::tracked_data::{TrackedData, TrackedId};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CUpdateMobEffect, Metadata};
use std::sync::atomic::Ordering::{self, Relaxed};

impl LivingEntity {
    pub fn heal(&self, additional_health: f32) {
        assert!(additional_health > 0.0);
        self.set_health(self.health.load() + additional_health);
    }

    pub fn set_health(&self, health: f32) {
        // Clamp to [0, max_health]
        let max_health = self.get_max_health();
        let clamped = health.max(0.0).min(max_health);
        self.health.store(clamped);
        // tell everyone entities health changed
        self.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::HEALTH_ID,
                MetaDataType::FLOAT,
                clamped,
            )],
            None,
        );
    }

    /// Returns the current maximum health for this entity
    pub fn get_max_health(&self) -> f32 {
        self.get_attribute_value(&Attributes::MAX_HEALTH) as f32
    }

    /// Sets the maximum health for this entity
    pub async fn set_max_health(&self, max_health: f32) {
        // Update base attribute
        self.set_attribute_base(&Attributes::MAX_HEALTH, max_health as f64);

        // Broadcast the attribute change
        crate::entity::attributes::send_attribute_updates_for_living(
            self,
            vec![Attributes::MAX_HEALTH],
        )
        .await;

        // Clamp current health to new max if needed and send metadata update
        let current_health = self.health.load();
        if current_health > max_health {
            self.set_health(max_health);
        }
    }

    /// Returns the current absorption amount for this entity (yellow hearts)
    pub fn get_absorption(&self) -> f32 {
        self.absorption.load()
    }

    /// Sets the current absorption amount for this entity (yellow hearts)
    pub async fn set_absorption(&self, new_abs: f32) {
        // Must be at least 0
        let new_abs = new_abs.max(0.0);

        // Set local state
        self.absorption.store(new_abs);

        // Broadcast attribute update for max_absorption so clients receive
        // the updated absorption value via the attribute packet.
        crate::entity::attributes::send_attribute_updates_for_living(
            self,
            vec![Attributes::MAX_ABSORPTION],
        )
        .await;

        // Send absorption metadata for players (visual yellow hearts)
        if let Some(tracked_id) = self.player_absorption_id() {
            self.entity.send_meta_data(
                &[Metadata::new(tracked_id, MetaDataType::FLOAT, new_abs)],
                None,
            );
        }
    }

    /// Returns the absorption ID for this (player) entity
    /// TODO: don't hardcode these here?
    fn player_absorption_id(&self) -> Option<TrackedId> {
        (self.entity.entity_type == &EntityType::PLAYER).then_some(TrackedId {
            v1_21: 17u8,
            v1_21_2: 17u8,
            v1_21_4: 17u8,
            v1_21_5: 17u8,
            v1_21_6: 17u8,
            v1_21_7: 17u8,
            v1_21_9: 17u8,
            v1_21_11: 17u8,
            v26_1: 17u8, // ?
            v26_2: 17u8,
        })
    }

    /// Convenience helper to mutate an attribute instance. Automatically inserts
    /// a new instance populated from the registry base if needed.
    pub fn update_attribute<F: FnOnce(&mut AttributeInstance)>(
        &self,
        attribute: &Attributes,
        f: F,
    ) {
        let mut map = self.attributes.write().unwrap();

        let inst = map.entry(attribute.id).or_insert_with(|| {
            let base = self
                .entity
                .entity_type
                .attributes
                .iter()
                .find(|a| a.0.id == attribute.id)
                .map_or_else(
                    || {
                        tracing::warn!(
                            "Entity type {:?} has no base value for attribute {:?}; falling back to default {}",
                            self.entity.entity_type,
                            attribute.id,
                            attribute.default_value,
                        );
                        attribute.default_value
                    },
                    |a| a.1,
                );
            AttributeInstance::new(base)
        });

        f(inst);
        inst.dirty.store(true, Ordering::Relaxed);
    }

    /// Returns the computed value for `attribute` using the local instance, falling back
    /// to `attribute.default_value` if no local instance exists.
    pub fn get_attribute_value(&self, attribute: &Attributes) -> f64 {
        let map = self.attributes.read().unwrap();
        map.get(&attribute.id)
            .map_or(attribute.default_value, AttributeInstance::value)
    }

    /// Returns the base attribute value for `attribute` for this entity's type.
    pub fn get_attribute_base(&self, attribute: &Attributes) -> f64 {
        // Check the local base value first (could be modified)
        let map = self.attributes.read().unwrap();
        if let Some(instance) = map.get(&attribute.id) {
            return instance.base_value;
        }

        // Fall back to registry base value if no local instance exists
        self.entity
            .entity_type
            .attributes
            .iter()
            .find(|a| a.0.id == attribute.id)
            .unwrap()
            .1
    }

    /// Update or insert the base value for an attribute on this entity.
    /// If the attribute doesn't exist locally yet, it will be inserted.
    pub fn set_attribute_base(&self, attribute: &Attributes, new_base: f64) {
        let mut map = self.attributes.write().unwrap();
        if let Some(inst) = map.get_mut(&attribute.id) {
            inst.base_value = new_base;
            inst.dirty.store(true, Ordering::Relaxed);
        } else {
            let ai = AttributeInstance::new(new_base);
            ai.dirty.store(true, Ordering::Relaxed);
            map.insert(attribute.id, ai);
        }
    }

    pub async fn reset_effects_and_attributes(&self) {
        // Clear active effects and reset modified attributes
        let effects_to_remove: Vec<_> = {
            let lock = self.active_effects.lock().await;
            lock.keys().copied().collect()
        };

        for effect_type in effects_to_remove {
            self.remove_effect(effect_type).await;
        }
    }

    pub async fn add_effect(&self, effect: Effect) {
        // Apply instant effects immediately before storing
        if effect.effect_type == &StatusEffect::INSTANT_HEALTH {
            let heal_amount = 4.0 * (1 << effect.amplifier) as f32;
            self.heal(heal_amount);
        } else if effect.effect_type == &StatusEffect::INSTANT_DAMAGE {
            let damage_amount = 6.0 * (1 << effect.amplifier) as f32;
            if let Some(dyn_self) = self
                .entity
                .world
                .load()
                .get_entity_by_id(self.entity.entity_id)
            {
                dyn_self
                    .damage(&*dyn_self, damage_amount, DamageType::MAGIC)
                    .await;
            }
        } else {
            // Apply non-instant effects
            self.active_effects
                .lock()
                .await
                .insert(effect.effect_type, effect.clone());

            // Effects that modify attributes (ex. speed) should also update the
            // entity's attribute instances (server-side) and then notify clients.
            if !effect.effect_type.attribute_modifiers.is_empty() {
                // Apply each attribute modifier into the local AttributeInstance
                for m in effect.effect_type.attribute_modifiers {
                    let id = m.id.to_string();
                    let op = match m.operation {
                        Operation::AddValue => ModifierOperation::Add,
                        Operation::AddMultipliedBase => ModifierOperation::MultiplyBase,
                        Operation::AddMultipliedTotal => ModifierOperation::MultiplyTotal,
                    };
                    let scaled_amount = m.base_value * (f64::from(effect.amplifier) + 1.);
                    let mod_inst = Modifier {
                        id,
                        amount: scaled_amount,
                        operation: op,
                    };

                    self.update_attribute(m.attribute, |inst| {
                        inst.add_or_replace_modifier(mod_inst.clone());
                    });
                }

                // Recompute packet modifiers from active effects for each affected attribute
                let mut touched_attrs: Vec<pumpkin_data::attributes::Attributes> = Vec::new();
                for m in effect.effect_type.attribute_modifiers {
                    if !touched_attrs.iter().any(|a| a.id == m.attribute.id) {
                        touched_attrs.push(m.attribute.clone());
                    }
                }

                if !touched_attrs.is_empty() {
                    crate::entity::attributes::send_attribute_updates_for_living(
                        self,
                        touched_attrs,
                    )
                    .await;
                }
            }

            // Apply absorption effect (+4 absorption per level)
            if effect.effect_type == &StatusEffect::ABSORPTION {
                let added = 4.0 * (effect.amplifier as f32 + 1.0);
                let max_abs = self.get_attribute_value(&Attributes::MAX_ABSORPTION) as f32;
                let new_abs = (self.absorption.load() + added).min(max_abs);
                self.set_absorption(new_abs).await;
            }

            // Apply invisible effect
            if effect.effect_type == &StatusEffect::INVISIBILITY {
                self.entity.set_invisible(true).await;
            }

            // Apply glowing effect
            if effect.effect_type == &StatusEffect::GLOWING {
                self.entity.set_glowing(true).await;
            }
        }

        // Broadcast effect to nearby players
        let mut flag: i8 = 0;
        if effect.ambient {
            flag |= 1;
        }
        if effect.show_particles {
            flag |= 2;
        }
        if effect.show_icon {
            flag |= 4;
        }
        if effect.blend {
            flag |= 8;
        }

        let packet = CUpdateMobEffect::new(
            self.entity.entity_id.into(),
            VarInt(i32::from(effect.effect_type.id)),
            effect.amplifier.into(),
            effect.duration.into(),
            flag,
        );

        self.entity.world.load().broadcast_packet_all(&packet);
    }

    pub async fn remove_effect(&self, effect_type: &'static StatusEffect) -> bool {
        // Remove the effect
        let succeeded = self
            .active_effects
            .lock()
            .await
            .remove(&effect_type)
            .is_some();

        // Broadcast effect removal
        self.entity
            .world
            .load()
            .send_remove_mob_effect(&self.entity, effect_type);

        // Remove attribute modifiers, if any
        if !effect_type.attribute_modifiers.is_empty() {
            let mut touched_attrs = Vec::new();

            for m in effect_type.attribute_modifiers {
                let id = m.id.to_string();

                // Clean local server state
                self.update_attribute(m.attribute, |inst| {
                    inst.remove_modifier(&id);
                });

                // Track unique attributes for the packet update
                if !touched_attrs
                    .iter()
                    .any(|a: &Attributes| a.id == m.attribute.id)
                {
                    touched_attrs.push(m.attribute.clone());
                }
            }

            // Sync the clean state to the client
            if !touched_attrs.is_empty() {
                crate::entity::attributes::send_attribute_updates_for_living(self, touched_attrs)
                    .await;
            }
        }

        // If absorption effect removed, clear current absorption amount and notify clients
        if effect_type == &StatusEffect::ABSORPTION {
            self.set_absorption(0.0).await;
        }

        // If health boost effect removed, clamp current health to new max and notify clients
        if effect_type == &StatusEffect::HEALTH_BOOST {
            let new_max = self.get_max_health();
            if self.health.load() > new_max {
                // Update local health and send both health and absorption metadata together
                self.set_health(new_max.max(0.0));
            }
        }

        // If invisible effect removed, disable invisibility
        if effect_type == &StatusEffect::INVISIBILITY {
            self.entity.set_invisible(false).await;
        }

        // If glowing effect removed, disable glowing
        if effect_type == &StatusEffect::GLOWING {
            self.entity.set_glowing(false).await;
        }

        succeeded
    }

    pub async fn has_effect(&self, effect: &'static StatusEffect) -> bool {
        let effects = self.active_effects.lock().await;
        effects.contains_key(&effect)
    }

    pub async fn get_effect(&self, effect: &'static StatusEffect) -> Option<Effect> {
        let effects = self.active_effects.lock().await;
        effects.get(&effect).cloned()
    }

    pub(super) async fn tick_effects(&self) {
        let mut effects_to_remove = Vec::new();
        let mut effects_to_apply = Vec::new();

        {
            let mut effects = self.active_effects.lock().await;
            let entity_age = self.entity.age.load(Relaxed);
            for effect in effects.values_mut() {
                if effect.duration == 0 {
                    effects_to_remove.push(effect.effect_type);
                    continue;
                }

                let tick_duration = if effect.duration == -1 {
                    entity_age
                } else {
                    effect.duration
                };

                if Self::should_apply_effect_tick(effect, tick_duration) {
                    effects_to_apply.push((effect.effect_type, effect.amplifier));
                }

                if effect.duration != -1 {
                    effect.duration -= 1;
                }
            }
        }

        // Call the central removal function for each expired effect
        // This will now trigger your logs and absorption resets!
        for effect_type in effects_to_remove {
            self.remove_effect(effect_type).await;
        }

        for (effect_type, amplifier) in effects_to_apply {
            self.apply_effect_tick(effect_type, amplifier).await;
        }
    }

    /// Determines if an effect should apply its tick effect this frame
    /// Based on vanilla Minecraft's effect tick frequencies
    ///
    /// TODO: villager, beacon, and other effects.
    fn should_apply_effect_tick(effect: &pumpkin_data::potion::Effect, duration: i32) -> bool {
        let effect_type = effect.effect_type;

        if effect_type == &StatusEffect::REGENERATION {
            if duration <= 0 {
                return false;
            }
            let tick_rate = 50 >> effect.amplifier.min(4);
            duration % tick_rate == 0
        } else if effect_type == &StatusEffect::POISON {
            if duration <= 0 {
                return false;
            }
            let tick_rate = 25 >> effect.amplifier.min(4);
            duration % tick_rate == 0
        } else if effect_type == &StatusEffect::WITHER {
            if duration <= 0 {
                return false;
            }
            let tick_rate = 40 >> effect.amplifier.min(4);
            duration % tick_rate == 0
        } else if effect_type == &StatusEffect::HUNGER {
            // Hunger every 20 ticks
            duration % 20 == 0
        } else if effect_type == &StatusEffect::SATURATION {
            // Saturation every tick
            true
        } else {
            // Other effects that don't tick
            false
        }
    }

    /// Applies the actual effect to the entity
    /// This is called by `tick_effects` when an effect should trigger this tick
    async fn apply_effect_tick(&self, effect_type: &'static StatusEffect, amplifier: u8) {
        if effect_type == &StatusEffect::REGENERATION {
            let current_health = self.health.load();
            let max_health = self.get_max_health();
            if current_health < max_health && current_health > 0.0 {
                self.heal(1.0);
            }
        } else if effect_type == &StatusEffect::POISON {
            let current_health = self.health.load();
            if current_health > 1.0
                && let Some(dyn_self) = self
                    .entity
                    .world
                    .load()
                    .get_entity_by_id(self.entity.entity_id)
            {
                let damage_amount = (current_health - 1.0).min(1.0);
                if damage_amount > 0.0 {
                    dyn_self
                        .damage(&*dyn_self, damage_amount, DamageType::MAGIC)
                        .await;
                }
            }
        } else if effect_type == &StatusEffect::WITHER {
            let damage_amount = 1.0;
            if let Some(dyn_self) = self
                .entity
                .world
                .load()
                .get_entity_by_id(self.entity.entity_id)
            {
                dyn_self
                    .damage(&*dyn_self, damage_amount, DamageType::WITHER)
                    .await;
            }
        } else if effect_type == &StatusEffect::HUNGER {
            let world = self.entity.world.load();
            if let Some(entity) = world.get_entity_by_id(self.entity.entity_id)
                && let Some(player) = entity.get_player()
            {
                // Add exhaustion to trigger hunger decrease
                let exhaustion = 0.1 * (amplifier as f32 + 1.0);
                player.hunger_manager.add_exhaustion(exhaustion);
            }
            drop(world);
        } else if effect_type == &StatusEffect::SATURATION {
            let world = self.entity.world.load();
            if let Some(entity) = world.get_entity_by_id(self.entity.entity_id)
                && let Some(player) = entity.get_player()
            {
                // Add hunger and saturation
                let hunger = amplifier + 1;
                player.hunger_manager.add_hunger(hunger);
                player.hunger_manager.add_saturation(hunger as f32 * 2.0);
            }
        }
    }
}
