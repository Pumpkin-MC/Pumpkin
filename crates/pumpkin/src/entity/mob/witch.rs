use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    ConsumableImpl, DataComponentImpl, EquipmentSlot, PotionContentsImpl,
};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Potion;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};

use crate::entity::attributes::{AttributeInstance, Modifier, ModifierOperation};
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        nearest_attackable_witch_target::NearestAttackableWitchTargetGoal,
        nearest_healable_raider_target::NearestHealableRaiderTargetGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal, witch_attack::WitchAttackGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::item::potion::{PotionApplicationSource, PotionContents};

/// Vanilla: `Witch.SPEED_MODIFIER_DRINKING_ID`/`SPEED_MODIFIER_DRINKING` (`-0.25`, `ADD_VALUE`).
const SPEED_MODIFIER_DRINKING_ID: &str = "witch_drinking";
/// Vanilla `Items.POTION`'s `Consumable` component: `consume_seconds: 1.6` (32 ticks), used as
/// the fallback use-duration if the component can't be read off the built stack for some reason.
const DEFAULT_POTION_USE_TICKS: i32 = 32;

pub struct WitchEntity {
    pub mob_entity: MobEntity,
    /// Vanilla: `NearestHealableRaiderTargetGoal.cooldown`. Kept on the entity (rather than
    /// inside the goal) so `mob_tick` can drive it externally, matching `Witch.aiStep` step 1.
    pub heal_cooldown: AtomicI32,
    /// Vanilla: `NearestAttackableWitchTargetGoal.canAttack`, set from `aiStep` step 1.
    pub can_attack_players: AtomicBool,
    /// Vanilla: `Witch.usingTime`.
    drink_ticks_remaining: AtomicI32,
}

impl WitchEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let witch = Self {
            mob_entity,
            heal_cooldown: AtomicI32::new(0),
            can_attack_players: AtomicBool::new(false),
            drink_ticks_remaining: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(witch);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let witch_weak = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(WitchAttackGoal::new(witch_weak, 60, 10.0)));
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));

            // Witch.java:72: `HurtByTargetGoal(this, Raider.class)`.
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true).exclude_raiders()));
            target_selector.add_goal(2, Box::new(NearestHealableRaiderTargetGoal::new()));
            target_selector.add_goal(
                3,
                Box::new(NearestAttackableWitchTargetGoal::new(&mob_arc.mob_entity)),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn is_drinking_potion(&self) -> bool {
        self.drink_ticks_remaining.load(Relaxed) > 0
    }

    /// Vanilla `Witch.aiStep`'s potion-drinking state machine (non-client only).
    async fn tick_drinking(&self) {
        if self.is_drinking_potion() {
            if self.drink_ticks_remaining.fetch_sub(1, Relaxed) - 1 <= 0 {
                self.drink_ticks_remaining.store(0, Relaxed);
                self.finish_drinking().await;
            }
            return;
        }
        self.maybe_start_drinking().await;
    }

    /// Vanilla: the `usingTime-- <= 0` branch -- clears the mainhand potion, applies its effects,
    /// and removes the drinking speed penalty.
    async fn finish_drinking(&self) {
        let living = &self.mob_entity.living_entity;
        let item = living
            .entity_equipment
            .lock()
            .await
            .get(&EquipmentSlot::MAIN_HAND)
            .lock()
            .await
            .clone();
        living
            .entity_equipment
            .lock()
            .await
            .put(&EquipmentSlot::MAIN_HAND, ItemStack::EMPTY.clone())
            .await;
        if item.item.registry_key == Item::POTION.registry_key {
            let effects = PotionContents::read_potion_effects(&item);
            PotionContents::apply_effects_to(living, effects, 1.0, PotionApplicationSource::Normal)
                .await;
        }
        let mut attributes = living.attributes.write().unwrap();
        if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
            speed.remove_modifier(SPEED_MODIFIER_DRINKING_ID);
        }
    }

    /// Vanilla: the `else` branch of the drinking state machine -- rolls the four candidate
    /// self-potion triggers in vanilla's exact priority order (only one can fire per tick).
    async fn maybe_start_drinking(&self) {
        let living = &self.mob_entity.living_entity;
        let mut roll = rand::random::<f32>();
        let has_water_breathing = living
            .get_effect(&StatusEffect::WATER_BREATHING)
            .await
            .is_some();
        let has_fire_resistance = living
            .get_effect(&StatusEffect::FIRE_RESISTANCE)
            .await
            .is_some();
        let has_speed = living.get_effect(&StatusEffect::SPEED).await.is_some();
        let health = living.health.load();
        let max_health = living.get_attribute_value(&Attributes::MAX_HEALTH) as f32;
        let target = self.mob_entity.target.lock().await.clone();
        let target_dist_sq = target.as_ref().map(|t| {
            t.get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&living.entity.pos.load())
        });

        // Vanilla evaluates these as an exclusive if/else-if chain, each gated by its own roll.
        let potion =
            if roll < 0.15 && living.entity.touching_water.load(Relaxed) && !has_water_breathing {
                Some(&Potion::WATER_BREATHING)
            } else if {
                roll = rand::random::<f32>();
                roll < 0.15
            } && living.entity.fire_ticks.load(Relaxed) > 0
                && !has_fire_resistance
            {
                Some(&Potion::FIRE_RESISTANCE)
            } else if {
                roll = rand::random::<f32>();
                roll < 0.05
            } && health < max_health
            {
                Some(&Potion::HEALING)
            } else if {
                roll = rand::random::<f32>();
                roll < 0.5
            } && target.is_some()
                && !has_speed
                && target_dist_sq.is_some_and(|d| d > 121.0)
            {
                Some(&Potion::SWIFTNESS)
            } else {
                None
            };

        let Some(potion) = potion else {
            return;
        };

        let stack = ItemStack::new_with_component(
            1,
            &Item::POTION,
            vec![(
                DataComponent::PotionContents,
                Some(
                    PotionContentsImpl {
                        potion_id: Some(potion.id as i32),
                        custom_color: None,
                        custom_effects: Vec::new(),
                        custom_name: None,
                    }
                    .to_dyn(),
                ),
            )],
        );
        let use_ticks = stack
            .get_data_component::<ConsumableImpl>()
            .map_or(DEFAULT_POTION_USE_TICKS, ConsumableImpl::consume_ticks);
        living
            .entity_equipment
            .lock()
            .await
            .put(&EquipmentSlot::MAIN_HAND, stack)
            .await;
        self.drink_ticks_remaining.store(use_ticks, Relaxed);

        living.entity.world.load().play_sound(
            Sound::EntityWitchDrink,
            SoundCategory::Hostile,
            &living.entity.pos.load(),
        );

        let base = living.get_attribute_base(&Attributes::MOVEMENT_SPEED);
        let mut attributes = living.attributes.write().unwrap();
        let speed = attributes
            .entry(Attributes::MOVEMENT_SPEED.id)
            .or_insert_with(|| AttributeInstance::new(base));
        speed.remove_modifier(SPEED_MODIFIER_DRINKING_ID);
        speed.add_or_replace_modifier(Modifier {
            id: SPEED_MODIFIER_DRINKING_ID.to_string(),
            amount: -0.25,
            operation: ModifierOperation::Add,
        });
    }
}

impl NBTStorage for WitchEntity {}

impl Mob for WitchEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla: `Witch.aiStep` (non-client-only steps): heal-cooldown/attack-gate, the
    /// potion-drinking state machine, and the ambient particle-puff roll.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let cooldown = self.heal_cooldown.fetch_sub(1, Relaxed) - 1;
            self.can_attack_players.store(cooldown <= 0, Relaxed);

            self.tick_drinking().await;

            // Vanilla: `7.5E-4F` chance per tick to broadcast entity event 15 (ambient
            // ground-particle puff, ornamental only -- no server-observable effect, so this is
            // skipped as a scope reduction rather than wired to a client entity-status event.
        })
    }

    /// Vanilla: `Witch.getDamageAfterMagicAbsorb`'s `WITCH_RESISTANT_TO`-tag 85% reduction.
    ///
    /// Scope reduction: the `damageSource.getEntity() == this` self-damage-zeroing branch is not
    /// ported -- `modify_incoming_damage` has no source/attacker parameter, and widening its
    /// signature would touch every `Mob` implementor for one mob's self-damage-immunity rule.
    fn modify_incoming_damage(&self, amount: f32, damage_type: DamageType) -> f32 {
        if damage_type.has_tag(&tag::DamageType::MINECRAFT_WITCH_RESISTANT_TO) {
            amount * 0.15
        } else {
            amount
        }
    }
}
