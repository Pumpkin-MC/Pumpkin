use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Potion;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, ranged_attack::RangedAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity, RangedAttackMob,
        patrol::{PatrolData, PatrollingMonster},
        raider::{
            PathfindToRaidGoal, Raider, RaiderCelebrationGoal, RaiderData,
            RaiderMoveThroughVillageGoal,
        },
    },
    projectile::splash_potion::SplashPotionEntity,
};

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
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                2,
                Box::new(RangedAttackGoal::new(ranged_weak, 1.0, 60, 10.0)),
            );
            goal_selector.add_goal(3, Box::new(RaiderMoveThroughVillageGoal::new(1.05)));
            goal_selector.add_goal(3, Box::new(PathfindToRaidGoal::default()));
            goal_selector.add_goal(4, Box::new(RaiderCelebrationGoal));
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub fn set_drinking_potion(&self, drinking: bool) {
        self.drinking_potion.store(drinking, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                tracked_data::witch::DATA_USING_ITEM,
                drinking,
            )],
            None,
        );
    }

    #[must_use]
    pub fn is_drinking_potion(&self) -> bool {
        self.drinking_potion.load(Ordering::Relaxed)
    }

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
            let r: f32 = rand::random();
            if dist >= 8.0 && !target_living.has_effect(&StatusEffect::SLOWNESS) {
                potion = &Potion::SLOWNESS;
            } else if target_living.health.load() >= 8.0
                && !target_living.has_effect(&StatusEffect::POISON)
            {
                potion = &Potion::POISON;
            } else if dist <= 3.0 && !target_living.has_effect(&StatusEffect::WEAKNESS) && r < 0.25
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

        if !entity.silent.load(Ordering::Relaxed) {
            world.play_sound(Sound::EntityWitchThrow, SoundCategory::Hostile, &witch_pos);
        }

        let splash_arc: Arc<dyn EntityBase> = Arc::new(splash);
        world.spawn_entity(splash_arc);
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
        if damage_type == DamageType::MAGIC
            || damage_type == DamageType::INDIRECT_MAGIC
            || damage_type == DamageType::THORNS
            || damage_type == DamageType::WITHER
        {
            amount *= 0.15;
        }
        amount
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        let living = &self.mob_entity.living_entity;
        let world = entity.world.load();

        if self.is_drinking_potion() {
            let remaining = self.using_time.fetch_sub(1, Ordering::Relaxed) - 1;
            if remaining <= 0 {
                self.set_drinking_potion(false);
                if let Some(witch) = caller.cast_any().downcast_ref::<Self>() {
                    let living = &witch.mob_entity.living_entity;
                    let mut equipment = living
                        .entity_equipment
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let stack = equipment.get(&EquipmentSlot::MAIN_HAND);
                    equipment.put(&EquipmentSlot::MAIN_HAND, ItemStack::EMPTY.clone());
                    drop(equipment);
                    living.send_equipment_changes(&[(
                        EquipmentSlot::MAIN_HAND,
                        ItemStack::EMPTY.clone(),
                    )]);

                    let effects = crate::item::potion::PotionContents::read_potion_effects(&stack);
                    crate::item::potion::PotionContents::apply_effects_to(
                        living,
                        effects,
                        1.0,
                        crate::item::potion::PotionApplicationSource::Normal,
                    );
                }
            }
        } else {
            let mut potion: Option<&'static Potion> = None;
            let r: f32 = rand::random();

            if r < 0.15
                && entity.touching_water.load(Ordering::Relaxed)
                && !living.has_effect(&StatusEffect::WATER_BREATHING)
            {
                potion = Some(&Potion::WATER_BREATHING);
            } else if r < 0.15
                && entity.fire_ticks.load(Ordering::Relaxed) > 0
                && !living.has_effect(&StatusEffect::FIRE_RESISTANCE)
            {
                potion = Some(&Potion::FIRE_RESISTANCE);
            } else if r < 0.05 && living.health.load() < living.get_max_health() {
                potion = Some(&Potion::HEALING);
            } else if r < 0.5
                && let Some(target) = self.mob_entity.get_target()
                && !living.has_effect(&StatusEffect::SPEED)
            {
                let target_pos = target.get_entity().pos.load();
                let self_pos = entity.pos.load();
                if self_pos.squared_distance_to_vec(&target_pos) > 121.0 {
                    potion = Some(&Potion::SWIFTNESS);
                }
            }

            if let Some(potion) = potion {
                let stack = create_potion_stack(&Item::POTION, potion);
                if let Some(witch) = caller.cast_any().downcast_ref::<Self>() {
                    let living = &witch.mob_entity.living_entity;
                    let mut equipment = living
                        .entity_equipment
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    equipment.put(&EquipmentSlot::MAIN_HAND, stack.clone());
                    drop(equipment);
                    living.send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, stack)]);
                }

                self.using_time.store(32, Ordering::Relaxed);
                self.set_drinking_potion(true);

                if !entity.silent.load(Ordering::Relaxed) {
                    let pos = entity.pos.load();
                    world.play_sound(Sound::EntityWitchDrink, SoundCategory::Hostile, &pos);
                }
            }
        }
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
