use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::{
    item::Item, meta_data_type::MetaDataType, sound::Sound, tracked_data::TrackedData,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::Metadata};
use pumpkin_world::item::ItemStack;
use rand::RngExt;

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

const DYE_WHITE: i32 = 0;
const DYE_ORANGE: i32 = 1;
const DYE_LIGHT_BLUE: i32 = 3;
const DYE_YELLOW: i32 = 4;
const DYE_LIME: i32 = 5;
const DYE_PINK: i32 = 6;
const DYE_GRAY: i32 = 7;
const DYE_CYAN: i32 = 9;
const DYE_PURPLE: i32 = 10;
const DYE_BLUE: i32 = 11;
const DYE_RED: i32 = 14;

const PATTERN_KOB: i32 = 0;
const PATTERN_SUNSTREAK: i32 = 1 << 8;
const PATTERN_SNOOPER: i32 = 2 << 8;
const PATTERN_DASHER: i32 = 3 << 8;
const PATTERN_BRINELY: i32 = 4 << 8;
const PATTERN_SPOTTY: i32 = 5 << 8;
const PATTERN_FLOPPER: i32 = 1;
const PATTERN_STRIPEY: i32 = (1 << 8) | 1;
const PATTERN_GLITTER: i32 = (2 << 8) | 1;
const PATTERN_BLOCKFISH: i32 = (3 << 8) | 1;
const PATTERN_BETTY: i32 = (4 << 8) | 1;
const PATTERN_CLAYFISH: i32 = (5 << 8) | 1;

const fn pack_variant(pattern: i32, base_color: i32, pattern_color: i32) -> i32 {
    (pattern & 0xFFFF) | ((base_color & 0xFF) << 16) | ((pattern_color & 0xFF) << 24)
}

const DEFAULT_VARIANT: i32 = pack_variant(PATTERN_KOB, DYE_WHITE, DYE_WHITE);
const COMMON_VARIANTS: [i32; 22] = [
    pack_variant(PATTERN_STRIPEY, DYE_ORANGE, DYE_GRAY),
    pack_variant(PATTERN_FLOPPER, DYE_GRAY, DYE_GRAY),
    pack_variant(PATTERN_FLOPPER, DYE_GRAY, DYE_BLUE),
    pack_variant(PATTERN_CLAYFISH, DYE_WHITE, DYE_GRAY),
    pack_variant(PATTERN_SUNSTREAK, DYE_BLUE, DYE_GRAY),
    pack_variant(PATTERN_KOB, DYE_ORANGE, DYE_WHITE),
    pack_variant(PATTERN_SPOTTY, DYE_PINK, DYE_LIGHT_BLUE),
    pack_variant(PATTERN_BLOCKFISH, DYE_PURPLE, DYE_YELLOW),
    pack_variant(PATTERN_CLAYFISH, DYE_WHITE, DYE_RED),
    pack_variant(PATTERN_SPOTTY, DYE_WHITE, DYE_YELLOW),
    pack_variant(PATTERN_GLITTER, DYE_WHITE, DYE_GRAY),
    pack_variant(PATTERN_CLAYFISH, DYE_WHITE, DYE_ORANGE),
    pack_variant(PATTERN_DASHER, DYE_CYAN, DYE_PINK),
    pack_variant(PATTERN_BRINELY, DYE_LIME, DYE_LIGHT_BLUE),
    pack_variant(PATTERN_BETTY, DYE_RED, DYE_WHITE),
    pack_variant(PATTERN_SNOOPER, DYE_GRAY, DYE_RED),
    pack_variant(PATTERN_BLOCKFISH, DYE_RED, DYE_WHITE),
    pack_variant(PATTERN_FLOPPER, DYE_WHITE, DYE_YELLOW),
    pack_variant(PATTERN_KOB, DYE_RED, DYE_WHITE),
    pack_variant(PATTERN_SUNSTREAK, DYE_GRAY, DYE_WHITE),
    pack_variant(PATTERN_DASHER, DYE_CYAN, DYE_YELLOW),
    pack_variant(PATTERN_FLOPPER, DYE_YELLOW, DYE_YELLOW),
];
const ALL_PATTERNS: [i32; 12] = [
    PATTERN_KOB,
    PATTERN_SUNSTREAK,
    PATTERN_SNOOPER,
    PATTERN_DASHER,
    PATTERN_BRINELY,
    PATTERN_SPOTTY,
    PATTERN_FLOPPER,
    PATTERN_STRIPEY,
    PATTERN_GLITTER,
    PATTERN_BLOCKFISH,
    PATTERN_BETTY,
    PATTERN_CLAYFISH,
];

pub struct TropicalFishEntity {
    pub mob_entity: MobEntity,
    from_bucket: AtomicBool,
    packed_variant: AtomicI32,
    school_leader_id: Arc<AtomicI32>,
}

impl TropicalFishEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let tropical_fish = Self {
            mob_entity: MobEntity::new(entity),
            from_bucket: AtomicBool::new(false),
            packed_variant: AtomicI32::new(Self::roll_spawn_variant()),
            school_leader_id: Arc::new(AtomicI32::new(0)),
        };
        let mob_arc = Arc::new(tropical_fish);
        let school_leader_id = mob_arc.school_leader_id.clone();

        mob_arc.mob_entity.living_entity.movement_speed.store(0.1);
        *mob_arc.mob_entity.move_control.lock().await = MoveControl::fish(90);
        mob_arc.sync_from_bucket_metadata().await;
        mob_arc.sync_variant_metadata().await;

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            goal_selector.add_goal(0, Box::new(FishPanicGoal::new()));
            goal_selector.add_goal(2, Box::new(FishAvoidPlayerGoal::new()));
            goal_selector.add_goal(4, Box::new(FishSwimGoal::new(school_leader_id.clone())));
            goal_selector.add_goal(5, Box::new(FollowSchoolLeaderGoal::new(school_leader_id)));
        };

        mob_arc
    }

    fn roll_spawn_variant() -> i32 {
        let mut rng = rand::rng();
        if rng.random::<f32>() < 0.9 {
            let idx = rng.random_range(0..COMMON_VARIANTS.len());
            COMMON_VARIANTS[idx]
        } else {
            let pattern = ALL_PATTERNS[rng.random_range(0..ALL_PATTERNS.len())];
            let base_color = rng.random_range(0..=15);
            let pattern_color = rng.random_range(0..=15);
            pack_variant(pattern, base_color, pattern_color)
        }
    }

    fn is_from_bucket(&self) -> bool {
        self.from_bucket.load(Relaxed)
    }

    fn set_from_bucket(&self, from_bucket: bool) {
        self.from_bucket.store(from_bucket, Relaxed);
    }

    fn get_variant(&self) -> i32 {
        self.packed_variant.load(Relaxed)
    }

    fn set_variant(&self, variant: i32) {
        self.packed_variant.store(variant, Relaxed);
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

    async fn sync_variant_metadata(&self) {
        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_VARIANT,
                MetaDataType::Integer,
                VarInt(self.get_variant()),
            )])
            .await;
    }
}

impl NBTStorage for TropicalFishEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.entity.write_nbt(nbt).await;
            nbt.put_bool("FromBucket", self.is_from_bucket());
            nbt.put_int("Variant", self.get_variant());
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
            self.set_variant(nbt.get_int("Variant").unwrap_or(DEFAULT_VARIANT));
            self.sync_from_bucket_metadata().await;
            self.sync_variant_metadata().await;
        })
    }
}

impl Mob for TropicalFishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.mob_entity.living_entity.dead.load(Relaxed) {
                return;
            }
            maybe_flop(self, Sound::EntityTropicalFishFlop).await;
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Player,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            try_bucket_mob_pickup(self, player, item_stack, &Item::TROPICAL_FISH_BUCKET).await
        })
    }
}
