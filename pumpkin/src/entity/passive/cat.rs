use std::sync::{
    Arc, Weak,
    atomic::{AtomicU8, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::particle::Particle;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        non_tame_random_target::NonTameRandomTargetGoal, sit::SitGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::COD, &Item::SALMON];

const NATURAL_CAT_VARIANTS: [&str; 10] = [
    "tabby",
    "black",
    "red",
    "siamese",
    "british_shorthair",
    "calico",
    "persian",
    "ragdoll",
    "white",
    "jellie",
];

/// Moon brightness per lunar phase index (0 = full moon), from vanilla
/// `DimensionType.MOON_BRIGHTNESS_PER_PHASE`.
const MOON_BRIGHTNESS_PER_PHASE: [f32; 8] = [1.0, 0.75, 0.5, 0.25, 0.0, 0.25, 0.5, 0.75];

/// `MoonBrightnessCheck` gate used by the `all_black` cat variant's spawn selector.
///
/// `CatVariants.java` pairs `all_black` with `MinMaxBounds.Doubles.atLeast(0.9)`
/// against the current moon phase's brightness, which only the full moon
/// (phase 0, brightness 1.0) satisfies.
#[must_use]
pub fn moon_brightness_allows_all_black(time_of_day: i64) -> bool {
    let phase = time_of_day.div_euclid(24000).rem_euclid(8) as usize;
    MOON_BRIGHTNESS_PER_PHASE[phase] >= 0.9
}

/// Picks a natural-spawn cat variant.
///
/// Replicates `CatVariants.bootstrap`'s `PriorityProvider.pick` outcome with
/// the `StructureCheck` selector for `all_black` (swamp-hut detection)
/// dropped: Pumpkin has no structure-manager lookup by structure tag. On a
/// full moon, `all_black` joins the uniform pool of the 10 base variants;
/// otherwise only the base variants are eligible.
#[must_use]
pub fn select_natural_cat_variant(time_of_day: i64) -> &'static str {
    if moon_brightness_allows_all_black(time_of_day) {
        const POOL: [&str; 11] = [
            "tabby",
            "black",
            "red",
            "siamese",
            "british_shorthair",
            "calico",
            "persian",
            "ragdoll",
            "white",
            "jellie",
            "all_black",
        ];
        POOL[rand::random_range(0..POOL.len())]
    } else {
        NATURAL_CAT_VARIANTS[rand::random_range(0..NATURAL_CAT_VARIANTS.len())]
    }
}

pub struct CatEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
}

impl CatEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cat = Self {
            mob_entity,
            variant: AtomicU8::new(9), // Default to tabby
        };
        let mob_arc = Arc::new(cat);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(2, SitGoal::new());
            goal_selector.add_goal(4, Box::new(TemptGoal::new(0.6, TEMPT_ITEMS)));
            goal_selector.add_goal(5, BreedGoal::new(0.8));
            // goal_selector.add_goal(7, FollowOwnerGoal::new(1.0, 10.0, 5.0, false));
            goal_selector.add_goal(9, Box::new(FollowParentGoal::new(0.8)));
            goal_selector.add_goal(11, Box::new(WanderAroundGoal::new(0.8)));
            goal_selector.add_goal(
                12,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
            goal_selector.add_goal(12, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(
                1,
                NonTameRandomTargetGoal::without_predicate(
                    &mob_arc.mob_entity,
                    &[&EntityType::RABBIT],
                    false,
                ),
            );
            target_selector.add_goal(
                1,
                NonTameRandomTargetGoal::new(
                    &mob_arc.mob_entity,
                    crate::entity::ai::goal::non_tame_random_target::TURTLE_TYPES,
                    false,
                    Some(crate::entity::ai::goal::non_tame_random_target::baby_turtle_on_land),
                ),
            );
        };

        mob_arc
    }
}

impl NBTStorage for CatEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            let variant_str = match self.variant.load(Ordering::Relaxed) {
                0 => "minecraft:all_black",
                1 => "minecraft:black",
                2 => "minecraft:british_shorthair",
                3 => "minecraft:calico",
                4 => "minecraft:jellie",
                5 => "minecraft:persian",
                6 => "minecraft:ragdoll",
                7 => "minecraft:red",
                8 => "minecraft:siamese",
                10 => "minecraft:white",
                _ => "minecraft:tabby",
            };
            nbt.put_string("variant", variant_str.to_string());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            if let Some(variant_str) = nbt.get_string("variant") {
                let variant = match variant_str
                    .strip_prefix("minecraft:")
                    .unwrap_or(variant_str)
                {
                    "all_black" => 0,
                    "black" => 1,
                    "british_shorthair" => 2,
                    "calico" => 3,
                    "jellie" => 4,
                    "persian" => 5,
                    "ragdoll" => 6,
                    "red" => 7,
                    "siamese" => 8,
                    "white" => 10,
                    _ => 9,
                };
                self.variant.store(variant, Ordering::Relaxed);
            }
        })
    }
}

impl Mob for CatEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let is_food = TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id);
            if self.mob_entity.is_tamed() || !is_food {
                return false;
            }

            item_stack.decrement_unless_creative(player.gamemode.load(), 1);

            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let pos = entity.pos.load() + Vector3::new(0.0, f64::from(entity.height()), 0.0);

            if self.get_random().random_range(0..3) == 0 {
                self.mob_entity.set_owner(player.gameprofile.id);
                world.spawn_particle(pos, Vector3::new(0.5, 0.5, 0.5), 1.0, 7, Particle::Heart);
            } else {
                world.spawn_particle(pos, Vector3::new(0.5, 0.5, 0.5), 1.0, 7, Particle::Smoke);
            }

            true
        })
    }

    fn mob_set_variant_name(&self, name: &str) {
        let variant = match name.strip_prefix("minecraft:").unwrap_or(name) {
            "all_black" => 0,
            "black" => 1,
            "british_shorthair" => 2,
            "calico" => 3,
            "jellie" => 4,
            "persian" => 5,
            "ragdoll" => 6,
            "red" => 7,
            "siamese" => 8,
            "white" => 10,
            _ => 9,
        };
        self.variant.store(variant, Ordering::Relaxed);
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let is_baby = entity.age.load(Ordering::Relaxed) < 0;
            if is_baby {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::CAT_VARIANT,
                    MetaDataType::CAT_VARIANT,
                    VarInt(self.variant.load(Ordering::Relaxed) as i32),
                )],
                None,
            );
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{moon_brightness_allows_all_black, select_natural_cat_variant};

    #[test]
    fn only_full_moon_allows_all_black() {
        for phase in 0..8 {
            let time_of_day = phase * 24000;
            let allowed = moon_brightness_allows_all_black(time_of_day);
            assert_eq!(
                allowed,
                phase == 0,
                "phase {phase} should only allow all_black on full moon"
            );
        }
    }

    #[test]
    fn phase_wraps_across_multiple_lunar_cycles() {
        assert!(moon_brightness_allows_all_black(8 * 24000));
        assert!(!moon_brightness_allows_all_black(9 * 24000));
    }

    #[test]
    fn all_black_only_selectable_on_full_moon() {
        // Day 4 (new moon, brightness 0.0) is never a full moon.
        for _ in 0..200 {
            assert_ne!(select_natural_cat_variant(4 * 24000), "all_black");
        }
        let mut saw_all_black = false;
        for _ in 0..500 {
            if select_natural_cat_variant(0) == "all_black" {
                saw_all_black = true;
                break;
            }
        }
        assert!(saw_all_black);
    }
}
