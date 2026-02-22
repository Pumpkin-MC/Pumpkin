use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, AtomicU8, Ordering::Relaxed},
};

use pumpkin_data::{
    entity::EntityType, item::Item, meta_data_type::MetaDataType, tracked_data::TrackedData,
};
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::Metadata};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::{
        goal::{
            bee_wander::BeeWanderGoal, escape_danger::EscapeDangerGoal,
            follow_parent::FollowParentGoal, look_around::LookAroundGoal,
            look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        },
        move_control::MoveControl,
    },
    mob::{Mob, MobEntity},
};

const BEE_TEMPT_ITEMS: &[&Item] = &[
    &Item::DANDELION,
    &Item::POPPY,
    &Item::BLUE_ORCHID,
    &Item::ALLIUM,
    &Item::AZURE_BLUET,
    &Item::RED_TULIP,
    &Item::ORANGE_TULIP,
    &Item::WHITE_TULIP,
    &Item::PINK_TULIP,
    &Item::OXEYE_DAISY,
    &Item::CORNFLOWER,
    &Item::LILY_OF_THE_VALLEY,
    &Item::TORCHFLOWER,
    &Item::SUNFLOWER,
    &Item::LILAC,
    &Item::ROSE_BUSH,
    &Item::PEONY,
    &Item::WITHER_ROSE,
];

const FLAG_ROLL: u8 = 2;
const FLAG_HAS_STUNG: u8 = 4;
const FLAG_HAS_NECTAR: u8 = 8;
const STING_DEATH_COUNTDOWN: i32 = 1200;

pub struct BeeEntity {
    pub mob_entity: MobEntity,
    bee_flags: AtomicU8,
    anger_end_time: AtomicI32,
    time_since_sting: AtomicI32,
    ticks_without_nectar: AtomicI32,
    stay_out_of_hive_countdown: AtomicI32,
    #[expect(dead_code)]
    under_water_ticks: AtomicI32,
}

impl BeeEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let bee = Self {
            mob_entity,
            bee_flags: AtomicU8::new(0),
            anger_end_time: AtomicI32::new(0),
            time_since_sting: AtomicI32::new(0),
            ticks_without_nectar: AtomicI32::new(0),
            stay_out_of_hive_countdown: AtomicI32::new(0),
            under_water_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(bee);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        mob_arc.mob_entity.living_entity.movement_speed.store(0.3);

        *mob_arc.mob_entity.move_control.lock().await = MoveControl::flying(20, true);

        mob_arc.sync_bee_flags_metadata().await;
        mob_arc.sync_anger_metadata().await;

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            // Priority order adapted from vanilla registerGoals()
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, BEE_TEMPT_ITEMS)));
            goal_selector.add_goal(5, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(7, Box::new(BeeWanderGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(9, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }

    fn get_flag(&self, flag: u8) -> bool {
        (self.bee_flags.load(Relaxed) & flag) != 0
    }

    fn set_flag(&self, flag: u8, value: bool) {
        let old = self.bee_flags.load(Relaxed);
        let new = if value { old | flag } else { old & !flag };
        self.bee_flags.store(new, Relaxed);
    }

    pub fn has_nectar(&self) -> bool {
        self.get_flag(FLAG_HAS_NECTAR)
    }

    pub fn has_stung(&self) -> bool {
        self.get_flag(FLAG_HAS_STUNG)
    }

    pub fn is_rolling(&self) -> bool {
        self.get_flag(FLAG_ROLL)
    }

    pub fn set_has_nectar(&self, value: bool) {
        self.set_flag(FLAG_HAS_NECTAR, value);
    }

    pub fn set_has_stung(&self, value: bool) {
        self.set_flag(FLAG_HAS_STUNG, value);
    }

    pub fn set_rolling(&self, value: bool) {
        self.set_flag(FLAG_ROLL, value);
    }

    // --- Metadata sync ---

    async fn sync_bee_flags_metadata(&self) {
        let flags = self.bee_flags.load(Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_BEE_FLAGS,
                MetaDataType::Byte,
                flags,
            )])
            .await;
    }

    async fn sync_anger_metadata(&self) {
        let anger = self.anger_end_time.load(Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_ANGER_END_TIME,
                MetaDataType::Integer,
                VarInt(anger),
            )])
            .await;
    }
}

impl NBTStorage for BeeEntity {}

impl Mob for BeeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Bees fly — no gravity
    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    /// Vanilla flying entity drag
    fn get_mob_y_velocity_drag(&self) -> Option<f64> {
        Some(0.6)
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.mob_entity.living_entity.dead.load(Relaxed) {
                return;
            }

            // Decrement hive cooldown
            if self.stay_out_of_hive_countdown.load(Relaxed) > 0 {
                self.stay_out_of_hive_countdown.fetch_sub(1, Relaxed);
            }

            // Track ticks without nectar
            if !self.get_flag(FLAG_HAS_NECTAR) {
                self.ticks_without_nectar.fetch_add(1, Relaxed);
            }

            if self.has_stung() {
                let ticks = self.time_since_sting.fetch_add(1, Relaxed) + 1;
                if ticks % 5 == 0 {
                    let clamp = (STING_DEATH_COUNTDOWN - ticks).max(1);
                    let mut rng = rand::rng();
                    if rand::RngExt::random_range(&mut rng, 0..clamp) == 0 {
                        self.mob_entity.living_entity.dead.store(true, Relaxed);
                    }
                }
            }

            self.sync_bee_flags_metadata().await;
        })
    }
}
