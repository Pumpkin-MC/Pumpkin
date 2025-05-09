use crate::server::Server;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use async_trait::async_trait;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::client::play::{MetaDataType, Metadata};
use tokio::sync::Mutex;

use crate::entity::{
    Entity, EntityBase,
    ai::{
        goal::{look_at_entity::LookAtEntityGoal, target_goal::TargetGoal},
        path::Navigator,
    },
    living::LivingEntity,
};

use super::MobEntity;

pub struct ZombieEntity {
    mob: MobEntity,
    /// Indicates if this is a Baby Zombie
    baby: AtomicBool,
    /// Indicates if this is a drowned Zombie
    drowned: AtomicBool,
}

impl ZombieEntity {
    pub fn new(living_entity: LivingEntity, baby: bool, drowned: bool) -> Self {
        let mob = MobEntity {
            living_entity,
            goals: Mutex::new(vec![
                (Arc::new(LookAtEntityGoal::new(8.0)), false),
                (Arc::new(TargetGoal::new(16.0)), false),
            ]),
            navigator: Mutex::new(Navigator::default()),
        };
        Self {
            mob,
            baby: AtomicBool::new(baby),
            drowned: AtomicBool::new(drowned),
        }
    }
}

#[async_trait]
impl EntityBase for ZombieEntity {
    fn get_entity(&self) -> &Entity {
        self.mob.get_entity()
    }
    fn get_living_entity(&self) -> Option<&LivingEntity> {
        self.mob.get_living_entity()
    }
    async fn tick(&self, server: &Server) {
        self.mob.tick(server).await;
    }

    async fn init_data_tracker(&self) {
        self.mob
            .living_entity
            .entity
            .send_meta_data(&[
                Metadata::new(16, MetaDataType::Boolean, self.baby.load(Ordering::Relaxed)),
                // 1.10 also had the Zombie type (e.g Husk), But that was removed
                Metadata::new(
                    18,
                    MetaDataType::Boolean,
                    self.drowned.load(Ordering::Relaxed),
                ),
            ])
            .await;
    }

    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.mob.write_nbt(nbt).await;
        nbt.put_bool("IsBaby", self.baby.load(Ordering::Relaxed));
        // TODO
    }

    async fn read_nbt(&self, nbt: &NbtCompound) {
        self.mob.read_nbt(nbt).await;
        self.baby
            .store(nbt.get_bool("IsBaby").unwrap_or(false), Ordering::Relaxed);
        // TODO
    }
}
