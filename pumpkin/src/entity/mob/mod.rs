use super::{Entity, EntityBase, NBTStorage, ai::pathfinder::Navigator, living::LivingEntity};
use crate::entity::EntityBaseFuture;
use crate::entity::ai::control::look_control::LookControl;
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::server::Server;
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::{CHeadRot, CUpdateEntityRot, Metadata};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};
use tokio::sync::Mutex;

pub mod creeper;
pub mod enderman;
pub mod silverfish;
pub mod skeleton;
pub mod zombie;
pub mod zombie_villager;

pub struct MobEntity {
    pub living_entity: LivingEntity,
    pub goals_selector: Mutex<GoalSelector>,
    pub target_selector: Mutex<GoalSelector>,
    pub navigator: Mutex<Navigator>,
    pub target: Mutex<Option<Arc<dyn EntityBase>>>,
    pub look_control: Mutex<LookControl>,
    pub position_target: AtomicCell<BlockPos>,
    pub position_target_range: AtomicI32,
    mob_flags: AtomicU8,
    last_sent_yaw: AtomicU8,
    last_sent_pitch: AtomicU8,
    last_sent_head_yaw: AtomicU8,
}

impl MobEntity {
    #[expect(dead_code)]
    const AI_DISABLED_FLAG: u8 = 1;
    #[expect(dead_code)]
    const LEFT_HANDED_FLAG: u8 = 2;
    const ATTACKING_FLAG: u8 = 4;

    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self {
            living_entity: LivingEntity::new(entity),
            goals_selector: Mutex::new(GoalSelector::default()),
            target_selector: Mutex::new(GoalSelector::default()),
            navigator: Mutex::new(Navigator::default()),
            target: Mutex::new(None),
            look_control: Mutex::new(LookControl::default()),
            position_target: AtomicCell::new(BlockPos::ZERO),
            position_target_range: AtomicI32::new(-1),
            mob_flags: AtomicU8::new(0),
            last_sent_yaw: AtomicU8::new(0),
            last_sent_pitch: AtomicU8::new(0),
            last_sent_head_yaw: AtomicU8::new(0),
        }
    }
    pub fn is_in_position_target_range(&self) -> bool {
        self.is_in_position_target_range_pos(&self.living_entity.entity.block_pos.load())
    }

    pub fn is_in_position_target_range_pos(&self, block_pos: &BlockPos) -> bool {
        let position_target_range = self.position_target_range.load(Relaxed);
        if position_target_range == -1 {
            true
        } else {
            self.position_target.load().squared_distance(block_pos)
                < position_target_range * position_target_range
        }
    }

    pub async fn set_attacking(&self, attacking: bool) {
        self.set_mob_flag(Self::ATTACKING_FLAG, attacking).await;
    }

    async fn set_mob_flag(&self, flag: u8, value: bool) {
        let old_b = self.mob_flags.load(Ordering::Relaxed);

        let new_b = if value { old_b | flag } else { old_b & !flag };

        if new_b != old_b {
            self.mob_flags.store(new_b, Ordering::Relaxed);

            self.living_entity
                .entity
                .send_meta_data(&[Metadata::new(
                    TrackedData::DATA_MOB_FLAGS,
                    MetaDataType::Byte,
                    new_b,
                )])
                .await;
        }
    }

    pub async fn is_in_attack_range(&self, target: &dyn EntityBase) -> bool {
        const DEFAULT_ATTACK_RANGE: f64 = 0.828_427_12; // sqrt(2.04) - 0.6

        // TODO: Implement DataComponent lookup for ATTACK_RANGE when components are ready
        let max_range = DEFAULT_ATTACK_RANGE;
        let min_range = 0.0;

        let target_hitbox = target.get_entity().bounding_box.load();

        let attack_box_max = self.get_attack_box(max_range).await;

        let intersects_max = attack_box_max.intersects(&target_hitbox);

        if !intersects_max {
            return false;
        }

        if min_range > 0.0 {
            let attack_box_min = self.get_attack_box(min_range).await;
            if attack_box_min.intersects(&target_hitbox) {
                return false;
            }
        }

        true
    }

    pub async fn try_attack(&self, caller: &dyn EntityBase, target: &dyn EntityBase) {
        // TODO: Use entity attributes for damage once implemented
        const ZOMBIE_ATTACK_DAMAGE: f32 = 3.0;

        target
            .damage_with_context(
                target,
                ZOMBIE_ATTACK_DAMAGE,
                DamageType::MOB_ATTACK,
                None,
                Some(caller),
                Some(caller),
            )
            .await;
    }

    async fn get_attack_box(&self, attack_range: f64) -> BoundingBox {
        let vehicle_lock = self.living_entity.entity.vehicle.lock().await;

        let base_box = vehicle_lock.as_ref().map_or_else(
            || self.living_entity.entity.bounding_box.load(),
            |vehicle| {
                let vehicle_box = vehicle.get_entity().bounding_box.load();
                let my_box = self.living_entity.entity.bounding_box.load();

                BoundingBox {
                    min: Vector3::new(
                        my_box.min.x.min(vehicle_box.min.x),
                        my_box.min.y,
                        my_box.min.z.min(vehicle_box.min.z),
                    ),
                    max: Vector3::new(
                        my_box.max.x.max(vehicle_box.max.x),
                        my_box.max.y,
                        my_box.max.z.max(vehicle_box.max.z),
                    ),
                }
            },
        );

        base_box.expand(attack_range, 0.0, attack_range)
    }
}

// This trait contains all overridable functions
pub trait Mob: EntityBase + Send + Sync {
    fn get_random(&self) -> rand::rngs::ThreadRng {
        rand::rng()
    }

    fn get_max_look_yaw_change(&self) -> f32 {
        10.0
    }

    fn get_max_look_pitch_change(&self) -> f32 {
        40.0
    }

    fn get_max_head_rotation(&self) -> f32 {
        75.0
    }

    fn get_mob_entity(&self) -> &MobEntity;

    fn get_path_aware_entity(&self) -> Option<&dyn PathAwareEntity> {
        None
    }
}

impl<T: Mob + Send + 'static> EntityBase for T {
    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let mob_entity = self.get_mob_entity();

            // AI runs BEFORE physics (vanilla order: goals → navigator → look → physics)
            let age = mob_entity.living_entity.entity.age.load(Relaxed);
            if (age + mob_entity.living_entity.entity.entity_id) % 2 != 0 && age > 1 {
                mob_entity
                    .target_selector
                    .lock()
                    .await
                    .tick_goals(self, false)
                    .await;
                mob_entity
                    .goals_selector
                    .lock()
                    .await
                    .tick_goals(self, false)
                    .await;
            } else {
                mob_entity.target_selector.lock().await.tick(self).await;
                mob_entity.goals_selector.lock().await.tick(self).await;
            }

            let mut navigator = mob_entity.navigator.lock().await;
            navigator.tick(&mob_entity.living_entity).await;
            drop(navigator);

            let mut look_control = mob_entity.look_control.lock().await;
            look_control.tick(self).await;
            drop(look_control);

            // Physics tick runs AFTER AI sets movement inputs
            mob_entity.living_entity.tick(caller, server).await;

            // Send rotation packets after look_control finalizes head_yaw and pitch
            let entity = &mob_entity.living_entity.entity;
            let yaw = (entity.yaw.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;
            let pitch = (entity.pitch.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;
            let head_yaw = (entity.head_yaw.load() * 256.0 / 360.0).rem_euclid(256.0) as u8;

            let last_yaw = mob_entity.last_sent_yaw.load(Relaxed);
            let last_pitch = mob_entity.last_sent_pitch.load(Relaxed);
            let last_head_yaw = mob_entity.last_sent_head_yaw.load(Relaxed);

            if yaw.abs_diff(last_yaw) >= 1 || pitch.abs_diff(last_pitch) >= 1 {
                let world = entity.world.load();
                world
                    .broadcast_packet_all(&CUpdateEntityRot::new(
                        entity.entity_id.into(),
                        yaw,
                        pitch,
                        entity.on_ground.load(Relaxed),
                    ))
                    .await;
                mob_entity.last_sent_yaw.store(yaw, Relaxed);
                mob_entity.last_sent_pitch.store(pitch, Relaxed);
            }

            if head_yaw.abs_diff(last_head_yaw) >= 1 {
                let world = entity.world.load();
                world
                    .broadcast_packet_all(&CHeadRot::new(entity.entity_id.into(), head_yaw))
                    .await;
                mob_entity.last_sent_head_yaw.store(head_yaw, Relaxed);
            }
        })
    }

    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            self.get_mob_entity()
                .living_entity
                .damage_with_context(caller, amount, damage_type, position, source, cause)
                .await
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.get_mob_entity().living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.get_mob_entity().living_entity)
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn get_gravity(&self) -> f64 {
        self.get_mob_entity().living_entity.get_gravity()
    }
}

#[expect(dead_code)]
const DEFAULT_PATHFINDING_FAVOR: f32 = 0.0;

pub trait PathAwareEntity: Mob + Send + Sync {
    fn get_pathfinding_favor(&self, _block_pos: BlockPos, _world: Arc<World>) -> f32 {
        0.0
    }

    // TODO: missing SpawnReason attribute
    fn can_spawn(&self, world: Arc<World>) -> bool {
        self.get_pathfinding_favor(
            self.get_mob_entity().living_entity.entity.block_pos.load(),
            world,
        ) >= 0.0
    }

    fn is_navigation<'a>(&'a self) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async {
            let navigator = self.get_mob_entity().navigator.lock().await;
            !navigator.is_idle()
        })
    }

    // TODO: implement
    fn is_panicking(&self) -> bool {
        false
    }

    fn should_follow_leash(&self) -> bool {
        true
    }

    fn on_short_leash_tick(&self) {
        // TODO: implement
    }

    fn before_leash_tick(&self) {
        // TODO: implement
    }

    fn get_follow_leash_speed(&self) -> f32 {
        1.0
    }
}
