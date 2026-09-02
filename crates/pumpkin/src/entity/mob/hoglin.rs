use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::world::World;

pub struct HoglinEntity {
    pub mob_entity: MobEntity,
    pub immune_to_zombification: AtomicBool,
    pub time_in_overworld: AtomicI32,
    pub cannot_be_hunted: AtomicBool,
    pub is_baby: AtomicBool,
}

impl HoglinEntity {
    pub const CONVERSION_TIME: i32 = 300;
    pub const XP_REWARD: u32 = 5;

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut attributes = mob_entity
                .living_entity
                .attributes
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(health) = attributes.get_mut(&Attributes::MAX_HEALTH.id) {
                health.base_value = 40.0;
                health.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
                speed.base_value = 0.3;
                speed.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(knockback_res) = attributes.get_mut(&Attributes::KNOCKBACK_RESISTANCE.id) {
                knockback_res.base_value = 0.6;
                knockback_res.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(attack_kb) = attributes.get_mut(&Attributes::ATTACK_KNOCKBACK.id) {
                attack_kb.base_value = 1.0;
                attack_kb.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(damage) = attributes.get_mut(&Attributes::ATTACK_DAMAGE.id) {
                damage.base_value = 6.0;
                damage.dirty.store(true, Ordering::Relaxed);
            }
        }
        mob_entity.living_entity.health.store(40.0);

        let hoglin = Self {
            mob_entity,
            immune_to_zombification: AtomicBool::new(false),
            time_in_overworld: AtomicI32::new(0),
            cannot_be_hunted: AtomicBool::new(false),
            is_baby: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(hoglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub fn is_immune_to_zombification(&self) -> bool {
        self.immune_to_zombification.load(Ordering::Relaxed)
    }

    pub fn set_immune_to_zombification(&self, immune: bool) {
        self.immune_to_zombification
            .store(immune, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::hoglin::DATA_IMMUNE_TO_ZOMBIFICATION,
                immune,
            )],
            None,
        );
    }

    pub fn is_converting(&self, world: &World) -> bool {
        !self.is_immune_to_zombification()
            && !self.mob_entity.is_no_ai()
            && world.dimension.minecraft_name != Dimension::THE_NETHER.minecraft_name
    }

    fn convert_to_zoglin(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        if world.level_info.load().difficulty != pumpkin_util::Difficulty::Peaceful {
            world.play_sound(
                Sound::EntityHoglinConvertedToZombified,
                SoundCategory::Hostile,
                &pos,
            );
        }

        let zoglin = crate::entity::r#type::from_type(
            &EntityType::ZOGLIN,
            pos,
            &world,
            uuid::Uuid::new_v4(),
        );

        let zoglin_base = zoglin.get_entity();
        zoglin_base.set_rotation(entity.yaw.load(), entity.pitch.load());
        zoglin_base.head_yaw.store(entity.head_yaw.load());
        zoglin_base.velocity.store(entity.velocity.load());

        if let Some(living) = zoglin.get_living_entity() {
            living.set_health(self.mob_entity.living_entity.health.load());
        }

        if let Some(custom_name) = &**entity.custom_name.load() {
            zoglin_base.set_custom_name(custom_name.clone());
        }

        world.spawn_entity(zoglin);
        entity.remove();
    }
}

impl Mob for HoglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        if self.is_immune_to_zombification() {
            nbt.put_bool("IsImmuneToZombification", true);
        }
        let time = self.time_in_overworld.load(Ordering::Relaxed);
        if time > 0 {
            nbt.put_int("TimeInOverworld", time);
        }
        if self.cannot_be_hunted.load(Ordering::Relaxed) {
            nbt.put_bool("CannotBeHunted", true);
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(immune) = nbt.get_bool("IsImmuneToZombification") {
            self.set_immune_to_zombification(immune);
        }
        if let Some(time) = nbt.get_int("TimeInOverworld") {
            self.time_in_overworld.store(time, Ordering::Relaxed);
        }
        if let Some(cannot_hunt) = nbt.get_bool("CannotBeHunted") {
            self.cannot_be_hunted.store(cannot_hunt, Ordering::Relaxed);
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let world = entity.world.load();
        if self.is_converting(&world) {
            let time = self.time_in_overworld.fetch_add(1, Ordering::Relaxed) + 1;
            if time > Self::CONVERSION_TIME {
                self.convert_to_zoglin();
            }
        } else {
            self.time_in_overworld.store(0, Ordering::Relaxed);
        }
    }

    fn on_attack(&self, target: &dyn EntityBase) {
        let my_pos = self.mob_entity.living_entity.entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let dx = target_pos.x - my_pos.x;
        let dz = target_pos.z - my_pos.z;
        let dist = dx.hypot(dz).max(0.001);
        let vel = target.get_entity().velocity.load();
        target.get_entity().velocity.store(Vector3::new(
            vel.x + (dx / dist) * 0.5,
            0.5,
            vel.z + (dz / dist) * 0.5,
        ));
    }
}
