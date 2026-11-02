use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::{
    data_component::DataComponent,
    data_component_impl::{
        DataComponentImpl, TropicalFishBaseColorImpl, TropicalFishPatternColorImpl,
        TropicalFishPatternImpl,
    },
    dye_color::DyeColor,
    entity_variant::{TROPICAL_FISH_COMMON_VARIANTS, TropicalFishPattern},
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
    mob::{Mob, MobEntity, spawn::SpawnGroupData},
};

const PATTERN_MASK: i32 = 0xFFFF;
const COLOR_MASK: i32 = 0xFF;
const BASE_COLOR_SHIFT: i32 = 16;
const PATTERN_COLOR_SHIFT: i32 = 24;
const COMMON_VARIANT_CHANCE: f64 = 0.9;

const fn pack_variant(
    pattern: TropicalFishPattern,
    base_color: DyeColor,
    pattern_color: DyeColor,
) -> i32 {
    pattern.id() & PATTERN_MASK
        | (base_color.id() as i32 & COLOR_MASK) << BASE_COLOR_SHIFT
        | (pattern_color.id() as i32 & COLOR_MASK) << PATTERN_COLOR_SHIFT
}

fn get_base_color(packed: i32) -> DyeColor {
    DyeColor::by_id_or_white(packed >> BASE_COLOR_SHIFT & COLOR_MASK)
}

fn get_pattern_color(packed: i32) -> DyeColor {
    DyeColor::by_id_or_white(packed >> PATTERN_COLOR_SHIFT & COLOR_MASK)
}

const fn get_pattern(packed: i32) -> TropicalFishPattern {
    TropicalFishPattern::by_id(packed & PATTERN_MASK)
}

const DEFAULT_VARIANT: i32 =
    pack_variant(TropicalFishPattern::Kob, DyeColor::White, DyeColor::White);

pub struct TropicalFishEntity {
    pub mob_entity: MobEntity,
    fish_data: FishData,
    schooling_data: SchoolingFishData,
    variant: AtomicI32,
    is_school: AtomicBool,
}

impl TropicalFishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        super::abstract_fish::initialize_fish_movement(&mut mob_entity);
        let mob_arc = Arc::new_cyclic(|weak: &std::sync::Weak<Self>| Self {
            mob_entity,
            fish_data: FishData::default(),
            schooling_data: SchoolingFishData::new(weak.clone()),
            variant: AtomicI32::new(DEFAULT_VARIANT),
            is_school: AtomicBool::new(true),
        });
        super::abstract_fish::register_fish_goals(&mob_arc.mob_entity);
        mob_arc
            .mob_entity
            .goals_selector
            .add_goal(5, Box::new(FollowFlockLeaderGoal::default()));

        mob_arc
    }

    fn get_packed_variant(&self) -> i32 {
        self.variant.load(Relaxed)
    }

    fn set_packed_variant(&self, packed: i32) {
        self.variant.store(packed, Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::tropical_fish::DATA_ID_TYPE_VARIANT,
            packed,
        );
    }

    pub fn get_pattern(&self) -> TropicalFishPattern {
        get_pattern(self.get_packed_variant())
    }

    pub fn get_base_color(&self) -> DyeColor {
        get_base_color(self.get_packed_variant())
    }

    pub fn get_pattern_color(&self) -> DyeColor {
        get_pattern_color(self.get_packed_variant())
    }

    fn set_pattern(&self, pattern: TropicalFishPattern) {
        self.set_packed_variant(pack_variant(
            pattern,
            self.get_base_color(),
            self.get_pattern_color(),
        ));
    }

    fn set_base_color(&self, base_color: DyeColor) {
        self.set_packed_variant(pack_variant(
            self.get_pattern(),
            base_color,
            self.get_pattern_color(),
        ));
    }

    fn set_pattern_color(&self, pattern_color: DyeColor) {
        self.set_packed_variant(pack_variant(
            self.get_pattern(),
            self.get_base_color(),
            pattern_color,
        ));
    }

    fn pattern_component(&self) -> TropicalFishPatternImpl {
        TropicalFishPatternImpl {
            value: self.get_pattern().name().into(),
        }
    }

    fn base_color_component(&self) -> TropicalFishBaseColorImpl {
        TropicalFishBaseColorImpl {
            value: self.get_base_color().name().into(),
        }
    }

    fn pattern_color_component(&self) -> TropicalFishPatternColorImpl {
        TropicalFishPatternColorImpl {
            value: self.get_pattern_color().name().into(),
        }
    }
}

impl Mob for TropicalFishEntity {
    crate::fish_mob_hooks!();
    crate::schooling_fish_mob_hooks!();

    fn finalize_spawn(
        &self,
        _world: &Arc<crate::world::World>,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let mut group_data = self.finalize_school_spawn(group_data);
        let variant = if let Some(SpawnGroupData::TropicalFish { variant, .. }) = group_data {
            variant
        } else {
            let mut rng = self.get_random();
            if f64::from(rng.random::<f32>()) < COMMON_VARIANT_CHANCE {
                let (pattern, base_color, pattern_color) = TROPICAL_FISH_COMMON_VARIANTS
                    [rng.random_range(0..TROPICAL_FISH_COMMON_VARIANTS.len())];
                let variant = pack_variant(pattern, base_color, pattern_color);
                group_data = Some(SpawnGroupData::TropicalFish {
                    leader: self.schooling_data.self_reference.clone(),
                    variant,
                });
                variant
            } else {
                self.is_school.store(false, Relaxed);
                let pattern =
                    TropicalFishPattern::ALL[rng.random_range(0..TropicalFishPattern::ALL.len())];
                let base_color = DyeColor::ALL[rng.random_range(0..DyeColor::ALL.len())];
                let pattern_color = DyeColor::ALL[rng.random_range(0..DyeColor::ALL.len())];
                pack_variant(pattern, base_color, pattern_color)
            }
        };
        self.set_packed_variant(variant);
        group_data
    }

    fn is_max_group_size_reached(&self, _group_size: i32) -> bool {
        !self.is_school.load(Relaxed)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.fish_write_nbt(nbt);
        nbt.put_int("Variant", self.get_packed_variant());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.fish_read_nbt(nbt);
        let packed = nbt.get_int("Variant").unwrap_or(DEFAULT_VARIANT);
        self.set_packed_variant(pack_variant(
            get_pattern(packed),
            get_base_color(packed),
            get_pattern_color(packed),
        ));
    }

    fn mob_init_data_tracker(&self) {
        self.set_from_bucket(self.is_from_bucket());
        self.set_packed_variant(self.get_packed_variant());
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        match component {
            DataComponent::TropicalFishPattern => Some(Box::new(self.pattern_component())),
            DataComponent::TropicalFishBaseColor => Some(Box::new(self.base_color_component())),
            DataComponent::TropicalFishPatternColor => {
                Some(Box::new(self.pattern_color_component()))
            }
            _ => None,
        }
    }

    fn mob_apply_implicit_component(&self, component: DataComponent, value: &str) {
        match component {
            DataComponent::TropicalFishPattern => {
                if let Some(pattern) = TropicalFishPattern::from_name(value) {
                    self.set_pattern(pattern);
                }
            }
            DataComponent::TropicalFishBaseColor => {
                if let Some(color) = DyeColor::by_name(value) {
                    self.set_base_color(color);
                }
            }
            DataComponent::TropicalFishPatternColor => {
                if let Some(color) = DyeColor::by_name(value) {
                    self.set_pattern_color(color);
                }
            }
            _ => {}
        }
    }
}

crate::impl_abstract_fish!(
    TropicalFishEntity,
    bucket: &Item::TROPICAL_FISH_BUCKET,
    flop: Sound::EntityTropicalFishFlop,
    ambient: Some(Sound::EntityTropicalFishAmbient),
    bucket_extra: {
        fn save_to_bucket_tag(&self, bucket: &mut ItemStack) {
            self.save_default_data_to_bucket_tag(bucket);
            bucket.set_data_component(self.pattern_component());
            bucket.set_data_component(self.base_color_component());
            bucket.set_data_component(self.pattern_color_component());
        }
    }
);
crate::impl_schooling_fish!(TropicalFishEntity);
