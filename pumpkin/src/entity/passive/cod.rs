use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::{
    item::Item, meta_data_type::MetaDataType, sound::Sound, tracked_data::TrackedData,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_world::item::ItemStack;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::{
        goal::{
            fish_avoid_player::FishAvoidPlayerGoal,
            fish_helpers::{maybe_flop, try_bucket_mob_pickup},
            fish_panic::FishPanicGoal,
            fish_swim::FishSwimGoal,
            follow_school_leader::FollowSchoolLeaderGoal,
        },
        move_control::MoveControl,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

pub struct CodEntity {
    pub mob_entity: MobEntity,
    from_bucket: AtomicBool,
    school_leader_id: Arc<AtomicI32>,
}

impl CodEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let cod = Self {
            mob_entity: MobEntity::new(entity),
            from_bucket: AtomicBool::new(false),
            school_leader_id: Arc::new(AtomicI32::new(0)),
        };
        let mob_arc = Arc::new(cod);
        let school_leader_id = mob_arc.school_leader_id.clone();

        mob_arc.mob_entity.living_entity.movement_speed.store(0.1);
        *mob_arc.mob_entity.move_control.lock().await = MoveControl::fish(90);
        mob_arc.sync_from_bucket_metadata().await;

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            goal_selector.add_goal(0, Box::new(FishPanicGoal::new()));
            goal_selector.add_goal(2, Box::new(FishAvoidPlayerGoal::new()));
            goal_selector.add_goal(4, Box::new(FishSwimGoal::new(school_leader_id.clone())));
            goal_selector.add_goal(5, Box::new(FollowSchoolLeaderGoal::new(school_leader_id)));
        };

        mob_arc
    }

    fn is_from_bucket(&self) -> bool {
        self.from_bucket.load(Relaxed)
    }

    fn set_from_bucket(&self, from_bucket: bool) {
        self.from_bucket.store(from_bucket, Relaxed);
    }

    async fn sync_from_bucket_metadata(&self) {
        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_FROM_BUCKET,
                MetaDataType::Boolean,
                self.is_from_bucket(),
            )])
            .await;
    }
}

impl NBTStorage for CodEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.entity.write_nbt(nbt).await;
            nbt.put_bool("FromBucket", self.is_from_bucket());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity
                .living_entity
                .entity
                .read_nbt_non_mut(nbt)
                .await;
            self.set_from_bucket(nbt.get_bool("FromBucket").unwrap_or(false));
            self.sync_from_bucket_metadata().await;
        })
    }
}

impl Mob for CodEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.mob_entity.living_entity.dead.load(Relaxed) {
                return;
            }
            maybe_flop(self, Sound::EntityCodFlop).await;
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Player,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(
            async move { try_bucket_mob_pickup(self, player, item_stack, &Item::COD_BUCKET).await },
        )
    }
}
