use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity,
        patrol::{LongDistancePatrolGoal, PatrolData, PatrollingMonster},
        raider::{
            HoldGroundAttackGoal, PathfindToRaidGoal, Raider, RaiderCelebrationGoal, RaiderData,
            RaiderMoveThroughVillageGoal,
        },
    },
};

pub struct RavagerEntity {
    pub mob_entity: MobEntity,
    pub raider_data: RaiderData,
    attack_tick: AtomicI32,
    stunned_tick: AtomicI32,
    roar_tick: AtomicI32,
}

impl RavagerEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let ravager = Self {
            mob_entity,
            raider_data: RaiderData::default(),
            attack_tick: AtomicI32::new(0),
            stunned_tick: AtomicI32::new(0),
            roar_tick: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(ravager);
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
            goal_selector.add_goal(2, Box::new(HoldGroundAttackGoal::new(10.0)));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(4, Box::new(LongDistancePatrolGoal::new(0.7, 0.595)));
            goal_selector.add_goal(4, Box::new(RaiderMoveThroughVillageGoal::new(1.05)));
            goal_selector.add_goal(4, Box::new(PathfindToRaidGoal::default()));
            goal_selector.add_goal(5, Box::new(RaiderCelebrationGoal));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::VILLAGER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn is_immobile(&self) -> bool {
        self.attack_tick.load(Ordering::Relaxed) > 0
            || self.stunned_tick.load(Ordering::Relaxed) > 0
            || self.roar_tick.load(Ordering::Relaxed) > 0
    }

    pub fn stun(&self) {
        self.stunned_tick.store(40, Ordering::Relaxed);
        let pos = self.mob_entity.living_entity.entity.pos.load();
        self.mob_entity
            .living_entity
            .entity
            .world
            .load()
            .play_sound(Sound::EntityRavagerStunned, SoundCategory::Hostile, &pos);
        self.mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    pub fn roar(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load_full();
        let my_pos = entity.pos.load();
        let my_id = entity.entity_id;

        let roar_bb = entity.bounding_box.load().expand_all(4.0);
        let targets = world.get_entities_at_box(&roar_bb);

        for target in targets {
            let target_entity = target.get_entity();
            if target_entity.entity_id == my_id {
                continue;
            }
            if target_entity.entity_type == &EntityType::RAVAGER
                || target_entity.entity_type == &EntityType::PILLAGER
                || target_entity.entity_type == &EntityType::EVOKER
                || target_entity.entity_type == &EntityType::VINDICATOR
                || target_entity.entity_type == &EntityType::ILLUSIONER
                || target_entity.entity_type == &EntityType::WITCH
            {
                continue;
            }

            if let Some(living) = target.get_living_entity() {
                if !living.entity.is_alive() {
                    continue;
                }
                target.damage(target.as_ref(), 6.0, DamageType::MOB_ATTACK);

                // Vanilla strong knockback
                let t_pos = target_entity.pos.load();
                let xd = t_pos.x - my_pos.x;
                let zd = t_pos.z - my_pos.z;
                let dd = (xd * xd + zd * zd).max(0.001);
                let push_x = (xd / dd) * 4.0;
                let push_z = (zd / dd) * 4.0;
                let mut vel = target_entity.velocity.load();
                vel.x += push_x.clamp(-2.0, 2.0);
                vel.y += 0.2;
                vel.z += push_z.clamp(-2.0, 2.0);
                target_entity.set_velocity(vel);
            }
        }
    }
}

impl CustomSound for RavagerEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityRavagerDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityRavagerHurt)
    }
}

impl Mob for RavagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn as_patrolling_monster(&self) -> Option<&dyn PatrollingMonster> {
        Some(self)
    }

    fn as_raider(&self) -> Option<&dyn Raider> {
        Some(self)
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let world = entity.world.load_full();

        // Vanilla leaf trample griefing
        if world.level_info.load().game_rules.mob_griefing {
            let bb = entity.bounding_box.load().expand_all(0.2);
            let min_pos = bb.min_block_pos();
            let max_pos = bb.max_block_pos();
            for x in min_pos.0.x..=max_pos.0.x {
                for y in min_pos.0.y..=max_pos.0.y {
                    for z in min_pos.0.z..=max_pos.0.z {
                        let block_pos = BlockPos::new(x, y, z);
                        let block = world.get_block(&block_pos);
                        if block.has_tag(&tag::Block::MINECRAFT_LEAVES) {
                            world.break_block(&block_pos, None, BlockFlags::NOTIFY_ALL);
                        }
                    }
                }
            }
        }

        // Roar countdown
        let r_tick = self.roar_tick.load(Ordering::Relaxed);
        if r_tick > 0 {
            let next = r_tick - 1;
            self.roar_tick.store(next, Ordering::Relaxed);
            if next == 10 {
                self.roar();
            }
        }

        // Stunned countdown
        let s_tick = self.stunned_tick.load(Ordering::Relaxed);
        if s_tick > 0 {
            let next = s_tick - 1;
            self.stunned_tick.store(next, Ordering::Relaxed);
            if next == 0 {
                let pos = entity.pos.load();
                world.play_sound(
                    Sound::EntityRavagerRoar,
                    SoundCategory::Hostile,
                    &pos,
                );
                self.roar_tick.store(20, Ordering::Relaxed);
            }
        }

        if self.attack_tick.load(Ordering::Relaxed) > 0 {
            self.attack_tick.fetch_sub(1, Ordering::Relaxed);
        }

        // Ambient sound
        if self.roar_tick.load(Ordering::Relaxed) <= 0
            && self.stunned_tick.load(Ordering::Relaxed) <= 0
            && !entity.silent.load(Ordering::Relaxed)
            && rand::random_range(0..100) == 0
        {
            world.play_sound_fine(
                Sound::EntityRavagerAmbient,
                SoundCategory::Hostile,
                &entity.pos.load(),
                1.0,
                1.0,
            );
        }
    }

    fn on_attack(&self, target: &dyn EntityBase) {
        if self.roar_tick.load(Ordering::Relaxed) > 0 {
            return;
        }
        if let Some(living) = target.get_living_entity()
            && living.is_blocking()
        {
            if rand::random::<f32>() < 0.5 {
                self.stun();
            } else {
                let entity = &self.mob_entity.living_entity.entity;
                let my_pos = entity.pos.load();
                let target_entity = target.get_entity();
                let t_pos = target_entity.pos.load();
                let xd = t_pos.x - my_pos.x;
                let zd = t_pos.z - my_pos.z;
                let dd = (xd * xd + zd * zd).max(0.001);
                let push_x = (xd / dd) * 4.0;
                let push_z = (zd / dd) * 4.0;
                let mut vel = target_entity.velocity.load();
                vel.x += push_x.clamp(-2.0, 2.0);
                vel.y += 0.2;
                vel.z += push_z.clamp(-2.0, 2.0);
                target_entity.set_velocity(vel);
            }
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_raider_nbt(nbt);
        nbt.put_int("AttackTick", self.attack_tick.load(Ordering::Relaxed));
        nbt.put_int("StunTick", self.stunned_tick.load(Ordering::Relaxed));
        nbt.put_int("RoarTick", self.roar_tick.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_raider_nbt(nbt);
        if let Some(tick) = nbt.get_int("AttackTick") {
            self.attack_tick.store(tick, Ordering::Relaxed);
        }
        if let Some(tick) = nbt.get_int("StunTick") {
            self.stunned_tick.store(tick, Ordering::Relaxed);
        }
        if let Some(tick) = nbt.get_int("RoarTick") {
            self.roar_tick.store(tick, Ordering::Relaxed);
        }
    }
}

impl PatrollingMonster for RavagerEntity {
    fn get_patrol_data(&self) -> &PatrolData {
        &self.raider_data.patrol_data
    }

    fn can_be_leader(&self) -> bool {
        false
    }
}

impl Raider for RavagerEntity {
    fn get_raider_data(&self) -> &RaiderData {
        &self.raider_data
    }

    fn get_celebrate_sound(&self) -> Sound {
        Sound::EntityRavagerCelebrate
    }
}
