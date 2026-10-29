use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::food::ConsumableImpl;
use pumpkin_data::data_component_impl::{EquipmentSlot, PotionDurationScaleImpl};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Potion;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        Controls, Goal, active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, ranged_attack::RangedAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, to_goal_ticks, wander_around::WanderAroundGoal,
    },
    attributes::{Modifier, ModifierOperation},
    custom_sound::CustomSound,
    living::LivingEntity,
    mob::{
        Mob, MobEntity, RangedAttackMob,
        patrol::{LongDistancePatrolGoal, PatrolData, PatrollingMonster},
        raider::{
            ObtainRaidLeaderBannerGoal, PathfindToRaidGoal, Raider, RaiderCelebrationGoal,
            RaiderData, RaiderMoveThroughVillageGoal,
        },
    },
    projectile::splash_potion::SplashPotionEntity,
};
use crate::world::World;
use crate::world::game_event::GameEventContext;

const SPEED_MODIFIER_DRINKING_ID: &str = "minecraft:drinking";
const SPEED_MODIFIER_DRINKING_AMOUNT: f64 = -0.25;
/// `NearestHealableRaiderTargetGoal.DEFAULT_COOLDOWN`.
const HEAL_RAIDERS_COOLDOWN: i32 = 200;

fn create_potion_stack(item: &'static Item, potion: &'static Potion) -> ItemStack {
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{DataComponentImpl, PotionContentsImpl};
    let mut stack = ItemStack::new(1, item);
    stack.patch.push((
        DataComponent::PotionContents,
        Some(
            PotionContentsImpl {
                potion_id: Some(i32::from(potion.id)),
                custom_color: None,
                custom_effects: Vec::new(),
                custom_name: None,
            }
            .to_dyn(),
        ),
    ));
    stack
}

/// Represents a Witch, a hostile ranged mob that throws splash potions and drinks restorative potions.
///
/// Wiki: <https://minecraft.wiki/w/Witch>
pub struct WitchEntity {
    pub mob_entity: MobEntity,
    pub raider_data: RaiderData,
    drinking_potion: AtomicBool,
    using_time: AtomicI32,
    /// The cooldown of the heal raiders target goal; while it runs the witch does not target
    /// players (vanilla `NearestAttackableWitchTargetGoal.canAttack`).
    heal_cooldown: AtomicI32,
}

impl WitchEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let witch = Self {
            mob_entity,
            raider_data: RaiderData::default(),
            drinking_potion: AtomicBool::new(false),
            using_time: AtomicI32::new(0),
            heal_cooldown: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(witch);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let ranged_weak: Weak<dyn RangedAttackMob> = {
            let ranged_arc: Arc<dyn RangedAttackMob> = mob_arc.clone();
            Arc::downgrade(&ranged_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let target_selector = &mob_arc.mob_entity.target_selector;

            // Vanilla registers PatrollingMonster's and Raider's goals first; the insertion
            // order breaks priority ties.
            goal_selector.add_goal(4, Box::new(LongDistancePatrolGoal::new(0.7, 0.595)));
            goal_selector.add_goal(1, Box::new(ObtainRaidLeaderBannerGoal));
            goal_selector.add_goal(3, Box::new(PathfindToRaidGoal::default()));
            goal_selector.add_goal(4, Box::new(RaiderMoveThroughVillageGoal::new(1.05)));
            goal_selector.add_goal(5, Box::new(RaiderCelebrationGoal));
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                2,
                Box::new(RangedAttackGoal::new(ranged_weak, 1.0, 60, 10.0)),
            );
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                1,
                Box::new(RevengeGoal::new(true).ignoring(|entity_type| {
                    entity_type.has_tag(&tag::EntityType::MINECRAFT_RAIDERS)
                })),
            );
            target_selector.add_goal(
                2,
                Box::new(NearestHealableRaiderTargetGoal::new(&mob_arc.mob_entity)),
            );
            target_selector.add_goal(
                3,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::PLAYER,
                    10,
                    true,
                    false,
                    None::<fn(&LivingEntity, &World) -> bool>,
                ))
                .gated_by(can_attack_players),
            );
        };

        mob_arc
    }

    pub fn set_drinking_potion(&self, drinking: bool) {
        self.drinking_potion.store(drinking, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .set_synced_data(tracked_data::witch::DATA_USING_ITEM, drinking);
    }

    #[must_use]
    pub fn is_drinking_potion(&self) -> bool {
        self.drinking_potion.load(Ordering::Relaxed)
    }

    fn set_main_hand(&self, stack: ItemStack) {
        let living = &self.mob_entity.living_entity;
        living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .put(&EquipmentSlot::MAIN_HAND, stack.clone());
        living.send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, stack)]);
    }

    fn play_sound_with_random_pitch(&self, sound: Sound) {
        let entity = &self.mob_entity.living_entity.entity;
        if entity.silent.load(Ordering::Relaxed) {
            return;
        }
        let pitch = 0.8 + rand::random::<f32>() * 0.4;
        entity.world.load().play_sound_raw(
            sound as u16,
            SoundCategory::Hostile,
            &entity.pos.load(),
            1.0,
            pitch,
        );
    }

    /// Vanilla `Witch.performRangedAttack`.
    pub fn throw_potion(&self, target: &Arc<dyn EntityBase>) {
        if self.is_drinking_potion() {
            return;
        }

        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load_full();

        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let target_vel = target_entity.velocity.load();
        let witch_pos = entity.pos.load();

        let xd = target_pos.x + target_vel.x - witch_pos.x;
        let yd = target_pos.y + target_entity.get_eye_height() - 1.1 - witch_pos.y;
        let zd = target_pos.z + target_vel.z - witch_pos.z;
        let dist = xd.hypot(zd);

        let mut potion = &Potion::HARMING;

        if let Some(target_living) = target.get_living_entity() {
            let is_raider = target
                .get_mob()
                .is_some_and(|mob| mob.as_raider().is_some());
            if is_raider {
                potion = if target_living.health.load() <= 4.0 {
                    &Potion::HEALING
                } else {
                    &Potion::REGENERATION
                };
                self.set_mob_target(None);
            } else if dist >= 8.0 && !target_living.has_effect(&StatusEffect::SLOWNESS) {
                potion = &Potion::SLOWNESS;
            } else if target_living.health.load() >= 8.0
                && !target_living.has_effect(&StatusEffect::POISON)
            {
                potion = &Potion::POISON;
            } else if dist <= 3.0
                && !target_living.has_effect(&StatusEffect::WEAKNESS)
                && rand::random::<f32>() < 0.25
            {
                potion = &Potion::WEAKNESS;
            }
        }

        let potion_stack = create_potion_stack(&Item::SPLASH_POTION, potion);

        let splash_entity = Entity::new(world.clone(), witch_pos, &EntityType::SPLASH_POTION);
        let splash = SplashPotionEntity::new_shot(splash_entity, entity);
        splash.set_item_stack(potion_stack);

        let speed = if dist <= 2.0 { 0.45 } else { 0.75 };
        let yo = dist * 0.2;

        splash.thrown.set_velocity(xd, yd + yo, zd, speed, 8.0);

        let splash_arc: Arc<dyn EntityBase> = Arc::new(splash);
        world.spawn_entity(splash_arc);

        self.play_sound_with_random_pitch(Sound::EntityWitchThrow);
    }

    /// The drink branch of vanilla `Witch.aiStep`: finish the potion in hand, or pick one.
    fn tick_drinking(&self) {
        if self.is_drinking_potion() {
            if self.using_time.fetch_sub(1, Ordering::Relaxed) <= 0 {
                self.finish_drinking();
            }
        } else if let Some(potion) = self.choose_potion_to_drink() {
            self.start_drinking(potion);
        }
    }

    /// Each condition rolls its own chance, like vanilla.
    fn choose_potion_to_drink(&self) -> Option<&'static Potion> {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        let on_fire = entity.fire_ticks.load(Ordering::Relaxed) > 0
            || living.get_last_damage_type().is_some_and(|damage_type| {
                damage_type.has_tag(&tag::DamageType::MINECRAFT_IS_FIRE)
            });

        if rand::random::<f32>() < 0.15
            && entity.is_submerged_in_water()
            && !living.has_effect(&StatusEffect::WATER_BREATHING)
        {
            Some(&Potion::WATER_BREATHING)
        } else if rand::random::<f32>() < 0.15
            && on_fire
            && !living.has_effect(&StatusEffect::FIRE_RESISTANCE)
        {
            Some(&Potion::FIRE_RESISTANCE)
        } else if rand::random::<f32>() < 0.05 && living.health.load() < living.get_max_health() {
            Some(&Potion::HEALING)
        } else if rand::random::<f32>() < 0.5
            && let Some(target) = self.mob_entity.get_target()
            && !living.has_effect(&StatusEffect::SPEED)
            && target
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&entity.pos.load())
                > 121.0
        {
            Some(&Potion::SWIFTNESS)
        } else {
            None
        }
    }

    fn start_drinking(&self, potion: &'static Potion) {
        let stack = create_potion_stack(&Item::POTION, potion);
        let use_duration = stack
            .get_data_component::<ConsumableImpl>()
            .map_or(0, ConsumableImpl::consume_ticks);
        self.set_main_hand(stack);
        self.using_time.store(use_duration, Ordering::Relaxed);
        self.set_drinking_potion(true);
        self.play_sound_with_random_pitch(Sound::EntityWitchDrink);

        self.mob_entity
            .living_entity
            .update_attribute(&Attributes::MOVEMENT_SPEED, |speed| {
                speed.remove_modifier(SPEED_MODIFIER_DRINKING_ID);
                speed.add_or_replace_modifier(Modifier {
                    id: SPEED_MODIFIER_DRINKING_ID.to_string(),
                    amount: SPEED_MODIFIER_DRINKING_AMOUNT,
                    operation: ModifierOperation::Add,
                });
            });
    }

    fn finish_drinking(&self) {
        let living = &self.mob_entity.living_entity;
        self.set_drinking_potion(false);
        let stack = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND);
        self.set_main_hand(ItemStack::EMPTY.clone());

        // Only a drinkable potion takes effect, scaled like vanilla's `forEachEffect`.
        if stack.item == &Item::POTION {
            let scale = stack
                .get_data_component::<PotionDurationScaleImpl>()
                .map_or(1.0, |scale| scale.scale);
            let effects = crate::item::potion::PotionContents::read_potion_effects(&stack);
            crate::item::potion::PotionContents::apply_effects_to(
                living,
                effects,
                scale,
                crate::item::potion::PotionApplicationSource::Normal,
            );
        }

        let entity = &living.entity;
        entity.world.load().emit_game_event_with_context(
            GameEvent::Drink.name(),
            entity.pos.load(),
            &GameEventContext {
                source_entity: Some(self),
            },
        );
        living.update_attribute(&Attributes::MOVEMENT_SPEED, |speed| {
            speed.remove_modifier(SPEED_MODIFIER_DRINKING_ID);
        });
    }
}

/// The gate of vanilla `NearestAttackableWitchTargetGoal`: no players while healing raiders.
fn can_attack_players(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<WitchEntity>()
        .is_none_or(|witch| witch.heal_cooldown.load(Ordering::Relaxed) <= 0)
}

/// Vanilla `NearestHealableRaiderTargetGoal`: a raider of the witch's active raid (not another
/// witch) on half of the attempts, then not again for the cooldown.
struct NearestHealableRaiderTargetGoal {
    target_goal: Box<ActiveTargetGoal>,
}

impl NearestHealableRaiderTargetGoal {
    fn new(mob: &MobEntity) -> Self {
        Self {
            // `canUse` skips the random interval and searches right away.
            target_goal: ActiveTargetGoal::predicated(mob, 0, true, |target, _| {
                let entity_type = target.entity.entity_type;
                entity_type.has_tag(&tag::EntityType::MINECRAFT_RAIDERS)
                    && entity_type != &EntityType::WITCH
            }),
        }
    }
}

impl Goal for NearestHealableRaiderTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(witch) = mob.cast_any().downcast_ref::<WitchEntity>() else {
            return false;
        };
        if witch.heal_cooldown.load(Ordering::Relaxed) > 0 || !mob.get_random().random::<bool>() {
            return false;
        }
        witch.has_active_raid() && self.target_goal.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(witch) = mob.cast_any().downcast_ref::<WitchEntity>() {
            witch
                .heal_cooldown
                .store(to_goal_ticks(HEAL_RAIDERS_COOLDOWN), Ordering::Relaxed);
        }
        self.target_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.target_goal.controls()
    }
}

impl Mob for WitchEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_patrolling_monster(&self) -> Option<&dyn PatrollingMonster> {
        Some(self)
    }

    fn as_raider(&self) -> Option<&dyn Raider> {
        Some(self)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(Sound::EntityWitchAmbient)
    }

    fn mob_write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.write_raider_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        self.read_raider_nbt(nbt);
    }

    fn pre_damage(&self, _damage_type: DamageType, source: Option<&dyn EntityBase>) -> bool {
        if let Some(src) = source
            && src.get_entity().entity_id == self.mob_entity.living_entity.entity.entity_id
        {
            return false;
        }
        true
    }

    fn modify_incoming_damage(&self, mut amount: f32, damage_type: DamageType) -> f32 {
        // Vanilla `Witch.getDamageAfterMagicAbsorb`.
        if damage_type.has_tag(&tag::DamageType::MINECRAFT_WITCH_RESISTANT_TO) {
            amount *= 0.15;
        }
        amount
    }

    /// Vanilla `Witch.aiStep`.
    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let living = &self.mob_entity.living_entity;
        if !living.is_alive() {
            return;
        }
        self.heal_cooldown.fetch_sub(1, Ordering::Relaxed);
        self.tick_drinking();

        if rand::random::<f32>() < 7.5E-4 {
            living.entity.world.load().send_entity_status(
                &living.entity,
                EntityStatus::WitchHatMagic,
                None,
            );
        }
    }
}

impl CustomSound for WitchEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityWitchHurt)
    }

    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityWitchDeath)
    }

    fn sound_category(&self) -> SoundCategory {
        SoundCategory::Hostile
    }
}

impl RangedAttackMob for WitchEntity {
    fn perform_ranged_attack(&self, target: &Arc<dyn EntityBase>, _power: f32) {
        self.throw_potion(target);
    }
}

impl PatrollingMonster for WitchEntity {
    fn get_patrol_data(&self) -> &PatrolData {
        &self.raider_data.patrol_data
    }

    fn can_be_leader(&self) -> bool {
        false
    }
}

impl Raider for WitchEntity {
    fn get_raider_data(&self) -> &RaiderData {
        &self.raider_data
    }

    fn get_celebrate_sound(&self) -> Sound {
        Sound::EntityWitchCelebrate
    }
}
