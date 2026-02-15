use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::{
    entity::EntityType,
    meta_data_type::MetaDataType,
    sound::{Sound, SoundCategory},
    tracked_data::TrackedData,
};
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::Metadata};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, creeper_ignite::CreeperIgniteGoal,
        look_around::LookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const FUSE_TIME: i32 = 30;
const EXPLOSION_RADIUS: f32 = 3.0;

pub struct CreeperEntity {
    pub mob_entity: MobEntity,
    pub fuse_speed: AtomicI32,
    pub current_fuse_time: AtomicI32,
    pub last_fuse_time: AtomicI32,
    pub ignited: AtomicBool,
    pub charged: AtomicBool,
}

impl CreeperEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let entity = Self {
            mob_entity,
            fuse_speed: AtomicI32::new(-1),
            current_fuse_time: AtomicI32::new(0),
            last_fuse_time: AtomicI32::new(0),
            ignited: AtomicBool::new(false),
            charged: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(entity);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(CreeperIgniteGoal::new(mob_arc.clone())));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, false)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.8)));

            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(LookAroundGoal::default()));

            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub async fn set_fuse_speed(&self, speed: i32) {
        self.fuse_speed.store(speed, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_FUSE_SPEED,
                MetaDataType::Integer,
                VarInt(speed),
            )])
            .await;
    }

    async fn explode(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let multiplier = if self.charged.load(Ordering::Relaxed) {
            2.0
        } else {
            1.0
        };
        self.mob_entity
            .living_entity
            .dead
            .store(true, Ordering::Relaxed);
        let world = entity.world.load();
        let pos = entity.pos.load();
        world.explode(pos, EXPLOSION_RADIUS * multiplier).await;
        // TODO: spawn area effect cloud with potion effects
        entity.remove().await;
    }
}

impl NBTStorage for CreeperEntity {}

impl Mob for CreeperEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            if !entity.is_alive() {
                return;
            }

            self.last_fuse_time.store(
                self.current_fuse_time.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );

            if self.ignited.load(Ordering::Relaxed) {
                self.set_fuse_speed(1).await;
            }

            let fuse_speed = self.fuse_speed.load(Ordering::Relaxed);
            let current = self.current_fuse_time.load(Ordering::Relaxed);

            if fuse_speed > 0 && current == 0 {
                let world = entity.world.load();
                world
                    .play_sound_fine(
                        Sound::EntityCreeperPrimed,
                        SoundCategory::Hostile,
                        &entity.pos.load(),
                        1.0,
                        0.5,
                    )
                    .await;
                // TODO: emit GameEvent::PRIME_FUSE
            }

            let new_fuse = (current + fuse_speed).max(0);
            self.current_fuse_time.store(new_fuse, Ordering::Relaxed);

            if new_fuse >= FUSE_TIME {
                self.current_fuse_time.store(FUSE_TIME, Ordering::Relaxed);
                self.explode().await;
            }
        })
    }
}
