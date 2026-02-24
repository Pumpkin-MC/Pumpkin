use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::{
    entity::EntityType,
    item::Item,
    meta_data_type::MetaDataType,
    sound::{Sound, SoundCategory},
    tracked_data::TrackedData,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::Metadata};
use pumpkin_world::item::ItemStack;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, creeper_ignite::CreeperIgniteGoal,
        look_around::LookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

const DEFAULT_FUSE_TIME: i32 = 30;
const DEFAULT_EXPLOSION_RADIUS: i32 = 3;

pub struct CreeperEntity {
    pub mob_entity: MobEntity,
    pub fuse_speed: AtomicI32,
    pub current_fuse_time: AtomicI32,
    pub last_fuse_time: AtomicI32,
    pub fuse_time: AtomicI32,
    pub explosion_radius: AtomicI32,
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
            fuse_time: AtomicI32::new(DEFAULT_FUSE_TIME),
            explosion_radius: AtomicI32::new(DEFAULT_EXPLOSION_RADIUS),
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
            target_selector.add_goal(2, Box::new(RevengeGoal::new(true)));
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
                MetaDataType::INTEGER,
                VarInt(speed),
            )])
            .await;
    }

    async fn explode(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let radius = self.explosion_radius.load(Ordering::Relaxed) as f32;
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
        world.explode(pos, radius * multiplier).await;
        // TODO: spawn area effect cloud with potion effects
        entity.remove().await;
    }
}

impl NBTStorage for CreeperEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.entity.write_nbt(nbt).await;
            nbt.put_bool("powered", self.charged.load(Ordering::Relaxed));
            nbt.put_short("Fuse", self.fuse_time.load(Ordering::Relaxed) as i16);
            nbt.put_byte(
                "ExplosionRadius",
                self.explosion_radius.load(Ordering::Relaxed) as i8,
            );
            nbt.put_bool("ignited", self.ignited.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity
                .living_entity
                .entity
                .read_nbt_non_mut(nbt)
                .await;
            if let Some(powered) = nbt.get_bool("powered") {
                self.charged.store(powered, Ordering::Relaxed);
            }
            if let Some(fuse) = nbt.get_short("Fuse") {
                self.fuse_time.store(i32::from(fuse), Ordering::Relaxed);
            }
            if let Some(radius) = nbt.get_byte("ExplosionRadius") {
                self.explosion_radius
                    .store(i32::from(radius), Ordering::Relaxed);
            }
            if let Some(ignited) = nbt.get_bool("ignited") {
                self.ignited.store(ignited, Ordering::Relaxed);
            }
        })
    }
}

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
            }

            let fuse_time = self.fuse_time.load(Ordering::Relaxed);
            let new_fuse = (current + fuse_speed).max(0);
            self.current_fuse_time.store(new_fuse, Ordering::Relaxed);

            if new_fuse >= fuse_time {
                self.current_fuse_time.store(fuse_time, Ordering::Relaxed);
                self.explode().await;
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Player,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if item_stack.item.id != Item::FLINT_AND_STEEL.id {
                return false;
            }

            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let pos = entity.pos.load();

            world
                .play_sound_fine(
                    Sound::ItemFlintandsteelUse,
                    SoundCategory::Hostile,
                    &pos,
                    1.0,
                    rand::random::<f32>() * 0.4 + 0.8,
                )
                .await;

            self.ignited.store(true, Ordering::Relaxed);
            entity
                .send_meta_data(&[Metadata::new(
                    TrackedData::DATA_IGNITED,
                    MetaDataType::BOOLEAN,
                    true,
                )])
                .await;

            if player.gamemode.load() != pumpkin_util::GameMode::Creative {
                item_stack.damage_item_with_context(1, false);
            }

            true
        })
    }
}
