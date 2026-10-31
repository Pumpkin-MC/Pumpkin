use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::{
    data_component::DataComponent,
    data_component_impl::{DataComponentImpl, SalmonSizeImpl},
    entity_variant::SalmonSize,
    item::Item,
    item_stack::ItemStack,
    sound::Sound,
};
use pumpkin_nbt::compound::NbtCompound;
use rand::RngExt;

use super::{
    abstract_fish::{AbstractFish, FishData},
    abstract_schooling_fish::{AbstractSchoolingFish, SchoolingFishData},
};
use crate::entity::{
    Entity, EntityBase,
    ai::goal::follow_flock_leader::FollowFlockLeaderGoal,
    bucketable::Bucketable,
    mob::{Mob, MobEntity},
};

const SMALL_WEIGHT: u32 = 30;
const MEDIUM_WEIGHT: u32 = 50;
const LARGE_WEIGHT: u32 = 15;

pub struct SalmonEntity {
    pub mob_entity: MobEntity,
    fish_data: FishData,
    schooling_data: SchoolingFishData,
    variant: AtomicI32,
}

impl SalmonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        super::abstract_fish::initialize_fish_movement(&mut mob_entity);
        let mob_arc = Arc::new_cyclic(|weak: &std::sync::Weak<Self>| Self {
            mob_entity,
            fish_data: FishData::default(),
            schooling_data: SchoolingFishData::new(weak.clone()),
            variant: AtomicI32::new(SalmonSize::DEFAULT.id()),
        });
        super::abstract_fish::register_fish_goals(&mob_arc.mob_entity);
        mob_arc
            .mob_entity
            .goals_selector
            .add_goal(5, Box::new(FollowFlockLeaderGoal::default()));

        mob_arc
    }

    pub fn get_variant(&self) -> SalmonSize {
        SalmonSize::by_id(self.variant.load(Relaxed))
    }

    pub fn set_variant(&self, variant: SalmonSize) {
        self.variant.store(variant.id(), Relaxed);
        self.get_entity()
            .set_synced_data(pumpkin_data::tracked_data::salmon::DATA_TYPE, variant.id());
        self.set_dimension_scale(variant.bounding_box_scale());
    }

    fn random_spawn_variant(&self) -> SalmonSize {
        let roll = self
            .get_random()
            .random_range(0..SMALL_WEIGHT + MEDIUM_WEIGHT + LARGE_WEIGHT);
        if roll < SMALL_WEIGHT {
            SalmonSize::Small
        } else if roll < SMALL_WEIGHT + MEDIUM_WEIGHT {
            SalmonSize::Medium
        } else {
            SalmonSize::Large
        }
    }

    fn size_component(&self) -> SalmonSizeImpl {
        SalmonSizeImpl {
            value: self.get_variant().name().into(),
        }
    }
}

impl Mob for SalmonEntity {
    crate::fish_mob_hooks!();
    crate::schooling_fish_mob_hooks!();

    fn finalize_spawn(
        &self,
        _world: &Arc<crate::world::World>,
        group_data: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        self.set_variant(self.random_spawn_variant());
        self.finalize_school_spawn(group_data)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.fish_write_nbt(nbt);
        nbt.put_string("type", self.get_variant().name().to_string());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.fish_read_nbt(nbt);
        self.set_variant(
            nbt.get_string("type")
                .and_then(SalmonSize::from_name)
                .unwrap_or(SalmonSize::DEFAULT),
        );
    }

    fn mob_init_data_tracker(&self) {
        self.set_from_bucket(self.is_from_bucket());
        self.set_variant(self.get_variant());
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        match component {
            DataComponent::SalmonSize => Some(Box::new(self.size_component())),
            _ => None,
        }
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(variant) = SalmonSize::from_name(name) {
            self.set_variant(variant);
        }
    }
}

crate::impl_abstract_fish!(
    SalmonEntity,
    bucket: &Item::SALMON_BUCKET,
    flop: Sound::EntitySalmonFlop,
    ambient: Some(Sound::EntitySalmonAmbient),
    bucket_extra: {
        fn save_to_bucket_tag(&self, bucket: &mut ItemStack) {
            self.save_default_data_to_bucket_tag(bucket);
            bucket.set_data_component(self.size_component());
        }
    }
);
crate::impl_schooling_fish!(SalmonEntity);
