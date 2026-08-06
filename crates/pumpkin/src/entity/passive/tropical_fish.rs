use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use rand::RngExt;

use crate::block::entities::sign::DyeColor;
use crate::entity::{
    Entity, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, escape_danger::EscapeDangerGoal,
        follow_flock_leader::FollowFlockLeaderGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::fish_variant::{
        dye_color_from_id, pack_variant, unpack_base_color_id, unpack_pattern_color_id,
        unpack_pattern_id,
    },
};

/// `TropicalFish.Base`: the body-shape flag packed into a pattern's low byte. Distinct from
/// Salmon's bounding-box scale - this is purely a pattern-shape identifier.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Base {
    Small = 0,
    Large = 1,
}

/// `TropicalFish.Pattern`: 12 patterns, each `base.id | index << 8`. Ported verbatim from the
/// decompiled source (order and indices matter - they are the wire values).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pattern {
    Kob,
    Sunstreak,
    Snooper,
    Dasher,
    Brinely,
    Spotty,
    Flopper,
    Stripey,
    Glitter,
    Blockfish,
    Betty,
    Clayfish,
}

impl Pattern {
    const ALL: [Self; 12] = [
        Self::Kob,
        Self::Sunstreak,
        Self::Snooper,
        Self::Dasher,
        Self::Brinely,
        Self::Spotty,
        Self::Flopper,
        Self::Stripey,
        Self::Glitter,
        Self::Blockfish,
        Self::Betty,
        Self::Clayfish,
    ];

    #[must_use]
    pub const fn base_and_index(self) -> (Base, u16) {
        match self {
            Self::Kob => (Base::Small, 0),
            Self::Sunstreak => (Base::Small, 1),
            Self::Snooper => (Base::Small, 2),
            Self::Dasher => (Base::Small, 3),
            Self::Brinely => (Base::Small, 4),
            Self::Spotty => (Base::Small, 5),
            Self::Flopper => (Base::Large, 0),
            Self::Stripey => (Base::Large, 1),
            Self::Glitter => (Base::Large, 2),
            Self::Blockfish => (Base::Large, 3),
            Self::Betty => (Base::Large, 4),
            Self::Clayfish => (Base::Large, 5),
        }
    }

    #[must_use]
    pub const fn packed_id(self) -> u16 {
        let (base, index) = self.base_and_index();
        base as u16 | (index << 8)
    }

    /// `Pattern.byId`: sparse lookup, falling back to `KOB` (matching `ByIdMap.sparse`'s
    /// documented fallback-to-first-argument behavior) for any packed id with no match.
    #[must_use]
    pub const fn from_packed_id(id: u16) -> Self {
        let mut i = 0;
        while i < Self::ALL.len() {
            if Self::ALL[i].packed_id() == id {
                return Self::ALL[i];
            }
            i += 1;
        }
        Self::Kob
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Kob => "kob",
            Self::Sunstreak => "sunstreak",
            Self::Snooper => "snooper",
            Self::Dasher => "dasher",
            Self::Brinely => "brinely",
            Self::Spotty => "spotty",
            Self::Flopper => "flopper",
            Self::Stripey => "stripey",
            Self::Glitter => "glitter",
            Self::Blockfish => "blockfish",
            Self::Betty => "betty",
            Self::Clayfish => "clayfish",
        }
    }
}

/// `TropicalFish.DEFAULT_VARIANT`: `KOB` / `WHITE` / `WHITE`.
const DEFAULT_PACKED_VARIANT: i32 = 0;

/// `TropicalFish.COMMON_VARIANTS`: 22 fixed (pattern, base color, pattern color) triples,
/// copied verbatim from the decompiled source (not paraphrased - these are player-recognized
/// named varieties).
const COMMON_VARIANTS: [(Pattern, DyeColor, DyeColor); 22] = [
    (Pattern::Stripey, DyeColor::Orange, DyeColor::Gray),
    (Pattern::Flopper, DyeColor::Gray, DyeColor::Gray),
    (Pattern::Flopper, DyeColor::Gray, DyeColor::Blue),
    (Pattern::Clayfish, DyeColor::White, DyeColor::Gray),
    (Pattern::Sunstreak, DyeColor::Blue, DyeColor::Gray),
    (Pattern::Kob, DyeColor::Orange, DyeColor::White),
    (Pattern::Spotty, DyeColor::Pink, DyeColor::LightBlue),
    (Pattern::Blockfish, DyeColor::Purple, DyeColor::Yellow),
    (Pattern::Clayfish, DyeColor::White, DyeColor::Red),
    (Pattern::Spotty, DyeColor::White, DyeColor::Yellow),
    (Pattern::Glitter, DyeColor::White, DyeColor::Gray),
    (Pattern::Clayfish, DyeColor::White, DyeColor::Orange),
    (Pattern::Dasher, DyeColor::Cyan, DyeColor::Pink),
    (Pattern::Brinely, DyeColor::Lime, DyeColor::LightBlue),
    (Pattern::Betty, DyeColor::Red, DyeColor::White),
    (Pattern::Snooper, DyeColor::Gray, DyeColor::Red),
    (Pattern::Blockfish, DyeColor::Red, DyeColor::White),
    (Pattern::Flopper, DyeColor::White, DyeColor::Yellow),
    (Pattern::Kob, DyeColor::Red, DyeColor::White),
    (Pattern::Sunstreak, DyeColor::Gray, DyeColor::White),
    (Pattern::Dasher, DyeColor::Cyan, DyeColor::Yellow),
    (Pattern::Flopper, DyeColor::Yellow, DyeColor::Yellow),
];

pub struct TropicalFishEntity {
    pub mob_entity: MobEntity,
    packed_variant: AtomicI32,
    /// `TropicalFish.isSchool`: per-instance, not synced/saved - only affects
    /// `isMaxGroupSizeReached` at runtime.
    is_school: AtomicBool,
}

impl TropicalFishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);

        // `TropicalFish.finalizeSpawn`: 90% picks a common named variant (vanilla shares one
        // roll across the whole spawn group via `TropicalFishGroupData`; this codebase has no
        // group-spawn-data mechanism for any mob yet, so each fish rolls independently here -
        // a known, explicitly-flagged deviation, not a silent approximation). The 10% branch
        // rolls a fully random triple and marks the fish as non-schooling.
        let mut rng = rand::rng();
        let (packed, is_school) = if rng.random::<f32>() < 0.9 {
            let (pattern, base, pattern_color) =
                COMMON_VARIANTS[rng.random_range(0..COMMON_VARIANTS.len())];
            (
                pack_variant(pattern.packed_id(), base as u8, pattern_color as u8),
                true,
            )
        } else {
            let pattern = Pattern::ALL[rng.random_range(0..Pattern::ALL.len())];
            let base_color = rng.random_range(0u8..16);
            let pattern_color = rng.random_range(0u8..16);
            (
                pack_variant(pattern.packed_id(), base_color, pattern_color),
                false,
            )
        };

        let tropical_fish = Self {
            mob_entity,
            packed_variant: AtomicI32::new(packed),
            is_school: AtomicBool::new(is_school),
        };
        let mob_arc = Arc::new(tropical_fish);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // Vanilla `AbstractFish.registerGoals` has no float/swim goal.
            goal_selector.add_goal(0, EscapeDangerGoal::new(1.25));
            goal_selector.add_goal(
                2,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 1.6, 1.4)),
            );
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new_with_interval(1.0, 40)));
            goal_selector.add_goal(5, FollowFlockLeaderGoal::new());
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn pattern(&self) -> Pattern {
        Pattern::from_packed_id(unpack_pattern_id(self.packed_variant.load(Relaxed)))
    }

    #[must_use]
    pub fn base_color(&self) -> DyeColor {
        dye_color_from_id(unpack_base_color_id(self.packed_variant.load(Relaxed)))
    }

    #[must_use]
    pub fn pattern_color(&self) -> DyeColor {
        dye_color_from_id(unpack_pattern_color_id(self.packed_variant.load(Relaxed)))
    }

    /// `TropicalFish.isMaxGroupSizeReached`: a non-common fish is immediately at max group
    /// size (i.e. refuses to group at all).
    #[must_use]
    pub fn is_max_group_size_reached(&self) -> bool {
        !self.is_school.load(Relaxed)
    }

    fn send_packed_variant(&self, packed: i32) {
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::ID_TYPE_VARIANT,
                MetaDataType::INT,
                VarInt(packed),
            )],
            None,
        );
    }
}

impl NBTStorage for TropicalFishEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            // `Variant.CODEC` is `Codec.INT.xmap(...)`: the raw packed int, not a compound.
            nbt.put_int("Variant", self.packed_variant.load(Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            let packed = nbt.get_int("Variant").unwrap_or(DEFAULT_PACKED_VARIANT);
            self.packed_variant.store(packed, Relaxed);
        })
    }
}

impl Mob for TropicalFishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.send_packed_variant(self.packed_variant.load(Relaxed));
        })
    }
}
