use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};

use crate::entity::{
    Entity, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, escape_danger::EscapeDangerGoal,
        follow_flock_leader::FollowFlockLeaderGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::fish_variant::pick_weighted,
};

/// `Salmon.Variant`: id, `StringRepresentable` name, and bounding-box scale.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum SalmonVariant {
    Small = 0,
    Medium = 1,
    Large = 2,
}

impl SalmonVariant {
    #[must_use]
    pub const fn bounding_box_scale(self) -> f32 {
        match self {
            Self::Small => 0.5,
            Self::Medium => 1.0,
            Self::Large => 1.5,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }

    #[must_use]
    pub const fn from_id(id: u8) -> Self {
        match id {
            0 => Self::Small,
            2 => Self::Large,
            _ => Self::Medium,
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "small" => Self::Small,
            "large" => Self::Large,
            _ => Self::Medium,
        }
    }
}

/// `Salmon.finalizeSpawn`: `SMALL` 30, `MEDIUM` 50, `LARGE` 15 (a 30/50/15 weighted split, not
/// a flat distribution - confirmed against the decompiled source rather than assumed).
const VARIANT_WEIGHTS: [(SalmonVariant, u32); 3] = [
    (SalmonVariant::Small, 30),
    (SalmonVariant::Medium, 50),
    (SalmonVariant::Large, 15),
];

pub struct SalmonEntity {
    pub mob_entity: MobEntity,
    variant: AtomicU8,
}

impl SalmonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        // Vanilla `Salmon.finalizeSpawn` rolls the variant on spawn; there is no
        // finalize-spawn hook in this codebase yet (see the Turtle home_pos commit for the
        // same gap), so it's rolled here instead. NBT restore (which runs after
        // construction on load) overwrites this for existing salmon.
        let variant = pick_weighted(&mut rand::rng(), &VARIANT_WEIGHTS);
        let salmon = Self {
            mob_entity,
            variant: AtomicU8::new(variant as u8),
        };
        salmon.refresh_dimensions(variant);
        let mob_arc = Arc::new(salmon);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
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
    pub fn variant(&self) -> SalmonVariant {
        SalmonVariant::from_id(self.variant.load(Relaxed))
    }

    /// `Salmon.getDefaultDimensions`: base dimensions scaled by `getSalmonScale()`.
    fn refresh_dimensions(&self, variant: SalmonVariant) {
        let entity = &self.mob_entity.living_entity.entity;
        let base = entity.entity_type.dimension;
        let scale = variant.bounding_box_scale();
        let dimensions = EntityDimensions {
            width: base[0] * scale,
            height: base[1] * scale,
            eye_height: entity.entity_type.eye_height * scale,
        };
        entity.entity_dimension.store(dimensions);
        let pos = entity.pos.load();
        entity
            .bounding_box
            .store(BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &dimensions));
    }

    fn set_variant(&self, variant: SalmonVariant) {
        self.variant.store(variant as u8, Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::VARIANT,
                MetaDataType::INT,
                VarInt(i32::from(variant as u8)),
            )],
            None,
        );
        // `Salmon.onSyncedDataUpdated`: variant changes must refresh dimensions reactively,
        // not just at spawn.
        self.refresh_dimensions(variant);
    }
}

impl NBTStorage for SalmonEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_string("type", self.variant().name().to_string());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            let variant = nbt
                .get_string("type")
                .map_or(SalmonVariant::Medium, SalmonVariant::from_name);
            self.set_variant(variant);
        })
    }
}

impl Mob for SalmonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::VARIANT,
                    MetaDataType::INT,
                    VarInt(i32::from(self.variant.load(Relaxed))),
                )],
                None,
            );
        })
    }
}
